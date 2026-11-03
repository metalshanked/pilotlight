# Pilot Light 0.1 validation — 2026-10-27

This Windows x64 development build uses Rust/Cargo 1.89.0. Independent accuracy labels and enterprise acceptance remain open. No migration or legacy compatibility layer is included.

## Executable and catalog

- Windows executable: 28,988,416 bytes; SHA-256 `7269db1738b8343a1c2421edbd7ea647cde162f3a32977c3e67e95000fdf6de5`. PE inspection confirms static CRT linkage with only Windows system DLL imports.
- Ten language parsers; 195 independent catalog entries, 338 package coordinates, 411 import prefixes, 240 explicit semantic symbols and 21,114 seeded exact Python API references.
- Two embedded packs, 29 framework-entry rules and 106 inline positive/negative fixtures. The separate fictional model-record pack passes four CLI fixtures.
- Five pack-defined model argument selectors cover reviewed Deep Agents/Semantic Kernel constructor and parser paths, including private/future IDs. Local dataclass inference has eight initial exact model IDs; it is intentionally narrower. See [the contract](DETECTION_PACKS.md).
- The optional seed preserves all 1,906,503 original rows, including duplicates/nulls, verified with bidirectional `EXCEPT ALL`. Its canonical document equals the default composed export and round-trips through the feature-enabled CLI. The raw upstream database remains unchanged; 800 unmapped callable rows remain visible.
- Selected new bindings have official SDK/registry source evidence. Prior registry audits and new evidence are separate; no claim is made that every coordinate or SDK version is qualified.

## Behavioral checks

All 71 default and 73 DuckDB-feature integration tests pass. Five model-record groups cover positional/keyword mapping, import aliases, private model IDs, data-only constructor selectors, shadowing, invalid/ambiguous calls and preservation of ordinary class decorators. A separate module test covers direct re-exports with resolution evidence and rejects unproven wrapper argument forwarding; instance values do not retain local dataclass constructor bindings. Pack fixtures also exercise current and earlier MCP adapter APIs, C# agent namespaces, Go constructors, TypeScript bindings and client/server role distinctions.

Earlier suites remain passing: artifact selection, application metadata and monorepo inheritance, leading-zero IDs, CSV and CycloneDX consistency, safe output handling, acquisition limits, package formats, ten-language bindings, Python module resolution, notebooks, handwritten loops, gateways, C# DI, Rust macros, deterministic pack composition, invalid/colliding packs, explicit configuration and checksum-pinned installation.

The final staged pattern smoke produces six assets and nine CSV evidence rows with application ID `0007` and valid CycloneDX 1.6. Installed-binary enterprise smoke is recorded in the delivery validation manifest. Packaging tests cover the five-platform matrix, pinned actions, archives, executable modes and overwrite rejection.

## Benchmarks

The unchanged 39-case authored probe corpus now yields 29/29 positive hits and 0/10 negative target emissions for Pilot Light. The unchanged Cisco main baseline yields 12/29 and 3/10. Its source/main pin was reverified; the Cisco probe outputs were reused at that identical pin while Pilot Light was rerun. These are development expectations, not independent accuracy estimates or results for Cisco's LLM mode. See [the probe report](PACK_BENCHMARK.md).

The [real-code report](REAL_CODE_BENCHMARK.md) records the fresh 34+4 fixed-repository replay. Every extracted source file matches the previous corpus, and each scanner receives the same trees. Pilot Light completes all 38 with partial-coverage reports and retains all prior finding IDs. Cisco runs its unmodified deterministic full pipeline, with three concurrent repositories and a 300-second per-process limit; every completion, warning and timeout is retained. Timeouts have no invented zero counts. Cisco consolidates occurrences and filters test-only components, so counts are not directly comparable accuracy units. Shared-workstation timings include integrity checks and do not establish speed ratios or percentiles.

## Remaining qualification

Linux/macOS native CI remains unrun. The release workflow defines Linux x64/ARM64, macOS Intel/Apple Silicon and Windows x64; only Windows is locally verified. Formatting/Clippy remain unverified because those components are unavailable and the earlier antivirus block was not bypassed.

General cross-file types, additional language adapters, broader direct HTTP semantics, SDK-version constraints, signed updates, incremental caching, enterprise/org acquisition, independent gold labels and production acceptance remain open. Optional decision-model enrichment is research; no model backend or weights ship. Static references do not establish runtime execution or universal absence of AI.
