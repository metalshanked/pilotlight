# Architecture

## Design decisions

Rust CLI → local tree or immutable GitHub archive → bounded file enumeration → manifest/Tree-sitter extraction → catalog matches → evidence relationships → report.

Keep acquisition, extraction, catalog data and reporting separate. Do not require executing the target's package manager or build. The local scan has no network path. File scans use a bounded Rayon pool and are merged in sorted order. Python module indexing has a separate bounded prepass; parsed trees for up to 16 MiB of source are reused by the finding pass. Beyond this cache, Python files are reparsed to bound retained tree memory. Other source adapters parse once per run. Graph input is limited to 128 MiB, 100,000 export facts and eight resolution hops; limits produce explicit diagnostics. These are engineering bounds, not achieved performance benchmarks. Package/import matching spans 195 family entries; seeded API identities use per-entry ordered sets for logarithmic exact lookup. The 1.9 million raw catalog rows are never queried per syntax node.

Evidence is an occurrence in a file, not an application-wide canonical asset. Records carry path, line/UTF-8 byte column, optional notebook cell, mechanism, symbol and framework. IDs are namespaced by repository. Report and finding metadata support nested application roots. Definition identity remains distinct future work; these are occurrence IDs, not runtime identities.

## DuckDB

[DuckDB's Rust client](https://duckdb.org/docs/current/clients/rust/overview) supports bundling the engine and opening local databases. This can retain one executable while increasing build time and distribution size. Thus the prototype makes it an explicit build option. It exports normalized lookup tables plus a canonical JSON document and loads that document once per scan. Database connections are not shared across scanner workers.

The initial catalog schema includes exact operation rules, optional provenance URLs/review dates and symbol-language restrictions. SDK version ranges, extractor minimum versions, fixture IDs, retirement metadata and signed release manifests remain planned. The reviewable overlay JSON is the maintained source of truth. An explicit pinned seed import preserves raw upstream rows and builds a conservative API-reference index; curated overlay classifications take precedence. Parameters, attributes and methods remain context data, without becoming standalone model/agent findings. Atomic update, last-known-good rollback and signature verification belong at the catalog boundary. Do not execute arbitrary SQL from a downloaded update manifest.

## Code graph

The scanner combines lexical bindings with a bounded Python module-export graph, named re-exports and synchronous single-return factory wrappers. Resolution chains are attached to evidence. Cycles, shadowing and ambiguous module paths abstain. It is not a full call graph; JS/TS modules, general wrappers, type inference and richer language-neutral facts remain next steps. Source hashes prevent reusing a cached tree for a changed file; mutable local trees still require snapshot discipline for coherent cross-file analysis. See [research](CODE_GRAPHS.md).

## Sources and deployment

GitHub archives use the [repository contents API](https://docs.github.com/en/rest/repos/contents#download-a-repository-archive-tar). Public full-commit pins use direct codeload downloads; other references are resolved through the API before retrieval. Authorization goes only to api.github.com; archive redirects are restricted to HTTPS codeload.github.com without forwarding the token. The archive reader bounds entry counts and expanded/compressed bytes and omits links with diagnostics and rejects traversal, other special entries and Windows filename aliases. No target code is executed. This is read-only static analysis, not a full hostile-filesystem sandbox: concurrently changing local trees should be scanned from snapshots.

For org scale, acquire a repository list outside the scanner or use `--repos-file`. Future org discovery should page GitHub APIs, record omissions/rate limits and use installation tokens. Incremental caching should key on source digest, extractor version, catalog digest and effective scan configuration; dependency changes must invalidate dependent symbols. No incremental cache is implemented yet.

## Performance evaluation

Measure cold and warm scans separately: download time, bytes/files parsed, parsing CPU, peak RSS, catalog lookup time, graph-resolution work, p50/p95 scan time and binary size. Use fixed repo commits and a labeled corpus, not framework counts alone. Accuracy metrics: per-language/per-kind precision, recall, duplicate rate and unresolved rate. Performance is a hypothesis to measure, not a guarantee derived from Rust.

## Declarative detection packs

`packs.rs` validates operator-selected pack data and composes an immutable catalog before scanning. The embedded full snapshot supplies the unowned baseline and compact seed references; bundled pack-owned entries are removed and reintroduced from the selected pack set. External packs compose transactionally, rejecting duplicate pack identities and scalar semantic conflicts. A custom `--catalog` is already a complete baseline. Exact pack hashes and entry-rule origins survive report serialization.

The engine provides bounded syntax, receiver and control-flow mechanisms; packs parameterize recognized bindings, configured loops, Rust attributes and C# DI registrations. Unknown mechanisms require an engine capability addition. Pack data cannot execute code or initiate downloads. `packs test` scans inline fixtures in isolation. See [the authoring contract](DETECTION_PACKS.md).

Python model references use two mechanisms: reviewed qualified-call selectors accept bounded literal identifiers, while conservative local dataclass field mapping requires an exact catalog model identity. Both retain lexical shadowing checks; neither uses a `ModelSpec` classname heuristic or executes constructors. Selectors and known IDs are pack data. The local mechanism does not infer arbitrary cross-file classes or constructor control flow.
