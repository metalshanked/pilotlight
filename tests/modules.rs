use pilotlight::{artifact::ScanConfig, catalog::Catalog, scan::{self, Options}, source::Source};
use std::fs;

fn scan_files(files: &[(&str, &str)]) -> pilotlight::model::Report {
    let temp = tempfile::tempdir().unwrap();
    for (path, source) in files {
        let target = temp.path().join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, source).unwrap();
    }
    scan::scan(&Source::open(temp.path().to_str().unwrap(), None).unwrap(), &Catalog::load(None).unwrap(), &Options { selection: ScanConfig { types: Some(vec!["agent".into(), "model".into()]), ..Default::default() }, ..Default::default() }).unwrap()
}

#[test]
fn namespace_imports_resolve_reviewed_exports_without_trusting_local_names() {
    let report = scan_files(&[
        ("factory.py", "from deepagents import create_deep_agent as make\ndef build():\n return make()\n"),
        ("app.py", "import factory as api\na = api.build()\napi = unrelated\nb = api.build()\n"),
    ]);
    let callers: Vec<_> = report.assets.iter().filter(|a|a.kind=="agent" && a.evidence[0].path=="app.py").collect();
    assert_eq!(callers.len(),1);
    assert_eq!(callers[0].confidence,"inferred_from_wrapper");
    assert!(callers[0].evidence[0].resolution_path.len()>=2);
}

#[test]
fn generic_reexports_and_factory_wrappers_have_cross_file_evidence() {
    let report = scan_files(&[
        ("factories.py", "from deepagents import create_deep_agent as factory\nfrom crewai import Agent as CrewAgent\ndef build():\n return factory(model='internal-model')\n"),
        ("bridge.py", "from factories import build as make\n"),
        ("app.py", "from bridge import make\nfrom factories import CrewAgent\na = make(model='not-forwarded')\nb = CrewAgent()\n"),
    ]);
    let callers: Vec<_> = report.assets.iter().filter(|a| a.kind == "agent" && a.evidence[0].path == "app.py").collect();
    assert_eq!(callers.len(), 2);
    assert!(callers.iter().any(|a| a.framework.as_deref() == Some("deepagents") && a.confidence == "inferred_from_wrapper"));
    assert!(callers.iter().any(|a| a.framework.as_deref() == Some("crewai") && a.confidence == "inferred_from_module"));
    assert!(callers.iter().all(|a| a.evidence[0].resolution_path.len() >= 2));
    assert!(!report.assets.iter().any(|a| a.kind == "model" && a.name == "not-forwarded"));
    assert!(report.assets.iter().any(|a| a.kind == "model" && a.name == "internal-model"));
}

#[test]
fn relative_imports_resolve_and_cycles_remain_partial() {
    let report = scan_files(&[
        ("pkg/__init__.py", ""),
        ("pkg/factories.py", "from deepagents import create_deep_agent as make\n"),
        ("pkg/app.py", "from .factories import make as build\na=build()\n"),
        ("cycle_a.py", "from cycle_b import make\n"),
        ("cycle_b.py", "from cycle_a import make\n"),
        ("bad.py", "from cycle_a import make\nx=make()\n"),
    ]);
    assert_eq!(report.assets.iter().filter(|a| a.kind == "agent").count(), 1);
    assert!(report.assets.iter().any(|a| a.evidence[0].path == "pkg/app.py"));
    assert_eq!(report.status, "partial");
    assert!(report.coverage.diagnostics.iter().any(|d| d.path == "bad.py"));
}

#[test]
fn local_shadow_modules_and_reassigned_wrapper_globals_do_not_establish_agents() {
    let report = scan_files(&[
        ("deepagents.py", "def create_deep_agent():\n return object()\n"),
        ("app.py", "from deepagents import create_deep_agent\na=create_deep_agent()\n"),
        ("namespace.py", "import deepagents\na=deepagents.create_deep_agent()\n"),
    ]);
    assert!(report.assets.is_empty());
    assert_eq!(report.status, "partial");
    let report = scan_files(&[
        ("factory.py", "from deepagents import create_deep_agent as make\ndef build():\n return make()\nmake = other\n"),
        ("app.py", "from factory import build\na=build()\n"),
    ]);
    assert!(!report.assets.iter().any(|a| a.evidence[0].path == "app.py"));
}

#[test]
fn complex_or_shadowed_wrappers_abstain() {
    for body in [
        "def build(make):\n return make()\n",
        "def build():\n if condition:\n  return make()\n return other()\n",
        "@decorator\ndef build():\n return make()\n",
        "async def build():\n return make()\n",
    ] {
        let factory = format!("from deepagents import create_deep_agent as make\n{body}");
        let report = scan_files(&[("factory.py", &factory), ("app.py", "from factory import build\na=build()\n")]);
        assert!(!report.assets.iter().any(|a| a.evidence[0].path == "app.py"), "{body}");
    }
}

#[test]
fn changed_source_does_not_reuse_stale_tree_byte_ranges() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("app.py");
    fs::write(&path, "from deepagents import create_deep_agent\na=create_deep_agent()\n").unwrap();
    let index = pilotlight::modules::ModuleIndex::build(temp.path(), &[path], 1000);
    assert!(index.tree("app.py").is_some());
    let result = pilotlight::syntax::scan_with_modules("app.py", "x=1\n", "python", &Catalog::load(None).unwrap(), Some(&index)).unwrap();
    assert!(result.assets.is_empty());
    assert!(result.unresolved_modules);
}

#[test]
fn model_selectors_preserve_reexports_but_do_not_forward_wrapper_arguments() {
    let report = scan_files(&[
        ("factory.py", "from deepagents_code.model_config import ModelSpec as Record\ndef build(provider, model):\n return Record('openai','actual-model')\n"),
        ("app.py", "from factory import build, Record\na=build('ignored','not-forwarded')\nb=Record('private','direct-model')\n"),
    ]);
    assert!(!report.assets.iter().any(|a| a.kind == "model" && a.name == "not-forwarded"));
    assert!(report.assets.iter().any(|a| a.kind == "model" && a.name == "actual-model"));
    let direct=report.assets.iter().find(|a| a.kind == "model" && a.name == "direct-model").unwrap();
    assert!(direct.evidence.iter().any(|e| !e.resolution_path.is_empty()));
}
