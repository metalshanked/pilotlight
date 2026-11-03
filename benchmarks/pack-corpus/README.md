# Fixed pack comparison cases

The 39 cases are authored development probes, not independently labeled accuracy data or runnable applications. The manifest and 40 source files were frozen before the first valid comparison. Both scanners receive identical file bytes. Each case scores presence of its requested artifact type; other types and duplicate counts are retained but not scored.

Cisco main is pinned at `317d72a9d0f4fe763a5f81a6fb51b5c78bee11c1`. Obtain that source and the SHA-pinned 1.10.0 catalog documented in `docs/CATALOG_POLICY.md`. Install the upstream frozen `uv.lock` base and `analysis` extra in an isolated environment (no project or development group required). Use its Python for the Cisco harness:

```sh
python scripts/benchmark-cisco-deterministic.py --source /path/to/cisco-checkout --kb /path/to/catalog.duckdb --corpus benchmarks/pack-corpus --output /path/to/new-cisco-results --allow-metadata-network
python scripts/compare-pack-corpus.py --pilotlight /path/to/pilotlight --corpus benchmarks/pack-corpus --cisco-results /path/to/new-cisco-results --output /path/to/new-comparison
```

The first command invokes unmodified `ScanPipeline` with `llm_config=None`. It redirects public metadata caches to the result parent, records network attempts, logs, scanner timings, all source hashes and cache hashes, and does not execute target code. The model metadata cache is not immutable upstream: retain the captured bytes to reproduce its exact state. The second command rejects source identity mismatches and incomplete Cisco cases; it retains Pilot Light partial coverage and diagnostics. New output directories are required.

Read `docs/PACK_BENCHMARK.md` for limitations and the complete result table. Do not use these timings as a performance comparison: the harnesses have different process/startup boundaries and run on a shared workstation. Future validation needs independently reviewed source labels and fixed real application repositories with positive and negative expectations.
