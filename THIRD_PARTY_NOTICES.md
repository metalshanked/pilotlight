# Third-party notices

Pilot Light includes a normalized API-reference index derived from Cisco AI BOM's 1.10.0 component catalog. The optional full seed database preserves the upstream component_catalog table and adds Pilot Light metadata. The independently maintained overlay supplies packages, language mappings and verified artifact classifications.

Upstream project: https://github.com/cisco-ai-defense/aibom
Source commit: 8d7bec099d5ddcd24dbda13a9777f3cf11e2c325
Catalog SHA-256: b1165384a646094aa6a4762d676f285c1c68b6d7f9367c90a129b529904a3268
License: Apache License 2.0; see licenses/cisco-aibom-LICENSE.

Changes: retained raw rows in the optional DuckDB seed; deduplicated mapped Python class/function identities into dependency-reference evidence; added independently maintained detection rules, provenance and application metadata. Upstream concept labels are retained as source data and are not treated as verified AI artifact roles.

The Kotlin parser uses the unmodified tree-sitter-kotlin-ng 1.1.0 grammar from
https://github.com/tree-sitter-grammars/tree-sitter-kotlin at commit
77dd60ea0a9003ce062c9728a513ffe1aaff8c82. Copyright (c) 2024 Amaan Qureshi.
License: MIT; see licenses/tree-sitter-kotlin-LICENSE. Pilot Light adds its own
import, binding and artifact extraction adapter around that grammar.

## Rust dependencies

The versioned inventory in [licenses/dependencies.json](licenses/dependencies.json)
records all 297 locked Cargo dependencies, including optional features and platform
alternatives, with license expressions, source locations and hashes of the retained
license/notice texts in `licenses/dependencies/`. Public upstream copyright holders
and attribution are deliberately preserved. These are not Pilot Light maintainer
identities. Alternative licenses remain alternatives; this project does not select
the LGPL option where a permissive option is offered.

The default binary excludes DuckDB. Distributors of optional DuckDB builds must
also review the native dependencies bundled by libduckdb-sys and retain their
applicable notices. The Cargo inventory alone is not a complete native dependency
SBOM or legal opinion. Regenerate and review notices whenever Cargo.lock changes:
`python scripts/dependency-notices.py --generate`.

The tested binary uses Rust 1.89.0. Its standard-library attribution is retained
in `licenses/rust-1.89.0/COPYRIGHT-library.html`, copied from that official
toolchain. Refresh these notices when changing the distribution toolchain.
