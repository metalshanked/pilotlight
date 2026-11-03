//! Lockfile facts are resolved declarations, never proof of installation or execution.
use anyhow::{Context, Result, ensure};
use crate::{catalog::Catalog, formats, model::{Asset, Evidence, PackageIdentity}};
use serde_json::Value;
use std::{collections::{BTreeMap, BTreeSet}, path::Path};

pub fn supported(path: &Path) -> bool {
    matches!(path.file_name().and_then(|p| p.to_str()), Some("package-lock.json" | "Cargo.lock" | "uv.lock" | "poetry.lock" | "composer.lock" | "packages.lock.json" | "pnpm-lock.yaml"))
}

pub fn scan(path: &str, text: &str, catalog: &Catalog) -> Result<Vec<Asset>> {
    let name = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("");
    let mut packages = BTreeMap::<(String, String, String), BTreeSet<String>>::new();
    let mut add = |eco: &str, name: &str, version: &str, scope: &str| {
        // Never export URL-like dependency versions that could carry credentials.
        let version = if version.len() <= 128 && !version.is_empty() && version.bytes().all(|b| b.is_ascii_alphanumeric() || b".-+_".contains(&b)) { version } else { "" };
        packages.entry((eco.into(), name.into(), version.into())).or_default().insert(scope.into());
    };
    match name {
        "package-lock.json" => {
            let root: Value = serde_json::from_str(text).context("invalid npm lockfile")?;
            match root["lockfileVersion"].as_u64() {
                Some(1) => {
                    fn walk(value: &Value, depth: usize, add: &mut impl FnMut(&str, &str, &str, &str)) -> Result<()> {
                        ensure!(depth <= 64, "npm dependency nesting exceeds limit");
                        if let Some(deps) = value.get("dependencies") {
                            for (name, entry) in deps.as_object().context("npm dependencies must be an object")? {
                                ensure!(entry.is_object(), "npm dependency must be an object");
                                let version = entry["version"].as_str().unwrap_or("");
                                let (name, version) = version.strip_prefix("npm:").and_then(|v| v.rsplit_once('@')).unwrap_or((name, version));
                                add("npm", name, version, if entry["dev"] == true { "development" } else { "unspecified" });
                                walk(entry, depth + 1, add)?;
                            }
                        }
                        Ok(())
                    }
                    walk(&root, 0, &mut add)?;
                }
                Some(2 | 3) => {
                    for (location, entry) in root["packages"].as_object().context("npm lockfile packages missing")? {
                        ensure!(entry.is_object(), "npm package must be an object");
                        let Some((_, name)) = location.rsplit_once("node_modules/") else { continue; };
                        if entry["link"] == true { continue; }
                        add("npm", entry["name"].as_str().unwrap_or(name), entry["version"].as_str().unwrap_or(""), if entry["dev"] == true { "development" } else { "unspecified" });
                    }
                }
                _ => anyhow::bail!("unsupported npm lockfile version"),
            }
        }
        "Cargo.lock" | "uv.lock" | "poetry.lock" => {
            let root: toml::Value = toml::from_str(text).context("invalid TOML lockfile")?;
            let ecosystem = if name == "Cargo.lock" { "cargo" } else { "pypi" };
            if name == "Cargo.lock" { ensure!(root.get("version").and_then(toml::Value::as_integer).is_none_or(|v| (1..=4).contains(&v)), "unsupported Cargo lockfile version"); }
            if name == "uv.lock" { ensure!(root.get("version").and_then(toml::Value::as_integer) == Some(1), "unsupported uv lockfile version"); }
            if name == "poetry.lock" {
                let version = root.get("metadata").and_then(|m| m.get("lock-version")).and_then(toml::Value::as_str);
                ensure!(matches!(version, Some("1.0" | "1.1" | "2.0" | "2.1")), "unsupported Poetry lockfile version");
            }
            for entry in root.get("package").and_then(toml::Value::as_array).context("lockfile package array missing")? {
                let package = entry.get("name").and_then(toml::Value::as_str).context("lockfile package name missing")?;
                let version = entry.get("version").and_then(toml::Value::as_str).unwrap_or("");
                add(ecosystem, package, version, "unspecified");
            }
        }
        "composer.lock" => {
            let root: Value = serde_json::from_str(text)?;
            ensure!(root.get("packages").is_some(), "Composer packages missing");
            for (section, scope) in [("packages", "production"), ("packages-dev", "development")] {
                if let Some(entries) = root.get(section) {
                    for entry in entries.as_array().context("Composer packages must be an array")? {
                        add("composer", entry["name"].as_str().context("Composer name missing")?, entry["version"].as_str().unwrap_or(""), scope);
                    }
                }
            }
        }
        "packages.lock.json" => {
            let root: Value = serde_json::from_str(text)?;
            ensure!(matches!(root["version"].as_u64(), Some(1 | 2)), "unsupported NuGet lockfile version");
            for target in root["dependencies"].as_object().context("NuGet dependencies missing")?.values() {
                for (name, entry) in target.as_object().context("NuGet target dependencies must be an object")? {
                    if entry["type"] == "Project" { continue; }
                    add("nuget", name, entry["resolved"].as_str().unwrap_or(""), "unspecified");
                }
            }
        }
        "pnpm-lock.yaml" => {
            let docs = formats::yaml_documents_with_budget(text, 250_000)?;
            ensure!(docs.len() == 1, "expected one pnpm document");
            let root = &docs[0];
            let version = root["lockfileVersion"].as_str().map(str::to_owned)
                .or_else(|| root["lockfileVersion"].as_i64().map(|n| n.to_string()))
                .or_else(|| if let yaml_rust2::Yaml::Real(v) = &root["lockfileVersion"] { Some(v.clone()) } else { None });
            ensure!(matches!(version.as_deref(), Some("6.0" | "6" | "9.0" | "9")), "unsupported pnpm lockfile version");
            if let Some(entries) = root["packages"].as_hash() {
                for key in entries.keys() {
                    let key = key.as_str().context("pnpm package key must be a string")?.trim_start_matches('/');
                    let key = key.split('(').next().unwrap_or(key);
                    let (name, version) = key.rsplit_once('@').context("unsupported pnpm package coordinate")?;
                    ensure!(!name.is_empty() && !version.is_empty(), "invalid pnpm coordinate");
                    add("npm", name, version, "unspecified");
                }
            } else { ensure!(root["packages"].is_badvalue(), "pnpm packages must be a mapping"); }
        }
        _ => anyhow::bail!("unsupported lockfile"),
    }
    Ok(packages.into_iter().filter_map(|((ecosystem, name, version), scopes)| {
        let entry = catalog.package(&ecosystem, &name)?;
        let display = if version.is_empty() { format!("{ecosystem}:{name}") } else { format!("{ecosystem}:{name}@{version}") };
        let mut asset = Asset::new("dependency", &display, Some(&entry.id), Evidence { path: path.into(), line: 1, column: 1, mechanism: "lockfile".into(), symbol: name.clone(), cell: None, resolution_path: vec![] });
        asset.package = Some(PackageIdentity { ecosystem, name, version: (!version.is_empty()).then_some(version), scope: scopes.into_iter().collect::<Vec<_>>().join("+"), resolution: "lockfile".into() });
        Some(asset)
    }).collect())
}
