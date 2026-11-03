use anyhow::{Context, Result};
use crate::{catalog::Catalog, model::{Asset, Evidence, PackageIdentity}};
use regex::Regex;
use serde_json::Value;
use std::path::Path;

pub fn supported(path: &Path) -> bool {
    if crate::lockfile::supported(path) { return true; }
    let name = path.file_name().and_then(|x| x.to_str()).unwrap_or("");
    matches!(name, "package.json" | "composer.json" | "pyproject.toml" | "Cargo.toml" | "go.mod" | "pom.xml" | "Gemfile" | "build.gradle" | "build.gradle.kts" | "Directory.Packages.props" | "packages.config")
        || (name.starts_with("requirements") && name.ends_with(".txt"))
        || name.ends_with(".csproj")
}

pub fn scan(path: &str, source: &str, catalog: &Catalog) -> Result<Vec<Asset>> {
    if crate::lockfile::supported(Path::new(path)) { return crate::lockfile::scan(path, source, catalog); }
    let name = Path::new(path).file_name().unwrap().to_string_lossy();
    let mut packages: Vec<(String, String)> = vec![];
    let mut add = |eco: &str, name: &str| packages.push((eco.into(), name.into()));
    if name == "composer.json" {
        let json: Value = serde_json::from_str(source).context("invalid composer.json")?;
        for section in ["require", "require-dev"] {
            if let Some(deps) = json.get(section).and_then(Value::as_object) { for name in deps.keys() { add("composer", name); } }
        }
    } else if name == "package.json" {
        let json: Value = serde_json::from_str(source).context("invalid package.json")?;
        for section in ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"] {
            if let Some(deps) = json.get(section).and_then(Value::as_object) {
                for (name, version) in deps {
                    let actual = version.as_str().and_then(|v| v.strip_prefix("npm:")).map(|v| v.rsplit_once('@').map_or(v, |(p, _)| p)).unwrap_or(name);
                    add("npm", actual);
                }
            }
        }
    } else if name == "pyproject.toml" {
        let toml: toml::Value = toml::from_str(source).context("invalid pyproject.toml")?;
        if let Some(groups) = toml.get("dependency-groups").and_then(toml::Value::as_table) {
            for deps in groups.values().filter_map(toml::Value::as_array) {
                for dep in deps.iter().filter_map(toml::Value::as_str) { add("pypi", requirement_name(dep)); }
            }
        }
        if let Some(deps) = toml.get("project").and_then(|p| p.get("dependencies")).and_then(toml::Value::as_array) {
            for dep in deps.iter().filter_map(toml::Value::as_str) { add("pypi", requirement_name(dep)); }
        }
        if let Some(groups) = toml.get("project").and_then(|p| p.get("optional-dependencies")).and_then(toml::Value::as_table) {
            for deps in groups.values().filter_map(toml::Value::as_array) {
                for dep in deps.iter().filter_map(toml::Value::as_str) { add("pypi", requirement_name(dep)); }
            }
        }
        if let Some(poetry) = toml.get("tool").and_then(|p| p.get("poetry")) {
            for section in ["dependencies", "dev-dependencies"] {
                if let Some(deps) = poetry.get(section).and_then(toml::Value::as_table) {
                    for name in deps.keys() { add("pypi", name); }
                }
            }
            if let Some(groups) = poetry.get("group").and_then(toml::Value::as_table) {
                for group in groups.values() {
                    if let Some(deps) = group.get("dependencies").and_then(toml::Value::as_table) {
                        for name in deps.keys() { add("pypi", name); }
                    }
                }
            }
        }
    } else if name == "Cargo.toml" {
        let toml: toml::Value = toml::from_str(source).context("invalid Cargo.toml")?;
        cargo_dependencies(&toml, &mut add);
    } else if name.starts_with("requirements") {
        for line in source.lines().map(str::trim).filter(|s| !s.starts_with(['#', '-'])) {
            add("pypi", requirement_name(line));
        }
    } else if name == "go.mod" {
        let re = Regex::new(r"(?m)^\s*(?:require\s+)?([a-zA-Z0-9.-]+/[^\s]+)\s+v[0-9]")?;
        for c in re.captures_iter(source) { add("go", &c[1]); }
    } else if name == "pom.xml" || name.ends_with(".csproj") || name == "Directory.Packages.props" || name == "packages.config" {
        let doc = roxmltree::Document::parse(source).context("invalid XML manifest")?;
        for node in doc.descendants() {
            if node.has_tag_name("dependency") {
                let field = |key: &str| node.children().find(|n| n.has_tag_name(key)).and_then(|n| n.text());
                if let (Some(group), Some(artifact)) = (field("groupId"), field("artifactId")) { add("maven", &format!("{group}:{artifact}")); }
            }
            if node.has_tag_name("PackageReference") || node.has_tag_name("PackageVersion") {
                if let Some(name) = node.attribute("Include").or_else(|| node.attribute("Update")) { add("nuget", name); }
            }
            if name == "packages.config" && node.has_tag_name("package") { if let Some(name) = node.attribute("id") { add("nuget", name); } }
        }
    } else if name == "Gemfile" {
        for c in Regex::new(r#"(?m)^\s*gem\s+["']([^"']+)["']"#)?.captures_iter(source) { add("gem", &c[1]); }
    } else if name.starts_with("build.gradle") {
        for c in Regex::new(r#"(?m)^\s*(?:implementation|api|compileOnly|runtimeOnly|testImplementation)\s*\(?\s*["']([^:"']+):([^:"']+):[^"']+["']"#)?.captures_iter(source) { add("maven", &format!("{}:{}", &c[1], &c[2])); }
    }
    packages.sort(); packages.dedup();
    Ok(packages.into_iter().filter_map(|(eco, name)| {
        let entry = catalog.package(&eco, &name)?;
        // Manifest evidence is file-scoped: do not invent a source line from substring matching.
        let mut asset = Asset::new("dependency", &format!("{eco}:{name}"), Some(&entry.id), Evidence { path: path.into(), line: 1, column: 1, mechanism: "manifest".into(), symbol: name.clone(), cell: None, resolution_path: vec![] });
        asset.package = Some(PackageIdentity { ecosystem: eco, name, version: None, scope: "unspecified".into(), resolution: "declared".into() });
        Some(asset)
    }).collect())
}

fn requirement_name(value: &str) -> &str {
    value.trim().split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))).next().unwrap_or("")
}

fn cargo_dependencies(value: &toml::Value, add: &mut impl FnMut(&str, &str)) {
    if let Some(table) = value.as_table() {
        for (key, value) in table {
            if matches!(key.as_str(), "dependencies" | "dev-dependencies" | "build-dependencies") {
                if let Some(deps) = value.as_table() {
                    for (name, config) in deps {
                        add("cargo", config.get("package").and_then(toml::Value::as_str).unwrap_or(name));
                    }
                }
            } else if matches!(key.as_str(), "target" | "workspace") || key.contains("cfg(") || key.contains('-') {
                cargo_dependencies(value, add);
            }
        }
    }
}
