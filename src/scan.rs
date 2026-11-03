use anyhow::{Context, Result, ensure};
use crate::{artifact::{self, ScanConfig, Selection}, catalog::Catalog, formats, indicator::{self, ApplicationConfig, Indicator}, manifest, model::{digest, Application, Asset, Coverage, Diagnostic, Relationship, Report}, source::Source, syntax};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::{collections::BTreeMap, fs, io::Read, path::Path};

pub struct Options { pub max_file_bytes: u64, pub jobs: usize, pub indicators: Vec<Indicator>, pub selection: ScanConfig, pub applications: Vec<ApplicationConfig> }
impl Default for Options {
    fn default() -> Self { Self { max_file_bytes: 2 * 1024 * 1024, jobs: 4, indicators: vec![], selection: ScanConfig::default(), applications: vec![] } }
}

struct FileResult { assets: Vec<Asset>, relationships: Vec<Relationship>, language: String, diagnostic: Option<String> }

pub fn scan(source: &Source, catalog: &Catalog, options: &Options) -> Result<Report> {
    ensure!((1..=64).contains(&options.jobs), "jobs must be between 1 and 64");
    ensure!((1..=64 * 1024 * 1024).contains(&options.max_file_bytes), "max-file-bytes must be between 1 and 67108864");
    let selection = Selection::resolve(&options.selection, catalog)?;
    let needs_code = selection.effective.iter().any(|kind| artifact::capabilities(catalog, kind).keys().any(|l| syntax_language(l)));
    let (metadata, warnings) = indicator::collect(&source.root, &options.indicators)?;
    let mut coverage = Coverage::default();
    coverage.files_skipped += source.omissions.len();
    coverage.diagnostics.extend(source.omissions.clone());
    coverage.scope_notes = vec!["Git ignore rules and generated/vendor directories are excluded; excluded trees are not enumerated.".into(), "Source capabilities cover catalog-qualified lexical calls/imports, not full program or runtime behavior.".into(), "Manifest and MCP configuration locations are file-scoped (line 1); skill identity is frontmatter-scoped.".into()];
    coverage.diagnostics.extend(warnings.into_iter().map(|reason| Diagnostic { path: ".".into(), reason }));
    let mut applications: Vec<Application> = Vec::new();
    let mut roots = std::collections::BTreeSet::new();
    let mut application_configs: Vec<_> = options.applications.iter().collect();
    application_configs.sort_by_key(|a| a.root.len());
    for application in application_configs {
        indicator::validate_path(&application.root)?;
        ensure!(roots.insert(&application.root), "duplicate application root");
        let mut root = source.root.clone();
        for part in Path::new(&application.root).components() {
            root.push(part);
            ensure!(!fs::symlink_metadata(&root).context("application root not found")?.file_type().is_symlink(), "application root cannot contain symlinks");
        }
        ensure!(root.is_dir() && root.canonicalize()?.starts_with(source.root.canonicalize()?), "application root escapes repository or is not a directory");
        let (local, warnings) = indicator::collect(&root, &application.indicators)?;
        coverage.diagnostics.extend(warnings.into_iter().map(|reason| Diagnostic { path: application.root.clone(), reason }));
        let parent = applications.iter().filter(|a| application.root.strip_prefix(&a.root).is_some_and(|rest| rest.starts_with('/'))).max_by_key(|a| a.root.len());
        let mut values = parent.map_or_else(|| metadata.clone(), |a| a.metadata.clone()); values.extend(local);
        applications.push(Application { root: application.root.clone(), metadata: values });
    }
    applications.sort_by(|a, b| a.root.cmp(&b.root));
    let mut files = vec![];
    let mut walker = WalkBuilder::new(&source.root);
    walker.hidden(false).follow_links(false).require_git(false).git_global(false).git_exclude(false)
        .filter_entry(|entry| !entry.file_type().is_some_and(|t| t.is_dir()) || !matches!(entry.file_name().to_str().unwrap_or(""), ".git" | ".venv" | "venv" | "node_modules" | "target" | "vendor" | "dist" | "build" | ".idea" | "__pycache__"));
    for result in walker.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => { coverage.diagnostics.push(Diagnostic { path: ".".into(), reason: "directory traversal failed; some paths were not scanned".into() }); continue; }
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) { continue; }
        let candidate = (needs_code && (syntax::language(entry.path()).is_some() || manifest::supported(entry.path()) || entry.path().extension().is_some_and(|e| e == "ipynb")))
            || (selection.effective.contains("mcp_server") && formats::is_mcp_config(entry.path()))
            || (selection.effective.contains("skill") && formats::is_skill(entry.path()));
        if !candidate {
            if needs_code {
                if let Some(language) = unsupported_language(entry.path()) { *coverage.unsupported_languages.entry(language.into()).or_default() += 1; }
            }
            coverage.files_skipped += 1; continue;
        }
        ensure!(files.len() < 100_000, "scan exceeds 100,000 candidate files; scan smaller roots");
        files.push(entry.into_path());
    }
    files.sort();
    let modules = crate::modules::ModuleIndex::build(&source.root, &files, options.max_file_bytes);
    if modules.limited { coverage.diagnostics.push(Diagnostic { path: ".".into(), reason: "Python module graph budget exceeded; some cross-file references remain unresolved".into() }); }
    coverage.scope_notes.push("Python named re-exports and single-return factory wrappers resolve up to 8 hops; namespace, dynamic and ambiguous imports remain unresolved. Local source snapshots are recommended.".into());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(options.jobs).build()?;
    let results: Vec<_> = pool.install(|| files.par_iter().map(|path| (path, scan_file(&source.root, path, catalog, options.max_file_bytes, &modules))).collect());
    let mut assets = vec![]; let mut relationships = vec![];
    for (path, result) in results {
        let path = path.strip_prefix(&source.root).unwrap_or(path).to_string_lossy().replace('\\', "/");
        match result {
            Ok(mut result) => {
                coverage.files_scanned += 1;
                *coverage.by_language.entry(result.language).or_default() += 1;
                assets.append(&mut result.assets); relationships.append(&mut result.relationships);
                if let Some(reason) = result.diagnostic { coverage.diagnostics.push(Diagnostic { path, reason }); }
            }
            Err(error) => { coverage.files_skipped += 1; coverage.files_failed += 1; coverage.diagnostics.push(Diagnostic { path, reason: error.to_string() }); }
        }
    }
    let mut unique: BTreeMap<String, Asset> = BTreeMap::new();
    for asset in assets { unique.entry(asset.id.clone()).or_insert(asset); }
    let mut assets: Vec<_> = unique.into_values().collect();
    assets.retain(|asset| selection.effective.contains(&asset.kind));
    // Repository-scoped occurrence IDs prevent collisions in fleet exports. No runtime identity is claimed.
    let mut remap = BTreeMap::new();
    for asset in &mut assets {
        crate::packs::annotate(catalog, asset);
        let id = format!("pl:{}", digest(format!("{}\0{}", source.label, asset.id).as_bytes()));
        remap.insert(asset.id.clone(), id.clone()); asset.id = id;
        let path = &asset.evidence[0].path;
        let application = applications.iter().filter(|a| path.strip_prefix(&a.root).is_some_and(|rest| rest.starts_with('/'))).max_by_key(|a| a.root.len());
        asset.metadata = application.map_or_else(|| metadata.clone(), |a| a.metadata.clone());
        asset.application_root = application.map(|a| a.root.clone());
    }
    relationships.retain_mut(|r| {
        if let (Some(from), Some(to)) = (remap.get(&r.from), remap.get(&r.to)) { r.from = from.clone(); r.to = to.clone(); true } else { false }
    });
    assets.sort_by(|a, b| (&a.evidence[0].path, a.evidence[0].line, &a.kind, &a.name).cmp(&(&b.evidence[0].path, b.evidence[0].line, &b.kind, &b.name)));
    relationships.sort(); relationships.dedup();
    coverage.diagnostics.sort_by(|a,b| (&a.path, &a.reason).cmp(&(&b.path, &b.reason)));
    let mut counts = BTreeMap::new();
    for asset in &assets { *counts.entry(asset.kind.clone()).or_default() += 1; }
    artifact::finish_coverage(&mut coverage, &selection, catalog, &counts);
    let status = if coverage.incomplete() { "partial" } else { "complete_within_declared_scope" }.into();
    let config_sha256 = config_fingerprint(options, &selection);
    Ok(Report { schema_version: 1, scanner_version: env!("CARGO_PKG_VERSION").into(), source: source.label.clone(), revision: source.revision.clone(), catalog_version: catalog.version.clone(), catalog_sha256: catalog.fingerprint(), detection_packs: catalog.packs.clone(), metadata, assets, relationships, coverage, selection, applications, status, config_sha256 })
}

/// Preserve a failed batch member without embedding raw network errors or credential-bearing URLs.
pub fn failed(source: &str, revision: Option<&str>, catalog: &Catalog, options: &Options) -> Result<Report> {
    let selection = Selection::resolve(&options.selection, catalog)?;
    let mut coverage = Coverage::default();
    coverage.diagnostics.push(Diagnostic { path: ".".into(), reason: "repository acquisition or scan failed; no inventory conclusion is available".into() });
    artifact::finish_coverage(&mut coverage, &selection, catalog, &BTreeMap::new());
    for item in coverage.by_type.values_mut().filter(|c| c.status != "not_requested") { item.status = "failed".into(); }
    // Invalid URLs can contain embedded credentials; keep them out of machine reports.
    let source = if source.contains("://") && crate::source::github_coordinates(source).is_err() { "[invalid repository URL]" } else { source };
    let config_sha256 = config_fingerprint(options, &selection);
    Ok(Report { schema_version: 1, scanner_version: env!("CARGO_PKG_VERSION").into(), source: source.into(), revision: revision.map(str::to_owned), catalog_version: catalog.version.clone(), catalog_sha256: catalog.fingerprint(), detection_packs: catalog.packs.clone(), metadata: BTreeMap::new(), assets: vec![], relationships: vec![], coverage, selection, applications: vec![], status: "failed".into(), config_sha256 })
}

fn config_fingerprint(options: &Options, selection: &Selection) -> String {
    // Worker count and output format do not alter inventory semantics.
    digest(&serde_json::to_vec(&serde_json::json!({"selection":selection,"indicators":options.indicators,"applications":options.applications,"max_file_bytes":options.max_file_bytes})).expect("serializable configuration"))
}

fn syntax_language(language: &str) -> bool { matches!(language, "python" | "javascript" | "typescript" | "java" | "go" | "rust" | "csharp" | "ruby" | "php" | "kotlin") }
fn unsupported_language(path: &Path) -> Option<&'static str> {
    Some(match path.extension()?.to_str()? { "php" => "php", "swift" => "swift", "kt" | "kts" => "kotlin", "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" => "c_cpp", "r" | "R" => "r", "jl" => "julia", "scala" => "scala", "ex" | "exs" => "elixir", "ipynb" => "notebook", _ => return None })
}

fn scan_file(root: &Path, path: &Path, catalog: &Catalog, max_bytes: u64, modules: &crate::modules::ModuleIndex) -> Result<FileResult> {
    let metadata = fs::symlink_metadata(path).context("file metadata unavailable")?;
    ensure!(metadata.file_type().is_file(), "file changed type during scan");
    ensure!(metadata.len() <= max_bytes, "file exceeds configured size limit");
    let mut bytes = Vec::new();
    fs::File::open(path).context("cannot open file")?.take(max_bytes + 1).read_to_end(&mut bytes).context("cannot read file")?;
    ensure!(bytes.len() <= max_bytes as usize, "file exceeds configured size limit");
    let text = std::str::from_utf8(&bytes).context("file is not UTF-8")?;
    let relative = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
    if manifest::supported(path) {
        let assets = manifest::scan(&relative, text, catalog).map_err(|_| anyhow::anyhow!("manifest could not be parsed"))?;
        Ok(FileResult { assets, relationships: vec![], language: "manifest".into(), diagnostic: None })
    } else if formats::is_mcp_config(path) {
        let assets = formats::mcp_config(&relative, text).map_err(|_| anyhow::anyhow!("MCP configuration could not be parsed or validated"))?;
        Ok(FileResult { assets, relationships: vec![], language: "mcp_config".into(), diagnostic: None })
    } else if formats::is_skill(path) {
        let assets = formats::skill(&relative, text).map_err(|_| anyhow::anyhow!("skill frontmatter could not be parsed or validated"))?;
        Ok(FileResult { assets, relationships: vec![], language: "skill".into(), diagnostic: None })
    } else if path.extension().is_some_and(|e| e == "ipynb") {
        let (result, language) = crate::notebook::scan(&relative, text, catalog).map_err(|_| anyhow::anyhow!("notebook format/language is unsupported or invalid"))?;
        Ok(FileResult { assets: result.assets, relationships: result.relationships, language, diagnostic: Some(if result.parse_errors { "notebook syntax errors; cells are analyzed independently, execution order is unknown" } else { "notebook cells are analyzed independently; cross-cell bindings and execution order are unresolved" }.into()) })
    } else {
        let language = syntax::language(path).unwrap();
        let graph = (language == "python" && modules.unchanged(&relative, text)).then_some(modules);
        let result = syntax::scan_with_modules(&relative, text, language, catalog, graph)?;
        let diagnostic = if result.parse_errors { Some("syntax errors or traversal depth limit; findings may be incomplete".into()) }
            else if result.unresolved_modules || (language == "python" && graph.is_none()) { Some("local module resolution is incomplete, ambiguous, cyclic or source changed; no dynamic imports were executed".into()) }
            else { None };
        Ok(FileResult { assets: result.assets, relationships: result.relationships, language: language.into(), diagnostic })
    }
}
