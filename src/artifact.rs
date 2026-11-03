//! Shared taxonomy, selection and capability contract for every detector/reporter.
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use crate::{catalog::{Catalog, prefix_matches}, model::{Coverage, TypeCoverage}};

pub const TYPES: &[&str] = &[
    "model", "llm_endpoint", "model_endpoint", "agent", "agent_proxy", "tool",
    "mcp_server", "mcp_client", "mcp_gateway", "embedding", "vector_store", "dataset",
    "retriever", "knowledge_base", "feature_store", "memory", "prompt", "training_run",
    "hyperparameter", "model_artifact", "experiment_tracker", "model_registry",
    "data_versioning", "ml_pipeline", "guardrail", "skill", "observability", "secret",
    "dependency", "other", "llm_client", "llm_call", "decision_call",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanConfig {
    pub types: Option<Vec<String>>,
    #[serde(default)] pub exclude_types: Vec<String>,
    #[serde(default)] pub strict_coverage: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Selection {
    pub requested: Vec<String>,
    pub effective: BTreeSet<String>,
    pub excluded: BTreeSet<String>,
}

fn expand(names: &[String]) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    ensure!(!names.is_empty(), "artifact type selection cannot be empty; use `pilotlight types list`");
    ensure!(!names.iter().any(|n| n == "all") || names.len() == 1, "all cannot be combined with individual types");
    for name in names {
        let group: Vec<&str> = match name.as_str() {
            "all" => TYPES.to_vec(),
            "agents" => vec!["agent", "agent_proxy"],
            "mcp" => vec!["mcp_server", "mcp_client", "mcp_gateway"],
            "models" => vec!["model", "model_artifact", "model_endpoint", "llm_endpoint"],
            "skills" => vec!["skill"],
            value if TYPES.contains(&value) => vec![value],
            _ => bail!("unknown artifact type '{name}'; use `pilotlight types list` (e.g. agent, agent_proxy, mcp_server, skill)"),
        };
        result.extend(group.into_iter().map(str::to_owned));
    }
    Ok(result)
}

/// Capability declarations are generated from the active catalog plus format adapters.
/// Capability claims are limited to catalog-qualified extraction per language.
pub fn capabilities(catalog: &Catalog, kind: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    if kind == "dependency" {
        for language in ["python", "javascript", "typescript", "java", "go", "rust", "csharp", "ruby", "php", "kotlin", "manifest"] {
            result.insert(language.into(), "package_or_import".into());
        }
    }
    if kind == "skill" { result.insert("skill".into(), "validated_frontmatter".into()); }
    if kind == "mcp_server" { result.insert("mcp_config".into(), "configuration_reference".into()); }
    for entry in &catalog.entries {
        if kind == "agent" {
            for language in ["python", "javascript", "typescript"] {
                let model_calls = entry.symbols.iter().any(|(s,k)|
                    catalog.symbol_kind(entry,language,s).is_some() && entry.imports.contains_key(language)
                    && (k=="llm_call" || k=="llm_client" && entry.operations.values().any(|k|k=="llm_call")));
                if model_calls { result.insert(language.into(), "bounded_tool_loop_patterns".into()); }
            }
        }
        for (symbol, output_kind) in &entry.symbols {
            let applicable = output_kind == kind
                || kind == "model"
                || (output_kind == "llm_client" && entry.operations.values().any(|output| output == kind));
            if !applicable { continue; }
            for language in ["python", "javascript", "typescript", "go", "java", "csharp", "rust", "php", "kotlin", "ruby"] {
                if kind == "model" && output_kind != "model" && !matches!(language, "python" | "javascript" | "typescript" | "ruby") { continue; }
                if catalog.symbol_kind(entry, language, symbol).is_none() { continue; }
                if entry.imports.get(language).is_some_and(|prefixes| prefixes.iter().any(|p| prefix_matches(symbol, p))) {
                    result.insert(language.into(), if language == "python" { "bounded_modules_and_lexical_calls" } else { "lexical_calls_and_literals" }.into());
                }
            }
        }
    }
    result
}

impl Selection {
    pub fn resolve(config: &ScanConfig, catalog: &Catalog) -> Result<Self> {
        let requested = config.types.clone().unwrap_or_else(|| TYPES.iter()
            .filter(|kind| **kind != "secret" && !capabilities(catalog, kind).is_empty())
            .map(|kind| (*kind).into()).collect());
        let mut effective = expand(&requested)?;
        let excluded = if config.exclude_types.is_empty() { BTreeSet::new() } else { expand(&config.exclude_types)? };
        effective.retain(|kind| !excluded.contains(kind));
        ensure!(!effective.is_empty(), "artifact type exclusions leave nothing to scan");
        ensure!(effective.iter().any(|kind| !capabilities(catalog, kind).is_empty()), "no detector is available for the requested types; use `pilotlight types list`");
        Ok(Self { requested, effective, excluded })
    }
}

pub fn finish_coverage(coverage: &mut Coverage, selection: &Selection, catalog: &Catalog, counts: &BTreeMap<String, usize>) {
    for kind in TYPES {
        let capabilities = capabilities(catalog, kind);
        let requested = selection.effective.contains(*kind);
        let unsupported_languages: Vec<_> = coverage.by_language.keys()
            .chain(coverage.unsupported_languages.keys())
            .filter(|lang| !matches!(lang.as_str(), "manifest" | "skill" | "mcp_config"))
            .filter(|lang| !capabilities.contains_key(*lang)).cloned().collect();
        let status = if !requested { "not_requested" }
            else if capabilities.is_empty() { "unsupported" }
            else if !coverage.diagnostics.is_empty() || (!unsupported_languages.is_empty() && *kind != "skill") { "partial" }
            else { "complete_within_declared_scope" };
        coverage.by_type.insert((*kind).into(), TypeCoverage {
            status: status.into(), findings: counts.get(*kind).copied().unwrap_or(0), capabilities,
            unsupported_languages: if *kind == "skill" { vec![] } else { unsupported_languages },
        });
    }
}
