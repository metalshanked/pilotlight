use pilotlight::{catalog::Catalog, manifest, syntax};

#[test]
fn newly_reviewed_python_semantics_resolve_aliases_and_abstain_on_decoys() {
    let catalog=Catalog::load(None).unwrap();
    let cases:Vec<serde_json::Value>=serde_json::from_str(include_str!("catalog-semantics.json")).unwrap();
    for case in cases {
        let source=case["source"].as_str().unwrap();
        let result=syntax::scan("app.py",source,"python",&catalog).unwrap();
        assert!(!result.parse_errors);
        assert!(result.assets.iter().any(|a|a.kind==case["kind"].as_str().unwrap() && a.name==case["symbol"].as_str().unwrap()),"{case}");
        let source=format!("{}\nselected = unrelated\nasset = selected()\n",source.lines().next().unwrap());
        let result=syntax::scan("app.py",&source,"python",&catalog).unwrap();
        assert!(!result.parse_errors);
        assert!(result.assets.iter().all(|a|a.kind=="dependency"),"shadowed rule {case}");
    }
}

#[test]
fn native_aliases_and_constructors_have_qualified_evidence() {
    let catalog=Catalog::load(None).unwrap();
    let cases=[
        ("go","main.go","package main\nimport api \"github.com/modelcontextprotocol/go-sdk/mcp\"\nfunc main(){ s := api.NewServer(nil,nil); _ = s }","mcp_server"),
        ("go","main.go","package main\nimport \"github.com/openai/openai-go/v3\"\nfunc main(){ c := openai.NewClient(); c.Chat.Completions.New(nil,nil) }","llm_call"),
        ("java","App.java","import io.modelcontextprotocol.client.McpClient; class App { void run() { var c = McpClient.sync(transport); } }","mcp_client"),
        ("csharp","App.cs","using OpenAI.Chat; class App { void Run() { var c = new ChatClient(\"model\", \"key\"); c.CompleteChat(\"hi\"); } }","llm_call"),
        ("csharp","App.cs","using Client = OpenAI.Chat.ChatClient; class App { void Run() { var c = new Client(\"model\", \"key\"); } }","llm_client"),
        ("rust","main.rs","use async_openai::{Client as AiClient}; fn main(){ let client = AiClient::new(); }","llm_client"),
        ("rust","main.rs","use rig::prelude::*; use rig::providers::openai::{self, OpenAI}; fn main(){ let agent = AgentBuilder::new(OpenAI::from_env()); }","agent"),
        ("java","App.java","import dev.langchain4j.model.openai.OpenAiChatModel; class App { void run() { var model = OpenAiChatModel.builder().apiKey(key).build(); } }","llm_client"),
        ("csharp","App.cs","using Microsoft.SemanticKernel.Agents; class App { void Run() { var agent = new ChatCompletionAgent(); } }","agent"),
        ("csharp","App.cs","using OpenAI.Chat; class App { void Run() { ChatClient client = new(\"m\", \"k\"); client.CompleteChat(\"hi\"); } }","llm_call"),
        ("php","app.php",r"<?php use PhpMcp\Server\Server as Mcp; $server = Mcp::make();","mcp_server"),
        ("php","app.php",r"<?php use PhpMcp\Server\{Server as Mcp}; $server = Mcp::make();","mcp_server"),
    ];
    for (language,path,source,kind) in cases {
        let result=syntax::scan(path,source,language,&catalog).unwrap();
        assert!(!result.parse_errors,"parse error {language}");
        assert!(result.assets.iter().any(|a|a.kind==kind),"missing {kind} in {language}: {:?}",result.assets);
    }
}

#[test]
fn split_mcp_packages_and_builder_roles_are_not_generic_name_matches() {
    let catalog=Catalog::load(None).unwrap();
    let source="import {McpServer as Server} from '@modelcontextprotocol/server'; import {Client} from '@modelcontextprotocol/client'; const server = new Server({name:'sample'}); const client = new Client({name:'sample'});";
    let result=syntax::scan("app.ts",source,"typescript",&catalog).unwrap();
    assert_eq!(result.assets.iter().filter(|a|a.kind=="mcp_server").count(),1);
    assert_eq!(result.assets.iter().filter(|a|a.kind=="mcp_client").count(),1);
    let source="import dev.langchain4j.model.openai.OpenAiChatModel; class App { void run() { OpenAiChatModel.builder(); Other.builder(); } }";
    let result=syntax::scan("App.java",source,"java",&catalog).unwrap();
    let clients:Vec<_>=result.assets.iter().filter(|a|a.kind=="llm_client").collect();
    assert_eq!(clients.len(),1);
    assert_eq!(clients[0].subtype.as_deref(),Some("configuration_builder"));
}

#[test]
fn large_flat_pnpm_lockfiles_fit_the_separate_budget_but_aliases_are_rejected() {
    let catalog=Catalog::load(None).unwrap();
    let mut source=String::from("lockfileVersion: '9.0'\npackages:\n  openai@1.0.0: {}\n");
    for n in 0..15000 {source.push_str(&format!("  neutral{n}@1.0.0:\n    resolution:\n      integrity: sha512-placeholder\n"));}
    let assets=pilotlight::lockfile::scan("pnpm-lock.yaml",&source,&catalog).unwrap();
    assert_eq!(assets.len(),1);
    assert_eq!(assets[0].framework.as_deref(),Some("openai"));
    assert!(pilotlight::lockfile::scan("pnpm-lock.yaml","lockfileVersion: '9.0'\npackages: &p {}\nsnapshots: *p\n",&catalog).is_err());
}

#[test]
fn native_shadowing_and_generic_calls_do_not_invent_ai_assets() {
    let catalog=Catalog::load(None).unwrap();
    for (lang,path,source) in [
        ("go","app.go","package main\nimport api \"github.com/modelcontextprotocol/go-sdk/mcp\"\nfunc f(api interface{}) { api.NewServer() }"),
        ("rust","app.rs","use async_openai::Client; fn f(){ let Client = other; Client::new(); }"),
        ("php","app.php",r"<?php class Mcp { static function make() {} } $server = Mcp::make();"),
        ("java","App.java","class McpClient { void run() { sync(transport); } }"),
    ] {
        let result=syntax::scan(path,source,lang,&catalog).unwrap();
        assert!(result.assets.iter().all(|a|a.kind=="dependency"),"unqualified {lang}: {:?}",result.assets);
    }
}

#[test]
fn composer_pep735_and_central_nuget_manifests_are_supported() {
    let catalog=Catalog::load(None).unwrap();
    for (path,source,expected) in [
        ("composer.json",r#"{"require":{"php-mcp/server":"^3"},"require-dev":{"prism-php/prism":"*"},"description":"openai-php/client"}"#,2),
        ("pyproject.toml","[dependency-groups]\nwork = ['datasets>=1', 'wandb', {include-group='dev'}]\n",2),
        ("Directory.Packages.props",r#"<Project><ItemGroup><PackageVersion Include="OpenAI" Version="2" /></ItemGroup></Project>"#,1),
        ("packages.config",r#"<packages><package id="OpenAI" version="2" /></packages>"#,1),
    ] {
        assert!(manifest::supported(std::path::Path::new(path)));
        assert_eq!(manifest::scan(path,source,&catalog).unwrap().len(),expected,"{path}");
    }
}

#[test]
fn catalog_identities_are_exact_and_provenance_is_present_for_additions() {
    let catalog=Catalog::load(None).unwrap();
    assert!(catalog.entries.len()>=170);
    let evidence:serde_json::Value=serde_json::from_str(include_str!("../catalog/discovery-evidence.json")).unwrap();
    for record in evidence["records"].as_array().unwrap() {
        let ecosystem=record["ecosystem"].as_str().unwrap();let package=record["package"].as_str().unwrap();
        let entry=catalog.package(ecosystem,package).unwrap();
        assert!(!entry.sources.is_empty());
        assert!(catalog.package(ecosystem,&format!("{package}-not-the-sdk")).is_none());
    }
}

#[test]
fn benchmark_rejects_unpinned_and_duplicate_targets_before_acquisition() {
    let temp=tempfile::tempdir().unwrap();let path=temp.path().join("corpus.json");
    std::fs::write(&path,r#"{"schema_version":1,"repositories":[{"repository":"https://github.com/org/repo","commit":"main"}]}"#).unwrap();
    assert!(pilotlight::benchmark::targets(&path).is_err());
    let row=serde_json::json!({"repository":"https://github.com/org/repo","commit":"a".repeat(40)});
    std::fs::write(&path,serde_json::to_vec(&serde_json::json!({"schema_version":1,"repositories":[row,row]})).unwrap()).unwrap();
    assert!(pilotlight::benchmark::targets(&path).is_err());
}

#[test]
fn acquisition_bounds_are_explicit_and_omissions_remain_visible() {
    use pilotlight::{source::{Source,AcquisitionLimits},scan::{self,Options},model::Diagnostic};
    assert!(AcquisitionLimits::default().validate().is_ok());
    assert!(AcquisitionLimits {max_download_bytes:0,max_expanded_bytes:1}.validate().is_err());
    assert!(AcquisitionLimits {max_download_bytes:1,max_expanded_bytes:u64::MAX}.validate().is_err());
    let temp=tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("app.py"),"from deepagents import create_deep_agent\na=create_deep_agent()\n").unwrap();
    let mut source=Source::open(temp.path().to_str().unwrap(),None).unwrap();
    source.omissions.push(Diagnostic {path:"link.py".into(),reason:"archive link omitted without following its target".into()});
    let report=scan::scan(&source,&Catalog::load(None).unwrap(),&Options::default()).unwrap();
    assert_eq!(report.status,"partial");
    assert_eq!(report.coverage.files_skipped,1);
    assert!(report.coverage.diagnostics.iter().any(|d|d.path=="link.py"));
    assert!(report.assets.iter().any(|a|a.kind=="agent"));
}
