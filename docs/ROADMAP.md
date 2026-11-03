# Roadmap

Current progress: [0.1 implementation status](IMPLEMENTATION_STATUS.md). This document retains the broader target contract; unimplemented requirements are not completion claims.

The authoritative next-release specification is [PRODUCTION_PLAN.md](PRODUCTION_PLAN.md), with [artifact types](ARTIFACT_TYPES.md), [coverage expansion](COVERAGE_PROGRAM.md) and the [fixed benchmark](../benchmarks/README.md). The summary below does not mark planned work as complete.

## 0.1 — started here

- Standalone Rust CLI, local/GitHub acquisition, commit attribution.
- Ten embedded parsers, exact package catalog, selected semantic constructions.
- Table/JSON/CSV/CycloneDX, repository indicators, multi-repo input list.
- Portable versioned catalog and optional bundled DuckDB, integrity-pinned updates.
- Fixture tests, cross-platform CI definition, honest coverage diagnostics.

## Next: useful enterprise breadth

Start with typed selection and initial report schema: agents, agent proxies, MCP servers and skills, including enterprise metadata and explicit partial/unsupported coverage. Close the Deep Agents gap in both Python and JS/TS with positive and negative evidence fixtures. Then follow production milestones M2–M7; the items below remain part of that scope.

1. GitHub org discovery and GitHub Enterprise URL support with explicit omissions/rate-limit handling.
2. JSONC/YAML/TOML MCP configuration, notebooks and lockfiles with actual resolved versions.
3. TypeScript/Python module graph, wrappers, re-exports, common SDK factories and configuration references.
4. Broaden Java, Go, C# and Rust semantic adapters; add languages driven by real repo samples.
5. Stable repository-scoped component identity and deduplication separate from finding occurrences.
6. Incremental cache with dependency invalidation; timing/memory benchmark corpus.
7. Signed catalog releases, rollback, provenance and versioned positive/negative tests per rule.

## Optional intelligence

Evaluate local Laya/Kev and hosted Jev/OpenAI Decisions in shadow mode. Publish measured code-inventory results before adding a default model. An LLM fallback, if ever added, should be explicit and budgeted. Neither LLMs nor decision models are required for the core inventory.

Runtime observation, active MCP interrogation, vulnerability scoring, hosted dashboards and compliance certification are outside the static 1.0 scope. The production taxonomy includes optional redacted credential-reference/pattern inventory; it never validates credentials or reports their values. The 0.1 executable does not implement that detector.
