//! Read-only declarative adapters. Contents never cause commands, fetches or model calls.
use anyhow::{ensure, Context, Result};
use crate::model::{Asset, Evidence};
use std::path::Path;
use serde_json::Value;
use yaml_rust2::{Yaml, YamlLoader, parser::{Event, EventReceiver, Parser}};

pub fn is_skill(path: &Path) -> bool { path.file_name().is_some_and(|n| n == "SKILL.md") }

/// Check parser events before constructing YAML values, preventing alias expansion.
pub fn yaml_documents(text: &str) -> Result<Vec<Yaml>> {
    yaml_documents_with_budget(text, 100_000)
}

pub(crate) fn yaml_documents_with_budget(text: &str, limit: usize) -> Result<Vec<Yaml>> {
    struct Budget { depth: usize, events: usize, invalid: bool, limit: usize }
    impl EventReceiver for Budget {
        fn on_event(&mut self, event: Event) {
            self.events += 1;
            match event {
                Event::Alias(_) => self.invalid = true,
                Event::SequenceStart(..) | Event::MappingStart(..) => self.depth += 1,
                Event::SequenceEnd | Event::MappingEnd => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
            self.invalid |= self.depth > 64 || self.events > self.limit;
        }
    }
    let mut budget = Budget { depth: 0, events: 0, invalid: false, limit: limit.min(250_000) };
    Parser::new_from_str(text).load(&mut budget, true).map_err(|_| anyhow::anyhow!("invalid YAML"))?;
    ensure!(!budget.invalid, "YAML aliases or resource limits are not supported");
    YamlLoader::load_from_str(text).map_err(|_| anyhow::anyhow!("invalid YAML"))
}

pub fn skill(path: &str, text: &str) -> Result<Vec<Asset>> {
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    ensure!(lines.next().is_some_and(|l| l.trim() == "---"), "SKILL.md requires YAML frontmatter");
    let mut header = String::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" { closed = true; break; }
        header.push_str(line); header.push('\n');
        ensure!(header.len() <= 64 * 1024, "skill frontmatter exceeds 64 KiB");
    }
    ensure!(closed, "unterminated skill frontmatter");
    let docs = yaml_documents(&header)?;
    ensure!(docs.len() == 1, "expected one skill frontmatter document");
    let name = docs[0]["name"].as_str().context("skill name must be a string")?;
    let description = docs[0]["description"].as_str().context("skill description must be a string")?;
    ensure!(!name.is_empty() && name.len() <= 64 && !name.starts_with('-') && !name.ends_with('-') && !name.contains("--")
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'), "invalid skill name");
    ensure!(!description.trim().is_empty() && description.chars().count() <= 1024, "invalid skill description");
    if let Some(parent) = Path::new(path).parent().and_then(Path::file_name) {
        ensure!(parent == name, "skill name must match its directory name");
    }
    // Only identification metadata is exported. Instructions and descriptions stay private.
    Ok(vec![Asset::new("skill", name, None, Evidence { path: path.into(), line: 2, column: 1, mechanism: "skill_frontmatter".into(), symbol: name.into(), cell: None, resolution_path: vec![] })])
}

pub fn is_mcp_config(path: &Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    matches!(name, "mcp.json" | ".mcp.json" | "mcp.jsonc" | ".mcp.jsonc" | "mcp.yaml" | "mcp.yml" | "mcp.toml" | ".mcp.yaml" | ".mcp.yml" | ".mcp.toml" | "claude_desktop_config.json")
        || (name == "config.toml" && path.parent().and_then(Path::file_name).is_some_and(|p| p == ".codex"))
}

fn yaml_json(node: &Yaml) -> Result<Value> {
    Ok(match node {
        Yaml::Hash(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map { out.insert(key.as_str().context("YAML keys must be strings")?.into(), yaml_json(value)?); }
            Value::Object(out)
        }
        Yaml::Array(array) => Value::Array(array.iter().map(yaml_json).collect::<Result<_>>()?),
        Yaml::String(s) => Value::String(s.clone()),
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Integer(n) => (*n).into(),
        Yaml::Real(s) => Value::String(s.clone()),
        Yaml::Null => Value::Null,
        _ => anyhow::bail!("unsupported YAML node"),
    })
}

/// Remove comments and trailing commas while preserving string bytes and line positions.
pub fn jsonc(text: &str) -> Result<Value> {
    let mut bytes = text.as_bytes().to_vec();
    let (mut i, mut string, mut escape) = (0, false, false);
    while i < bytes.len() {
        let b = bytes[i];
        if string {
            if escape { escape = false; } else if b == b'\\' { escape = true; } else if b == b'"' { string = false; }
        } else if b == b'"' { string = true; }
        else if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            while i < bytes.len() && bytes[i] != b'\n' { bytes[i] = b' '; i += 1; }
            continue;
        } else if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
            bytes[i] = b' '; bytes[i + 1] = b' '; i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                if !matches!(bytes[i], b'\n' | b'\r') { bytes[i] = b' '; } i += 1;
            }
            ensure!(i + 1 < bytes.len(), "unterminated JSON comment");
            bytes[i] = b' '; bytes[i + 1] = b' '; i += 2; continue;
        }
        i += 1;
    }
    string = false; escape = false;
    for i in 0..bytes.len() {
        let b = bytes[i];
        if string {
            if escape { escape = false; } else if b == b'\\' { escape = true; } else if b == b'"' { string = false; }
        } else if b == b'"' { string = true; }
        else if b == b',' && bytes[i + 1..].iter().find(|b| !b.is_ascii_whitespace()).is_some_and(|b| matches!(b, b'}' | b']')) { bytes[i] = b' '; }
    }
    serde_json::from_slice(&bytes).context("invalid JSON/JSONC")
}

pub fn mcp_config(path: &str, text: &str) -> Result<Vec<Asset>> {
    let extension = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
    let value = match extension {
        "yaml" | "yml" => { let docs = yaml_documents(text)?; ensure!(docs.len() == 1, "expected one YAML document"); yaml_json(&docs[0])? }
        "toml" => serde_json::to_value(toml::from_str::<toml::Value>(text).context("invalid MCP TOML")?)?,
        _ => jsonc(text)?,
    };
    ensure!(value.is_object(), "MCP configuration must be an object");
    let servers = value.get("mcpServers").or_else(|| value.get("mcp_servers")).or_else(|| value.get("servers")).or_else(|| value.get("mcp").and_then(|m| m.get("servers")));
    let mut assets = vec![];
    if let Some(servers) = servers {
        let servers = servers.as_object().context("MCP servers must be an object")?;
        for (name, config) in servers {
            ensure!(config.is_object(), "MCP server entry must be an object");
            ensure!(config.get("command").or_else(|| config.get("url")).is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty())), "MCP server entry requires a command or URL");
            let mut asset = Asset::new("mcp_server", name, None, Evidence { path: path.into(), line: 1, column: 1, mechanism: "mcp_configuration".into(), symbol: name.clone(), cell: None, resolution_path: vec![] });
            asset.subtype = Some("configuration_reference".into());
            assets.push(asset);
        }
    }
    Ok(assets)
}
