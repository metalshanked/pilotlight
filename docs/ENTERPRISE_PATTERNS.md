# Enterprise agent and MCP patterns — 2026-10-11

Pilot Light now detects selected handwritten tool-calling loops, configured SDK loops, FastMCP gateway/proxy factories, C# MCP dependency-injection registrations, and Rust rmcp macro declarations. This remains a static development build; it does not prove runtime execution or universal coverage.

## Priority and evidence

The priority set uses the official OpenAI/Anthropic SDK patterns, Vercel AI SDK, FastMCP, and the official C#/Rust MCP SDKs. These are concrete integration targets already represented by enterprise-facing SDK documentation and the development corpus. This is a coverage priority, not a measured market-share ranking. Existing LangChain, LangGraph agent factories, OpenAI Agents, ADK, CrewAI, AutoGen, Pydantic AI, Deep Agents, Semantic Kernel and Mastra rules remain in place.

| Pattern | Detection boundary | False-positive checks |
|---|---|---|
| Handwritten Python / JS / TS loop | A catalog-resolved model call inside a loop; assigned response; tools plus input/messages; iteration over that response; tool dispatch; result appended to the same history with the matching tool-call ID | Reject unrelated response data, parsing-only calls, different history, constant output, replaced response/tool/result/history variables, and nested unused helpers |
| Vercel SDK loop | Qualified generateText/streamText, tools, and explicit multi-step maxSteps or imported stepCountIs/isStepCount with literal count above one | Single-step, missing/empty tools, unrelated helpers/imports abstain |
| Anthropic runner | Qualified Python Anthropic/AsyncAnthropic beta.messages.tool_runner; beta_tool/beta_async_tool definitions | Regular messages.create remains a model call; an import alone is a dependency |
| MCP gateway | FastMCP.as_proxy, fastmcp.server.create_proxy, FastMCPProxy construction | Ordinary clients/servers and a list of MCP configurations do not become gateways |
| C# DI | AddMcpServer on resolved ServiceCollection, IServiceCollection parameter, or Services from Host.CreateApplicationBuilder / WebApplication.CreateBuilder | Unresolved receivers, missing extension namespace and reassigned receivers abstain |
| Rust rmcp | Imported/aliased tool attribute on impl methods; tool_handler on a resolved ServerHandler impl; tool_router with explicit server_handler flag | Unimported/unrelated macros, wrong traits, standalone tool functions, bare routers and ordinary function calls abstain |

Handwritten loops have subtype tool_calling_loop_candidate and confidence inferred_from_control_flow. Evidence records the loop, model call, dispatch and feedback locations, without exporting prompts or credentials. This is bounded structural correlation, not a complete control-flow or type proof. CSV has one row per evidence item, so one inferred loop can span four rows with the same asset_id. JSON retains the full evidence and relationship graph.

Vercel generateText/streamText were corrected from llm_client to llm_call. Rust scoped imports now retain the final symbol, which also improves existing Rust rules. The overlay contains 192 entries, 329 package coordinates, 396 import prefixes and 199 explicit symbols. The preserved seed still has 1,906,503 rows and 21,114 runtime API references. Rule sources are recorded in catalog/enterprise-pattern-evidence.json.

Default supported-type selection now includes mcp_gateway. The baseline had no executable gateway detector; a zero baseline count is not evidence that no gateways existed. Each report retains its effective type selection.

## Validation

59 default integration tests and 61 optional-DuckDB tests pass, including six new test groups with positive and negative cases. The installed Windows binary passes the existing application-ID/CSV/CycloneDX smoke, plus a focused multi-language scan validating four-location loop evidence and metadata retention. Linux/macOS native CI, formatting and Clippy remain unverified.

Both comparisons use identical complete extracted path/size/SHA-256 source inventories, unauthenticated public acquisition, four workers, 2 MiB file limits, 512 MiB download limits and 2 GiB expanded limits. Target source was never built or executed. Findings include tests and examples; counts measure detector output, not unique deployed assets or accuracy.

### Original fixed 34 repositories

Outcomes: `{"partial": 34}`.

| Artifact | Before | After | Change |
|---|---:|---:|---:|
| agent | 4597 | 4598 | +1 |
| mcp_gateway | 0 | 0 | +0 |
| mcp_server | 1819 | 1878 | +59 |
| mcp_client | 2024 | 2026 | +2 |
| tool | 1972 | 1976 | +4 |
| llm_call | 142 | 152 | +10 |
| llm_client | 1444 | 1440 | -4 |

| Repository | Type | Before | After | Change |
|---|---|---:|---:|---:|
| 0xPlaygrounds/rig | mcp_server | 0 | 1 | +1 |
| 0xPlaygrounds/rig | tool | 0 | 1 | +1 |
| modelcontextprotocol/csharp-sdk | mcp_client | 185 | 187 | +2 |
| modelcontextprotocol/csharp-sdk | mcp_server | 57 | 114 | +57 |
| chroma-core/chroma | agent | 0 | 1 | +1 |
| chroma-core/chroma | mcp_server | 0 | 1 | +1 |
| chroma-core/chroma | tool | 0 | 3 | +3 |

Baseline binary: `4f17705dcb5d740c9a46d5bba6c959b77ff77afbc5c2fb28fe15d044ef6baf0f`. Candidate: `0e108c4ec2d94bcb0c9c52726d00d3d4116122bacc299ad82e8840938657edf1`.

Corpus: `119081383fd1c5db734425dc8067d69c3c2eea2c7383e5352d849c7cd23e12cb`. Catalog: `f75439ec6cea80b99cec7ec6762f21be73d78b426520203392213f48f7ca12ab`.

### Separate four-repository extension

Outcomes: `{"partial": 4}`.

| Artifact | Before | After | Change |
|---|---:|---:|---:|
| agent | 124 | 392 | +268 |
| mcp_gateway | 0 | 85 | +85 |
| mcp_server | 2156 | 2180 | +24 |
| mcp_client | 892 | 892 | +0 |
| tool | 1405 | 1550 | +145 |
| llm_call | 46 | 1732 | +1686 |
| llm_client | 1958 | 272 | -1686 |

| Repository | Type | Before | After | Change |
|---|---|---:|---:|---:|
| modelcontextprotocol/rust-sdk | mcp_server | 3 | 27 | +24 |
| modelcontextprotocol/rust-sdk | tool | 3 | 65 | +62 |
| PrefectHQ/fastmcp | mcp_gateway | 0 | 85 | +85 |
| vercel/ai | agent | 118 | 381 | +263 |
| anthropics/anthropic-sdk-python | agent | 0 | 5 | +5 |
| anthropics/anthropic-sdk-python | tool | 0 | 83 | +83 |

Baseline binary: `4f17705dcb5d740c9a46d5bba6c959b77ff77afbc5c2fb28fe15d044ef6baf0f`. Candidate: `0e108c4ec2d94bcb0c9c52726d00d3d4116122bacc299ad82e8840938657edf1`.

Corpus: `9d7141afcadc414a25b668b2f56e41b5c2727184eae8055b48aad694f4f6636c`. Catalog: `f75439ec6cea80b99cec7ec6762f21be73d78b426520203392213f48f7ca12ab`.

Handwritten loop candidates are verified by positive/negative source fixtures and the end-to-end example. Neither public corpus produced a qualifying handwritten-loop candidate in this increment; general real-application validation of that detector remains open.

Requests, Express and ripgrep remain at zero findings. The four-repository extension is separate and has its own pinned file, source inventories and before/after runs; it does not alter the original 34 targets. Neither set has independent gold labels.

The initial candidate exposed the current FastMCP providers.proxy class path missing from the catalog. That rule was added with a pinned source reference, and both corpora were rerun from the beginning. Initial candidate reports are retained alongside the final runs.

## Remaining gaps

The handwritten detector currently requires a model call before response iteration within the same loop and direct variable/member forms. Loop-carried responses acquired outside the loop, response aliases, dispatch registries addressed only through indirect variables, streaming events, recursive agents, complex wrappers, arbitrary HTTP clients and cross-file dataflow remain incomplete. Condition feasibility, early returns, branch exclusivity and runtime invocation are not proven. Tools passed through an SDK variable may be empty at runtime.

C# ConfigureServices lambda inference, arbitrary DI containers, global usings in another file, service extension chains and Roslyn type resolution remain open. Rust fully qualified attributes without a bound rmcp namespace, macro expansion, conditional compilation and generated/custom macros are not resolved. Gateway coverage currently means reviewed FastMCP proxy factories; it does not cover every ContextForge, Docker, Kubernetes, cloud or reverse-proxy deployment. SDK-version constraints, CommonJS binding resolution and enterprise-private wrappers remain open.

The next qualification step is a labeled application corpus organized by pattern and language, including difficult negatives. Popularity alone cannot establish detector accuracy. Existing diagnostics remain visible and all partial scans must be treated as incomplete. Timing is a single shared-workstation observation with concurrent validation; no performance or memory guarantee is inferred.

## Run

```powershell
.\bin\pilotlight.exe scan . --types agents,mcp,tool,skill --format csv --output inventory.csv
```

Use examples/agents-mcp.toml for the selection profile and examples/enterprise.toml for configurable application-ID metadata. Original comparison: enterprise-pattern-comparison.csv/json. Supplemental comparison: enterprise-supplement-comparison.csv/json. New source-linked evidence is in the corresponding *-new-evidence.csv files; notebook lines are cell-relative. Both before/after sets are retained in enterprise-pattern-benchmark-runs.zip.
