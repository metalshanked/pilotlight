use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use pilotlight::{artifact::{self, Selection}, catalog::{self, Catalog}, indicator::{Config, Indicator}, output::{self, Format}, scan::{self, Options}, source::Source};
use std::{fs, io::{self, Write}, path::PathBuf};

#[derive(Parser)]
#[command(name="pilotlight", version, about="Find AI libraries, agents, models and MCP servers in source repositories.", after_help="Examples:\n  pilotlight scan .\n  pilotlight scan https://github.com/OWNER/REPO --format csv --output inventory.csv\n  pilotlight scan . --config pilotlight.toml\n\nOffline local scans require no accounts, LLMs, Git, or database server.")]
struct Cli { #[command(subcommand)] command: Command }

#[derive(Subcommand)]
enum Command {
    /// Run a pinned development corpus; preserve per-repository timing, hashes and failures.
    Benchmark {
        #[arg(long)] corpus_file: PathBuf,
        #[arg(long)] output_dir: PathBuf,
        #[arg(long)] catalog: Option<PathBuf>,
        #[command(flatten)] packs: PackArgs,
        #[arg(long, default_value_t=4)] jobs: usize,
        #[arg(long, default_value_t=2_097_152)] max_file_bytes: u64,
        #[arg(long, default_value_t=104_857_600)] max_download_bytes: u64,
        #[arg(long, default_value_t=536_870_912)] max_expanded_bytes: u64,
    },
    /// Scan a local directory or GitHub.com repository (without executing its code).
    Scan {
        #[arg(default_value=".")] source: String,
        /// Scan a newline-separated list of local paths or GitHub URLs; replaces SOURCE.
        #[arg(long, conflicts_with_all=["reference", "corpus_file"])] repos_file: Option<PathBuf>,
        /// Scan a pinned corpus manifest, respecting each repository's exact commit.
        #[arg(long, conflicts_with="reference")] corpus_file: Option<PathBuf>,
        /// Preserve completed reports and explicit failed records in a batch (exit 1 if any fail).
        #[arg(long)] continue_on_error: bool,
        /// Branch, tag, or commit for a GitHub URL; default is the default branch.
        #[arg(long="ref")] reference: Option<String>,
        #[arg(long, value_enum, default_value="table")] format: Format,
        #[arg(short, long)] output: Option<PathBuf>,
        /// Operator-owned TOML file with repository indicators; never auto-loaded from targets.
        #[arg(long)] config: Option<PathBuf>,
        /// External JSON or DuckDB catalog; default uses the bundled snapshot.
        #[arg(long)] catalog: Option<PathBuf>,
        #[command(flatten)] packs: PackArgs,
        #[arg(short, long, default_value_t=4)] jobs: usize,
        #[arg(long, default_value_t=2_097_152)] max_file_bytes: u64,
        /// Maximum compressed GitHub archive size (up to 1 GiB).
        #[arg(long, default_value_t=104_857_600)] max_download_bytes: u64,
        /// Maximum expanded GitHub archive size (up to 4 GiB).
        #[arg(long, default_value_t=536_870_912)] max_expanded_bytes: u64,
        /// Exit 2 if any files or optional indicators could not be analyzed.
        #[arg(long)] strict: bool,
        /// Canonical artifact types or groups: agents, mcp, models, skills, all.
        #[arg(long, value_delimiter=',')] types: Option<Vec<String>>,
        #[arg(long, value_delimiter=',')] exclude_types: Option<Vec<String>>,
        /// Exit 2 for partial or unsupported requested coverage, even with zero findings.
        #[arg(long, num_args=0..=1, require_equals=true, default_missing_value="true")] strict_coverage: Option<bool>,
        /// Coverage sidecar; CSV file output defaults to OUTPUT.coverage.json.
        #[arg(long)] coverage_output: Option<PathBuf>,
        /// Optional CSV of relationships between selected findings.
        #[arg(long)] relationships_output: Option<PathBuf>,
        /// Quick single indicator: repository-relative YAML/JSON/TOML path.
        #[arg(long, requires="indicator_key")] indicator_path: Option<String>,
        #[arg(long, requires="indicator_path")] indicator_key: Option<String>,
        #[arg(long, default_value="application_id")] indicator_name: String,
        #[arg(long, requires="indicator_path")] indicator_required: bool,
    },
    /// Inspect, validate, test or install declarative detection packs.
    Packs { #[command(subcommand)] command: PackCommand },
    /// Inspect, validate, update or export the framework catalog.
    Catalog { #[command(subcommand)] command: CatalogCommand },
    /// List canonical artifact types and capabilities of the active catalog.
    Types { #[command(subcommand)] command: TypesCommand },
}

#[derive(clap::Args)]
struct PackArgs {
    /// Add an operator-owned JSON detection pack; repeat for multiple packs.
    #[arg(long="pack")] paths: Vec<PathBuf>,
    /// Disable bundled agent/MCP packs; never auto-load packs from target code.
    #[arg(long)] no_default_packs: bool,
}
#[derive(Subcommand)]
enum PackCommand {
    List,
    Inspect { path: PathBuf },
    Validate { path: PathBuf },
    Test { path: PathBuf },
    Install { path: PathBuf, #[arg(long)] sha256: String, #[arg(short,long)] output: PathBuf },
}
#[derive(Subcommand)]
enum TypesCommand {
    List { #[arg(long)] catalog: Option<PathBuf>,
        #[command(flatten)] packs: PackArgs, #[arg(long, value_enum, default_value="table")] format: Format },
}

#[derive(Subcommand)]
enum CatalogCommand {
    /// Preserve a Cisco component_catalog database and add an independent Pilot Light overlay.
    ImportCisco {
        #[arg(long)] input: PathBuf,
        #[arg(long)] overlay: PathBuf,
        #[arg(long)] sha256: String,
        #[arg(long)] source_url: String,
        #[arg(short, long)] output: PathBuf,
    },
    List { #[arg(long)] catalog: Option<PathBuf>, #[command(flatten)] packs: PackArgs },
    Validate { path: PathBuf },
    /// Install an explicitly pinned JSON catalog. No background network access.
    Update { #[arg(long)] url: String, #[arg(long)] sha256: String, #[arg(short, long)] output: PathBuf },
    /// Export portable JSON or a DuckDB database (requires duckdb-catalog build feature).
    Export { #[arg(short, long)] output: PathBuf, #[arg(long)] catalog: Option<PathBuf>, #[command(flatten)] packs: PackArgs },
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    match cli.command {
        Command::Benchmark {corpus_file,output_dir,catalog,packs,jobs,max_file_bytes,max_download_bytes,max_expanded_bytes} => {
            let catalog=pilotlight::packs::load(catalog.as_deref(), &packs.paths, packs.no_default_packs)?;
            let options=Options {jobs,max_file_bytes,..Default::default()};
            pilotlight::benchmark::run(&corpus_file,&output_dir,&catalog,&options,pilotlight::source::AcquisitionLimits {max_download_bytes,max_expanded_bytes})
        }
        Command::Scan { source, repos_file, corpus_file, continue_on_error, reference, format, output, config, catalog, packs, jobs, max_file_bytes, max_download_bytes, max_expanded_bytes, strict, types, exclude_types, strict_coverage, coverage_output, relationships_output, indicator_path, indicator_key, indicator_name, indicator_required } => {
            let limits=pilotlight::source::AcquisitionLimits {max_download_bytes,max_expanded_bytes};limits.validate()?;
            let mut config = Config::load(config.as_deref())?;
            if let Some(types) = types { config.scan.types = Some(types); }
            if let Some(exclude_types) = exclude_types { config.scan.exclude_types = exclude_types; }
            if let Some(strict_coverage) = strict_coverage { config.scan.strict_coverage = strict_coverage; }
            let strict_coverage = config.scan.strict_coverage;
            if let Some(path) = indicator_path {
                ensure!(!config.indicators.iter().any(|r| r.name == indicator_name), "duplicate indicator name");
                config.indicators.push(Indicator { name: indicator_name, path, key: indicator_key.unwrap(), required: indicator_required });
            }
            let mut pack_paths = packs.paths;
            pack_paths.extend(config.packs.paths);
            let catalog = pilotlight::packs::load(catalog.as_deref(), &pack_paths, packs.no_default_packs || config.packs.disable_defaults)?;
            Selection::resolve(&config.scan, &catalog)?;
            let coverage_output = coverage_output.or_else(|| output.as_ref().filter(|_| matches!(format, Format::Csv)).map(|path| {
                let mut name = path.as_os_str().to_owned(); name.push(".coverage.json"); PathBuf::from(name)
            }));
            ensure!(!matches!(format, Format::Csv) || output.is_some() || coverage_output.is_some(), "CSV on stdout requires --coverage-output PATH so zero findings and partial scans remain auditable");
            validate_outputs(&[output.as_deref(), coverage_output.as_deref(), relationships_output.as_deref()])?;
            let inputs = if let Some(path) = repos_file {
                ensure!(source == ".", "use SOURCE or --repos-file, not both");
                ensure!(fs::metadata(&path)?.len() <= 1024 * 1024, "repository list exceeds 1 MiB");
                let lines: Vec<String> = fs::read_to_string(path)?.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_owned).collect();
                ensure!(!lines.is_empty() && lines.len() <= 1000, "repository list must contain 1–1000 entries");
                lines.into_iter().map(|url| (url, None)).collect::<Vec<_>>()
            } else if let Some(path) = corpus_file {
                ensure!(source == ".", "use SOURCE or --corpus-file, not both");
                ensure!(fs::metadata(&path)?.len() <= 2 * 1024 * 1024, "corpus manifest exceeds 2 MiB");
                let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?).context("invalid corpus JSON")?;
                ensure!(value["schema_version"] == 1, "unsupported corpus schema");
                let rows = value["repositories"].as_array().context("corpus repositories missing")?;
                ensure!(!rows.is_empty() && rows.len() <= 1000, "corpus must contain 1–1000 repositories");
                let mut entries = vec![];
                let mut seen = std::collections::BTreeSet::new();
                for row in rows {
                    let url = row["repository"].as_str().context("corpus repository missing")?;
                    let coordinate = pilotlight::source::github_coordinates(url)?;
                    let sha = row["commit"].as_str().context("corpus commit missing")?;
                    ensure!(sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()), "corpus requires full 40-character commits");
                    ensure!(seen.insert((coordinate.0.to_lowercase(), coordinate.1.to_lowercase(), sha.to_lowercase())), "duplicate pinned corpus target");
                    entries.push((url.to_owned(), Some(sha.to_owned())));
                }
                entries
            } else { vec![(source, reference)] };
            ensure!(inputs.len() == 1 || !matches!(format, Format::Cyclonedx), "use JSON or CSV for multiple repositories");
            let options = Options { max_file_bytes, jobs, indicators: config.indicators, selection: config.scan, applications: config.applications };
            let mut reports = vec![];
            let mut failures = 0;
            for (index, (input, reference)) in inputs.iter().enumerate() {
                if inputs.len() > 1 { eprintln!("Scanning repository {}/{}", index + 1, inputs.len()); }
                let result = Source::open_with_limits(input, reference.as_deref(),limits).and_then(|source| scan::scan(&source, &catalog, &options));
                match result {
                    Ok(report) => reports.push(report),
                    Err(_) if continue_on_error => {
                        failures += 1;
                        eprintln!("Repository {} failed; preserving an explicit failure record and continuing.", index + 1);
                        reports.push(scan::failed(input, reference.as_deref(), &catalog, &options)?);
                    }
                    Err(error) => return Err(error).with_context(|| format!("could not scan repository {}", index + 1)),
                }
            }
            let diagnostics: usize = reports.iter().map(|r| r.coverage.diagnostics.len()).sum();
            let findings: usize = reports.iter().map(|r| r.assets.len()).sum();
            let mut primary = vec![];
            output::write_many(&reports, format, &mut primary)?;
            let mut files = vec![];
            if let Some(path) = coverage_output {
                let mut bytes = vec![]; output::coverage(&reports, &primary, &mut bytes)?; files.push((path, bytes));
            }
            if let Some(path) = relationships_output {
                let mut bytes = vec![]; output::relationships(&reports, &mut bytes)?; files.push((path, bytes));
            }
            if let Some(path) = output {
                files.push((path.clone(), primary));
                save_outputs(files)?;
                eprintln!("Saved {findings} findings to {} ({diagnostics} diagnostics)", path.display());
            } else { save_outputs(files)?; io::stdout().lock().write_all(&primary)?; }
            if !matches!(format, Format::Table) && diagnostics > 0 { eprintln!("Warning: scan has {diagnostics} diagnostics; use JSON for details."); }
            let incomplete = reports.iter().any(|r| r.coverage.incomplete());
            if incomplete { eprintln!("Coverage is partial or unsupported for some requested types; inspect JSON or the coverage sidecar."); }
            Ok(if failures > 0 { 1 } else if (strict && diagnostics > 0) || (strict_coverage && incomplete) { 2 } else { 0 })
        }
        Command::Types { command: TypesCommand::List { catalog, packs, format } } => {
            ensure!(matches!(format, Format::Table | Format::Json), "types list supports table or json");
            let catalog = pilotlight::packs::load(catalog.as_deref(), &packs.paths, packs.no_default_packs)?;
            let types: Vec<_> = artifact::TYPES.iter().map(|kind| serde_json::json!({"type":kind,"capabilities":artifact::capabilities(&catalog, kind)})).collect();
            if matches!(format, Format::Json) { println!("{}", serde_json::to_string_pretty(&serde_json::json!({"catalog_version":catalog.version,"types":types}))?); }
            else {
                println!("{:<22} CAPABILITIES (empty = unsupported)", "TYPE");
                for kind in artifact::TYPES { println!("{kind:<22} {}", artifact::capabilities(&catalog, kind).keys().cloned().collect::<Vec<_>>().join(", ")); }
            }
            Ok(0)
        }
        Command::Packs { command } => {
            use pilotlight::packs;
            match command {
                PackCommand::List => {
                    let rows = packs::builtins()?.into_iter().map(|(p,i)|serde_json::json!({"pack":i,"description":p.description,"rules":p.rules.len(),"fixtures":p.fixtures.len(),"requires":p.requires})).collect::<Vec<_>>();
                    println!("{}",serde_json::to_string_pretty(&rows)?);
                }
                PackCommand::Inspect {path} => { let (p,i)=packs::read(&path)?; println!("{}",serde_json::to_string_pretty(&serde_json::json!({"pack":p,"identity":i}))?); }
                PackCommand::Validate {path} => { let (p,i)=packs::read(&path)?; println!("Valid pack {} {}: {} rules; SHA256 {}",p.id,p.version,p.rules.len(),i.sha256); }
                PackCommand::Test {path} => {
                    let (p,i)=packs::read(&path)?; let results=packs::test(&p,i)?;
                    println!("{}",serde_json::to_string_pretty(&results)?);
                    return Ok(if results.iter().all(|r|r.passed) {0} else {1});
                }
                PackCommand::Install {path,sha256,output} => {packs::install(&path,&sha256,&output)?;println!("Installed pack at {}",output.display());}
            } Ok(0)
        }
        Command::Catalog { command } => {
            match command {
                CatalogCommand::ImportCisco {input,overlay,sha256,source_url,output} => {
                    let overlay = Catalog::load(Some(&overlay))?;
                    let merged = pilotlight::cisco::import(&input,&output,&sha256,&source_url,&overlay)?;
                    println!("Imported {} raw rows; {} exact API references; {} independent catalog entries", merged.seed.as_ref().unwrap().rows, merged.seed.as_ref().unwrap().indexed_api_references, merged.entries.len());
                }
                CatalogCommand::List { catalog, packs } => {
                    let catalog = pilotlight::packs::load(catalog.as_deref(), &packs.paths, packs.no_default_packs)?;
                    println!("Catalog {}: {} frameworks", catalog.version, catalog.entries.len());
                    for entry in &catalog.entries { println!("{:<22} {:<18} {}", entry.id, entry.category, entry.name); }
                }
                CatalogCommand::Validate { path } => { let catalog = Catalog::load(Some(&path))?; println!("Valid catalog {}: {} entries", catalog.version, catalog.entries.len()); }
                CatalogCommand::Update { url, sha256, output } => { catalog::update(&url, &sha256, &output)?; println!("Installed catalog at {}", output.display()); }
                CatalogCommand::Export { output, catalog, packs } => {
                    let catalog = pilotlight::packs::load(catalog.as_deref(), &packs.paths, packs.no_default_packs)?;
                    if output.extension().is_some_and(|x| x == "duckdb") { catalog.export_duckdb(&output)?; }
                    else { ensure!(!output.exists(), "output already exists"); fs::write(output, serde_json::to_vec_pretty(&catalog)?)?; }
                }
            }
            Ok(0)
        }
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => { eprintln!("pilotlight: {error:#}"); std::process::exit(1); }
    }
}

fn validate_outputs(paths: &[Option<&std::path::Path>]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for path in paths.iter().flatten() {
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
        let resolved = if path.exists() { ensure!(path.is_file(), "output path is not a file"); path.canonicalize()? }
            else { parent.canonicalize().context("output directory does not exist")?.join(path.file_name().context("output filename missing")?) };
        let key = resolved.to_string_lossy().into_owned();
        let key = if cfg!(windows) { key.to_lowercase() } else { key };
        ensure!(seen.insert(key), "report and sidecar paths must be distinct");
    }
    Ok(())
}

fn save_outputs(files: Vec<(PathBuf, Vec<u8>)>) -> Result<()> {
    // Stage every output before replacing any destination. Each file is atomically replaced;
    // coverage carries the primary hash to detect an interrupted multi-file publication.
    let mut staged = vec![];
    for (path, bytes) in files {
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent).context("output directory not writable")?;
        file.write_all(&bytes)?; file.flush()?; file.as_file().sync_all()?;
        staged.push((path, file));
    }
    for (path, file) in staged { file.persist(path).context("could not save output bundle; verify coverage primary_sha256 before consuming existing files")?; }
    Ok(())
}
