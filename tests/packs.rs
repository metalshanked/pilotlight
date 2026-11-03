use pilotlight::{catalog::Catalog, packs::{self, Pack}, model::digest, scan::{self, Options}, source::Source, syntax};
use serde_json::{json, Value};
use std::{fs, process::Command};
fn company() -> Value { json!({
    "schema_version":1,"id":"company.agents","version":"0.1.0","minimum_engine_version":"0.1.0",
    "description":"Example private SDK", "license":"Apache-2.0","sources":["https://example.com/sdk"],"requires":["catalog-v1","configured-loops-v1"],
    "rules":[{"id":"company-sdk","entry":{
        "id":"company","name":"Company SDK","category":"agent_framework","homepage":"https://example.com/sdk",
        "packages":{"pypi":["company-agents"]},"imports":{"python":["company_agents"],"typescript":["@company/agents"],"rust":["company_macros"],"csharp":["Company.Extensions"]},
        "symbols":{"company_agents.Agent":"agent","@company/agents.run":"llm_call","company_macros::expose":"tool","Company.Extensions.ServiceExtensions.AddCompanyAgent":"agent"},
        "configured_loops":{"@company/agents.run":{"tools_argument":"functions","count_argument":"turns","stop_argument":"until","stop_functions":["@company/agents.limit"]}},
        "rust_attributes":{"company_macros::expose":{"target":"impl_method"}},
        "di_registrations":["Company.Extensions.ServiceExtensions.AddCompanyAgent"]
    }}],"fixtures":[{"id":"agent","language":"python","source":"from company_agents import Agent\na=Agent()\n","expect":{"agent":1}}]
}) }
fn file(value: &Value, path: &std::path::Path) { fs::write(path,serde_json::to_vec_pretty(value).unwrap()).unwrap(); }
#[test]
fn bundled_packs_have_executable_positive_and_negative_fixtures() {
    for (pack,info) in packs::builtins().unwrap() {
        let results=packs::test(&pack,info).unwrap();
        assert!(results.len()>=10);
        for result in results {assert!(result.passed,"{}: {:?}",pack.id,result);}
    }
}
#[test]
fn new_sdk_and_structural_mappings_work_without_engine_changes() {
    let d=tempfile::tempdir().unwrap();let path=d.path().join("company.json");file(&company(),&path);
    let catalog=packs::load(None,&[path],false).unwrap();
    for (lang,code,kind) in [
        ("python","from company_agents import Agent as Maker\na=Maker()\n","agent"),
        ("typescript","import {run,limit} from '@company/agents'; run({functions:{lookup},until:limit(3)});","agent"),
        ("rust","use company_macros::expose; impl Demo { #[expose] fn lookup(&self) {} }","tool"),
        ("csharp","using Microsoft.Extensions.DependencyInjection; using Company.Extensions; var services=new ServiceCollection(); services.AddCompanyAgent();","agent"),
    ] {
        let result=syntax::scan("app",code,lang,&catalog).unwrap();assert!(!result.parse_errors);
        assert_eq!(result.assets.iter().filter(|a|a.kind==kind).count(),1,"{code}");
    }
    for (lang,code) in [
        ("python","from company_agents import Agent\nAgent=other\na=Agent()\n"),
        ("typescript","import {run} from '@company/agents'; run({functions:{lookup},turns:1});"),
        ("csharp","using Microsoft.Extensions.DependencyInjection; var services=new ServiceCollection(); services.AddCompanyAgent();"),
    ] {assert_eq!(syntax::scan("app",code,lang,&catalog).unwrap().assets.iter().filter(|a|a.kind=="agent").count(),0,"{code}");}
}
#[test]
fn composition_is_order_independent_conflicts_fail_and_application_is_atomic() {
    let d=tempfile::tempdir().unwrap();let a=d.path().join("a.json");let b=d.path().join("b.json");file(&company(),&a);
    let mut second=company();second["id"]=json!("company.more");second["rules"][0]["entry"]["symbols"]["company_agents.Other"]=json!("agent");file(&second,&b);
    assert_eq!(packs::load(None,&[a.clone(),b.clone()],false).unwrap().fingerprint(),packs::load(None,&[b.clone(),a.clone()],false).unwrap().fingerprint());
    assert!(packs::load(None,&[a.clone(),a.clone()],false).is_err());
    second["rules"][0]["entry"]["symbols"]["company_agents.Agent"]=json!("mcp_server");file(&second,&b);
    let mut catalog=packs::load(None,&[a],false).unwrap();let before=catalog.fingerprint();let (pack,info)=packs::read(&b).unwrap();
    assert!(packs::apply(&mut catalog,&pack,info).is_err());assert_eq!(catalog.fingerprint(),before);
}
#[test]
fn invalid_and_incompatible_packs_are_rejected_before_scan() {
    for (key,value) in [("schema_version",json!(99)),("minimum_engine_version",json!("99.0.0")),("requires",json!(["execute-python"])),("id",json!("../escape")),("version",json!("latest")),("unknown",json!(true))] {
        let mut p=company();p[key]=value;assert!(Pack::parse(&serde_json::to_vec(&p).unwrap()).is_err(),"{key}");
    }
    let mut p=company();p["rules"][0]["entry"]["symbols"]["company_agents.Agent"]=json!("invented_type");assert!(Pack::parse(&serde_json::to_vec(&p).unwrap()).is_err());
    assert!(Pack::parse(&vec![b' ';8*1024*1024+1]).is_err());
}
#[test]
fn selection_and_reports_preserve_pack_identity_and_application_metadata() {
    let d=tempfile::tempdir().unwrap();let path=d.path().join("pack.json");file(&company(),&path);
    fs::write(d.path().join("app.py"),"from company_agents import Agent\na=Agent()\n").unwrap();
    let source=Source::open(d.path().to_str().unwrap(),None).unwrap();
    let base=Catalog::load(None).unwrap();assert!(!scan::scan(&source,&base,&Options::default()).unwrap().assets.iter().any(|a|a.framework.as_deref()==Some("company")));
    let catalog=packs::load(None,&[path.clone()],false).unwrap();let report=scan::scan(&source,&catalog,&Options::default()).unwrap();
    let asset=report.assets.iter().find(|a|a.kind=="agent").unwrap();assert_eq!(asset.detection_rules.len(),1);assert_eq!(asset.detection_rules[0].rule_id,"company-sdk");
    assert_eq!(asset.detection_rules[0].pack.sha256,digest(&fs::read(path).unwrap()));assert_eq!(report.detection_packs.len(),3);
    let reduced=packs::load(None,&[],true).unwrap();assert!(reduced.entries.iter().all(|e|e.id!="deepagents"));assert!(reduced.packs.is_empty());
    let mut csv=vec![];pilotlight::output::write(&report,pilotlight::output::Format::Csv,&mut csv).unwrap();assert!(String::from_utf8(csv).unwrap().contains("company.agents"));
}
#[test]
fn pinned_install_and_config_relative_cli_loading_are_safe() {
    let d=tempfile::tempdir().unwrap();let path=d.path().join("pack.json");file(&company(),&path);let destination=d.path().join("installed.json");
    assert!(packs::install(&path,&"0".repeat(64),&destination).is_err());assert!(!destination.exists());
    let sha=digest(&fs::read(&path).unwrap());packs::install(&path,&sha,&destination).unwrap();assert!(packs::install(&path,&sha,&destination).is_err());
    fs::write(d.path().join("app.py"),"from company_agents import Agent\na=Agent()\n").unwrap();
    fs::write(d.path().join("app.json"),"{\"appid\":\"0007\"}").unwrap();
    let config=d.path().join("operator.toml");fs::write(&config,"[packs]\npaths=['pack.json']\ndisable_defaults=true\n[[indicators]]\nname='application_id'\npath='app.json'\nkey='appid'\nrequired=true\n").unwrap();
    let output=Command::new(env!("CARGO_BIN_EXE_pilotlight")).args(["scan",d.path().to_str().unwrap(),"--config",config.to_str().unwrap(),"--format","json"]).output().unwrap();assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    let report:Value=serde_json::from_slice(&output.stdout).unwrap();assert_eq!(report["detection_packs"].as_array().unwrap().len(),1);
    let asset=report["assets"].as_array().unwrap().iter().find(|a|a["kind"]=="agent").unwrap();assert_eq!(asset["metadata"]["application_id"],"0007");assert_eq!(asset["detection_rules"][0]["pack"]["id"],"company.agents");
}
