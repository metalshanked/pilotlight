# Agent and MCP coverage — Pilot Light 0.1

Historical comparison from 2026-10-10. See [enterprise patterns](ENTERPRISE_PATTERNS.md) for the newer detector boundaries and results.

The focus is static evidence of agent construction/orchestration, MCP servers and clients, and tool definitions/registrations. Package or import presence alone remains dependency evidence. This release does not establish that code executes or that a server is deployed.

## Run a focused scan

```sh
pilotlight scan /path/to/repository --types agents,mcp,tool,skill --format csv --output inventory.csv
```

`examples/agents-mcp.toml` provides the same selection. Combine these types with enterprise metadata settings to preserve configured application IDs in the CSV. The coverage sidecar records unsupported selected types and incomplete scans, including zero-result repositories. MCP gateway detection is still unsupported; selecting the MCP group keeps that limitation visible.

## Detection matrix

All rows describe reviewed API shapes, not complete coverage of the named SDK. Exact paths, language constraints and provenance are in `catalog/overlay.json` and `catalog/agent-mcp-evidence.json`.

| Area | Implemented evidence | Important boundary |
|---|---|---|
| Python agents | Deep Agents, LangChain/LangGraph explicit agent factories; OpenAI Agents, ADK, CrewAI, AutoGen, Pydantic AI, Strands, smolagents, Agno, LlamaIndex and other cataloged constructors | Generic graph builders and imports alone do not establish an agent |
| JS/TS agents | Deep Agents, LangChain createAgent, LangGraph createReactAgent, OpenAI Agents, ADK workflow agents, Mastra Agent, Vercel ToolLoopAgent | ES import/alias resolution; CommonJS callable bindings and re-export graph remain open |
| Agent orchestration | ADK Sequential/Parallel/Loop agents, AutoGen teams, LlamaIndex AgentWorkflow; orchestration subtype on new workflow rules | Static composition does not establish autonomous behavior at runtime |
| Other agent languages | Selected C# Semantic Kernel and Rust Rig constructor paths | Language parser availability does not imply broad agent semantics |
| Python MCP | Reviewed v1 FastMCP/low-level Server and ClientSession paths, v2 MCPServer/Client paths, standalone FastMCP roles | SDK versions are not resolved from dependency constraints |
| JS/TS MCP | Unified and split SDK packages, Mastra MCPClient/MCPServer, AI SDK MCP client | Requires resolved import and cataloged factory/constructor |
| Other MCP languages | Selected Go NewServer/NewClient, Java/Kotlin sync builders, C# client/server static factories and PHP Server::make | C# dependency-injection extensions and Rust rmcp macros/traits remain unresolved |
| Tool evidence | Python LangChain and FastMCP decorators, including bare decorators; reviewed add_tool paths; TS registerTool and legacy SDK tool paths; C# McpServerTool attributes/factories | A server constructor alone does not imply tool registration; resources/prompts are not tool findings |
| MCP configuration | Fixed-name JSON/JSONC/YAML/TOML MCP configuration files and Codex config | Declared server references only; no probing, command execution, credentials or tool-schema collection |
| Agent proxies / skills | One reviewed ADK RemoteA2aAgent path; valid SKILL.md identity | Not general proxy, gateway or custom-agent detection |

The catalog retains 192 framework/library entries, 329 package coordinates and 395 import prefixes. Explicit semantic symbols grow from 138 to 187. Two existing StateGraph paths change from agent to dependency with workflow_builder subtype: a shared-state graph need not contain an LLM or an agent. The preserved 1,906,503-row seed and its 21,114 exact runtime API references are unchanged.

## Validation and fixed-corpus results

53 default integration tests and 55 optional-DuckDB integration tests pass. New positive and negative cases cover aliases, type-only imports, reassignment, unrelated constructors/receivers, bare decorators, C# attributes, MCP v2 APIs, orchestration and non-agent StateGraph use. Existing metadata/CSV, acquisition, module graph and exact-package regressions remain in the suite.

All 34 pinned public repositories were rerun without authentication. Outcomes: `{"partial": 34}`. Complete extracted path/size/SHA-256 source inventories match the prior run for all 34 targets. Both runs use four workers, a 2 MiB scan limit, 512 MiB download limit and 2 GiB expanded limit. Target code was not executed.

Counts are static evidence occurrences, including examples and tests. They are not deployed assets, accuracy scores or runtime inventory.

| Artifact type | Before | After | Change |
|---|---:|---:|---:|
| agent | 5416 | 4597 | -819 |
| agent_proxy | 0 | 0 | +0 |
| mcp_server | 1056 | 1819 | +763 |
| mcp_client | 864 | 2024 | +1160 |
| tool | 69 | 1972 | +1903 |

### Repository changes

| Repository | Artifact type | Before | After | Change |
|---|---|---:|---:|---:|
| openai/openai-agents-python | mcp_server | 3 | 9 | +6 |
| openai/openai-agents-python | tool | 0 | 11 | +11 |
| crewAIInc/crewAI-examples | agent | 115 | 112 | -3 |
| crewAIInc/crewAI-examples | tool | 0 | 14 | +14 |
| langchain-ai/langgraph | agent | 894 | 55 | -839 |
| langchain-ai/langgraph | tool | 0 | 104 | +104 |
| mastra-ai/mastra | mcp_client | 16 | 40 | +24 |
| mastra-ai/mastra | mcp_server | 8 | 37 | +29 |
| mastra-ai/mastra | tool | 1 | 8 | +7 |
| langflow-ai/langflow | tool | 55 | 64 | +9 |
| FlowiseAI/Flowise | agent | 2 | 0 | -2 |
| FlowiseAI/Flowise | tool | 1 | 3 | +2 |
| microsoft/semantic-kernel | mcp_client | 3 | 4 | +1 |
| microsoft/semantic-kernel | tool | 1 | 3 | +2 |
| modelcontextprotocol/servers | tool | 0 | 24 | +24 |
| modelcontextprotocol/python-sdk | mcp_client | 20 | 1152 | +1132 |
| modelcontextprotocol/python-sdk | mcp_server | 452 | 1123 | +671 |
| modelcontextprotocol/python-sdk | tool | 0 | 490 | +490 |
| modelcontextprotocol/typescript-sdk | tool | 0 | 420 | +420 |
| modelcontextprotocol/csharp-sdk | mcp_server | 0 | 57 | +57 |
| modelcontextprotocol/csharp-sdk | tool | 0 | 493 | +493 |
| a2aproject/a2a-samples | mcp_client | 2 | 5 | +3 |
| a2aproject/a2a-samples | tool | 9 | 12 | +3 |
| mlflow/mlflow | agent | 96 | 89 | -7 |
| mlflow/mlflow | tool | 0 | 15 | +15 |
| feast-dev/feast | tool | 0 | 22 | +22 |
| NVIDIA-NeMo/Guardrails | tool | 0 | 3 | +3 |
| langchain-ai/deepagents | agent | 305 | 280 | -25 |
| langchain-ai/deepagents | tool | 1 | 284 | +283 |
| langchain-ai/deepagentsjs | agent | 44 | 101 | +57 |
| langchain-ai/deepagentsjs | tool | 0 | 1 | +1 |

Requests, Express and ripgrep remain at zero findings. These controls and regression fixtures do not establish a corpus-wide false-positive rate. StateGraph reclassification can reduce agent counts; this is intentional. Tools now use exact receiver paths instead of assuming any MCP server exposes a tool API.

## Remaining work

Handwritten LLM/tool loops, dynamic wrappers, general cross-file typing, conditional control flow, dependency-injection wiring, CommonJS bindings, Rust macros/traits, arbitrary MCP protocol implementations and SDK-version qualification remain incomplete. Agent/gateway products with only package mappings still produce dependency evidence. The bounded Python module graph helps reviewed imports, re-exports and simple wrappers; it is not whole-program analysis.

All 34 repositories have informed development. An independently labeled, held-out corpus with agent/MCP positives and hard negatives is still needed to measure precision and recall. All partial reports retain their diagnostics; no missing finding is silently scored as correct. The ten language parsers do not imply ten languages with equal semantic depth. Linux/macOS native CI and formatting/Clippy remain unverified. No runtime scanner ships here.

## Reproducibility

Baseline binary: `5628b4fb44959fff30cb056348c4c414e975825768e6d1e436603c510d881aef`. Candidate binary: `4f17705dcb5d740c9a46d5bba6c959b77ff77afbc5c2fb28fe15d044ef6baf0f`.

Catalog fingerprint: `87ed37d2c6d7cd4c1c0d69a1c640ba98a9c6590e5c76d1d2b6e7f64bc4195e65`. Corpus: `119081383fd1c5db734425dc8067d69c3c2eea2c7383e5352d849c7cd23e12cb`.

Deliverables: agent-mcp-comparison.csv (every repository/type), agent-mcp-comparison.json (source identity and diagnostics), agent-mcp-new-evidence.csv (new evidence with pinned source links), and agent-mcp-benchmark-runs.zip (both complete raw runs). Earlier coverage reports remain available as historical snapshots. Single-workstation timings are observational; peak memory and performance percentiles were not measured.
