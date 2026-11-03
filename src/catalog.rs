use anyhow::{Context, Result, ensure};
#[cfg(not(feature = "duckdb-catalog"))]
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, io::{Read, Write}, path::Path};
use crate::model::digest;

const EMBEDDED: &str = include_str!("../catalog/catalog.json");
const MAX_CATALOG: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub version: String,
    pub entries: Vec<Entry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<Seed>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<crate::packs::PackInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pack_rules: Vec<crate::packs::AppliedRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    pub source_sha256: String,
    pub source_url: String,
    pub rows: u64,
    pub rows_by_label: BTreeMap<String, u64>,
    pub indexed_api_references: u64,
    pub unmapped_callable_rows: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub homepage: String,
    pub packages: BTreeMap<String, Vec<String>>,
    pub imports: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub symbols: BTreeMap<String, String>,
    /// Exact receiver operation suffixes; never infer a call from a generic method name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub operations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbol_languages: BTreeMap<String, Vec<String>>,
    /// Declared Go package names when they differ from the import-path basename.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub module_names: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub wildcard_exports: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbol_subtypes: BTreeMap<String, String>,
    /// Upstream class/function identities are dependency evidence, not verified AI roles.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub api_references: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub configured_loops: BTreeMap<String, ConfiguredLoop>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rust_attributes: BTreeMap<String, RustAttribute>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub di_registrations: BTreeSet<String>,
    /// Exact reviewed IDs used for context-qualified local model-record references.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub model_identifiers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub model_arguments: BTreeMap<String, ModelArguments>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelArguments {
    pub position: Option<usize>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub require_provider_prefix: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredLoop {
    pub tools_argument: String,
    pub count_argument: String,
    pub stop_argument: String,
    pub stop_functions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustAttribute {
    pub target: String,
    #[serde(default)] pub trait_symbol: Option<String>,
    #[serde(default)] pub flag: Option<String>,
}

impl Catalog {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        crate::packs::load(path, &[], false)
    }
    pub(crate) fn load_base(path: Option<&Path>) -> Result<Self> {
        if let Some(path) = path {
            if path.extension().is_some_and(|e| e == "duckdb") {
                return Self::from_duckdb(path);
            }
            ensure!(fs::metadata(path)?.len() <= MAX_CATALOG, "catalog exceeds 8 MiB");
            Self::parse(&fs::read(path)?)
        } else {
            Self::parse(EMBEDDED.as_bytes())
        }
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let catalog: Self = serde_json::from_slice(bytes).context("invalid catalog JSON")?;
        catalog.validate()?;
        Ok(catalog)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 1, "unsupported catalog schema");
        ensure!(!self.version.is_empty() && self.version.len() < 128, "invalid catalog version");
        ensure!(!self.entries.is_empty(), "empty catalog");
        if let Some(seed) = &self.seed {
            ensure!(seed.source_sha256.len() == 64 && seed.source_sha256.bytes().all(|b| b.is_ascii_hexdigit()) && seed.source_url.starts_with("https://"), "invalid seed provenance");
        }
        let mut pack_ids = BTreeSet::new();
        for pack in &self.packs {
            ensure!(pack_ids.insert(&pack.id) && !pack.id.is_empty() && pack.sha256.len()==64 && pack.sha256.bytes().all(|b|b.is_ascii_hexdigit()), "invalid/duplicate applied pack identity");
        }
        for rule in &self.pack_rules {
            ensure!(self.packs.contains(&rule.origin.pack) && !rule.origin.rule_id.is_empty() && self.entries.iter().any(|e|e.id==rule.framework), "orphan pack rule provenance");
        }
        let mut ids = BTreeSet::new();
        let mut packages = BTreeSet::new();
        let mut imports = BTreeSet::new();
        let mut models = BTreeSet::new();
        for e in &self.entries {
            ensure!(!e.id.is_empty() && ids.insert(&e.id), "empty or duplicate catalog id: {}", e.id);
            ensure!(!e.name.is_empty() && e.homepage.starts_with("https://"), "invalid entry: {}", e.id);
            for (eco, names) in &e.packages {
                for name in names {
                    ensure!(!name.is_empty() && packages.insert((eco.clone(), normalize_package(eco, name))), "duplicate/empty package: {eco}:{name}");
                }
            }
            for (lang, names) in &e.imports {
                for name in names {
                    ensure!(!name.is_empty() && imports.insert((lang, name)), "duplicate/empty import: {lang}:{name}");
                }
            }
            for kind in e.symbols.values().chain(e.operations.values()) {
                ensure!(crate::artifact::TYPES.contains(&kind.as_str()), "unknown asset kind: {kind}");
            }
            for operation in e.operations.keys() {
                ensure!(!operation.is_empty() && operation.len() <= 256 && operation.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')), "invalid operation path");
            }
            for source in &e.sources { ensure!(source.starts_with("https://") && source.len() <= 2048, "catalog provenance must use HTTPS URLs"); }
            for (symbol, languages) in &e.symbol_languages {
                ensure!(e.symbols.contains_key(symbol) && !languages.is_empty() && languages.iter().all(|l| e.imports.contains_key(l)), "invalid symbol language restriction");
            }
            for (module, name) in &e.module_names {
                ensure!(e.imports.get("go").is_some_and(|paths| paths.contains(module)) && !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'), "invalid Go module name");
            }
            for (module, exports) in &e.wildcard_exports {
                ensure!(e.imports.get("rust").is_some_and(|paths| paths.iter().any(|p| prefix_matches(module, p))) && !exports.is_empty()
                    && exports.iter().all(|name| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')), "invalid wildcard export declaration");
            }
            for (symbol, subtype) in &e.symbol_subtypes { ensure!(e.symbols.contains_key(symbol) && !subtype.is_empty() && subtype.len() <= 80, "invalid symbol subtype"); }
            for (symbol, rule) in &e.configured_loops {
                ensure!(e.symbols.get(symbol).is_some_and(|k| k=="llm_call"), "configured loop requires an llm_call symbol");
                ensure!([&rule.tools_argument,&rule.count_argument,&rule.stop_argument].iter().all(|s| !s.is_empty() && s.len()<100), "invalid configured loop argument");
                ensure!(!rule.stop_functions.is_empty() && rule.stop_functions.iter().all(|s|!s.is_empty()), "missing stop function bindings");
            }
            for (symbol, rule) in &e.rust_attributes {
                ensure!(e.symbols.contains_key(symbol) && e.imports.contains_key("rust"), "attribute requires a Rust symbol");
                ensure!(matches!(rule.target.as_str(), "impl_method"|"trait_impl"|"impl_flag"), "unsupported Rust attribute target");
                ensure!(rule.target!="trait_impl" || rule.trait_symbol.as_ref().is_some_and(|s|!s.is_empty()), "attribute requires a trait binding");
                ensure!(rule.target!="impl_flag" || rule.flag.as_ref().is_some_and(|s|!s.is_empty()), "attribute requires a flag");
            }
            for symbol in &e.di_registrations { ensure!(e.symbols.contains_key(symbol) && e.imports.contains_key("csharp"), "DI registration requires a C# symbol"); }
            for (symbol, rule) in &e.model_arguments {
                ensure!(e.imports.get("python").is_some_and(|ps|ps.iter().any(|p|prefix_matches(symbol,p))) && rule.position.is_none_or(|p|p<32) && !rule.keywords.is_empty() && rule.keywords.iter().all(|k|!k.is_empty() && k.bytes().all(|b|b.is_ascii_alphanumeric() || b==b'_')), "invalid model argument selector");
            }
            for model in &e.model_identifiers {
                ensure!(!model.is_empty() && model.len()<=160 && model.bytes().all(|b|b.is_ascii_alphanumeric() || b"-_/.:".contains(&b)) && models.insert(model), "invalid or duplicate model identifier");
            }
            for symbol in &e.api_references {
                ensure!(symbol.len() <= 1024 && e.imports.get("python").is_some_and(|prefixes| prefixes.iter().any(|p| prefix_matches(symbol,p))), "unmapped API reference");
            }
        }
        Ok(())
    }
    pub fn fingerprint(&self) -> String {
        digest(&serde_json::to_vec(self).expect("serializable catalog"))
    }
    pub fn package(&self, ecosystem: &str, name: &str) -> Option<&Entry> {
        let name = normalize_package(ecosystem, name);
        self.entries.iter().find(|e| e.packages.get(ecosystem).is_some_and(|ps| ps.iter().any(|p| normalize_package(ecosystem, p) == name)))
    }
    pub fn import(&self, language: &str, symbol: &str) -> Option<&Entry> {
        self.entries.iter().flat_map(|e| e.imports.get(language).into_iter().flatten().map(move |p| (e, p)))
            .filter(|(_, p)| prefix_matches(symbol, p)).max_by_key(|(_, p)| p.len()).map(|(e, _)| e)
    }

    pub fn symbol_kind<'a>(&self, entry: &'a Entry, language: &str, symbol: &str) -> Option<&'a str> {
        if entry.symbol_languages.get(symbol).is_some_and(|languages| !languages.iter().any(|l| l == language)) { return None; }
        entry.symbols.get(symbol).map(String::as_str).or_else(|| {
            (language == "python" && entry.api_references.contains(symbol)).then_some("dependency")
        })
    }

    #[cfg(feature = "duckdb-catalog")]
    pub fn from_duckdb(path: &Path) -> Result<Self> {
        use duckdb::{AccessMode, Config, Connection};
        let config = Config::default().access_mode(AccessMode::ReadOnly)?;
        let conn = Connection::open_with_flags(path, config)?;
        conn.execute_batch("SET enable_external_access=false; SET autoinstall_known_extensions=false; SET autoload_known_extensions=false;")?;
        let json: String = conn.query_row("SELECT document FROM pilotlight_catalog WHERE id=1", [], |r| r.get(0))?;
        ensure!(json.len() <= MAX_CATALOG as usize, "catalog exceeds 8 MiB");
        Self::parse(json.as_bytes())
    }
    #[cfg(not(feature = "duckdb-catalog"))]
    pub fn from_duckdb(_path: &Path) -> Result<Self> {
        bail!("DuckDB support requires a build with --features duckdb-catalog; JSON catalogs work in every build")
    }
    #[cfg(feature = "duckdb-catalog")]
    pub fn export_duckdb(&self, path: &Path) -> Result<()> {
        use duckdb::Connection;
        ensure!(!path.exists(), "output already exists");
        let conn = Connection::open(path)?;
        conn.execute_batch("CREATE TABLE pilotlight_catalog (id INTEGER PRIMARY KEY, document VARCHAR); CREATE TABLE framework (id VARCHAR, name VARCHAR, category VARCHAR); CREATE TABLE package (framework_id VARCHAR, ecosystem VARCHAR, name VARCHAR); CREATE TABLE symbol (framework_id VARCHAR, qualified_name VARCHAR, kind VARCHAR); CREATE TABLE operation (framework_id VARCHAR, operation VARCHAR, kind VARCHAR); CREATE TABLE provenance (framework_id VARCHAR, source_url VARCHAR, reviewed_at VARCHAR);")?;
        conn.execute("INSERT INTO pilotlight_catalog VALUES (1, ?)", [serde_json::to_string(self)?])?;
        for e in &self.entries {
            conn.execute("INSERT INTO framework VALUES (?, ?, ?)", [&e.id, &e.name, &e.category])?;
            for (eco, names) in &e.packages {
                for name in names { conn.execute("INSERT INTO package VALUES (?, ?, ?)", [&e.id, eco, name])?; }
            }
            for (symbol, kind) in &e.symbols { conn.execute("INSERT INTO symbol VALUES (?, ?, ?)", [&e.id, symbol, kind])?; }
            for (operation, kind) in &e.operations { conn.execute("INSERT INTO operation VALUES (?, ?, ?)", [&e.id, operation, kind])?; }
            for source in &e.sources { conn.execute("INSERT INTO provenance VALUES (?, ?, ?)", [e.id.as_str(), source.as_str(), e.reviewed_at.as_deref().unwrap_or("")])?; }
        }
        Ok(())
    }
    #[cfg(not(feature = "duckdb-catalog"))]
    pub fn export_duckdb(&self, _path: &Path) -> Result<()> {
        bail!("rebuild with --features duckdb-catalog to export DuckDB")
    }
}

pub fn prefix_matches(symbol: &str, prefix: &str) -> bool {
    symbol == prefix || symbol.strip_prefix(prefix).is_some_and(|s| s.starts_with('.') || s.starts_with('/') || s.starts_with("::") || s.starts_with('\\'))
}

pub fn normalize_package(eco: &str, name: &str) -> String {
    match eco {
        "pypi" => {
            let mut normalized = String::new();
            for ch in name.chars() {
                if matches!(ch, '_' | '.' | '-') {
                    if !normalized.ends_with('-') { normalized.push('-'); }
                } else { normalized.push(ch.to_ascii_lowercase()); }
            }
            normalized
        }
        "nuget" => name.to_ascii_lowercase(),
        _ => name.to_owned(),
    }
}

/// A caller-supplied digest pins content; authenticity depends on obtaining it from a trusted channel.
pub fn update(url: &str, sha256: &str, destination: &Path) -> Result<()> {
    ensure!(sha256.len() == 64 && sha256.bytes().all(|b| b.is_ascii_hexdigit()), "expected a SHA-256 digest");
    let url = reqwest::Url::parse(url)?;
    ensure!(url.scheme() == "https" && url.username().is_empty() && url.password().is_none(), "catalog URL must be HTTPS without credentials");
    let client = crate::source::http_client()?;
    let mut bytes = Vec::new();
    // Signed download URLs can carry credentials in their query string.
    client.get(url).send().map_err(reqwest::Error::without_url)?
        .error_for_status().map_err(reqwest::Error::without_url)?
        .take(MAX_CATALOG + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= MAX_CATALOG as usize, "catalog exceeds 8 MiB");
    ensure!(digest(&bytes) == sha256.to_ascii_lowercase(), "catalog checksum mismatch");
    Catalog::parse(&bytes)?;
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(&bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(destination).context("could not install catalog atomically")?;
    Ok(())
}
