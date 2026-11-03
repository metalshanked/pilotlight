# Independent catalog maintenance

Pilot Light maintains independent package coordinates, import mappings and semantic rules. The 29 bundled agent/MCP framework definitions live in `packs/*.json`; other definitions live in `catalog/overlay.json`. Run `scripts/sync-builtin-packs.py` after editing a bundled definition to refresh its complete overlay/runtime snapshot entries while preserving seed references. The current overlay contains 195 entries, 338 exact package coordinates, 411 import prefixes and 240 explicit semantic symbols. The generated `catalog/catalog.json` is the complete offline runtime snapshot embedded in the binary.

## Seed data and matching

The optional DuckDB seed preserves the upstream `component_catalog` table with its original seven VARCHAR columns: `id`, `label`, `concept`, `framework`, `sig_name`, `type`, `catalog_label`. No upstream rows are deleted or reclassified. A separate `pilotlight_catalog` table stores the merged runtime document and source provenance.

The pinned source has 1,906,503 rows and 1,879,220 distinct IDs across eleven framework labels:

| Row label | Rows |
|---|---:|
| parameter | 1,335,355 |
| method | 419,972 |
| attribute | 129,255 |
| class | 16,795 |
| function | 5,126 |

This is API-detail depth, not two million packages or detectors. The source has no explicit ecosystem/language columns. Some IDs are synthetic and some concepts include generic utility types. The importer therefore maps only valid Python class/function paths under independently reviewed import prefixes, deduplicates exact identities, and emits dependency-reference evidence. The resulting index contains 21,114 exact references (15 overlap independently classified symbols); 800 callable rows could not be mapped and remain raw data only. Parameter, method and attribute rows remain available for inspection/context but are not standalone findings. Verified overlay rules take precedence over generic references. Upstream concept labels never automatically establish a model, agent or LLM call.

The normal executable embeds this compact index and does not need the raw database or database engine at scan time. Exact API references use ordered-set lookup, not a linear scan of all upstream rows. The full seed is an optional sidecar for inspection and import workflows. A normalized JSON snapshot is also usable with `--catalog` in every build.

## Rebuild the seed explicitly

Use a build with the `duckdb-catalog` feature for import/export. No source download or background synchronization occurs inside this command:

```sh
pilotlight catalog import-cisco --input upstream.duckdb --overlay catalog/overlay.json --sha256 b1165384a646094aa6a4762d676f285c1c68b6d7f9367c90a129b529904a3268 --source-url https://github.com/cisco-ai-defense/aibom/releases/tag/1.10.0 --output seeded.duckdb
pilotlight catalog export --catalog seeded.duckdb --output generated-catalog.json
pilotlight scan . --catalog generated-catalog.json
```

Outputs must be new paths. The importer verifies the copied input digest before opening it, validates the table schema and bounds, disables extension loading/external access, and retains the original input unchanged. Runtime JSON is bounded to 8 MiB; raw seed input to 512 MiB and three million rows. A supplied digest pins content, not publisher identity. Raw DuckDB files must come from trusted publishers.

For the embedded release, replace `catalog/catalog.json` with the reviewed generated snapshot and rebuild. The maintained overlay must be unseeded; this prevents accidentally carrying stale seed references into a refreshed import. Do not edit the generated file as the primary source. JSON-to-DuckDB `catalog export` creates normalized Pilot Light inspection tables; it does not recreate raw upstream rows from the compact index. Preserve the imported seed database when those rows are needed.

## Independent additions

1. Verify exact registry coordinates and import/call forms from official repositories or documentation. Record provenance and review date.
2. Edit the owning `packs/*.json` definition and fixtures, or `catalog/overlay.json` for entries outside bundled packs. New package names must be exact; import names cannot be safely guessed from package names.
3. Add positive and negative fixtures for semantic rules, including aliases, shadowing and near-miss APIs.
4. Regenerate the seeded snapshot, run tests and validate JSON/DuckDB equivalence. An overlay-only snapshot can be used when the seed is unnecessary.
5. Review moves/deprecations and missing language adapters separately. Package existence does not prove API semantics or supported SDK versions.

`scripts/audit-catalog.py` explicitly queries PyPI, npm and Composer without installing anything or promoting proposals. The current audit verified 262 coordinates, reported 33 without a registry adapter and had zero failed verifications. Registry metadata and an upstream catalog are discovery inputs; neither overrides reviewed rules automatically. More registry adapters, SDK-version qualification, signed releases and a hosted update service remain work to do.

Source snapshot, license and attribution are recorded in [third-party notices](../THIRD_PARTY_NOTICES.md). The source release is 1.10.0, pinned by the digest above and manifest commit `8d7bec099d5ddcd24dbda13a9777f3cf11e2c325`. The original database is 76,296,192 bytes. Pilot Light remains a greenfield 0.1 project with independent maintenance and no automatic upstream coupling.

This increment records package identity responses and pinned corpus source references in `catalog/gap-discovery-evidence.json`. Dify Agent is a workspace package, not a verified public PyPI publication. New coordinates establish dependency evidence; only explicitly reviewed symbols establish stronger artifact types.

Agent/MCP symbol provenance is maintained in `catalog/agent-mcp-evidence.json`; the independently maintained overlay defines semantic roles. The seed remains unchanged when these role rules evolve. Generic graph builders are not sufficient evidence of agentic behavior.

Sources for the enterprise-pattern rules and structural detectors are recorded in `catalog/enterprise-pattern-evidence.json`.

## Detection-pack composition

See [detection packs](DETECTION_PACKS.md) for local loading, checksums, capabilities and provenance. The seed retains its original schema and all raw rows; semantic pack changes are independent. A seed import produces a complete baseline without pack attribution. To serialize the current default composition with pack identities, use `catalog export` without `--catalog`; a complete custom catalog is otherwise authoritative and does not auto-load bundled packs. See [the pinned upstream comparison](PACK_BENCHMARK.md) for the current source revision.
