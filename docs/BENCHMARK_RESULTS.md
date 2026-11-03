# Earlier fixed GitHub development results — 2026-10-10

This report preserves the earlier baseline. See [coverage improvements](COVERAGE_IMPROVEMENTS.md) for the latest same-source rerun and comparison.

The final run attempted all 34 immutable public repository pins using unauthenticated access. Outcomes: `{"partial": 34}`.

All 34 repositories have informed development. This is an unlabeled development corpus, not a held-out acceptance benchmark. No precision, recall or production-readiness claim is made. Partial means the report retains unresolved coverage or explicit omissions; successful acquisition does not imply complete inventory.

## Reproducibility

Binary SHA-256: `3aadc8b4e871323626fa9e6293631ae625c127b1ac2a5e2074effc5f9124f66c`. Catalog fingerprint: `4df896af8dc59d7ea2639674b69260130efd732d69997c424af34bb3ca421108`. Corpus SHA-256: `119081383fd1c5db734425dc8067d69c3c2eea2c7383e5352d849c7cd23e12cb`.

The final profile uses four file workers, a 2 MiB per-file scan limit, 512 MiB compressed / 2 GiB expanded archive limits, and the seeded 170-entry catalog with 21,114 exact API references. Every run retains the exact catalog and corpus, source-file hashes, download digest, findings and diagnostics. The original pin file retains historical planning metadata; run.json and summary.json record actual execution.

Timings separate acquisition, source hashing and scanning. Each target has one trial on a shared Windows workstation; tests/builds overlapped parts of the run. These are observations, not controlled performance comparisons or p95 guarantees. Peak memory was not measured. The default inventory profile has no enterprise indicator configuration; application-ID behavior is verified separately with the enterprise fixture.

## Runs retained

| Run | Recorded outcomes | Status counts |
|---|---:|---|
| benchmark-expanded-initial | 34 | {"acquisition_failed": 14, "partial": 20} |
| benchmark-expanded-fixed | 34 | {"acquisition_failed": 3, "partial": 31} |
| benchmark-expanded-final (interrupted) | 25 | {"partial": 25} |
| benchmark-seeded-verified | 34 | {"partial": 34} |

The interrupted seeded run was stopped for a correctness fix: generic dependency-reference calls must not infer a model from arbitrary model= arguments. Its completed results and interruption record are preserved. The final run starts again from all 34 targets; earlier failures are not erased.

## Gaps fixed

- Archive links now produce explicit omissions without following targets; pinned public downloads avoid redundant REST ref-resolution requests.
- Configurable acquisition bounds allow larger repositories while preserving hard limits and failures.
- Split MCP TypeScript server/client packages and constructors are recognized.
- Java client builders, Rust grouped imports and curated prelude exports, Go declared package aliases, and C# namespace/target-typed constructors have fixtures.
- A separate bounded pnpm event budget handles Flowise’s large lockfile without enabling YAML aliases.
- Seeded API identities provide exact dependency-reference evidence; generic source concepts and model keywords do not become model/agent findings.

Changes in finding counts reflect rule scope and occurrence counts, not measured accuracy improvements. The initial MCP TypeScript run found 20 occurrences; the fixed catalog found 1,545. Both outputs remain available for review.

## Final outcomes

| Repository | Status | Findings | Scanned files | Scan seconds |
|---|---|---:|---:|---:|
| [openai/openai-agents-python](https://github.com/openai/openai-agents-python) | partial | 9587 | 1049 | 3.345 |
| [openai/openai-agents-js](https://github.com/openai/openai-agents-js) | partial | 1114 | 1215 | 1.492 |
| [crewAIInc/crewAI-examples](https://github.com/crewAIInc/crewAI-examples) | partial | 739 | 166 | 0.163 |
| [langchain-ai/langgraph](https://github.com/langchain-ai/langgraph) | partial | 5845 | 552 | 1.734 |
| [mastra-ai/mastra](https://github.com/mastra-ai/mastra) | partial | 8729 | 16021 | 10.128 |
| [langgenius/dify](https://github.com/langgenius/dify) | partial | 244 | 11384 | 14.040 |
| [langflow-ai/langflow](https://github.com/langflow-ai/langflow) | partial | 1389 | 7387 | 7.404 |
| [FlowiseAI/Flowise](https://github.com/FlowiseAI/Flowise) | partial | 858 | 1873 | 1.041 |
| [microsoft/semantic-kernel](https://github.com/microsoft/semantic-kernel) | partial | 10037 | 3882 | 2.541 |
| [spring-projects/spring-ai](https://github.com/spring-projects/spring-ai) | partial | 10658 | 2478 | 1.286 |
| [langchain4j/langchain4j](https://github.com/langchain4j/langchain4j) | partial | 19751 | 3765 | 1.698 |
| [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig) | partial | 3120 | 1826 | 1.135 |
| [crmne/ruby_llm](https://github.com/crmne/ruby_llm) | partial | 33 | 913 | 0.438 |
| [ollama/ollama](https://github.com/ollama/ollama) | partial | 1402 | 1000 | 1.054 |
| [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers) | partial | 190 | 103 | 0.075 |
| [modelcontextprotocol/python-sdk](https://github.com/modelcontextprotocol/python-sdk) | partial | 4133 | 884 | 1.060 |
| [modelcontextprotocol/typescript-sdk](https://github.com/modelcontextprotocol/typescript-sdk) | partial | 1545 | 762 | 0.761 |
| [modelcontextprotocol/go-sdk](https://github.com/modelcontextprotocol/go-sdk) | partial | 253 | 152 | 0.173 |
| [modelcontextprotocol/csharp-sdk](https://github.com/modelcontextprotocol/csharp-sdk) | partial | 835 | 755 | 0.462 |
| [modelcontextprotocol/java-sdk](https://github.com/modelcontextprotocol/java-sdk) | partial | 961 | 253 | 0.177 |
| [a2aproject/a2a-samples](https://github.com/a2aproject/a2a-samples) | partial | 698 | 588 | 0.339 |
| [anthropics/skills](https://github.com/anthropics/skills) | partial | 31 | 95 | 0.152 |
| [huggingface/transformers](https://github.com/huggingface/transformers) | partial | 12129 | 4980 | 12.805 |
| [mlflow/mlflow](https://github.com/mlflow/mlflow) | partial | 2561 | 5995 | 7.969 |
| [treeverse/dvc](https://github.com/treeverse/dvc) | partial | 1 | 555 | 0.664 |
| [feast-dev/feast](https://github.com/feast-dev/feast) | partial | 5281 | 1664 | 2.128 |
| [qdrant/qdrant](https://github.com/qdrant/qdrant) | partial | 15 | 2179 | 1.589 |
| [chroma-core/chroma](https://github.com/chroma-core/chroma) | partial | 291 | 1475 | 1.453 |
| [NVIDIA-NeMo/Guardrails](https://github.com/NVIDIA-NeMo/Guardrails) | partial | 741 | 1029 | 1.522 |
| [psf/requests](https://github.com/psf/requests) | partial | 0 | 40 | 0.080 |
| [expressjs/express](https://github.com/expressjs/express) | partial | 0 | 142 | 0.054 |
| [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | partial | 0 | 125 | 0.133 |
| [langchain-ai/deepagents](https://github.com/langchain-ai/deepagents) | partial | 6678 | 1247 | 4.334 |
| [langchain-ai/deepagentsjs](https://github.com/langchain-ai/deepagentsjs) | partial | 488 | 358 | 0.285 |

## Remaining qualification

Independent exhaustive gold labels, representative enterprise applications, per-type/language accuracy, repeated controlled performance and native Linux/macOS runs remain required. PHP currently has local fixtures but no dedicated public PHP corpus member. Ruby remains import-focused. General wrappers, SDK-version constraints, direct HTTP APIs, unresolved module graphs and dynamic behavior remain coverage gaps.

The deliverable includes benchmark-outcomes.csv and benchmark-runs.zip with detailed retained evidence. Source repositories themselves are not redistributed in that archive.
