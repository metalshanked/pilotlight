use pilotlight::{catalog::Catalog, indicator::{self, Config, Indicator}, manifest, output::{self, Format}, scan::{self, Options}, source::{Source, github_coordinates, extract_archive}, syntax};
use std::fs;

#[test]
fn catalog_is_valid_and_has_boundary_matching() {
    let catalog = Catalog::load(None).unwrap();
    assert!(catalog.entries.len() >= 50);
    assert!(catalog.import("python", "openai_fake").is_none());
    assert_eq!(catalog.package("pypi", "Langchain_OpenAI").unwrap().id, "langchain");
    assert_eq!(catalog.package("pypi", "Langchain__..OpenAI").unwrap().id, "langchain");
    let mut invalid = catalog.clone(); invalid.entries.push(invalid.entries[0].clone());
    assert!(invalid.validate().is_err());
}

#[test]
fn decisions_calls_are_distinguished_from_generative_calls() {
    let source = "from openai import OpenAI\nclient = OpenAI()\nclient.decisions.create(model='gpt-6-luna')\n";
    let found = syntax::scan("app.py", source, "python", &Catalog::load(None).unwrap()).unwrap();
    assert!(found.assets.iter().any(|a| a.kind == "decision_call"));
    assert!(!found.assets.iter().any(|a| a.kind == "llm_call"));
    assert!(found.assets.iter().any(|a| a.kind == "model" && a.name == "gpt-6-luna"));
}

#[test]
fn python_aliases_clients_models_and_tools_have_evidence_edges() {
    let source = "from openai import OpenAI as Client\nfrom fastmcp import FastMCP\nclient = Client()\nclient.chat.completions.create(model='example-model')\nmcp = FastMCP('demo')\n@mcp.tool()\ndef read_data():\n    pass\n";
    let found = syntax::scan("app.py", source, "python", &Catalog::load(None).unwrap()).unwrap();
    assert!(!found.parse_errors);
    for kind in ["dependency", "llm_client", "llm_call", "model", "mcp_server", "tool"] {
        assert!(found.assets.iter().any(|a| a.kind == kind), "missing {kind}: {:?}", found.assets);
    }
    assert!(found.assets.iter().any(|a| a.name == "example-model"));
    assert!(found.relationships.iter().any(|r| r.kind == "references_model"));
    for relationship in &found.relationships {
        assert!(found.assets.iter().any(|a| a.id == relationship.from));
        assert!(found.assets.iter().any(|a| a.id == relationship.to));
    }
}

#[test]
fn comments_strings_and_shadowed_names_are_not_ai_calls() {
    let source = "# from crewai import Agent\ntext = 'from openai import OpenAI'\nfrom crewai import Agent\ndef ordinary(Agent):\n    return Agent()\nAgent = other\nAgent()\n";
    let found = syntax::scan("app.py", source, "python", &Catalog::load(None).unwrap()).unwrap();
    assert!(found.assets.iter().all(|a| a.kind == "dependency"));
    assert!(found.assets.iter().all(|a| a.framework.as_deref() == Some("crewai")));
}

#[test]
fn typescript_resolves_named_and_default_imports() {
    let source = "import OpenAI from 'openai';\nimport { Agent as Worker } from '@openai/agents';\nconst api = new OpenAI();\napi.responses.create({model: 'example-model'});\nconst a = new Worker({name: 'worker', model: 'other-model'});";
    let found = syntax::scan("app.ts", source, "typescript", &Catalog::load(None).unwrap()).unwrap();
    assert!(!found.parse_errors);
    assert!(found.assets.iter().any(|a| a.kind == "agent"));
    assert!(found.assets.iter().any(|a| a.kind == "llm_call"));
    assert_eq!(found.assets.iter().filter(|a| a.kind == "model").count(), 2);
}

#[test]
fn all_eight_grammars_find_imports() {
    let examples = [
        ("a.py","python","import openai\n"),
        ("a.js","javascript","import OpenAI from 'openai';"),
        ("a.ts","typescript","import { Agent } from '@openai/agents';"),
        ("A.java","java","import dev.langchain4j.model.openai.OpenAiChatModel; class A {}"),
        ("a.go","go","package main\nimport \"github.com/openai/openai-go\"\n"),
        ("a.rs","rust","use async_openai::Client;\nfn main() {}"),
        ("A.cs","csharp","using Microsoft.SemanticKernel;\nclass A {}"),
        ("a.rb","ruby","require 'ruby_llm'\n"),
    ];
    let catalog = Catalog::load(None).unwrap();
    for (path, language, source) in examples {
        let found = syntax::scan(path, source, language, &catalog).unwrap();
        assert!(!found.parse_errors, "parse error: {language}");
        assert!(!found.assets.is_empty(), "no import: {language}");
    }
}

#[test]
fn manifests_match_exact_packages_not_comments_or_substrings() {
    let catalog = Catalog::load(None).unwrap();
    let found = manifest::scan("package.json", r#"{"dependencies":{"openai":"^1","openai-fake":"1"},"description":"anthropic"}"#, &catalog).unwrap();
    assert_eq!(found.len(), 1);
    let found = manifest::scan("requirements.txt", "# anthropic\nopenai>=1\nopenai-fake==2\n-r other.txt", &catalog).unwrap();
    assert_eq!(found.len(), 1);
    let found = manifest::scan("Cargo.toml", "[dependencies]\nllm = { package = 'rig-core', version = '0.1' }", &catalog).unwrap();
    assert_eq!(found[0].framework.as_deref(), Some("rig"));
}

#[test]
fn metadata_is_collected_from_yaml_json_toml_and_required_is_enforced() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".company/.source")).unwrap();
    fs::write(temp.path().join(".company/.source/appinfo.yaml"), "appid: 1234\napplication:\n  owner: platform\n").unwrap();
    fs::write(temp.path().join("info.json"), r#"{"app":{"id":"001234"}}"#).unwrap();
    fs::write(temp.path().join("info.toml"), "active = true\n").unwrap();
    let rules = vec![
        Indicator { name:"application_id".into(),path:".company/.source/appinfo.yaml".into(),key:"appid".into(),required:true },
        Indicator { name:"owner".into(),path:".company/.source/appinfo.yaml".into(),key:"application.owner".into(),required:false },
        Indicator { name:"string_id".into(),path:"info.json".into(),key:"app.id".into(),required:false },
        Indicator { name:"active".into(),path:"info.toml".into(),key:"active".into(),required:false },
    ];
    let (metadata, warnings) = indicator::collect(temp.path(), &rules).unwrap();
    assert_eq!(metadata["application_id"], "1234"); assert_eq!(metadata["string_id"], "001234");
    assert_eq!(metadata["owner"], "platform"); assert_eq!(metadata["active"], "true"); assert!(warnings.is_empty());
    let mut missing = rules[0].clone(); missing.key = "missing".into();
    assert!(indicator::collect(temp.path(), &[missing.clone()]).is_err());
    missing.required = false;
    assert_eq!(indicator::collect(temp.path(), &[missing]).unwrap().1.len(), 1);
    for path in ["../outside.yaml", "/outside.yaml", "C:/outside.yaml", "a/../../outside.yaml"] { assert!(indicator::validate_path(path).is_err()); }
}

#[test]
fn config_typos_fail_and_scan_is_deterministic_with_csv_metadata() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("app.py"), "from crewai import Agent\na = Agent()\n").unwrap();
    fs::write(temp.path().join("meta.yaml"), "appid: 1234\nteam: '=EVIL()'\n").unwrap();
    fs::write(temp.path().join("pilotlight.toml"), "[[indicators]]\nname='application_id'\npath='meta.yaml'\nkey='appid'\nrequired=true\n\n[[indicators]]\nname='team'\npath='meta.yaml'\nkey='team'\n").unwrap();
    let config = Config::load(Some(&temp.path().join("pilotlight.toml"))).unwrap();
    let source = Source::open(temp.path().to_str().unwrap(), None).unwrap();
    let catalog = Catalog::load(None).unwrap();
    let report = scan::scan(&source, &catalog, &Options { indicators:config.indicators.clone(), ..Default::default() }).unwrap();
    let again = scan::scan(&source, &catalog, &Options { jobs:1, indicators:config.indicators, ..Default::default() }).unwrap();
    assert_eq!(serde_json::to_string(&report).unwrap(), serde_json::to_string(&again).unwrap());
    let mut csv = vec![]; output::write(&report, Format::Csv, &mut csv).unwrap();
    let csv = String::from_utf8(csv).unwrap();
    assert!(csv.contains("metadata.application_id")); assert!(csv.contains("1234")); assert!(csv.contains("'=EVIL()"));
    let mut bom = vec![]; output::write(&report, Format::Cyclonedx, &mut bom).unwrap();
    let bom: serde_json::Value = serde_json::from_slice(&bom).unwrap();
    assert_eq!(bom["specVersion"], "1.6");
    fs::write(temp.path().join("bad.toml"), "indcators = []").unwrap();
    assert!(Config::load(Some(&temp.path().join("bad.toml"))).is_err());
}

#[test]
fn scan_skips_generated_ignored_and_oversized_files_and_reports_errors() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join(".gitignore"), "ignored.py\n").unwrap();
    fs::write(temp.path().join("ignored.py"), "import openai\n").unwrap();
    fs::create_dir(temp.path().join("node_modules")).unwrap();
    fs::write(temp.path().join("node_modules/a.js"), "import OpenAI from 'openai';").unwrap();
    fs::write(temp.path().join("large.py"), "#".repeat(100)).unwrap();
    fs::write(temp.path().join("package.json"), "{invalid").unwrap();
    let source = Source::open(temp.path().to_str().unwrap(), None).unwrap();
    let report = scan::scan(&source, &Catalog::load(None).unwrap(), &Options { max_file_bytes:50,..Default::default() }).unwrap();
    assert!(report.assets.is_empty()); assert_eq!(report.coverage.diagnostics.len(), 2);
}

#[test]
fn mcp_inventory_does_not_include_credentials_or_execute_commands() {
    let json = r#"{"mcpServers":{"internal":{"command":"DO_NOT_EXECUTE","args":["SECRET"],"env":{"API_KEY":"SECRET"}}}}"#;
    let assets = pilotlight::formats::mcp_config(".mcp.json", json).unwrap();
    assert_eq!(assets.len(), 1);
    assert!(!serde_json::to_string(&assets).unwrap().contains("SECRET"));
}

#[test]
fn github_urls_are_restricted_to_repository_coordinates() {
    assert_eq!(github_coordinates("https://github.com/org/repo.git").unwrap(), ("org".into(),"repo".into()));
    for url in ["https://evil.com/org/repo", "https://github.com/org/repo/tree/main", "http://github.com/org/repo", "https://token@github.com/org/repo", "https://github.com/org/repo?token=secret"] { assert!(github_coordinates(url).is_err()); }
}

#[test]
fn archive_extraction_rejects_links_and_extracts_regular_files() {
    use flate2::{Compression, write::GzEncoder};
    let build = |symlink: bool| {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar = tar::Builder::new(encoder);
        let mut global = tar::Header::new_gnu();
        let metadata = b"20 comment=fixture1\n";
        global.set_entry_type(tar::EntryType::new(b'g'));
        global.set_mode(0o644);
        global.set_size(metadata.len() as u64);
        global.set_cksum();
        tar.append_data(&mut global, "pax_global_header", &metadata[..]).unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        if symlink { header.set_entry_type(tar::EntryType::Symlink); header.set_size(0); header.set_link_name("../../escape").unwrap(); }
        else { header.set_size(3); }
        header.set_cksum();
        tar.append_data(&mut header, "root/app.py", if symlink { &b""[..] } else { &b"abc"[..] }).unwrap();
        tar.into_inner().unwrap().finish().unwrap()
    };
    let temp = tempfile::tempdir().unwrap();
    assert!(extract_archive(build(true).as_slice(), temp.path()).is_err());
    let omissions = pilotlight::source::extract_inventory_archive(build(true).as_slice(), temp.path()).unwrap();
    assert_eq!(omissions.len(), 1);
    assert_eq!(omissions[0].path, "app.py");
    assert!(!temp.path().join("app.py").exists());
    extract_archive(build(false).as_slice(), temp.path()).unwrap();
    assert_eq!(fs::read_to_string(temp.path().join("app.py")).unwrap(), "abc");
}

#[cfg(feature="duckdb-catalog")]
#[test]
fn duckdb_catalog_round_trip_matches_embedded_fingerprint() {
    let catalog = Catalog::load(None).unwrap();
    let temp = tempfile::tempdir().unwrap(); let path = temp.path().join("catalog.duckdb");
    catalog.export_duckdb(&path).unwrap();
    let loaded = Catalog::load(Some(&path)).unwrap();
    assert_eq!(catalog.fingerprint(), loaded.fingerprint());
}
