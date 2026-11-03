# Contributing

Pilot Light is at the prototype stage. Keep the common path small and offline.

Maintain additions in `catalog/overlay.json`; regenerate `catalog/catalog.json` using the pinned seed workflow in docs/CATALOG_POLICY.md. Retain required upstream license/attribution notices for reused code and data.

For catalog additions, supply exact ecosystem package names, language import prefixes, a primary-source homepage, qualified construction symbols where supported, and positive/negative fixtures. A library's presence is not proof of an agent. Avoid broad substring matches. New language support needs parser, extraction and coverage documentation together.

Use Rust 1.89+ and Python 3.11+ for maintainer tooling. Run `cargo test --locked`, `python scripts/test-audit-catalog.py`, `python scripts/test-public-source.py`, `python scripts/public_source.py --check-git`, `python scripts/dependency-notices.py`, and `python scripts/audit-dependencies.py`. The advisory check requires network access and sends only public dependency names/versions. Changes to the DuckDB integration also require `cargo test --locked --features duckdb-catalog`. Keep dependency versions in Cargo.lock and regenerate/review notices when they change. Do not check in credentials, proprietary source or model weights.

Formatting and lint cleanup are still pending; the local audit environment lacks rustfmt and Clippy. Maintainers with those components should run `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` and address findings before declaring a qualified release. These checks are not claimed to pass in this development snapshot.

Changes to detection should state expected false-positive/false-negative effects. Performance claims require reproducible corpus commits, machine details, flags and measurements. Release automation should pin actions to reviewed commit SHAs and include dependency license review, checksums and signatures before public distribution.
