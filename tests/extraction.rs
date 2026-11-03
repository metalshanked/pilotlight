use pilotlight::{catalog::Catalog, manifest, notebook, scan::{self, Options}, source::Source};
use std::fs;

#[test]
fn lockfiles_resolve_exact_packages_versions_and_aliases() {
    let catalog = Catalog::load(None).unwrap();
    for (path, text, ecosystem, package, version) in [
        ("package-lock.json", r#"{"lockfileVersion":1,"dependencies":{"alias":{"version":"npm:deepagents@0.1.0"},"deepagents-fake":{"version":"1.0"}}}"#, "npm", "deepagents", "0.1.0"),
        ("package-lock.json", r#"{"lockfileVersion":3,"packages":{"":{"name":"workspace"},"node_modules/alias":{"name":"deepagents","version":"0.1.0"},"node_modules/fake":{"version":"1.0"}}}"#, "npm", "deepagents", "0.1.0"),
        ("Cargo.lock", "version=4\n[[package]]\nname='rig-core'\nversion='0.1.0'\n", "cargo", "rig-core", "0.1.0"),
        ("uv.lock", "version=1\n[[package]]\nname='deepagents'\nversion='0.1.0'\n", "pypi", "deepagents", "0.1.0"),
        ("poetry.lock", "[[package]]\nname='deepagents'\nversion='0.1.0'\n[metadata]\nlock-version='2.1'\n", "pypi", "deepagents", "0.1.0"),
        ("pnpm-lock.yaml", "lockfileVersion: '9.0'\npackages:\n  deepagents@0.1.0:\n    resolution: {}\n  deepagents-fake@9.0.0: {}\n", "npm", "deepagents", "0.1.0"),
        ("pnpm-lock.yaml", "lockfileVersion: 6.0\npackages:\n  /@openai/agents@0.1.0(zod@3.0.0):\n    resolution: {}\n", "npm", "@openai/agents", "0.1.0"),
        ("packages.lock.json", r#"{"version":1,"dependencies":{"net8.0":{"OpenAI":{"type":"Direct","resolved":"0.1.0"}}}}"#, "nuget", "OpenAI", "0.1.0"),
    ] {
        let found = manifest::scan(path, text, &catalog).unwrap();
        assert_eq!(found.len(), 1, "{path}: {text}");
        let identity = found[0].package.as_ref().unwrap();
        assert_eq!(identity.ecosystem, ecosystem);
        assert_eq!(identity.name, package);
        assert_eq!(identity.version.as_deref(), Some(version));
        assert_eq!(identity.resolution, "lockfile");
    }
    let mut custom = catalog.clone();
    custom.entries[0].packages.insert("composer".into(), vec!["example/ai-client".into()]);
    let found = manifest::scan("composer.lock", r#"{"packages":[],"packages-dev":[{"name":"example/ai-client","version":"1.2.3"}]}"#, &custom).unwrap();
    assert_eq!(found[0].package.as_ref().unwrap().scope, "development");
}

#[test]
fn invalid_lock_versions_fail_and_url_versions_are_not_disclosed() {
    let catalog = Catalog::load(None).unwrap();
    assert!(manifest::scan("package-lock.json", r#"{"lockfileVersion":999,"packages":{}}"#, &catalog).is_err());
    assert!(manifest::scan("uv.lock", "version=2\npackage=[]", &catalog).is_err());
    assert!(manifest::scan("pnpm-lock.yaml", "lockfileVersion: 123\npackages: {}", &catalog).is_err());
    let assets = manifest::scan("package-lock.json", r#"{"lockfileVersion":1,"dependencies":{"deepagents":{"version":"git+https://SECRET@host/path"}}}"#, &catalog).unwrap();
    assert_eq!(assets.len(), 1);
    assert!(assets[0].package.as_ref().unwrap().version.is_none());
    assert!(!serde_json::to_string(&assets).unwrap().contains("SECRET"));
}

#[test]
fn notebook_cells_have_distinct_identity_and_ignore_outputs_and_markdown() {
    let catalog = Catalog::load(None).unwrap();
    let cell = serde_json::json!({"cell_type":"code","source":["from deepagents import create_deep_agent\n", "a=create_deep_agent()\n"],"outputs":[{"text":"SECRET import openai"}]});
    let doc = serde_json::json!({"nbformat":4,"metadata":{"language_info":{"name":"python"}},"cells":[cell.clone(),cell,{"cell_type":"markdown","source":"from crewai import Agent\na=Agent()"}]});
    let (result, language) = notebook::scan("experiment.ipynb", &doc.to_string(), &catalog).unwrap();
    assert_eq!(language, "python");
    let agents: Vec<_> = result.assets.iter().filter(|a| a.kind == "agent").collect();
    assert_eq!(agents.len(), 2);
    assert_ne!(agents[0].id, agents[1].id);
    assert_eq!(agents[0].evidence[0].path, "experiment.ipynb");
    assert_eq!(agents[0].evidence[0].cell, Some(1));
    assert_eq!(agents[1].evidence[0].cell, Some(2));
    assert!(!serde_json::to_string(&result.assets).unwrap().contains("SECRET"));
    assert!(notebook::scan("x.ipynb", r#"{"nbformat":4,"metadata":{},"cells":[]}"#, &catalog).is_err());
}

#[test]
fn nested_application_scopes_inherit_generic_metadata_with_longest_root_winning() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("services/api")).unwrap();
    fs::write(temp.path().join("services/team.yaml"), "owner: Platform\nappid: parent\n").unwrap();
    fs::write(temp.path().join("services/api/info.yaml"), "appid: child\n").unwrap();
    fs::write(temp.path().join("services/api/app.py"), "import deepagents\n").unwrap();
    let config: pilotlight::indicator::Config = toml::from_str("[[applications]]\nroot='services/api'\n[[applications.indicators]]\nname='application_id'\npath='info.yaml'\nkey='appid'\n[[applications]]\nroot='services'\n[[applications.indicators]]\nname='owner'\npath='team.yaml'\nkey='owner'\n[[applications.indicators]]\nname='application_id'\npath='team.yaml'\nkey='appid'\n").unwrap();
    let report = scan::scan(&Source::open(temp.path().to_str().unwrap(), None).unwrap(), &Catalog::load(None).unwrap(), &Options { applications: config.applications, ..Default::default() }).unwrap();
    assert_eq!(report.assets[0].metadata["owner"], "Platform");
    assert_eq!(report.assets[0].metadata["application_id"], "child");
    assert_eq!(report.assets[0].application_root.as_deref(), Some("services/api"));
}
