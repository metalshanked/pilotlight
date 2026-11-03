use pilotlight::{catalog::Catalog, syntax};

#[test]
fn kotlin_import_aliases_and_shadowing_are_structural() {
    let catalog=Catalog::load(None).unwrap();
    for source in [
        "import dev.langchain4j.model.openai.OpenAiChatModel as Model\nfun main() { val client = Model.builder() }\n",
        "import org.springframework.ai.chat.client.ChatClient\nclass App {\n fun run() { val client = ChatClient.builder(model) }\n}\n",
    ] {
        let result=syntax::scan("App.kt",source,"kotlin",&catalog).unwrap();
        assert!(!result.parse_errors);
        assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_client").count(),1,"{:?}",result.assets);
    }
    for source in [
        "import dev.langchain4j.model.openai.OpenAiChatModel as Model\nfun main(Model: Any) { Model.builder() }\n",
        "import dev.langchain4j.model.openai.OpenAiChatModel as Model\nfun main() { val Model = unrelated; Model.builder() }\n",
        "import dev.langchain4j.model.openai.*\nfun main() { OpenAiChatModel.builder() }\n",
    ] {
        let result=syntax::scan("App.kt",source,"kotlin",&catalog).unwrap();
        assert!(!result.assets.iter().any(|a|a.kind=="llm_client"),"{:?}",result.assets);
    }
}

#[test]
fn ruby_qualified_clients_and_operations_abstain_on_shadowed_constants() {
    let catalog=Catalog::load(None).unwrap();
    let result=syntax::scan("app.rb","chat = RubyLLM.chat(model: 'sample')\nchat.ask('hello')\nchat.files()\n", "ruby", &catalog).unwrap();
    assert!(!result.parse_errors);
    assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_client").count(),1);
    assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_call").count(),1);
    assert_eq!(result.assets.iter().filter(|a|a.kind=="model" && a.name=="sample").count(),1);
    assert!(pilotlight::artifact::capabilities(&catalog,"model").contains_key("ruby"));
    assert!(result.assets.iter().filter(|a|a.kind=="llm_client" || a.kind=="llm_call").all(|a|a.confidence=="qualified_symbol"));
    let chained=syntax::scan("app.rb","RubyLLM.chat.ask('hello')\n", "ruby", &catalog).unwrap();
    assert_eq!(chained.assets.iter().filter(|a|a.kind=="llm_call").count(),1);
    for source in ["class RubyLLM; end\nRubyLLM.chat\n", "RubyLLM = unrelated\nRubyLLM.chat\n", "chat = unrelated\nchat.ask('hello')\n", "# RubyLLM.chat\ntext = 'RubyLLM.chat'\n"] {
        assert!(!syntax::scan("app.rb",source,"ruby",&catalog).unwrap().assets.iter().any(|a|a.kind=="llm_client" || a.kind=="llm_call"));
    }
}

#[test]
fn csharp_static_factories_and_lifecycle_rules_have_typed_evidence() {
    let catalog=Catalog::load(None).unwrap();
    for assignment in ["client := api.NewClient(base, transport)", "client, err := api.ClientFromEnvironment()"] {
        let source=format!("package main\nimport api \"github.com/ollama/ollama/api\"\nfunc main() {{ {assignment}; client.Chat(ctx, req, callback); client.List(ctx) }}\n");
        let result=syntax::scan("main.go",&source,"go",&catalog).unwrap();
        assert!(!result.parse_errors);
        assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_client").count(),1);
        assert_eq!(result.assets.iter().filter(|a|a.kind=="llm_call").count(),1);
    }
    let source="package main\nimport api \"github.com/ollama/ollama/api\"\nfunc main() { _, client := api.ClientFromEnvironment(); client.Chat(ctx, req, callback) }\n";
    assert!(!syntax::scan("main.go",source,"go",&catalog).unwrap().assets.iter().any(|a|a.kind=="llm_call"));
    let result=syntax::scan("App.cs","using ModelContextProtocol.Client; class App { async void Run() { var c = await McpClient.CreateAsync(transport); Other.CreateAsync(); } }", "csharp", &catalog).unwrap();
    assert_eq!(result.assets.iter().filter(|a|a.kind=="mcp_client").count(),1);
    for (source,kind) in [
        ("from feast import FeatureStore as Store\nstore = Store(repo_path='.')\n","feature_store"),
        ("import mlflow\nmlflow.start_run()\n","experiment_tracker"),
        ("import mlflow\nmlflow.register_model('runs:/example','sample')\n","model_registry"),
        ("from dvc.api import DVCFileSystem\nfs = DVCFileSystem('.')\n","data_versioning"),
    ] {
        let result=syntax::scan("app.py",source,"python",&catalog).unwrap();
        assert_eq!(result.assets.iter().filter(|a|a.kind==kind).count(),1,"{source}");
    }
}

#[test]
fn corpus_discovered_packages_are_exact_and_have_evidence() {
    let catalog=Catalog::load(None).unwrap();
    let evidence:serde_json::Value=serde_json::from_str(include_str!("../catalog/gap-discovery-evidence.json")).unwrap();
    for row in evidence["records"].as_array().unwrap() {
        let ecosystem=row["ecosystem"].as_str().unwrap();let name=row["package"].as_str().unwrap();
        assert!(catalog.package(ecosystem,name).is_some(),"{name}");
        assert!(catalog.package(ecosystem,&format!("{name}-unrelated")).is_none());
    }
}
