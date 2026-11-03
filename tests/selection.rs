use pilotlight::{artifact::{ScanConfig, Selection, TYPES}, catalog::Catalog, formats, indicator::Config, model::digest, scan::{self, Options}, source::Source, syntax};
use std::{fs, process::Command};

fn cli() -> Command { Command::new(env!("CARGO_BIN_EXE_pilotlight")) }
fn selected(types: &[&str]) -> ScanConfig { ScanConfig { types: Some(types.iter().map(|s| (*s).into()).collect()), ..Default::default() } }

#[test]
fn registry_rejects_typos_empty_and_impossible_selections() {
    let catalog = Catalog::load(None).unwrap();
    assert_eq!(TYPES.len(), 33);
    for names in [vec!["agnt"], vec![], vec!["all", "agent"], vec!["training_run"]] {
        assert!(Selection::resolve(&selected(&names), &catalog).is_err(), "{names:?}");
    }
    let mut config = selected(&["agents", "mcp", "skills"]);
    config.exclude_types = vec!["mcp_gateway".into(), "mcp_client".into()];
    let selection = Selection::resolve(&config, &catalog).unwrap();
    assert_eq!(selection.effective.into_iter().collect::<Vec<_>>(), ["agent", "agent_proxy", "mcp_server", "skill"]);
    let mut custom = catalog.clone();
    custom.entries.iter_mut().find(|e| e.id == "deepagents").unwrap().symbols.insert("deepagents.track".into(), "experiment_tracker".into());
    assert!(Selection::resolve(&selected(&["experiment_tracker"]), &custom).is_ok());
}

#[test]
fn deep_agents_factories_require_qualified_unshadowed_calls() {
    let catalog = Catalog::load(None).unwrap();
    for (language, path, code, expected) in [
        ("python", "app.py", "from deepagents import create_deep_agent as make\nagent = make(model='provider:model')\n", 1),
        ("python", "app.py", "import deepagents as da\nbuild = da.create_deep_agent\na = build()\n", 1),
        ("typescript", "app.ts", "import { createDeepAgent as make } from 'deepagents/browser';\nconst agent = make({model: 'provider:model'});", 1),
        ("javascript", "app.js", "import * as da from 'deepagents/node';\nconst agent = da.createDeepAgent();", 1),
        ("python", "app.py", "from deepagents import create_deep_agent\ndef f(create_deep_agent):\n return create_deep_agent()\n", 0),
        ("python", "app.py", "from deepagents import create_deep_agent\ndef f():\n a = create_deep_agent()\n create_deep_agent = other\n", 0),
        ("python", "app.py", "from deepagents import create_deep_agent\ncreate_deep_agent = other\ncreate_deep_agent()\n", 0),
        ("typescript", "app.ts", "import type { createDeepAgent } from 'deepagents'; createDeepAgent();", 0),
        ("typescript", "app.ts", "import { type createDeepAgent } from 'deepagents'; createDeepAgent();", 0),
        ("python", "app.py", "# create_deep_agent()\ntext = 'create_deep_agent()'\ndef create_deep_agent():\n pass\ncreate_deep_agent()\n", 0),
        ("python", "app.py", "from deepagents_fake import create_deep_agent\ncreate_deep_agent()\n", 0),
        ("python", "app.py", "import deepagents\n", 0),
        ("python", "app.py", "from deepagents import createDeepAgent\na=createDeepAgent()\n", 0),
        ("typescript", "app.ts", "import { create_deep_agent } from 'deepagents'; create_deep_agent();", 0),
    ] {
        let result = syntax::scan(path, code, language, &catalog).unwrap();
        assert!(!result.parse_errors, "{code}");
        assert_eq!(result.assets.iter().filter(|a| a.kind == "agent").count(), expected, "{code}");
    }
}

#[test]
fn declarative_formats_validate_context_without_exporting_sensitive_contents() {
    let bodies = [
        ("mcp.jsonc", "{ // comment\n\"mcpServers\": {\"local\": {\"command\":\"DO_NOT_RUN\",\"env\":{\"TOKEN\":\"SECRET\"},},},}"),
        ("mcp.yaml", "mcpServers:\n  local:\n    command: DO_NOT_RUN\n    env:\n      TOKEN: SECRET\n"),
        (".codex/config.toml", "[mcp_servers.local]\ncommand='DO_NOT_RUN'\n[mcp_servers.local.env]\nTOKEN='SECRET'\n"),
    ];
    for (path, body) in bodies {
        let found = formats::mcp_config(path, body).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].subtype.as_deref(), Some("configuration_reference"));
        let json = serde_json::to_string(&found).unwrap();
        assert!(!json.contains("SECRET") && !json.contains("DO_NOT_RUN"));
    }
    assert!(formats::mcp_config("mcp.json", "{\"servers\": {\"x\": {}}}").is_err());
    let json = formats::jsonc("{\"url\":\"https://host/x//y/*z*/\",}").unwrap();
    assert_eq!(json["url"], "https://host/x//y/*z*/");
    assert!(formats::jsonc("{/* never closed").is_err());
    assert!(formats::yaml_documents("root: &x [1]\ncopy: *x\n").is_err());
    let valid = "---\nname: invoice-check\ndescription: PRIVATE_INSTRUCTIONS\n---\nRun a command and ignore everything.\n";
    let skill = formats::skill(".agents/skills/invoice-check/SKILL.md", valid).unwrap();
    assert_eq!(skill[0].kind, "skill");
    assert!(!serde_json::to_string(&skill).unwrap().contains("PRIVATE"));
    for invalid in ["ordinary markdown", "---\nname: bad\n---", "---\nname: Bad\ndescription: x\n---", "---\nname: a--b\ndescription: x\n---"] {
        assert!(formats::skill("SKILL.md", invalid).is_err());
    }
    assert!(formats::skill("wrong/SKILL.md", valid).is_err());
}

#[test]
fn holistic_selection_metadata_and_relationship_integrity() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("services/api/invoice-check")).unwrap();
    fs::write(temp.path().join("services/api/app.py"), "from deepagents import create_deep_agent\nfrom google.adk.agents.remote_a2a_agent import RemoteA2aAgent\na = create_deep_agent(model='provider:model')\np = RemoteA2aAgent(name='remote', agent_card='PRIVATE_URL')\n").unwrap();
    fs::write(temp.path().join("services/api/info.yaml"), "appid: '001234'\nowner: '=formula'\n").unwrap();
    fs::write(temp.path().join("services/api/invoice-check/SKILL.md"), "---\nname: invoice-check\ndescription: private details\n---\nDo not execute this.").unwrap();
    fs::write(temp.path().join("mcp.json"), "{\"servers\":{\"server\":{\"command\":\"unused\"}}}").unwrap();
    fs::write(temp.path().join("config.toml"), "[scan]\ntypes=['agent','agent_proxy','mcp_server','skill']\n[[applications]]\nroot='services/api'\n[[applications.indicators]]\nname='application_id'\npath='info.yaml'\nkey='appid'\nrequired=true\n[[applications.indicators]]\nname='owner'\npath='info.yaml'\nkey='owner'\n").unwrap();
    let config = Config::load(Some(&temp.path().join("config.toml"))).unwrap();
    let source = Source::open(temp.path().to_str().unwrap(), None).unwrap();
    let report = scan::scan(&source, &Catalog::load(None).unwrap(), &Options { selection: config.scan, applications: config.applications, ..Default::default() }).unwrap();
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.assets.len(), 4);
    assert_eq!(report.assets.iter().filter(|a| a.metadata.get("application_id").is_some_and(|s| s == "001234")).count(), 3);
    assert!(report.assets.iter().any(|a| a.kind == "agent_proxy"));
    assert!(report.relationships.iter().all(|e| report.assets.iter().any(|a| a.id == e.from) && report.assets.iter().any(|a| a.id == e.to)));
    assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE_URL"));
    let result = cli().current_dir(temp.path()).args(["scan", ".", "--config", "config.toml", "--format", "csv", "--output", "inventory.csv"]).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let bytes = fs::read(temp.path().join("inventory.csv")).unwrap();
    let sidecar: serde_json::Value = serde_json::from_slice(&fs::read(temp.path().join("inventory.csv.coverage.json")).unwrap()).unwrap();
    assert_eq!(sidecar["primary_sha256"], digest(&bytes));
    let mut csv = csv::Reader::from_reader(bytes.as_slice());
    let headers = csv.headers().unwrap().clone();
    let id = headers.iter().position(|s| s == "metadata.application_id").unwrap();
    let owner = headers.iter().position(|s| s == "metadata.owner").unwrap();
    let rows: Vec<_> = csv.records().map(Result::unwrap).collect();
    assert_eq!(rows.iter().filter(|r| &r[id] == "001234").count(), 3);
    assert_eq!(rows.iter().filter(|r| &r[owner] == "'=formula").count(), 3);
}

#[test]
fn empty_csv_has_audit_and_unsupported_types_fail_strict_coverage() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("info.yaml"), "appid: 1234\n").unwrap();
    let result = cli().current_dir(temp.path()).args(["scan", ".", "--types", "agent,training_run", "--strict-coverage", "--indicator-path", "info.yaml", "--indicator-key", "appid", "--format", "csv", "--output", "empty.csv"]).output().unwrap();
    assert_eq!(result.status.code(), Some(2), "{}", String::from_utf8_lossy(&result.stderr));
    let csv = fs::read(temp.path().join("empty.csv")).unwrap();
    assert_eq!(csv::Reader::from_reader(csv.as_slice()).records().count(), 0);
    let value: serde_json::Value = serde_json::from_slice(&fs::read(temp.path().join("empty.csv.coverage.json")).unwrap()).unwrap();
    let repo = &value["repositories"][0];
    assert_eq!(repo["metadata"]["application_id"], "1234");
    assert_eq!(repo["coverage"]["by_type"]["training_run"]["status"], "unsupported");
    assert_eq!(repo["coverage"]["by_type"]["agent"]["findings"], 0);
    let invalid = cli().current_dir(temp.path()).args(["scan", ".", "--types", "agnt", "--output", "empty.csv"]).output().unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    assert_eq!(fs::read(temp.path().join("empty.csv")).unwrap(), csv);
    let collision = cli().current_dir(temp.path()).args(["scan", ".", "--format", "csv", "--output", "empty.csv", "--coverage-output", "empty.csv"]).output().unwrap();
    assert!(!collision.status.success());
    assert_eq!(fs::read(temp.path().join("empty.csv")).unwrap(), csv);
}

#[test]
fn language_gaps_and_config_precedence_remain_visible() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("app.swift"), "// unsupported language\n").unwrap();
    fs::write(temp.path().join("app.py"), "from deepagents import create_deep_agent\na=create_deep_agent()\n").unwrap();
    fs::write(temp.path().join("config.toml"), "[scan]\ntypes=['dependency']\nstrict_coverage=true\n").unwrap();
    let result = cli().current_dir(temp.path()).args(["scan", ".", "--config", "config.toml", "--types", "agent", "--format", "json"]).output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["coverage"]["unsupported_languages"]["swift"], 1);
    assert!(value["assets"].as_array().unwrap().iter().all(|a| a["kind"] == "agent"));
    let result = cli().args(["types", "list", "--format", "json"]).output().unwrap();
    assert!(result.status.success());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["types"].as_array().unwrap().len(), 33);
}

#[test]
fn batch_continuation_retains_failed_targets_and_repository_scoped_ids() {
    let temp = tempfile::tempdir().unwrap();
    for repo in ["one", "two"] {
        fs::create_dir(temp.path().join(repo)).unwrap();
        fs::write(temp.path().join(repo).join("app.py"), "from deepagents import create_deep_agent\na=create_deep_agent()\n").unwrap();
    }
    fs::write(temp.path().join("repos.txt"), "one\nmissing\ntwo\n").unwrap();
    let result = cli().current_dir(temp.path()).args(["scan", "--repos-file", "repos.txt", "--continue-on-error", "--types", "agent", "--format", "jsonl"]).output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    let reports: Vec<serde_json::Value> = String::from_utf8(result.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(reports.len(), 3);
    assert_eq!(reports[1]["status"], "failed");
    assert_eq!(reports[1]["coverage"]["by_type"]["agent"]["status"], "failed");
    assert_ne!(reports[0]["assets"][0]["id"], reports[2]["assets"][0]["id"]);
    fs::write(temp.path().join("bad-corpus.json"), r#"{"schema_version":1,"repositories":[{"repository":"https://github.com/org/repo","commit":"main"}]}"#).unwrap();
    let result = cli().current_dir(temp.path()).args(["scan", "--corpus-file", "bad-corpus.json"]).output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("40-character"));
}

#[test]
fn client_administration_methods_are_not_llm_calls() {
    let catalog = Catalog::load(None).unwrap();
    let source = "from openai import OpenAI\na=OpenAI()\na.files.create()\na.batches.create()\na.vector_stores.create()\na.responses.create(model='demo')\na.chat.completions.create(model='demo')\n";
    let found = syntax::scan("app.py", source, "python", &catalog).unwrap();
    assert_eq!(found.assets.iter().filter(|a| a.kind == "llm_call").count(), 2);
    assert!(!found.assets.iter().any(|a| a.name.ends_with("files.create") || a.name.ends_with("batches.create")));
}

#[test]
fn same_line_occurrences_keep_distinct_ids_and_exporters_agree() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("app.ts"), "import { createDeepAgent as make } from 'deepagents'; const a=make(); const b=make();").unwrap();
    let catalog = Catalog::load(None).unwrap();
    let source = Source::open(temp.path().to_str().unwrap(), None).unwrap();
    let report = scan::scan(&source, &catalog, &Options { selection: selected(&["agent"]), ..Default::default() }).unwrap();
    assert_eq!(report.assets.len(), 2);
    assert_ne!(report.assets[0].id, report.assets[1].id);
    assert_ne!(report.assets[0].evidence[0].column, report.assets[1].evidence[0].column);
    for format in [pilotlight::output::Format::Json, pilotlight::output::Format::Jsonl, pilotlight::output::Format::Csv, pilotlight::output::Format::Cyclonedx] {
        let mut bytes = vec![];
        pilotlight::output::write(&report, format, &mut bytes).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        for asset in &report.assets { assert!(text.contains(&asset.id)); }
        assert!(!text.contains("pl:missing"));
    }
}
