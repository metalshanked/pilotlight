# Pilot Light

**Know where AI lives in your code.**

Pilot Light is a Rust command-line scanner for evidence-backed AI inventory in local and GitHub.com repositories. One executable, an embedded catalog, and no account or LLM required for local scans.

**Status: working 0.1 development build.** Ten language parsers, 195 independent catalog entries and 21,114 seeded API references are included. Coverage is deliberately documented by capability: this is not universal detection, runtime monitoring, or proof that an asset executes. See [coverage](docs/COVERAGE.md).

The [33-type registry](docs/ARTIFACT_TYPES.md), [coverage program](docs/COVERAGE_PROGRAM.md), and [fixed benchmark protocol](benchmarks/README.md) describe supported capabilities and remaining work. See [implementation status](docs/IMPLEMENTATION_STATUS.md) for the tested subset; independent gold labels and acceptance scoring remain pending. See the [historical repository comparison](docs/ENTERPRISE_PATTERNS.md) and the [real-repository Cisco comparison](docs/REAL_CODE_BENCHMARK.md).

## Start here

```sh
cargo build --release --locked
./target/release/pilotlight scan .
./target/release/pilotlight scan examples/demo --config examples/pilotlight.toml
```

On Windows use `target\release\pilotlight.exe`. A C/C++ compiler is needed to build the embedded Tree-sitter grammars; end users do not need Rust, Python, Node, Git, or a database server to run the resulting executable. Requires Rust 1.89 or later; the lockfile records the tested dependencies. No public binary releases or package registry publication have been created yet.

For a Windows x64 distribution with the Visual C++ runtime statically linked and local build paths remapped, run `./scripts/build-windows.ps1` from PowerShell. This maintainer script requires Python 3.11+. It writes `target/x86_64-pc-windows-msvc/release/pilotlight.exe`; pass `-DuckDB` to include the optional database engine. The ordinary Cargo build can depend on the installed Visual C++ runtime and embed local build paths. See [publication checks](docs/PUBLISHING.md) before distributing a build.

## Scan GitHub and multiple repositories

```sh
pilotlight scan https://github.com/OWNER/REPO
pilotlight scan https://github.com/OWNER/REPO --ref main --format json --output inventory.json
pilotlight scan --repos-file repositories.txt --format csv --output inventory.csv
```

`repositories.txt` is one local path or GitHub URL per line. Blank lines and lines starting with `#` are ignored. Local paths are relative to the current working directory. A batch runs sequentially across repositories, with parallel file scanning within each repo. A failed repository aborts the batch before replacing the output by default. Add `--continue-on-error` to preserve successful reports plus explicit failed-target records; any failure still exits 1. JSONL emits one report per repository. Batch JSON has a `reports` array; single-repository JSON is a report object.

For private GitHub repositories, set `GITHUB_TOKEN` to a token with repository **Contents: read** access. Tokens are read only from the environment. GitHub scans resolve the requested ref to a commit SHA and download that exact archive. Nothing is cloned or executed. GitHub Enterprise, org enumeration, submodules and LFS content are roadmap items. Selected lockfile formats are now supported. `--corpus-file benchmarks/corpus.lock.json` uses each target's full pinned commit; it does not score accuracy or replace gold labeling. Archive links are omitted without following their targets, with explicit partial-coverage diagnostics. Traversal and other special entries are rejected. Public full-commit pins use direct codeload access without consuming REST ref-resolution requests. Acquisition defaults to 100 MiB compressed / 512 MiB expanded; explicitly raise `--max-download-bytes` and `--max-expanded-bytes` for larger repositories.

## Application IDs and enterprise metadata

Suppose each repository has `.company/.source/appinfo.yaml`:

```yaml
appid: 1234
application:
  owner: platform-engineering
```

Create an operator-owned `pilotlight.toml`:

```toml
[[indicators]]
name = "application_id"
path = ".company/.source/appinfo.yaml"
key = "appid"
required = true

[[indicators]]
name = "owner"
path = ".company/.source/appinfo.yaml"
key = "application.owner"
required = false
```

Then run:

```sh
pilotlight scan . --config pilotlight.toml --format csv --output inventory.csv
pilotlight scan --repos-file repositories.txt --config pilotlight.toml --format json --output inventory.json
```

Every CSV finding carries `metadata.application_id` and `metadata.owner`; JSON includes report and per-asset `metadata` objects; CycloneDX uses `pilotlight:metadata:*` properties. Multiple named indicators are supported. YAML, JSON and TOML scalar values are supported with dot-separated nested keys. Arrays and keys containing literal dots are not supported. Quote IDs with leading zeros to preserve them. A required missing/invalid value fails the scan; an optional missing value generates a diagnostic. Configure only metadata that you intend to export.

For one indicator without a config file:

```sh
pilotlight scan . --indicator-path .company/.source/appinfo.yaml --indicator-key appid --indicator-name application_id --indicator-required --format csv --output inventory.csv
```

Indicator paths are fixed relative paths, including hidden directories. They cannot escape the repository. Config is only read when explicitly selected with `--config`; target repositories cannot enable arbitrary behavior by shipping their own config.

## Artifact selection and monorepos

```sh
pilotlight types list --format json
pilotlight scan . --types agent,agent_proxy,mcp_server,skill --format csv --output inventory.csv
pilotlight scan . --types all --exclude-types secret,other --strict-coverage --format json
pilotlight scan examples/enterprise --config examples/enterprise.toml --format csv --output enterprise.csv
```

The shared registry has 33 canonical names. `types list` reports actual catalog/language capabilities, including unsupported types. Groups `agents`, `mcp`, `models` and `skills` expand explicitly; unknown names fail. Defaults select available detectors except secrets. `--strict-coverage` exits 2 for unsupported/partial requested coverage. An empty report means only that no selected evidence was found within the declared scope.

Operator TOML supports `[scan]` with `types`, `exclude_types` and `strict_coverage`, and `[[applications]]` with a repository-relative `root` plus `[[applications.indicators]]`. The most specific root wins and inherits parent metadata. Field names are arbitrary, so application ID, owner, business unit and other enterprise keys share one implementation. See [the complete example](examples/enterprise.toml) and [implementation notes](docs/IMPLEMENTATION_STATUS.md).

## Output and automation

| Format | Use |
|---|---|
| `table` | Human-readable terminal inventory and diagnostics; default |
| `json` | Complete evidence, relationships, metadata and coverage |
| `jsonl` | One complete repository report per line |
| `csv` | Spreadsheet/CMDB import; one row per finding evidence, with dynamic metadata columns |
| `cyclonedx` | CycloneDX 1.6 JSON with inventory properties and dependencies; single repo only |

CSV cells that could be interpreted as spreadsheet formulas are prefixed with an apostrophe. CSV finding rows remain empty for zero findings, but file output automatically writes `OUTPUT.coverage.json` with every repository, metadata and type status. CSV on stdout requires `--coverage-output PATH`. The sidecar includes the exact primary-report SHA-256. Optional `--relationships-output PATH` exports selected edges. Output order and IDs are deterministic for unchanged files, source location and catalog. IDs are repository-scoped occurrences using path/line/column; they are not global runtime identities or refactor-stable definition IDs. Manifest/config evidence uses line 1 as file-scoped evidence, not a precise key location.

```sh
pilotlight scan . --format json --output inventory.json --strict --jobs 4
```

Exit codes: `0` completed under normal policy, `1` fatal failure or any continued failed batch member, `2` diagnostics under `--strict` or incomplete requested coverage under `--strict-coverage`. Unsupported source languages are counted and affect requested type coverage; use `--strict-coverage` to gate them. Other noncandidate files are skipped. Ignored directories/files are outside the enumeration count. A clean exit does not prove that unsupported AI usage is absent.

## Catalogs that evolve separately from the binary

Pilot Light combines a preserved upstream component catalog with an independently maintained overlay. The standalone binary embeds a compact lookup index; the optional full DuckDB retains 1,906,503 source records. Updates are explicit and reviewed. See [catalog maintenance](docs/CATALOG_POLICY.md) and [third-party notices](THIRD_PARTY_NOTICES.md).

```sh
pilotlight catalog list
pilotlight catalog validate catalog/catalog.json
pilotlight catalog export --output company-catalog.json
pilotlight scan . --catalog company-catalog.json
pilotlight catalog update --url https://YOUR-HOST/catalog.json --sha256 TRUSTED_SHA256 --output company-catalog.json
```

The bundled snapshot works offline. An external catalog replaces it; edit `catalog/overlay.json` and regenerate the seeded snapshot to add internal packages and symbols. The update command validates schema and a caller-supplied SHA-256 before an atomic install. Obtain the digest through a trusted channel: a checksum alone is not publisher authentication. There is no automatic update schedule or hosted Pilot Light catalog feed yet. Language grammars and new extraction behavior still require a binary update; adding known package/import/symbol mappings does not.

The compact default build uses the JSON catalog directly. DuckDB is an optional **statically bundled build feature**, with no separate DB service:

```sh
cargo build --release --locked --features duckdb-catalog
pilotlight catalog export --output catalog.duckdb
pilotlight scan . --catalog catalog.duckdb
```

A JSON-to-DuckDB export provides `framework`, `package`, `symbol`, operation and provenance tables. A seed import preserves the upstream `component_catalog` table and adds the canonical Pilot Light document. The scanner loads and validates the document once; it does not issue a SQL query per syntax node. Use JSON for the smallest build. The initial catalog schema includes explicit operation rules, language restrictions and provenance. Only load catalog databases from trusted publishers. See [architecture](docs/ARCHITECTURE.md) for tradeoffs.

## Fixed public development benchmark

```sh
pilotlight benchmark --corpus-file benchmarks/corpus.lock.json --output-dir run-001 --max-download-bytes 536870912 --max-expanded-bytes 2147483648
```

Each run uses a fresh directory and retains source hashes, findings, failures, catalog/binary hashes and separate acquisition/scan timings. The 34 repositories are development data, not independently labeled acceptance data. See [results](docs/BENCHMARK_RESULTS.md).

## Development

Linux x64/ARM64, macOS Intel/Apple Silicon and Windows x64 packaging is defined in the [binary build workflow](.github/workflows/binaries.yml). Windows has been tested locally; the Linux/macOS jobs still need to run on GitHub. See [release targets and status](docs/RELEASES.md).

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --features duckdb-catalog
```

The demo is a static fixture and does not need its application dependencies installed. Scanning never installs dependencies, runs build scripts, starts MCP servers, or makes model calls. The optional decision-model layer is currently a researched design, not an enabled integration: [decision models](docs/DECISION_MODELS.md).

[Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [Roadmap](docs/ROADMAP.md) · [Code graph research](docs/CODE_GRAPHS.md) · [Related projects](docs/LANDSCAPE.md)

Licensed under Apache-2.0. Framework names are used for identification; this project is not affiliated with their vendors.

## Agent and MCP focused scans

```sh
pilotlight scan /path/to/repository --types agents,mcp,tool,skill --format csv --output inventory.csv
# Equivalent checked-in profile:
pilotlight scan /path/to/repository --config examples/agents-mcp.toml --format csv --output inventory.csv
```

Use [the enterprise-pattern coverage report](docs/ENTERPRISE_PATTERNS.md) for exact detection boundaries and benchmark evidence. Imports alone establish dependencies; explicit reviewed constructors/factories establish candidate roles. A generic StateGraph is dependency evidence, not an agent. MCP configuration declarations are references to servers, not runtime discovery. Gateway coverage includes selected FastMCP proxy factories. Language limits and unsupported patterns remain visible in the coverage sidecar.

The `examples/enterprise-patterns` scan fixture demonstrates handwritten loops, gateways, C# DI, Rust macros and application IDs; use `examples/enterprise-patterns.toml` as its configuration. It is sample source for scanning, not an executable application.

## Extend detection without rebuilding

Two embedded [detection packs](docs/DETECTION_PACKS.md) provide reviewed agent and MCP mappings. Load additional operator-owned JSON packs explicitly:

```sh
pilotlight packs list
pilotlight packs test examples/company-pack.json
pilotlight scan ./my-repo --pack ./company-pack.json --format csv --output inventory.csv
```

Packs declare exact bindings and supported structural parameters, source provenance, engine capabilities and positive/negative fixtures. Reports retain pack hashes and rule IDs alongside application IDs. No pack is loaded automatically from a scanned repository. New syntax mechanisms or languages still require engine work.
