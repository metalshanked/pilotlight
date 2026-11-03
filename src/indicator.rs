use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::{Component, Path}};
use yaml_rust2::Yaml;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Indicator {
    pub name: String,
    pub path: String,
    /// Dot-separated object keys, e.g. application.id; arrays are not traversed.
    pub key: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub indicators: Vec<Indicator>,
    #[serde(default)]
    pub scan: crate::artifact::ScanConfig,
    #[serde(default)]
    pub applications: Vec<ApplicationConfig>,
    #[serde(default)]
    pub packs: PackConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackConfig {
    #[serde(default)] pub paths: Vec<std::path::PathBuf>,
    #[serde(default)] pub disable_defaults: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationConfig {
    pub root: String,
    pub indicators: Vec<Indicator>,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        // Only explicitly selected operator config is loaded, never config from a scanned repo.
        let mut config = if let Some(path) = path {
            ensure!(fs::metadata(path)?.len() <= 1024 * 1024, "config exceeds 1 MiB");
            // TOML errors can quote the original line, including operator secrets.
            toml::from_str::<Self>(&fs::read_to_string(path)?)
                .map_err(|_| anyhow::anyhow!("invalid Pilot Light config; check TOML syntax and supported fields"))?
        } else { Self::default() };
        if let Some(path) = path {
            let parent = path.parent().unwrap_or(Path::new("."));
            for pack in &mut config.packs.paths { if pack.is_relative() { *pack = parent.join(&*pack); } }
        }
        validate_rules(&config.indicators)?;
        let mut roots = std::collections::BTreeSet::new();
        for application in &config.applications {
            validate_path(&application.root)?;
            ensure!(roots.insert(&application.root), "duplicate application root");
            validate_rules(&application.indicators)?;
        }
        Ok(config)
    }
}

pub fn validate_rules(rules: &[Indicator]) -> Result<()> {
        let mut names = std::collections::BTreeSet::new();
        for rule in rules {
            ensure!(!rule.name.is_empty() && names.insert(&rule.name), "indicator names must be unique and nonempty");
            ensure!(!rule.key.is_empty(), "indicator key is empty");
            validate_path(&rule.path)?;
        }
        Ok(())
}

pub fn validate_path(path: &str) -> Result<()> {
    ensure!(!path.is_empty() && !path.contains([':', '\\']) && path.split('/').all(|p| !matches!(p, "" | "." | "..")) && Path::new(path).components().all(|p| matches!(p, Component::Normal(_))), "indicator path must be relative to the repository, using / and no empty, . or .. components");
    Ok(())
}

pub fn collect(root: &Path, rules: &[Indicator]) -> Result<(BTreeMap<String, String>, Vec<String>)> {
    validate_rules(rules)?;
    let mut values = BTreeMap::new(); let mut warnings = Vec::new();
    for rule in rules {
        match read(root, rule) {
            Ok(value) => { values.insert(rule.name.clone(), value); }
            Err(error) if rule.required => bail!("required indicator '{}' ({}): {error:#}", rule.name, rule.path),
            Err(error) => warnings.push(format!("indicator '{}' ({}): {error:#}", rule.name, rule.path)),
        }
    }
    Ok((values, warnings))
}

fn read(root: &Path, rule: &Indicator) -> Result<String> {
    validate_path(&rule.path)?;
    let mut path = root.to_path_buf();
    for part in Path::new(&rule.path).components() {
        path.push(part);
        let meta = fs::symlink_metadata(&path).context("indicator file or parent not found")?;
        ensure!(!meta.file_type().is_symlink(), "indicator path cannot contain symlinks");
    }
    ensure!(path.canonicalize()?.starts_with(root.canonicalize()?), "indicator escapes repository");
    ensure!(fs::metadata(&path)?.len() <= 1024 * 1024, "indicator file exceeds 1 MiB");
    let text = fs::read_to_string(&path).context("indicator is not UTF-8")?;
    let keys: Vec<_> = rule.key.split('.').collect();
    let value = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "yaml" | "yml" => {
            let documents = crate::formats::yaml_documents(&text)?;
            ensure!(documents.len() == 1, "expected one YAML document");
            let mut node = &documents[0];
            for key in keys { node = &node[key]; }
            match node {
                Yaml::String(s) => s.clone(), Yaml::Integer(n) => n.to_string(), Yaml::Boolean(b) => b.to_string(),
                Yaml::Real(s) => s.clone(), _ => bail!("key '{}' missing or not a scalar", rule.key),
            }
        }
        "json" => {
            let json: serde_json::Value = serde_json::from_str(&text).map_err(|_| anyhow::anyhow!("invalid JSON"))?;
            let mut node = &json;
            for key in keys { node = node.get(key).context("indicator key not found")?; }
            match node { serde_json::Value::String(s) => s.clone(), serde_json::Value::Number(n) => n.to_string(), serde_json::Value::Bool(b) => b.to_string(), _ => bail!("indicator must be a scalar") }
        }
        "toml" => {
            let toml: toml::Value = toml::from_str(&text).map_err(|_| anyhow::anyhow!("invalid TOML"))?;
            let mut node = &toml;
            for key in keys { node = node.get(key).context("indicator key not found")?; }
            match node { toml::Value::String(s) => s.clone(), toml::Value::Integer(n) => n.to_string(), toml::Value::Boolean(b) => b.to_string(), _ => bail!("indicator must be a string, integer or boolean") }
        }
        _ => bail!("indicators support YAML, JSON and TOML"),
    };
    ensure!(value.len() <= 1024, "indicator value exceeds 1024 bytes");
    Ok(value)
}
