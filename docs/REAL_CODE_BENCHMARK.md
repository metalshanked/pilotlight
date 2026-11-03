# Fixed GitHub repository replay — 2026-10-27

Pilot Light's new build completes all 38 fixed-source scans and retains every finding ID produced by the previous binary. The new occurrences include **64 agents, 145 MCP clients and 569 tools**. The model-record change adds one literal `ModelSpec.parse` reference in a Deep Agents test; its broader behavior is covered by fixtures. The original 34 pins and separate four-repository SDK supplement were not changed.

Cisco main [`317d72a9d0f4`](https://github.com/cisco-ai-defense/aibom/tree/317d72a9d0f4fe763a5f81a6fb51b5c78bee11c1) completes **20/38** repositories under this harness; **18/38** hit the fixed 300-second process budget. Full outcomes are below. A timeout means there is no assembled result to compare; it never means zero detections. No Cisco source changes were made.

## What improved relative to the previous Pilot Light binary

| Type | Added occurrences | In test paths | Other paths, including examples |
|---|---:|---:|---:|
| agent | 64 | 60 | 4 |
| mcp_client | 145 | 112 | 33 |
| mcp_server | 0 | 0 | 0 |
| mcp_gateway | 0 | 0 | 0 |
| tool | 569 | 480 | 89 |
| model | 1 | 1 | 0 |
| dependency | 5,913 | 3,314 | 2,599 |

There are **zero removed finding IDs**. Test paths use the exact upstream `_is_test_file` helper at the pinned commit. This is path context, not an accuracy label: Cisco may retain test evidence when it consolidates it with other occurrences. In particular, 60 of the 64 new agent occurrences are in SDK tests; the other four are Microsoft Agent Framework C# examples in Semantic Kernel. The totals do not represent distinct deployed applications or independently verified true positives.

The packs now have **29 framework-entry rules and 106 inline fixtures**, versus 23 and 38 previously. Three new catalog families are LangChain MCP adapters, MCP-use and MCP-Go; three existing families moved into pack ownership. New bindings cover OpenAI Agents tools/core/MCP clients, Strands TypeScript, Microsoft Agent Framework Python/C#, Haystack, FastMCP Python/TypeScript and reviewed Pydantic/LangChain MCP API generations. Some additions are fixture-tested but not exercised by this fixed corpus. See [the pack contract and sources](DETECTION_PACKS.md).

## Concrete observations against Cisco

These two added occurrences were manually checked in the pinned OpenAI Agents TypeScript repository. Neither requested type appears anywhere in Cisco's completed default assembled report for that repository. Both are in example paths, not paths classified as tests by Cisco's policy:

- [examples/docs/mcp/agentMcpConfig.ts:3](https://github.com/openai/openai-agents-js/blob/89a9ddf1e241e75f2e0ad4c15d89de9e215ebac6/examples/docs/mcp/agentMcpConfig.ts#L3): `@openai/agents.MCPServerStreamableHttp` → `mcp_client`.
- [examples/agent-patterns/agents-as-tools-streaming.ts:6](https://github.com/openai/openai-agents-js/blob/89a9ddf1e241e75f2e0ad4c15d89de9e215ebac6/examples/agent-patterns/agents-as-tools-streaming.ts#L6): `@openai/agents.tool` → `tool`.

These are concrete gaps for these source forms and the selected deterministic mode. They do not establish that an entire framework is unsupported. Full source links, hashes and observations are in `real-code-reviewed-examples.json`. Other added findings in `real-code-new-evidence.csv` include a file/type-presence comparison against Cisco's primary and consolidated evidence paths. That column is a navigation aid, not a scored false-negative label.

The unchanged authored 39-case corpus now yields Pilot Light **29/29 positives and 0/10 negative target emissions**, versus Cisco **12/29 and 3/10**. The Cisco probe outputs were reused after re-verifying the identical main/source pin; the 38-repository Cisco replay here is fresh. These expectations are development probes, not held-out accuracy labels. See [all probe cases](PACK_BENCHMARK.md).

## Reproducibility and scope

- All **120,275 extracted files** match the previous fixed corpus by path, byte count and SHA-256. The harness checks the source inventory before and after every replay. Archive-link omissions are retained in metadata and the CSV. No target code, build or package manager was executed.
- The baseline is the previously delivered executable (`1cb2059e40f1e400a29d165d661fdf25f3299b3f2e224f27fc16235318776fd3`); the candidate is `7269db1738b8343a1c2421edbd7ea647cde162f3a32977c3e67e95000fdf6de5`. Both scan the same local trees, default artifact selection, four file workers and 2 MiB source-file limit. The candidate and Cisco replay each use three concurrent repositories; the baseline used one. Worker concurrency does not change the semantic scan configuration, but it makes baseline timing unsuitable for a before/after speed claim.
- Cisco uses its original `ScanPipeline(..., llm_config=None, include_code_snippets=False)`, frozen base/analysis dependencies and original 1.10.0 DuckDB (`b1165384a646094aa6a4762d676f285c1c68b6d7f9367c90a129b529904a3268`). All **244 original upstream source files** are unchanged, and main was rechecked during the run. Progress callbacks/stack snapshots aid timeout diagnosis; only cache placement is redirected. Public metadata networking is allowed; no LLM or GitHub credentials are used.
- Cisco's full pipeline runs its applicable deterministic scanners, cross-reference and assembly, including work such as secret and vulnerability scanning. It consolidates components and filters test-only components. Pilot Light reports occurrence/evidence records and has different declared capabilities and exclusions. Identical input trees do not imply identical visited scopes, work or output units.
- All 38 Pilot Light reports are **partial coverage**, with diagnostics and per-language/type limits retained. The designated Requests, Express and ripgrep controls still produce zero Pilot Light findings. Cisco's counts for those controls remain visible; different workflow/context classifications are not scored as accuracy here.
- The raw Pilot Light replay times range from **0.38 to 43.99 seconds**, with an across-repository median of **4.10 seconds**. Replay timing includes source-integrity checks. These are single observations on a shared Windows workstation, with concurrent builds/scans, warm OS caches and different scanner workloads; they are not controlled speed ratios, percentile guarantees or peak-memory measurements. Cisco timeouts are censored observations.
- The superseded sequential Cisco attempt is retained: five outcomes were recorded before its sixth worker was cancelled to start the complete three-worker replay. Intermediate Pilot Light runs are also retained. Final reporting uses `pilotlight-delivery` and `cisco-main-parallel` consistently, not selected successful outcomes from different runs.

## Every fixed repository

The two count columns have different units: Pilot Light occurrences versus Cisco assembled components. **Do not rank accuracy by their size.** A dash is an incomplete Cisco result. All Pilot Light before/after statuses are `partial`.

| Pinned repository | Pilot Light occurrences, before → after | Cisco assembled components | Cisco outcome |
|---|---:|---:|---|
| [openai/openai-agents-python ](https://github.com/openai/openai-agents-python/tree/125efa029b4bfd84238bd2c4fd69c3406f802663) | 9,604 → 10,187 | — | timed_out |
| [openai/openai-agents-js ](https://github.com/openai/openai-agents-js/tree/89a9ddf1e241e75f2e0ad4c15d89de9e215ebac6) | 1,115 → 1,542 | 40 | completed |
| [crewAIInc/crewAI-examples ](https://github.com/crewAIInc/crewAI-examples/tree/da94a91e691e1cf5b3151416bb15b5b62729bea8) | 739 → 739 | 325 | completed |
| [langchain-ai/langgraph ](https://github.com/langchain-ai/langgraph/tree/b4991f1ba36a2449ab59c38ab71828ee865bff84) | 5,932 → 5,932 | — | timed_out |
| [mastra-ai/mastra ](https://github.com/mastra-ai/mastra/tree/8e98b38a2e15a78c56178a978bf49281e4eda80e) | 8,797 → 8,797 | — | timed_out |
| [langgenius/dify ](https://github.com/langgenius/dify/tree/2ac4c221d5f7e99ddcfaa14e22775ecd1769a191) | 830 → 830 | — | timed_out |
| [langflow-ai/langflow ](https://github.com/langflow-ai/langflow/tree/504c02fc47e76087b82b0e7cbe4186e9cdd916d4) | 1,398 → 1,399 | — | timed_out |
| [FlowiseAI/Flowise ](https://github.com/FlowiseAI/Flowise/tree/9291856d1ea4a4ceea9f8fef8ce14f4f6c81e8eb) | 1,039 → 1,039 | 60 | completed |
| [microsoft/semantic-kernel ](https://github.com/microsoft/semantic-kernel/tree/11f072362e24207da9db55779c0fd73eb15123b1) | 10,042 → 10,084 | — | timed_out |
| [spring-projects/spring-ai ](https://github.com/spring-projects/spring-ai/tree/00f230f9ee828f12517b2234e5a279cc9c061851) | 10,683 → 10,683 | 22 | completed |
| [langchain4j/langchain4j ](https://github.com/langchain4j/langchain4j/tree/872eca4b97b85a42f8409d0437ed4fc1994157b5) | 19,833 → 19,833 | 160 | completed |
| [0xPlaygrounds/rig ](https://github.com/0xPlaygrounds/rig/tree/9e7cb6000355b7ae1a89c913bf98b8a6f17661ee) | 3,128 → 3,128 | — | timed_out |
| [crmne/ruby_llm ](https://github.com/crmne/ruby_llm/tree/a999ef9552125d137ee797ff54e13a6d59584ea1) | 505 → 505 | — | timed_out |
| [ollama/ollama ](https://github.com/ollama/ollama/tree/eab97e9f92b9a25c2d52d2cc6c1b1c99bd9fae21) | 1,512 → 1,512 | 51 | completed |
| [modelcontextprotocol/servers ](https://github.com/modelcontextprotocol/servers/tree/956d36f8a148908b2e2394669ef5ae1f1230c9e9) | 215 → 215 | 15 | completed |
| [modelcontextprotocol/python-sdk ](https://github.com/modelcontextprotocol/python-sdk/tree/91941ed4d3985d59def99e090baa3f880c626cc8) | 6,253 → 6,253 | — | timed_out |
| [modelcontextprotocol/typescript-sdk ](https://github.com/modelcontextprotocol/typescript-sdk/tree/b022522089a0c8b632595c6e7b536453945ed5a9) | 1,965 → 1,965 | 19 | completed |
| [modelcontextprotocol/go-sdk ](https://github.com/modelcontextprotocol/go-sdk/tree/218aedde7c7e2c3c24f8f3b09e093e8eb1fb08c3) | 253 → 253 | 2 | completed |
| [modelcontextprotocol/csharp-sdk ](https://github.com/modelcontextprotocol/csharp-sdk/tree/3338e88e15c42cfc27465143c140d7d10f1a2707) | 1,629 → 1,629 | 5 | completed |
| [modelcontextprotocol/java-sdk ](https://github.com/modelcontextprotocol/java-sdk/tree/e6c532e54b3f8bc34687542a301f6b6103e6ee6c) | 961 → 961 | 5 | completed |
| [a2aproject/a2a-samples ](https://github.com/a2aproject/a2a-samples/tree/6603ba3f2c31a7ef33e70b9d8b5b5f8be42ac9a3) | 701 → 708 | 254 | completed |
| [anthropics/skills ](https://github.com/anthropics/skills/tree/dbd4588f9e1033efb41dad4bef2f7947c8993d44) | 31 → 31 | 8 | completed |
| [huggingface/transformers ](https://github.com/huggingface/transformers/tree/536ecc007387a50e77603bb5d92100e9b07514cc) | 12,129 → 12,129 | — | timed_out |
| [mlflow/mlflow ](https://github.com/mlflow/mlflow/tree/b7a5c98c19edde5adc503ca36db8eaef23790566) | 2,587 → 2,590 | — | timed_out |
| [treeverse/dvc ](https://github.com/treeverse/dvc/tree/56e59829512ff134aa269099a2099587b810b4dd) | 1 → 1 | 7 | completed |
| [feast-dev/feast ](https://github.com/feast-dev/feast/tree/4fc97261f1c11211f6c1e34ba41b0cdc4e3810c1) | 5,475 → 5,475 | — | timed_out |
| [qdrant/qdrant ](https://github.com/qdrant/qdrant/tree/016542aa5deb6c66380bb137badf73d54f742bde) | 15 → 15 | 7 | completed |
| [chroma-core/chroma ](https://github.com/chroma-core/chroma/tree/5f30478c11ffdcf8cba1f872993c29d8f84ce0d9) | 296 → 296 | — | timed_out |
| [NVIDIA-NeMo/Guardrails ](https://github.com/NVIDIA-NeMo/Guardrails/tree/4512d081cb970be8a99ac2ccd1c6a39b1a81d352) | 744 → 744 | — | timed_out |
| [psf/requests ](https://github.com/psf/requests/tree/611c6162cbc4ac2020a2f91c7cfa4f3abf9bbb60) | 0 → 0 | 5 | completed |
| [expressjs/express ](https://github.com/expressjs/express/tree/a419aa7350389a275ab45fde2e849ed1f02449ef) | 0 → 0 | 1 | completed |
| [BurntSushi/ripgrep ](https://github.com/BurntSushi/ripgrep/tree/3fce3b5bb0236da2df6d99672afb8a719642eca7) | 0 → 0 | 3 | completed |
| [langchain-ai/deepagents ](https://github.com/langchain-ai/deepagents/tree/9f0e39d55a5f31003b206b31b955154f2f1bc49a) | 6,893 → 12,508 | — | timed_out |
| [langchain-ai/deepagentsjs ](https://github.com/langchain-ai/deepagentsjs/tree/b448f220866bbb5d41924dfdbb1e0079c3b11838) | 557 → 557 | 46 | completed |
| [modelcontextprotocol/rust-sdk ](https://github.com/modelcontextprotocol/rust-sdk/tree/c1a8eea312613519b93f53b89a1dc74dc993f5b0) | 1,383 → 1,383 | 4 | completed |
| [PrefectHQ/fastmcp ](https://github.com/PrefectHQ/fastmcp/tree/e2fcc412001ff7414095c5dc282f94c70299a612) | 8,284 → 8,291 | — | timed_out |
| [vercel/ai ](https://github.com/vercel/ai/tree/ba05943b69ad070558bb5bc912da99ec66d7d263) | 7,126 → 7,133 | — | timed_out |
| [anthropics/anthropic-sdk-python ](https://github.com/anthropics/anthropic-sdk-python/tree/b4b7916deaf4e5570cd627b1ae7ee4394a4de39f) | 1,339 → 1,339 | — | timed_out |

## Validation and files

**71 default and 73 DuckDB-feature integration tests pass**, including the new model/graph regressions. The fictional model pack passes four CLI fixtures. Windows static-CRT imports, CSV application ID `0007`, sidecar integrity and CycloneDX 1.6 are checked. The independent seed preserves all 1,906,503 raw rows and round-trips its canonical document through the feature-enabled CLI.

Delivery files: `real-code-comparison.csv` and `.json` retain every outcome/type count; `real-code-new-evidence.csv` links added evidence to fixed commits; `real-code-removed-evidence.json` is empty; `real-code-benchmark-runs.zip` retains raw reports/logs, inventories, pins, baseline binary/source, dependency freeze, harness and verification records. Large downloaded source trees are not bundled; fixed pins and hashes support reacquisition. [Reproduction commands](../benchmarks/README.md#fixed-source-cisco-main-replay) are included. Native Linux/macOS CI, independent accuracy labels and enterprise acceptance remain unrun.
