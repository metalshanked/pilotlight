# Comprehensive coverage program

Current progress: [0.1 implementation status](IMPLEMENTATION_STATUS.md). This document retains the broader target contract; unimplemented requirements are not completion claims.

Status: production specification and discovery backlog, 2026-10-10. This is not the runtime catalog or a claim of implemented support. The maintained overlay has 192 entries plus a separately derived seed index, including Deep Agents; broader qualification remains open. Pilot Light aims for broad, independently maintained AI inventory with explicit evidence, not a finite claim to detect every AI system.

## The coverage unit

Measure an exact ecosystem/package/version range, language, artifact type and detection mechanism together. A family such as LangChain is not a package; a package dependency is not an instantiated agent; an import is not a runtime call. A model family name is not proof of deployment. Keep all four distinctions in reports and release notes.

Publish five independent views: discovery backlog; verified package coordinates; import/config recognition; resolved artifact construction/calls; benchmark-qualified coverage. A package may be present in one and missing from another. Separate catalog support from parser support and from measured accuracy. Never count parameter rows, aliases, transitive packages or generated signatures as additional supported frameworks.

The Cisco reference audit found 1,906,503 rows, including 1,335,355 parameter rows. See [the audit and its exact snapshot](CATALOG_POLICY.md). This motivates independent normalization; it does not establish that any particular package is absent from Cisco's current product. Its own use of a framework also does not prove it can inventory that framework.

## Deep Agents: required first gap closure

Pilot Light 0.1 includes initial Deep Agents factories with regression fixtures. Official sources verify Python `deepagents` / `create_deep_agent` and npm `deepagents` / `createDeepAgent`. The JS project also documents `deepagents/browser` and `deepagents/node` entrypoints. Sources: [Python SDK](https://github.com/langchain-ai/deepagents), [Python manifest](https://github.com/langchain-ai/deepagents/blob/main/libs/deepagents/pyproject.toml), [JavaScript SDK](https://github.com/langchain-ai/deepagentsjs). The implemented mappings are described in the implementation status; the qualification checklist below remains the broader target.

Required implementation ticket PL-COV-001:

1. Record the PyPI/npm coordinates separately, public import paths, version intervals and source revision. Verify terminal/partner packages independently; don't guess renamed packages or assign every family package the SDK's semantics.
2. Detect qualified factory calls as `agent`, including import aliases and JS server/browser entrypoints. Resolve supported wrappers and re-exports using the bounded graph. An unused import stays dependency/import evidence.
3. Extract explicitly configured model, tools, subagents, memory/backend and skill-path references only when supported by the source. Distinguish subagent declarations from remote `agent_proxy` references. Do not invent built-in assets just because a library can provide them.
4. Test aliases, qualified calls, missing dependencies, shadowed names, local functions with the same factory name, strings/comments, import-only cases, generated/test/example roles and unresolved dynamic configuration.
5. Verify type-selected JSON/CSV plus application attribution. Add positive and negative fixtures before enabling semantic coverage. Test the pinned Python/JS repositories after gold labeling; keep related LangChain projects in the same evaluation family.

Both official projects are pinned in [the benchmark manifest](../benchmarks/corpus.lock.json). Their source is now part of the fixed public development runs. Package recognition is the first step, not completion of this ticket.

## Discovery workstreams

The names below are candidates to research and verify, not assertions that every project is maintained, supported or absent. Some already have partial prototype mappings. Exact coordinates, versions, provenance and evidence must be verified before promotion. This list seeds the registry; it is not the denominator for a claim of universal coverage.

| Workstream | Candidate families and sources of gaps | Artifact mapping to validate |
|---|---|---|
| Agent harnesses and orchestration | Deep Agents, LangChain/LangGraph, CrewAI, AutoGen/Microsoft Agent Framework, Semantic Kernel, OpenAI Agents, Claude Agent SDK, Google ADK, Strands, Pydantic AI, Agno, smolagents, Mastra, Haystack, LlamaIndex, DSPy | agent, tool, memory, prompt, llm_client, llm_call |
| Remote agents and protocols | A2A SDKs/cards, ACP integrations, delegated agents, agent gateways; distinguish protocol versions and proxy factories | agent_proxy, agent, tool; MCP types only with MCP evidence |
| MCP implementations and registries | Official SDKs across languages, FastMCP, standalone servers, client configs, embedded servers, gateway frameworks; registry records are discovery leads | mcp_server, mcp_client, mcp_gateway, tool, prompt |
| Skills and agent extensions | SKILL.md metadata, skill directories, plugin manifests, agent configuration and referenced tool bundles; malformed/ordinary Markdown controls | skill, tool, prompt, agent references |
| Model providers and enterprise clouds | OpenAI, Anthropic, Google, Azure, Bedrock, Cohere, Mistral, Groq, Together, OpenRouter, Replicate, Fireworks, DeepSeek, xAI, Cerebras, SambaNova, NVIDIA NIM; regional/private providers | model, llm_endpoint, model_endpoint, llm_client, llm_call, embedding |
| Model gateways and routing | LiteLLM, Portkey, provider routers, self-hosted OpenAI-compatible APIs, deployment base URLs and routing policies | llm_endpoint, model_endpoint, llm_client; not agent_proxy without agent delegation evidence |
| Local inference and serving | Ollama, vLLM, SGLang, llama.cpp, LM Studio, MLX, ONNX Runtime, TensorRT-LLM, Triton, OpenVINO, TGI, BentoML, Ray Serve, KServe | model, model_endpoint, model_artifact, ml_pipeline |
| Multimodal and specialized models | Diffusers, Transformers, sentence-transformers, speech/TTS/ASR, image/video generators, rerankers and classifiers | model with modality/task subtype, model_endpoint, embedding; not every generative model is an LLM |
| Retrieval and storage | Chroma, Qdrant, Pinecone, Weaviate, Milvus, LanceDB, FAISS, pgvector, Elasticsearch/OpenSearch, Redis vector features; generic DB clients are insufficient | embedding, vector_store, retriever, knowledge_base |
| Training and adaptation | PyTorch, TensorFlow/Keras, JAX, Hugging Face Trainer/PEFT/TRL/Accelerate, DeepSpeed, Lightning, Unsloth, Axolotl, XGBoost, LightGBM, CatBoost, scikit-learn | model, training_run, hyperparameter, model_artifact, ml_pipeline |
| Data and lifecycle | Hugging Face Datasets, DVC, lakeFS, Feast, Tecton, MLflow, W&B, ClearML, Kubeflow, Metaflow, ZenML, Dagster/Prefect/Airflow with AI context | dataset, data_versioning, feature_store, experiment_tracker, model_registry, ml_pipeline |
| Evaluation, observability and guardrails | Langfuse, LangSmith, Phoenix, OpenInference, OpenLLMetry, Braintrust, Ragas, DeepEval, promptfoo, TruLens, Guardrails AI, NeMo Guardrails, LLM Guard, model validation | observability with evaluation/tracing subtype, guardrail, prompt |
| Language-native ecosystems | Spring AI, LangChain4j, official Java SDKs; Microsoft.Extensions.AI/.NET integrations; Go SDKs; Rig/Rust SDKs; RubyLLM/Langchain.rb; PHP/Swift/Kotlin/C++ libraries | Same evidence taxonomy; don't downgrade all non-Python coverage to package names |
| Infrastructure and packaged artifacts | Dockerfiles, Kubernetes/Helm, Terraform, CI, notebook code cells, model manifests/headers, inference service configs, declarative AI platforms | Declared model/endpoint/pipeline/artifact references; static configuration is not proof of a live service |
| Internal and uncataloged AI | Enterprise wrappers, renamed SDK facades, private packages, raw HTTP calls and dynamic endpoints | Qualified local overrides and graph evidence; unresolved cases retain explicit reasons |
| Optional fast decision models | Laya/Kev, Jev, hosted decision APIs and other verifiable classifiers | decision_call; optional enrichment stays separate from inventory evidence |

Maintain `model.modality` (text/image/audio/video/multimodal/other/unknown) and `model.task` (generation/embedding/reranking/classification/regression/other/unknown) in the initial schema as those features are implemented, each with evidence. Taxonomy is not restricted to chat agents. Add types only through versioned governance when a subtype cannot express the asset accurately. Do not invent a modality when a generic serving client hides it.

## Independent catalog supply chain

Maintainer tooling performs discovery outside end-user scans. It can read official registry metadata, release manifests and upstream repositories, extract exported API/config changes without installing or executing packages, and propose reviewable changes. Signed runtime snapshots remain deterministic and offline. A suggested symbol never becomes a trusted rule automatically.

1. Ingest approved primary sources: PyPI, npm, Maven Central, NuGet, Go modules, crates.io, RubyGems, Packagist, Swift/C++ package metadata and official protocol registries. Use versioned adapters, conditional requests and host-specific credentials. A registry's AI keyword is a candidate signal, not a classifier.
2. Normalize package identities according to each ecosystem; record aliases, package moves, official/community ownership, license, version constraints, source revision/hash and last review. Deduplicate by coordinate and version, not display name.
3. Compare against the coverage matrix: missing coordinate, missing import, missing operation, incompatible version, new config schema, language-adapter gap, or missing benchmark. Every gap becomes a distinct review item.
4. Generate candidate API manifests using static source parsing. Parameter/signature metadata can improve rules but cannot inflate supported-asset counts. Never execute setup/build hooks or pickle/model payloads.
5. A maintainer verifies semantics and reviews fixtures; a second reviewer checks new high-confidence detectors and taxonomy changes. Reject ambiguous suffix-only matches. Record negative boundaries and unresolved cases.
6. Build JSON/compact indexes and optional DuckDB from one normalized source. Validate equivalence, uniqueness, rule references, compatibility, fixtures and license attribution. Sign releases, retain rollback snapshots and publish an intelligible coverage delta.
7. Proposed operating cadence: automated discovery proposals weekly, priority upstream breakage triaged within two business days, monthly reviewed catalog releases, quarterly long-tail audit. These are maintainer targets; no automation or update service is deployed by this plan.

Use a published lifecycle: discovered → identity_verified → dependency_tested → semantics_tested → benchmark_qualified; also deprecated, rejected and blocked_by_adapter. The initial catalog does not yet express all of this metadata; add explicit fields as these capabilities are implemented. A release can include tested dependency recognition while clearly withholding deeper claims.

## What to measure and expose

- Percentage of the explicitly versioned priority package set with verified identities; separately, percentage with semantic rules and qualified benchmark cells. Publish the denominator and changes to it.
- Per-type/per-language precision and recall, unknown-resolution rate, unsupported-file rate, scope exclusions and stale version mappings. Show sample counts and confidence intervals.
- Coverage gaps per source: missing SDK, unsupported language, unresolved wrapper, dynamic configuration, unsupported format or scan resource limit. A zero-result file cannot become a negative label without review.
- New upstream releases waiting for review and time since last verification. Export `coverage.json` with every machine-readable scan, including scans with no findings.
- No implicit source telemetry. Operators may explicitly export a scrubbed gap report or submit a fixture. Internal package names and code stay local by default.

Release qualification requires every selected priority family and artifact type to have an explicit disposition. A missing implementation blocks that declared support claim; it is not fixed by marking a row green. Independent catalog breadth, language semantics and benchmark results are all necessary.
