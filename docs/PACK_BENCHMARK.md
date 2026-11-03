# Detection packs and pinned Cisco comparison — 2026-10-27

Pilot Light now has two embedded, independently maintained detection packs with 23 framework-entry rules and 38 positive/negative inline fixtures. This packages existing reviewed coverage and moves configured-loop, Rust-attribute and C# registration parameters into data. It does not add 23 new frameworks. New operator packs can extend existing engine mechanisms without rebuilding; new languages or analyses still need engine work. See [the pack contract](DETECTION_PACKS.md).

## Shared-source results

| Authored case group | Pilot Light | Cisco deterministic pipeline |
|---|---:|---:|
| Positive cases with the requested type present | 28 / 29 | 12 / 29 |
| Negative cases emitting the requested type | 0 / 10 | 3 / 10 |

There are 17 positive cases where only Pilot Light emits the requested type, 11 where both do, and one where only Cisco does. The Pilot Light-only cases include Deep Agents Python/TypeScript, LangGraph TypeScript, Semantic Kernel C#, configured Vercel loops, Anthropic tool runners, handwritten Python/TypeScript loops, the tested FastMCP bare decorator/proxy forms, MCP TypeScript server/client constructors, C# DI, Rust handler/tool declarations, Go server construction and cross-file tool registration. These observations apply to these concrete source forms and this scanner mode, not entire framework support matrices. Most of this coverage predates the pack feature; packs make the relevant mappings maintainable without rebuilding.

The first valid Pilot Light run hit 27/29. Its Strands MCP-client miss was fixed by adding qualified constructor mappings to pack data, with positive and shadowing-negative fixtures. The existing binary detected the client using `--no-default-packs --pack packs/agents.json --pack packs/mcp.json` before embedded defaults were rebuilt. The case corpus stayed unchanged. The positional `ModelSpec` miss is now fixed through bounded dataclass mapping and pack-defined model argument selectors. The corpus and expectations remain unchanged; see the [model-record contract](DETECTION_PACKS.md#model-records-and-future-identifiers).

Cisco emitted agent findings for these three authored negative cases: a plain StateGraph without a model/tool loop, an agent import without construction, and a reassigned imported Agent name. This is a difference against our explicitly chosen occurrence-level expectations; Cisco may intentionally report broader reference evidence. Do not generalize this to a production false-positive rate.

## Scope and reproducibility

- Cisco main pinned at [`317d72a9d0f4fe763a5f81a6fb51b5c78bee11c1`](https://github.com/cisco-ai-defense/aibom/tree/317d72a9d0f4fe763a5f81a6fb51b5c78bee11c1), checked 2026-10-27. All 244 archived source files were verified unchanged after the run. Base and analysis dependencies were installed from its frozen uv.lock in an isolated Python 3.13.12 environment.
- The harness invokes the actual unmodified `ScanPipeline(..., llm_config=None)` with the 1.10.0 raw DuckDB and upstream built-in catalog. Normal deterministic scanners, cross-reference and assembly run. **This excludes Cisco's LLM-enriched CLI mode.** It is not an end-to-end comparison with that mode.
- There are 39 authored development cases (29 positive, 10 negative), across Python, TypeScript, C#, Rust and Go, containing 40 source files. Both scanners receive identical bytes; manifest and source hashes are checked before scoring. Case labels are authored expectations, not independent expert annotations, and the cases are not held out. They do not replace the historical 34-repository plus four-repository-extension development corpus.
- Scoring checks presence of the requested artifact type. Full raw counts and other findings are retained; duplicates are not extra successes. All 39 Cisco runs completed without scanner warnings/errors in the selected final run. Pilot Light has 22 complete-within-declared-scope and 17 partial reports because broader capability limitations remain. The cross-file registration case includes an unresolved-local-module diagnostic despite its tool finding; it is retained in the CSV.
- Public model metadata networking was allowed for Cisco; no LLM credentials or target-code execution were used. Cache bytes and hashes are retained. Initial harness attempts incorrectly blocked Windows loopback sockets, then encountered disabled model-metadata access. Those attempts are retained but excluded from scoring; the final run corrected harness conditions without modifying Cisco detection code.
- Timings are raw observations, not a speed comparison: process/startup boundaries differ and both run on a shared workstation. No production precision/recall, universal coverage, performance percentile or runtime claim follows from these results.

Reproduction commands are in [the corpus README](../benchmarks/pack-corpus/README.md). The delivery includes `pack-comparison.csv`, `pack-comparison.json` and `detection-pack-benchmark-runs.zip`, with the selected and excluded runs, pins, dependency freeze, metadata cache and source corpus. `run.json` identifies final Cisco inputs; the comparison records the final Pilot Light executable and every source-file hash.

## Content pins

- Pilot Light Windows executable SHA-256: `7269db1738b8343a1c2421edbd7ea647cde162f3a32977c3e67e95000fdf6de5` (28,988,416 bytes).
- Corpus manifest SHA-256: `832f45debd9a793f12e1c3df03cfdf8314abebfd6e4730f50f43e6b7c1c63b3f`.
- Cisco source archive SHA-256: `b1ac268914d9bce7932ea74495dfcfb312c47ce87fd252a4ebb23375a626f921`.
- Raw catalog SHA-256: `b1165384a646094aa6a4762d676f285c1c68b6d7f9367c90a129b529904a3268` (1,906,503 rows).

## Every case

Counts below are findings of the requested type, not total findings.

| Case | Type | Expected present | Pilot Light | Cisco |
|---|---|---|---:|---:|
| deepagents-python | agent | yes | 1 | 0 |
| deepagents-typescript | agent | yes | 1 | 0 |
| openai-agents-python | agent | yes | 1 | 1 |
| openai-agents-typescript | agent | yes | 1 | 1 |
| langgraph-python | agent | yes | 1 | 1 |
| langgraph-typescript | agent | yes | 1 | 0 |
| crewai-python | agent | yes | 1 | 1 |
| autogen-python | agent | yes | 1 | 2 |
| adk-python | agent | yes | 1 | 1 |
| pydantic-ai-python | agent | yes | 1 | 1 |
| strands-python | agent | yes | 1 | 1 |
| mastra-typescript | agent | yes | 1 | 1 |
| semantic-kernel-csharp | agent | yes | 1 | 0 |
| vercel-configured-loop | agent | yes | 1 | 0 |
| anthropic-tool-runner | agent | yes | 1 | 0 |
| handwritten-python | agent | yes | 1 | 0 |
| handwritten-typescript | agent | yes | 1 | 0 |
| langgraph-state-only | agent | no | 0 | 1 |
| agent-import-only | agent | no | 0 | 1 |
| agent-unrelated-name | agent | no | 0 | 0 |
| agent-shadowed | agent | no | 0 | 1 |
| vercel-single-step | agent | no | 0 | 0 |
| llm-retry-not-agent | agent | no | 0 | 0 |
| fastmcp-server | mcp_server | yes | 1 | 1 |
| fastmcp-tool | tool | yes | 1 | 0 |
| fastmcp-proxy | mcp_gateway | yes | 1 | 0 |
| mcp-typescript-server | mcp_server | yes | 1 | 0 |
| mcp-typescript-client | mcp_client | yes | 1 | 0 |
| mcp-csharp-di | mcp_server | yes | 1 | 0 |
| mcp-rust-handler | mcp_server | yes | 1 | 0 |
| mcp-rust-tool | tool | yes | 1 | 0 |
| mcp-go-server | mcp_server | yes | 1 | 0 |
| mcp-import-only | mcp_server | no | 0 | 0 |
| mcp-unrelated-server | mcp_server | no | 0 | 0 |
| mcp-rust-wrong-trait | mcp_server | no | 0 | 0 |
| mcp-csharp-unresolved | mcp_server | no | 0 | 0 |
| strands-mcp-client | mcp_client | yes | 1 | 1 |
| model-spec-positional | model | yes | 1 | 1 |
| registered-cross-file-tool | tool | yes | 1 | 0 |

## Next qualification

Prioritize SDK-version-qualified mappings, real application repositories with independently reviewed source-level labels, and the remaining ModelSpec case with negative controls. Expand pack coverage only when its positive and negative fixtures establish the requested artifact semantics. Optional signed distribution and a hosted registry remain separate work. Linux/macOS native CI and full Cisco LLM-mode evaluation were not run.

## Current increment

Pilot Light was rerun on all 39 unchanged cases using the executable pinned above. The Cisco probe results were reused after re-verifying that main and all 244 upstream source files were unchanged. The fresh 38-repository Cisco replay is separate: [real-code results](REAL_CODE_BENCHMARK.md). Current probe outputs are included in `real-code-benchmark-runs.zip`; `detection-pack-benchmark-runs.zip` preserves the earlier Cisco probe run and harness history.
