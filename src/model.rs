use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Evidence {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub mechanism: String,
    /// A symbol/package identifier, never a source snippet, prompt, or credential.
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolution_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub framework: Option<String>,
    pub confidence: String,
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageIdentity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub detection_rules: Vec<crate::packs::RuleOrigin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIdentity {
    pub ecosystem: String,
    pub name: String,
    pub version: Option<String>,
    pub scope: String,
    pub resolution: String,
}

impl Asset {
    pub fn new(kind: &str, name: &str, framework: Option<&str>, evidence: Evidence) -> Self {
        let identity = format!("{kind}\0{name}\0{}\0{}\0{}\0{}", framework.unwrap_or(""), evidence.path, evidence.line, evidence.column);
        Self {
            id: format!("pl:{}", digest(identity.as_bytes())),
            kind: kind.into(), name: name.into(), framework: framework.map(str::to_owned),
            confidence: "static_evidence".into(), evidence: vec![evidence],
            subtype: None, metadata: BTreeMap::new(), application_root: None, package: None, detection_rules: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Coverage {
    pub files_scanned: usize,
    pub files_skipped: usize,
    pub by_language: BTreeMap<String, usize>,
    pub diagnostics: Vec<Diagnostic>,
    pub files_failed: usize,
    pub unsupported_languages: BTreeMap<String, usize>,
    pub by_type: BTreeMap<String, TypeCoverage>,
    pub scope_notes: Vec<String>,
}

impl Coverage {
    pub fn incomplete(&self) -> bool {
        self.by_type.values().any(|c| matches!(c.status.as_str(), "partial" | "unsupported" | "failed"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeCoverage {
    pub status: String,
    pub findings: usize,
    pub capabilities: BTreeMap<String, String>,
    pub unsupported_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub root: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub scanner_version: String,
    pub source: String,
    pub revision: Option<String>,
    pub catalog_version: String,
    pub catalog_sha256: String,
    #[serde(default)]
    pub detection_packs: Vec<crate::packs::PackInfo>,
    pub metadata: BTreeMap<String, String>,
    pub assets: Vec<Asset>,
    pub relationships: Vec<Relationship>,
    pub coverage: Coverage,
    pub selection: crate::artifact::Selection,
    pub applications: Vec<Application>,
    pub status: String,
    pub config_sha256: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

