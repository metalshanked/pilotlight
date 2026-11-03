# Pilot Light 0.1 implementation status

This is a tested development increment, not a production-readiness declaration. This greenfield 0.1 codebase uses one initial report and catalog schema. There are no legacy mappings or migration paths.

## Implemented

- Python model-record mapping and pack-defined model argument selectors, including private/future SDK model IDs. Simple local dataclass inference remains restricted to reviewed IDs. See [boundaries](DETECTION_PACKS.md#model-records-and-future-identifiers). The current [38-repository replay](REAL_CODE_BENCHMARK.md) retains the original 34 pins plus the four-repository SDK supplement.

- Detection packs: two embedded packs, 29 framework-entry rules and 106 positive/negative fixtures. Explicit local packs extend mappings and supported structural parameters without rebuilding. Validation, capability checks, deterministic composition, checksum-pinned installation and JSON/CSV/CycloneDX provenance are implemented. See [pack guide](DETECTION_PACKS.md) and [39-case Cisco comparison](PACK_BENCHMARK.md).

- Catalog expansion: 195 independent entries, 338 package coordinates, 411 import prefixes and 240 explicit semantic symbols. A preserved 1,906,503-row seed adds 21,114 exact Python API references. These are dependency evidence unless an independent semantic rule assigns a stronger type.
- Ten parsers including PHP and Kotlin; native alias/constructor improvements for Go, Java, Rust and C#. Composer, PEP 735 and central NuGet manifests, split MCP TypeScript packages and larger bounded pnpm parsing are covered by regression fixtures.
- Corpus-driven rules add RubyLLM direct clients/operations, .NET MCP client factories, Ollama Go clients/operations, Feast feature stores, MLflow tracking/registration and DVC filesystem evidence. Package-only additions include Flowise integrations, Dify workspace packages, memory services, sandboxes and MCP tools. See [coverage improvements](COVERAGE_IMPROVEMENTS.md).
- Explicit registry audit supports PyPI, npm and Composer: the prior 295-coordinate snapshot verified 262 identities, with 33 awaiting adapters and zero failures. This increment records 34 live registry responses and one pinned workspace identity in `catalog/gap-discovery-evidence.json` (one coordinate was already present). Identity checks do not validate all SDK semantics.
- The benchmark runner records all fixed targets, source hashes, input/catalog/binary fingerprints and timing/failure data. Public pinned scans support unauthenticated codeload access and configurable archive size limits.

- Shared 33-type registry, active-catalog capability discovery, `types list`, type/group selection, exclusions, operator-config precedence and strict coverage exit status. Unsupported types are visible; types with no executable detector cannot be selected alone. The registry does not imply 33 working detectors.
- Canonical report kinds: `dependency` includes imports, `model` uses a reference subtype, and cataloged agent workflows use the agent type. Repository-scoped occurrence IDs include source columns; they are not runtime identities or refactor-stable definition IDs. Relationships are filtered with their endpoints.
- Deep Agents PyPI/npm coordinates, Python factory/alias calls, JS/TS named/namespace factory calls including browser/node entrypoints, and language-specific signature constraints. ADK's cataloged Python `RemoteA2aAgent` constructor provides initial proxy coverage. No generic remote-proxy detector is claimed for other languages.
- Skill identity from validated ASCII-name/frontmatter fields, without exporting instruction bodies or executing anything. MCP JSON/JSONC/YAML/TOML configuration references, including `.codex/config.toml`; source constructors remain separate evidence. YAML aliases are rejected before value construction.
- Arbitrary named YAML/JSON/TOML metadata indicators. Nested monorepo application roots inherit parent metadata, with the most specific configured root winning. Missing required indicators fail; missing optional indicators produce diagnostics. Paths are explicit and confined to the repository.
- JSON, JSONL, table, CSV and CycloneDX use the same selected findings. CSV file output gets a coverage sidecar automatically; CSV on stdout requires `--coverage-output`. Sidecars retain empty/failed repositories, metadata, selection and a primary-output hash. Optional relationship CSV has selected endpoints only. Output files are staged before replacement and individually atomic; the bundle is not a filesystem transaction.
- Lockfile extraction: npm v1/v2/v3, Cargo v1–v4, uv v1, Poetry 1.0/1.1/2.0/2.1, pnpm 6/9, Composer, NuGet v1/v2. Exact package/version evidence is separate from manifest declarations and installation/runtime claims. Unsupported version formats are not guessed. Not all dependency scopes or workspace links are resolved.
- Notebook format 4: Python/JS/TS code cells are scanned independently and carry cell numbers. Markdown, outputs and attachments are ignored. Cross-cell state/order is unresolved and produces a partial-coverage diagnostic.
- Exact catalog receiver-operation rules replace generic method-name LLM inference. SDK administration operations are not automatically LLM calls. Initial rules cover selected OpenAI, Anthropic, Google Gen AI, Together, Groq, OpenRouter and LangChain paths; missing operation mappings are still gaps.
- Catalog provenance and language restrictions have schema fields. Optional DuckDB export includes operation/provenance tables and retains the complete canonical document. Existing catalog entries still need a systematic provenance/version audit.
- `--continue-on-error` records failed batch targets while preserving successful results and exiting 1. `--corpus-file` respects every target's full pinned GitHub commit; this is acquisition/scanning input, not an accuracy scorer or evidence that the corpus was evaluated.

- Initial Python module graph: named re-exports, namespace import aliases, relative imports and undecorated synchronous single-return factory wrappers resolve across up to eight modules using the active catalog. Evidence records the resolution path. Cycles, ambiguous paths, local SDK-name shadowing, reassigned wrapper globals and complex wrappers abstain. `src/` layouts, wildcard imports and general interprocedural data flow remain unresolved. No caller model argument is attributed through a wrapper unless directly evidenced elsewhere.

## Operator examples

```sh
pilotlight types list
pilotlight scan examples/enterprise --config examples/enterprise.toml --format csv --output inventory.csv
pilotlight scan . --types agents,mcp,skills --exclude-types mcp_gateway --format json
pilotlight scan . --types agent,skill --strict-coverage --format csv --coverage-output coverage.json > findings.csv
pilotlight scan --repos-file repositories.txt --continue-on-error --format jsonl --output fleet.jsonl
```

`inventory.csv.coverage.json` binds to `inventory.csv` using SHA-256 and includes metadata even for zero findings. Dynamic columns use `metadata.<configured-name>`; application ID is one convention, not a special hardcoded field. A quoted `"0007"` ID stays a string. CSV formula-like values are neutralized. `--relationships-output relationships.csv` exports the selected graph separately.

Exit codes: 0 completed within normal CLI policy; 1 fatal error or any explicitly continued failed target; 2 strict diagnostics/coverage gate failed. With `--strict-coverage=false`, an operator can override the configuration's coverage gate. A normal zero exit does not prove universal absence of AI.

The semantic config hash includes effective selection, indicator rules, application roots and file-size limits; worker count/output formatting do not change this fingerprint. Coverage is relative to the declared lexical/config detectors and default exclusions. Entire ignored trees are not counted. The supported type/language matrix is catalog-derived rather than a blanket language claim.

## Milestones still open

M1's usable selection/attribution/reporting slice is implemented; full entity-versus-occurrence identity, rule-level explain and more comprehensive failure accounting remain. M2 has initial catalog/format/lockfile breadth; signed updates, SDK version constraints, broader coordinates, additional registry adapters and format completeness remain. M3 has an initial bounded Python named-export/simple-wrapper graph. JS/TS module resolution, richer wrappers, class/type resolution and new language adapters remain open. M4 has initial batch failure accounting and pinned inputs; GitHub Apps/Enterprise, org discovery, retries/rate budgets, resume, incremental caching and policy/diff remain. M5 native Linux/macOS execution, formatting/Clippy and signing remain unverified. M6 gold labeling/scoring and M7 enterprise qualification have not run.

The 34-repository corpus remains pinned and unlabeled. It has been used for development runs and gap fixes; none of it is a held-out acceptance set. A separate four-repository SDK extension has its own pinned baseline/candidate comparison. See [latest coverage improvements](ENTERPRISE_PATTERNS.md). Passing fixtures and producing findings do not establish precision, recall or production acceptance.

Agent/MCP increment: 49 additional exact symbols, Python bare tool decorators, C# tool attributes, MCP Python v2 role paths, Mastra MCP roles and explicit agent orchestration. Two generic StateGraph paths were reclassified as dependencies. The subsequent enterprise-pattern increment adds reviewed FastMCP gateway detection; other gateway platforms remain gaps. See [focused coverage](AGENT_MCP_COVERAGE.md).

The enterprise-pattern increment adds six regression groups for handwritten loop candidates, SDK loops, gateway factories, C# service registration and Rust macros. See [current pattern boundaries and benchmark comparison](ENTERPRISE_PATTERNS.md).
