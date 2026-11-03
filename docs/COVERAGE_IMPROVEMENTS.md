# Coverage improvements on the fixed 34 repositories

The candidate reruns all 34 original public commit pins with unauthenticated access. Complete extracted source-file path/size/SHA-256 inventories are equal for every baseline/candidate pair. Both runs use four workers, 2 MiB file limits, 512 MiB compressed and 2 GiB expanded archive limits. No source repositories were built or executed.

Outcomes: `{"partial": 34}`. All reports retain explicit coverage gaps; this is not a complete inventory guarantee.

| Capability | Before | After |
|---|---:|---:|
| Independent catalog entries | 170 | 192 |
| Exact package coordinates | 295 | 329 |
| Import prefixes | 324 | 395 |
| Explicit semantic symbols | 125 | 138 |
| Language parsers | 9 | 10 |
| Kotlin source files parsed | 0 | 39 |

The unchanged raw seed has 1,906,503 rows; the compact runtime seed index has 21,114 exact API references. Raw seed rows are not packages or detectors. New package provenance covers Flowise integrations, Dify Agent and Graphon, Pydantic harnesses, memory services, sandboxes and MCP tooling. Dify Agent is a pinned workspace package, not a verified PyPI release.

## Observable typed findings

Counts below are static evidence occurrences, including examples and tests. They measure changed detector output, not unique deployed services, precision or recall. Newly available feature-store, model-registry and data-versioning detectors join the default supported-type selection; both reports retain their effective selection.

| Repository | Artifact type | Before | After | Change |
|---|---|---:|---:|---:|
| langgenius/dify | dependency | 224 | 808 | +584 |
| langgenius/dify | experiment_tracker | 0 | 2 | +2 |
| FlowiseAI/Flowise | dependency | 832 | 1011 | +179 |
| microsoft/semantic-kernel | mcp_client | 0 | 3 | +3 |
| crmne/ruby_llm | embedding | 0 | 32 | +32 |
| crmne/ruby_llm | llm_call | 0 | 54 | +54 |
| crmne/ruby_llm | llm_client | 0 | 357 | +357 |
| crmne/ruby_llm | model | 0 | 29 | +29 |
| ollama/ollama | llm_call | 0 | 14 | +14 |
| ollama/ollama | llm_client | 0 | 96 | +96 |
| modelcontextprotocol/csharp-sdk | mcp_client | 0 | 185 | +185 |
| mlflow/mlflow | experiment_tracker | 0 | 12 | +12 |
| feast-dev/feast | experiment_tracker | 0 | 33 | +33 |
| feast-dev/feast | feature_store | 0 | 139 | +139 |

Requests, Express and ripgrep remain at zero findings. Shadowed-name, unrelated-call and exact-package near-miss fixtures also pass. These three repositories do not by themselves establish a false-positive rate.

## What changed

- Kotlin imports and aliases now use a structural parser with selected JVM SDK builders and local binding handling. Wildcard imports do not invent callable bindings.
- RubyLLM chat/embed factories and direct ask/complete calls are recognized, including direct factory-call chains. Ruby constant evidence and downstream operations remain `qualified_symbol`; no verified require/load graph is claimed.
- C# namespace imports resolve reviewed static MCP client factories. General dependency-injection MCP server configuration remains unresolved.
- Ollama Go factories and selected Chat/Generate operations have explicit rules. Conventional `(client, err)` factory assignments propagate the first client result; unrelated methods and reversed tuples abstain.
- Feast FeatureStore, MLflow tracking/registration and DVC filesystem constructions have dedicated types. Package-only matches never automatically become agents or servers.
- Python namespace imports resolve reviewed local exports and simple factories through the bounded module index. Unresolved local definitions retain diagnostics.

## Remaining gaps

Unsupported source files: `{"c_cpp": 58, "r": 64, "scala": 11, "swift": 1}`. R, C/C++, Scala and Swift adapters remain open. Kotlin parse/traversal errors, when present, remain visible in diagnostics.

The DVC repository still produces only one dependency finding: its local implementation is not resolved by the new external-API rule. General local definitions, `src/` layouts, dynamic imports, complex wrappers, whole-program types, direct HTTP AI calls, SDK-version constraints and fluent chains remain incomplete. Detection in framework implementation code is especially limited when a local symbol cannot be traced to a reviewed external API. No missing result is silently scored as correct.

49 default integration tests and 51 with DuckDB pass, plus enterprise application-ID/CSV/CycloneDX smoke checks and distribution packaging checks. Windows x64 is locally verified; native Linux/macOS CI and formatting/Clippy remain unverified.

All 34 repositories informed development; there is no independently labeled or held-out accuracy set. Timings are single shared-workstation observations, with other validation overlapping parts of execution. Peak memory is not measured, and no speedup or performance-percentile claim is made.

## Reproducibility

Baseline binary: `3aadc8b4e871323626fa9e6293631ae625c127b1ac2a5e2074effc5f9124f66c`. Candidate binary: `5628b4fb44959fff30cb056348c4c414e975825768e6d1e436603c510d881aef`.

Candidate catalog: `68308293a85a83deeb66c0bc3038ca3adac371d7c972d0d5454e9308d0897beb`. Corpus: `119081383fd1c5db734425dc8067d69c3c2eea2c7383e5352d849c7cd23e12cb`.

An initial candidate run was interrupted for a Ruby confidence-label correction. A subsequent complete candidate exposed a Ruby model-capability reporting mismatch, fixed before the final verified run. All candidate outcomes are retained; the final candidate starts again at repository 1. Earlier development history remains in benchmark-runs.zip.

Deliverables: coverage-comparison.csv (every repository/type), coverage-comparison.json (source identity and aggregate diagnostics), new-typed-evidence.csv (new typed/Kotlin evidence with pinned source links; notebook line numbers are cell-relative), and coverage-benchmark-runs.zip (baseline, interrupted candidate, capability-audit candidate and final candidate raw reports).
