# Pilot Light production plan

Current progress: [0.1 implementation status](IMPLEMENTATION_STATUS.md). This document retains the broader target contract; unimplemented requirements are not completion claims.

Date: 2026-10-10. Status: implementation specification, not a claim that the features below exist. Product target: an independent open-source Rust static AI inventory scanner, usable as one executable in local workflows and enterprise CI. Unlabeled public development runs now exercise acquisition and expose gaps; acceptance scoring still requires the labeling gates below. Regression fixtures run throughout development.

## What needs to change

The 0.1 prototype contains 56 framework/library entries, 146 ecosystem package coordinates, 175 language import prefixes and 84 explicit symbol mappings. Those are different measures. Neither the entry count nor a successful demo establishes production quality. Current lexical extraction is strongest in Python/JS/TS; other parsers largely provide import evidence. Current limits include no module graph, incomplete client-method semantics, no agent-proxy or skill detector, no artifact-type selector, no incremental cache, sparse formats/lockfiles and unexecuted Linux/macOS release jobs.

Keep the working acquisition/output/indicator foundations, but treat the evidence schema and detection pipeline as a versioned redesign. Do not fill a database with unreviewed rows to manufacture apparent breadth. Cisco AIBOM remains one research reference; Pilot Light has independent data, rules, taxonomy governance and releases.

The [coverage program](COVERAGE_PROGRAM.md) defines discovery, verification and semantic qualification across 16 AI workstreams, including an explicit Deep Agents gap-closure ticket. The [benchmark protocol](../benchmarks/README.md) and [34-repository lockfile](../benchmarks/corpus.lock.json) define the evaluation preparation. Package discovery, implemented recognition and measured accuracy are separate deliverables.

## Production outcome

An operator can scan a local snapshot, a GitHub repository or an explicitly selected fleet; choose AI artifact types; attach application ownership; export actionable findings and truthful coverage; reproduce results using exact scanner/catalog/config/source versions; and see every omission. No target code or package installation is needed. Deterministic scanning works offline. Each platform has a tested binary with verified distribution metadata.

The target artifact contract is [ARTIFACT_TYPES.md](ARTIFACT_TYPES.md): 30 Cisco-compatible core names plus Pilot Light's three call/client extensions. Users can request agents, remote-agent proxies, MCP servers/clients/gateways, skills, models, data/retrieval, lifecycle and operational assets. A type is selectable independently of language support; unsupported combinations are explicit, not presented as clean scans.

## Features required for the static 1.0 release

| Area | Required implementation | Completion evidence |
|---|---|---|
| CLI and configuration | Typed selection, include/exclude, named scope profiles, config validation, doctor/explain, progress on stderr, quiet/machine modes, cancellation, stable exit codes | CLI contract fixtures; no prompts in CI; invalid/unknown fields fail |
| Artifact identity and reports | Initial schema, repository/application entities, distinct occurrences, typed/evidenced relationships, schema validation and deterministic ordering | Golden reports; no dangling references; stable rescan/diff identities |
| Local acquisition | Git-aware and plain directory snapshots, ignore overrides, explicit untracked/generated/vendor policy, immutable local snapshot option, symlink handling with coverage reasons | Concurrent-change, symlink, path, encoding, permission and ignore fixtures |
| GitHub acquisition | Public/private repos, exact refs, org/repo lists, archived/fork policies, pagination, token auth, verified redirect rules, retries/rate-limit budgets, repository inventory and omissions | Recorded HTTP fixtures for 401/403/404/429/5xx, rename, token expiry, pagination and partial batches |
| Enterprise acquisition | GitHub Enterprise Server/Cloud configurable allowed hosts, proxy/custom CA, GitHub App installations, resumable per-repo jobs | Test server fixtures and an operator-provided enterprise integration environment before claiming support |
| Dependencies | Manifest and lockfile parsing, exact ecosystem coordinates, aliases, workspaces, direct/transitive/dev distinctions and resolved versions | Labeled fixtures across the ecosystem matrix; no build/package-manager execution |
| Source semantics | AST adapters, import aliases, shadowing, type-only imports, assignments, constants, module resolution, bounded wrapper/re-export tracing | Positive/negative fixtures and qualified cross-file edges with resolution reasons |
| Artifact detectors | All 33 target types have explicit definitions, minimum evidence, supported-language/format entries and tests | Per-type capability matrix and reviewed labels; no type registered as universally supported |
| MCP/A2A/skills | Protocol-specific config/code adapters, server/client/proxy distinction, tools/resources/prompts, skill metadata and references | False-positive controls and fixtures proving nothing is executed or fetched from target instructions |
| Configuration and deployment | JSON/JSONC/YAML/TOML, dotenv variable names, notebooks, Docker/Kubernetes/Helm/Terraform/workflow evidence, model-file headers | File-to-artifact attribution, source spans and unknown-template diagnostics |
| Application attribution | Multiple indicators, fixed paths and monorepo application roots, typed scalar selectors, precedence and conflict rules, required/optional semantics | Every entity mapped to application or explicit unassigned/conflict state |
| Catalog lifecycle | Independent rule/package registry, version constraints, provenance, review dates, positive/negative fixtures, compatibility bounds, deprecation, signed snapshots, local overrides and rollback | Reproducible build; signature/downgrade/compatibility tests; no implicit external update |
| Scale and performance | Bounded workers/queues, parser reuse, compiled lookup indexes, parse-once IR, deterministic merge, content-addressed cache and invalidation, streaming/partial batch output | Reference hardware measurements and memory/cancellation/resume tests |
| Exports | JSON and JSONL, findings CSV plus coverage/relationship files, terminal summary, schema-validated CycloneDX, SARIF with documented inventory mapping | Consumer fixtures; CSV formula safety; zero-findings/partial scans still visible |
| Change and policy workflows | Baseline/diff, added/removed/changed evidence, suppressions with reason/expiry, rule allowlists, fail-on conditions separate from scan failures | Stable diff fixtures; suppressed findings counted; report version compatibility |
| Operations and release | Five platform binaries, checksums and publisher verification, dependency SBOM/license review, reproducible catalog builds, support policy, vulnerability handling, CI gates | Native tests/smoke tests, release upgrade/rollback exercises and operator documentation |

Inventory is not automatic vulnerability scoring or compliance certification. Explicit IaC references describe declared infrastructure; this phase does not require cloud credentials, enumerate live accounts or probe endpoints. A registry/type list can be comprehensive while deep semantic coverage grows through measured releases.

## Coverage matrix to implement and publish

Track language × ecosystem × artifact type × detection mechanism × SDK version × fixture status. Publish levels: unsupported; dependency-only; import/config evidence; resolved calls; bounded cross-file inference. The scanner must export the level used, not a generic checkmark for a language.

| Language/format | Initial production depth | Packages and formats |
|---|---|---|
| Python | Tier A: binding/module graph, factories, context-manager clients, async calls, decorators, wrappers and notebooks | requirements/pyproject, Poetry/uv/pip lock formats with explicit version support |
| JS/TS/JSX/TSX | Tier A: ESM/CJS, namespace/destructuring aliases, type-only imports, tsconfig paths, workspaces, re-exports and factories | npm/yarn/pnpm/Bun manifest/lock formats with explicit version support |
| Java/Kotlin | Tier B: imports, constructors, annotations, common injection/config patterns; bounded resolution | Maven/Gradle declarations and lock/version-catalog data; no build invocation |
| C# | Tier B: namespaces/aliases, constructors, extension/builder patterns and config references | csproj, central package versions and NuGet lock/assets data |
| Go | Tier B: module imports, alias/receiver construction, SDK calls and config | go.mod/go.sum; distinguish checksum presence from resolved dependency graph |
| Rust | Tier B: grouped uses/aliases, constructors/builders and selected receiver patterns | Cargo manifests/lockfile/workspaces; unresolved macros/traits reported |
| Ruby | Tier B: literal requires, constants, common client/DSL patterns | Gemfile/Gemfile.lock without evaluating Ruby |
| PHP | Tier B: namespaces/use aliases, constructors and provider calls | composer.json/composer.lock |
| Swift | Tier B: imports, common clients/builders and package evidence | Package.swift literals and Package.resolved without evaluating Swift |
| C/C++ | Tier B: includes, model runtime construction/config and artifact references | CMake/vcpkg/Conan manifests/lock data; preprocessor ambiguity reported |
| Protocol/config documents | Tier A for validated schemas; bounded variable/reference resolution | MCP JSON/JSONC/TOML/YAML, A2A cards, SKILL.md, model configs, serving/IaC manifests |

Tier A/B are release targets, not existing features. Ten parsers today do not meet this matrix. PHP and Kotlin now have initial adapters; Swift and C/C++ remain planned. Tier B does not claim full compiler semantics. Scala/R/Julia/Elixir and other languages follow measured demand; until then report unsupported source plus any supported package/config evidence. There is no honest finite promise of “all known languages/frameworks.”

Prioritize maintained official provider SDKs, OpenAI-compatible endpoints, enterprise cloud SDKs, agent/orchestration frameworks, MCP/A2A/skills, embeddings/retrievers/stores, model serving/training and evaluation/guardrail tooling. For each framework family enumerate its actual published ecosystem packages and SDK versions, not one family row. Discovery can use registries and upstream docs, but additions pass review and fixtures; target scanning never installs those packages.

## Architecture

Acquisition and authorization → immutable source inventory → scope/type plan → manifest/config/source adapters → language-neutral facts → bounded name/value/module resolution → independent catalog/rules → entities/findings/relationships → selection and attribution → reports/coverage/diff.

Refactor into separable Rust modules/crates only when boundaries stabilize: acquisition, formats, syntax/IR, resolver, catalog, detectors, inventory schema, reporting and CLI. Use deterministic data contracts so a defect in one adapter can be localized. Parser cancellation, file budgets, graph depth/node budgets and output budgets are shared services. Unresolved facts retain reasons rather than being guessed away.

### Catalog and DuckDB

Reviewable source data should distinguish framework family, package coordinates, version intervals, language import paths, symbol aliases, constructors, operation semantics, endpoint signatures, config selectors and rules. Store source URLs/revisions, review dates, licenses, retired aliases, extractor compatibility and fixture IDs. Add conflict resolution between built-in and operator catalogs, with explicit override intent and effective hashes.

Generate compact offline lookup structures for the scan hot path. DuckDB can provide indexed catalog inspection/build joins and optional rich local inventory queries; it need not be queried for every AST node. Measure size/startup/throughput before making it mandatory. The optional seed importer accepts the upstream component_catalog schema while keeping the maintained overlay and report taxonomy independent. Signed catalog releases and scanner releases evolve independently within declared compatibility ranges.

### Code graph

Start with imports, declarations, references, assignments, calls, returns and config bindings. Trace only relevant candidate subgraphs, with bounded depth and explicit unknowns. Resolve Python modules and TS/JS workspaces first, then each Tier B adapter. Cache both positive and negative resolutions and invalidate on dependency/module/config/catalog changes. Do not execute user code to obtain types. Optional SCIP imports may improve resolution when indexes already exist; a graph server, compiler build and Joern are not required for the base executable.

### Credential and content boundaries

Read GitHub credentials from environment or an explicit credential helper; never command-line values, report fields or logs. Use host-specific authorization and redact signed archive URLs/errors. The current binary supports only `GITHUB_TOKEN`; `GH_TOKEN` fallback and GitHub App refresh are planned. A GitHub token was not present during this planning session.

For the benchmark, use a fine-grained token scoped to necessary repositories/read permissions; for enterprise fleet operation prefer installation-scoped GitHub App credentials. Repository visibility and org-discovery permissions are checked separately. Do not assume all credential types have the same rate budget. Serial API scheduling, conditional requests and bounded backoff follow [GitHub's recommendations](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api); observe response headers and report exhaustion rather than sleeping indefinitely. See [installation authentication](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation).

Local scans have no implicit network use. Archive links, path traversal, huge/binary files, malicious YAML aliases, recursive graphs, malformed syntax, encoding failures and hostile model formats need adversarial fixtures. Skill files and source comments are data, never instructions. Redact source/secret content by default; application indicators are explicit export choices. Catalogs and model artifacts are data, not executable plugins.

## Delivery milestones and dependencies

| Milestone | Work | Exit gate |
|---|---|---|
| M0 — production contract | Finalize taxonomy, selection/report schemas, evidence identity, supported matrix and benchmark protocol; capture prototype baseline | Schemas and feature acceptance list reviewed; no ambiguous “supported” claims |
| M1 — reliable inventory core | Typed detector registry/selection, initial report schema, attribution, diagnostics, coverage sidecars, CLI contracts | Agents/proxies/MCP/skills selections behave consistently in fixtures; unknown/partial states visible |
| M2 — catalog and broad extraction | Independent catalog and seed overlay, package/lockfile coverage, MCP/A2A/skill/config/notebook adapters and core Tier A semantics | Each mapped package/rule has provenance and positives/negatives; zero execution/network side effects |
| M3 — semantic breadth | Bounded cross-file graph, factory/constant resolution, Tier B adapters, remaining artifact families and evidence relationships | Declared type/language matrix has matching gold fixtures; no unsupported cells silently upgraded |
| M4 — enterprise reliability | GitHub fleet/auth/rate limiting, partial/resumable batches, stable application IDs, proxy/CA support, caching, diff and policy | Failure injection, resume, deterministic cache invalidation and memory-limit tests pass |
| M5 — release candidate | Native platform jobs, format/lint/security checks, artifact/schema validation, signing/verification, operator guides and upgrade checks | Reproducible release candidate and independently labeled fixed corpus ready |
| M6 — fixed public evaluation | Authenticated scans of pinned repos; score gold labels and adversarial fixtures; measure cold/warm/local/network paths | Published scorecard meets acceptance thresholds; failures fixed and rerun on the same pins |
| M7 — enterprise pilot | User-provided private/enterprise samples and deployment constraints | Documented coverage delta, permission behavior and operational acceptance; only then declare production readiness |

M1 depends on M0. M2/M3 share the M1 contracts; M4 can progress on acquisition after M1. M5 needs M2–M4. M6 requires M5 and gold labels, not just a list of URLs. M7 cannot be inferred from public GitHub success. No calendar estimate is asserted before the first representative vertical slice is measured.

The first vertical slice is exactly the requested flow: `--types agent,agent_proxy,mcp_server,skill`, application ID extraction, consistent JSON/CSV/coverage, deterministic local fixtures, then the corresponding pinned public repos after the release-candidate gate. This is a concrete slice of the production architecture, not a substitute for the remaining milestones.

## Proposed acceptance gates

These are proposed engineering targets, not achieved metrics. Freeze thresholds and scope before opening the final acceptance labels. Revisit targets using measured pilot requirements transparently; do not weaken them after seeing failures without versioning the protocol.

- Taxonomy/reporting: every requested type gets explicit coverage; all selected-output fixtures pass; no dangling relationships; zero silently lost repositories; stable repeat-scan identities.
- Accuracy: on fully labeled supported scopes, high-confidence findings target at least 98% precision and 95% recall overall. Each qualified type/language stratum targets at least 95% precision and 90% recall. Publish sample sizes and 95% intervals; cells with fewer than 50 positive and 50 negative examples remain provisional. A high aggregate cannot hide a failed stratum.
- Gold quality: exhaustively label designated bounded scopes for recall. Two reviewers adjudicate acceptance labels and disagreements. If second review or adequate labels are unavailable, mark evaluation provisional; scanner output is never its own ground truth.
- Reliability: every pinned target yields a complete report or explicit failure; unsupported/failed targets remain in the denominator. Cancellation and per-repo timeout must preserve completed results and a recoverable manifest. No panics on the adversarial corpus.
- Safety: no target code execution, unrequested network egress or credential/prompt disclosure in fixture tests. Archive/parser/graph fuzz jobs and dependency review have no unresolved release-blocking findings.
- Performance: proposed reference machine is 4 dedicated CPU cores, 8 GiB RAM, SSD, no GPU. On pre-downloaded supported source totaling at most 100 MiB/20,000 files, target p95 at most 60 seconds and peak RSS at most 1 GiB. For a separate 1 GiB/100,000-file stress set, target at most 10 minutes and 2 GiB RSS. Measure download separately; exceeding configured limits is explicit partial coverage, not a speed success.
- Incremental: for controlled changes affecting at most 1% of files without broad dependency invalidation, target at most 25% of warm full-scan time, with identical relevant findings to a full scan. Broad invalidation is allowed but explained.
- Releases: every advertised OS/architecture passes native tests, selector/report smoke tests and archive verification. Linux/macOS workflow definitions alone do not satisfy this gate. Formatting/Clippy remain open from the prototype and must be resolved.

## Optional extensions after the static foundation

Local decision models (Laya/Kev) and hosted Jev/OpenAI Decisions are optional classifiers for unresolved candidates, with abstention, versioned rubrics, bounded inputs, explicit egress permission and independent accuracy/latency tests. They cannot erase deterministic evidence or decide to skip entire repos. Disabled remains a fully supported mode. No model performance claim is inherited from vendor benchmarks.

Later runtime ingestion can correlate instrumented model/agent/MCP observations with deployment commit/image/application identities. Use an adapter boundary for evolving [OpenTelemetry GenAI conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/), keep static declarations separate from observed events, and retain first/last observation times. Active MCP introspection, cloud discovery, UI/control plane, RBAC/SSO, alerts, vulnerability feeds and compliance mappings are separate product tracks; they must not turn the standalone static CLI into a required service deployment.

## Standards and maintenance

Keep native JSON as the lossless inventory contract. Map compatible facts to [CycloneDX AI/ML-BOM](https://cyclonedx.org/capabilities/mlbom/) without manufacturing training/license facts. SARIF carries location-bearing findings but is not the complete graph. Revisit SPDX mappings as a separately tested exporter rather than claiming standards compliance from a JSON header.

Maintain a coverage dashboard generated from rules/fixtures, a supported-SDK/version policy, deprecation notices, changelog, reproduction templates, issue triage and catalog review ownership. All external-source-derived mappings retain provenance and license obligations. A production release needs these maintenance processes as much as a bigger catalog.
