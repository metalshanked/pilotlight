use pilotlight::{catalog::Catalog, syntax, packs::{self,Pack}, model::digest};
use serde_json::json;
fn count(source:&str)->usize {syntax::scan("app.py",source,"python",&Catalog::load(None).unwrap()).unwrap().assets.iter().filter(|a|a.kind=="model").count()}
const RECORD:&str="from dataclasses import dataclass\n@dataclass\nclass Settings:\n    label: str\n    model_name: str\n";
#[test]
fn local_model_record_maps_fields_and_aliases_without_class_name_heuristics() {
    assert_eq!(count(&format!("{RECORD}x=Settings('demo','gpt-4o')\ny=Settings(model_name='gpt-4.1',label='other')\n")),2);
    assert_eq!(count(&format!("{RECORD}Alias=Settings\nx=Alias('demo','gpt-4o')\n")),1);
    assert_eq!(count("import dataclasses as dc\n@dc.dataclass()\nclass Configuration:\n model: str\nx=Configuration('gpt-4o')\n"),1);
    assert_eq!(count(include_str!("../benchmarks/pack-corpus/cases/model-spec-positional/app.py")),1);
}
#[test]
fn local_model_records_reject_unrelated_values_shadowing_and_ambiguous_constructors() {
    assert_eq!(count(&format!("{RECORD}instance=Settings('demo','gpt-4o')\ninstance('demo','gpt-4o')\n")),1);
    for suffix in ["x=Settings('gpt-4o','sedan')", "x=Settings('demo',unknown)", "Settings=other\nx=Settings('demo','gpt-4o')", "x=Settings(*values)", "x=Settings('demo','gpt-4o',model_name='gpt-4o')", "x=Settings('demo','private-unknown-model')", "def f(Settings):\n x=Settings('demo','gpt-4o')"] {
        assert_eq!(count(&format!("{RECORD}{suffix}\n")),0,"{suffix}");
    }
    for code in [
        "class ModelSpec:\n model: str\nx=ModelSpec('gpt-4o')\n",
        "from other import dataclass\n@dataclass\nclass ModelSpec:\n model: str\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\ndataclass=other\n@dataclass\nclass ModelSpec:\n model: str\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\n@dataclass(init=False)\nclass ModelSpec:\n model: str\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\n@dataclass\nclass ModelSpec(Base):\n model: str\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\n@dataclass\nclass ModelSpec:\n model: str\n def __init__(self, model): pass\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\nfrom typing import ClassVar\n@dataclass\nclass ModelSpec:\n model: ClassVar[str]\nx=ModelSpec('gpt-4o')\n",
        "from dataclasses import dataclass\n@dataclass\nclass ModelSpec:\n model: str\nx=ModelSpec(f'gpt-4o')\n",
    ] {assert_eq!(count(code),0,"{code}");}
}
#[test]
fn model_identity_catalog_is_extensible_and_retains_pack_attribution() {
    let value=json!({"schema_version":1,"id":"company.models","version":"0.1.0","minimum_engine_version":"0.1.0","description":"Private model identifiers","license":"Apache-2.0","sources":["https://example.com/models"],"requires":["model-records-v1"],"rules":[{"id":"private-model","entry":{"id":"private-model","name":"Private Model","category":"model","homepage":"https://example.com/models","imports":{},"packages":{},"model_identifiers":["internal/model-v3"]}}]});
    let bytes=serde_json::to_vec(&value).unwrap();let pack=Pack::parse(&bytes).unwrap();let info=packs::PackInfo{id:pack.id.clone(),version:pack.version.clone(),sha256:digest(&bytes)};
    let mut c=Catalog::load(None).unwrap();packs::apply(&mut c,&pack,info).unwrap();
    let code=format!("{RECORD}x=Settings('demo','internal/model-v3')\n");assert_eq!(count(&code),0);
    let result=syntax::scan("app.py",&code,"python",&c).unwrap();let mut model=result.assets.into_iter().find(|a|a.kind=="model").unwrap();packs::annotate(&c,&mut model);
    assert_eq!(model.name,"internal/model-v3");assert_eq!(model.detection_rules[0].rule_id,"private-model");
    let mut invalid=value;invalid["rules"][0]["entry"]["model_identifiers"]=json!(["contains a secret / query?token=bad"]);assert!(Pack::parse(&serde_json::to_vec(&invalid).unwrap()).is_err());
}
#[test]
fn imported_model_constructor_selectors_are_generic_and_reject_ambiguity() {
    let mut c=Catalog::load(None).unwrap();
    let value=json!({"schema_version":1,"id":"company.records","version":"0.1.0","minimum_engine_version":"0.1.0","description":"Private record constructor","license":"Apache-2.0","sources":["https://example.com/models"],"requires":["model-arguments-v1"],"rules":[{"id":"private-record","entry":{"id":"private-record","name":"Private record","category":"model","homepage":"https://example.com/models","imports":{"python":["company_specs"]},"packages":{},"model_identifiers":["private/model"],"model_arguments":{"company_specs.Setup":{"position":2,"keywords":["deployment"]}}}}]});
    let bytes=serde_json::to_vec(&value).unwrap();let pack=Pack::parse(&bytes).unwrap();packs::apply(&mut c,&pack,packs::PackInfo{id:pack.id.clone(),version:pack.version.clone(),sha256:digest(&bytes)}).unwrap();
    for (call,want) in [("Setup('x','y','private/model')",1),("Setup(deployment='private/model')",1),("Setup('private/model')",0),("Setup(*args)",0),("Setup('x','y','private/model',deployment='private/model')",0),("Setup(deployment='unknown')",1),("Setup(deployment='white space')",0),("Setup(deployment='')",0)] {
        let code=format!("from company_specs import Setup\nx={call}\n");let result=syntax::scan("app.py",&code,"python",&c).unwrap();assert_eq!(result.assets.iter().filter(|a|a.kind=="model").count(),want,"{call}");
    }
    assert_eq!(count("from dataclasses import dataclass\n@dataclass(frozen=True)\nclass Options:\n model: str\n def validate(self): pass\nx=Options('gpt-4o')\n"),1);
}
#[test]
fn ordinary_class_decorators_keep_their_existing_evidence() {
    let c=Catalog::load(None).unwrap();let source="from crewai.flow.persistence import persist\n@persist()\nclass Workflow:\n    pass\n";
    let r=syntax::scan("app.py",source,"python",&c).unwrap();assert!(r.assets.iter().any(|a|a.evidence.iter().any(|e|e.mechanism=="ast_call" && e.symbol=="crewai.flow.persistence.persist")));
}
