use pilotlight::{catalog::Catalog, manifest, syntax};

#[test]
fn independent_provider_packages_have_exact_manifest_matches() {
    let catalog = Catalog::load(None).unwrap();
    let python = manifest::scan("requirements.txt", "together>=1\nopenrouter\nreplicate\ntogether-fake\n", &catalog).unwrap();
    assert_eq!(python.len(), 3);
    let npm = manifest::scan("package.json", r#"{"dependencies":{"together-ai":"*","@openrouter/sdk":"*","@openrouter/agent":"*","replicate":"*","replicate-fake":"*"}}"#, &catalog).unwrap();
    assert_eq!(npm.len(), 4);
}

#[test]
fn new_provider_calls_resolve_aliases_without_classifying_unrelated_send_calls() {
    let catalog = Catalog::load(None).unwrap();
    let py = "from together import Together as Client\na = Client()\na.chat.completions.create(model='example-model')\nfrom openrouter import OpenRouter\nb = OpenRouter()\nb.chat.send(model='other-model')\nb.files.send()\n";
    let result = syntax::scan("a.py", py, "python", &catalog).unwrap();
    assert_eq!(result.assets.iter().filter(|a| a.kind == "llm_call").count(), 2);
    let ts = "import { OpenRouter as Router } from '@openrouter/sdk';\nconst api = new Router();\napi.chat.send({model: 'example-model'});\napi.files.send();\n";
    let result = syntax::scan("a.ts", ts, "typescript", &catalog).unwrap();
    assert_eq!(result.assets.iter().filter(|a| a.kind == "llm_call").count(), 1);
    // Replicate hosts non-LLM models too: import evidence is valid, blanket LLM inference is not.
    let result = syntax::scan("r.py", "import replicate\nreplicate.run('image-model')\n", "python", &catalog).unwrap();
    assert!(result.assets.iter().any(|a| a.kind == "dependency"));
    assert!(!result.assets.iter().any(|a| a.kind == "llm_call"));
}
