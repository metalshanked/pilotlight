# Fixed public benchmark protocol

Status: 34 fixed repositories have been downloaded and scanned using unauthenticated public access. The original pin file is retained unchanged, including its historical planning metadata. All 34 are development data after gap-driven tuning. No independent gold labels, accuracy results or acceptance partition exist. See [development results](../docs/BENCHMARK_RESULTS.md) and the [latest same-source coverage comparison](../docs/ENTERPRISE_PATTERNS.md).

## Freeze and label before evaluating

Use [corpus.lock.json](corpus.lock.json) as the immutable source list. Repository purpose/language fields are selection hypotheses. They are not expected findings. A framework's own implementation is not equivalent to a consuming application; label package code, example, test, generated, vendored and deployment roles separately. Negative-control candidates are not assumed AI-free.

Before tuning, assign related framework/application families to development, calibration or acceptance partitions. Keep Deep Agents/LangChain/LangGraph and closely derived examples together; avoid train/test leakage through duplicated snippets and shared templates. Freeze partitions, source pins, bounded path scopes, versions of gold labels, selection profiles and thresholds. Public acceptance data cannot be truly blind; a separate private enterprise pilot remains required.

Two reviewers independently annotate the selected source scopes, then adjudicate disagreements. Labels need repository/commit, path, file-content SHA-256, line/column convention (1-based UTF-8 byte columns, exclusive end), canonical artifact type, declaration/reference role, symbol/config identity, evidence rationale, application attribution when available, reviewer identifiers and disposition. Relationship labels need both endpoints and evidence. Negative labels must identify concrete near-miss constructs and their rationale. Keep unresolved cases separate; do not count them as negatives.

Annotate bounded scopes exhaustively for recall. Finding examples by searching scanner output is insufficient. Do not manufacture labels or reviewer approvals. If independent review is unavailable, evaluation remains provisional. Record source licenses per relevant path; pins do not authorize redistributing arbitrary source or fixtures. Prefer minimal original fixtures and maintain source references.

This initial set does not qualify all 33 types or every planned language. PHP, Swift, Kotlin, C/C++ and rare type/language combinations require additional real applications and fixtures before support qualification. Add them through a new versioned corpus amendment before freezing acceptance; do not silently replace difficult repositories. Framework-heavy scopes require additional ordinary enterprise application samples. Fewer than 50 positives and 50 negative controls in a claimed type/language cell means provisional coverage.

## Runner contract

The prototype can scan one URL at an exact ref:

```sh
pilotlight scan https://github.com/OWNER/REPO --ref FULL_COMMIT_SHA --format json --output report.json
```

The 0.1 `benchmark --corpus-file benchmarks/corpus.lock.json --output-dir NEW_DIRECTORY` command records separate acquisition, hashing and scan times and preserves each outcome. It continues after acquisition or scan errors. `scan --corpus-file` also supports ordinary batch reporting. Gold preparation and scoring remain pending. The `--repos-file` format cannot express a different ref per repository. Download/acquisition tests and local-source scanner tests are separate stages. Never run repository builds, install project dependencies, start servers or call models.

Every target gets an outcome: completed, partial, unsupported, failed, timed_out or cancelled. Record attempted/succeeded/failed files, selection, diagnostics and exclusion reasons. Continue independent jobs after a failure and preserve completed results. A retry records its reason and prior outcome. No silent skips or selective retries that improve reported scores. A source exceeding limits is a coverage failure/partial outcome, not a fast successful scan.

Use `GITHUB_TOKEN` from the process environment for the current binary; do not paste a key into reports, source files, commands or this chat. Use repository Contents read permissions for needed private targets. Proposed GitHub App/fallback authentication requires separate implementation/tests. Obey rate-limit response headers and bounded retry budgets; acquisition time and errors remain visible. Pin redirects and authorization to trusted GitHub hosts.

## Reproducibility record

Capture scanner version and binary SHA-256, Cargo.lock hash, compiler/target, catalog/schema/config hashes, corpus and gold-label hashes, full source commit and extracted per-file hashes, flags, ignore rules, limits, cache policy, concurrency, OS/hardware and clock measurements. Record archive SHA-256 for that particular download; do not assume GitHub's compressed archive bytes remain identical across downloads of the same source commit. Verify source content identity separately. Record benchmark tooling versions and any random seeds.

Freeze the prototype baseline and candidate independently on identical inputs. An optional Cisco comparison must use documented settings/input scopes and report incompatible semantics; another scanner's output is not gold truth. Optional decision models have a separate opt-in experiment, pinned model/rubric and latency/accuracy/abstention measurements. No source egress occurs implicitly.

## Required test profiles

- Each individual artifact type, all supported types, and the user's `agent,agent_proxy,mcp_server,skill` combination. Type selectors are implemented in 0.1; protocol qualification is still pending.
- Unknown names, aliases, conflicting selectors, exclusions, wholly unsupported and partly supported requests. Verify requested/effective types are recorded.
- JSON/CSV/table and standards exports agree on selected entities, application IDs and relationship endpoints. Zero findings still produces repository/type coverage. Context entities remain separate.
- Direct imports, aliases, namespace imports, re-exports, wrappers, static constants, shadowing, dynamic imports, comments/string decoys, generic HTTP/database calls and SDK administration operations.
- Multiple `.company/.source/appinfo.yaml`-style application roots; required/optional missing indicators, leading-zero IDs, conflicts and malformed values.
- Archive/path abuse, symlinks, huge/malformed/encoded files, deep graphs, malicious skills, cancellation and rate-limit/network errors using local fixtures.
- Cold and warm cache; cache disabled/enabled results; controlled edits and dependency/catalog/config changes. Canonical findings must match full scans after normalization of time/performance metadata.

## Scoring and release gates

Use one-to-one entity/evidence matching under a frozen identity and localization policy; do not reward duplicate occurrences as extra correct assets. Publish entity and occurrence scores separately, with cross-file relationship/localization and application-attribution accuracy. Match canonical types exactly; a dependency cannot count as a correctly detected agent. Count misclassifications as a false positive for the predicted type and a false negative for the gold type.

For fully labeled scopes: precision = TP/(TP+FP), recall = TP/(TP+FN). Undefined denominators are N/A, not 100%. Report per-type/per-language and macro averages alongside totals; package-only detections must not swamp rare agents, proxies or skills. Report 95% intervals, negative-control false positives per KLOC, exclusions and unresolved labels. Publish conditional supported-scope accuracy and end-to-end coverage separately. Gold positives in failed/unsupported in-scope scans count as missed in end-to-end recall; unlabeled failures stay in reliability/coverage denominators and cannot be scored as correct negatives.

Proposed targets: high-confidence overall precision >=98%, recall >=95%; each qualified type/language cell >=95% precision and >=90% recall. Minimum 50 positives and 50 negatives is only a provisional sample floor, not proof of statistical sufficiency; publish intervals and require more data when uncertainty changes the release decision. Report both high-confidence and all-emitted results so lowering confidence cannot hide false positives. Every requested type must have an explicit coverage outcome. Targets are not achieved results.

Measure download separately from parsing/resolution/reporting. On a fixed 4-core/8-GiB/SSD host, perform at least five cold-process and five warm-cache trials; disclose OS page-cache treatment. Use at least 20 independent measured trials for any p95 release claim; five-trial exploratory results report raw runs/median/range only. Measure process-tree peak RSS and CPU time, not only the parent. Target 100 MiB/20,000 supported files within p95 60 seconds and 1 GiB peak RSS; a separate 1 GiB/100,000-file stress set within 10 minutes and 2 GiB. The actual corpus sizes are not yet measured. Controlled <=1% edits target <=25% of full warm-scan time with identical facts; broad invalidation must be explained.

Run the authenticated public evaluation after implementation, native release checks and gold preparation. Publish raw normalized outputs, failure manifests, scorer/config versions, metrics and limitations with appropriate redaction/license handling. A successful command or a large findings count does not establish production readiness.

See [the production milestones](../docs/PRODUCTION_PLAN.md), [type contract](../docs/ARTIFACT_TYPES.md) and [coverage expansion program](../docs/COVERAGE_PROGRAM.md).

## Enterprise-pattern development extension

`enterprise-patterns.lock.json` pins four additional SDK repositories: the official Rust MCP SDK, FastMCP, Vercel AI SDK and Anthropic Python SDK. It is a separate development supplement; the original 34 pins remain unchanged. Both sets have baseline/candidate source-inventory comparisons. Neither has independent accuracy labels.

Reproduce the final acquisition and scan limits with a new output directory:

```sh
pilotlight benchmark --corpus-file benchmarks/enterprise-patterns.lock.json --output-dir results-enterprise --jobs 4 --max-file-bytes 2097152 --max-download-bytes 536870912 --max-expanded-bytes 2147483648
```

The documented runs used unauthenticated public access. Reproduction needs network access; scanning the target never builds or executes its source. Use `benchmarks/corpus.lock.json` and a different output directory to rerun the original 34.


## Fixed-source Cisco main replay

`scripts/benchmark-real-code.py` acquires the two existing lockfiles without changing pins, records every extracted file's digest and replays the same trees through the two scanners. It never imports/builds target code. Archive links are omitted and recorded. Python 3.13 was used for the recorded run; Cisco requires its own frozen base/analysis environment and original DuckDB. Acquire Cisco main at the exact commit in the report, verify the archive/database hashes and install its frozen dependencies before these commands. No LLM stage is enabled.

```powershell
$env:PYTHONUTF8='1'
python scripts/benchmark-real-code.py prepare --work ./benchmark-work --corpus benchmarks/corpus.lock.json --corpus benchmarks/enterprise-patterns.lock.json
python scripts/benchmark-real-code.py run --work ./benchmark-work --scanner pilotlight --run pilotlight-candidate --exe ./target/release/pilotlight --workers 3 --timeout 300
/path/to/cisco-venv/bin/python scripts/benchmark-real-code.py run --work ./benchmark-work --scanner cisco --run cisco-main --cisco /path/to/cisco/source --kb /path/to/cisco/aibom.duckdb --commit 317d72a9d0f4fe763a5f81a6fb51b5c78bee11c1 --workers 3 --timeout 300
python scripts/summarize-real-code.py --work ./benchmark-work --before pilotlight-baseline --after pilotlight-candidate --cisco cisco-main --output comparison
```

Create `pilotlight-baseline` using the same `run` command and the retained baseline executable before summarizing. Each run directory must be new; outputs are not silently replaced. The final shared-source report records the exact binary hashes and outcomes. All 38 outcomes must exist before summarization. The supplied 300-second budget is per child process; replay timing also includes source-integrity checks and can exceed that budget. Cisco metadata caches are isolated per repository in concurrent runs. The CPython 3.13 Windows venv launcher owns a kill-on-close job for its child process, so a subprocess timeout also terminates that worker; Unix Python has no Windows redirector.

The summary is a coverage/differencing aid, not an accuracy scorer. Cisco's default assembly drops test-only components and consolidates occurrences; Pilot Light retains occurrence records. Missing outputs after timeout remain null, not zero. See [the real-code report](../docs/REAL_CODE_BENCHMARK.md) for scope, timings and limitations.

For the optional test-path breakdown, run `python scripts/cisco-test-paths.py --work <benchmark-directory> --cisco <pinned-cisco-source> --commit <same-sha>` using Cisco's environment before summarizing. It applies the unmodified upstream path policy; test-path evidence may still be retained when Cisco consolidates it with non-test occurrences. This is context, not an accuracy label.
