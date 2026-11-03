# Prepublication audit — 2026-11-03

This is a local engineering audit of the 0.1.0 source and Windows x64 binary,
not an independent penetration test or a production certification. No Git commit,
remote repository, tag or release was created. The repository has no commit history.

## Fixes and publication preparation

- Updated the locked dependencies: rustls 0.23.45 → 0.23.46, rustls-webpki
  0.103.15 → 0.103.16, webpki-roots 1.0.9 → 1.0.10, and Tokio 1.53.2 → 1.53.3.
  The original lockfile matched seven RustSec advisories (some also have GHSA
  aliases). A fresh [OSV](https://osv.dev/) query of all **297** locked registry
  dependencies returns **zero findings**. This is a point-in-time advisory check,
  not proof that every dependency is secure.
- Removed compiler-machine paths from the release executable using Rust and
  native compiler path remapping. The old executable had embedded development
  paths; it should not be distributed as the audited build. Packaging now rejects
  binaries containing developer home paths, including UTF-16 paths.
- Malformed operator configuration errors no longer echo TOML source lines.
  Failed signed catalog downloads omit their URL from error chains. A regression
  test checks private config-value redaction; a failed-download smoke check
  verifies signed query-string redaction and no output installation.
- Added an allowlisted, deterministic source packager and Git candidate audit.
  Local environments, IDE files, credentials, raw databases, executables and raw
  benchmark reports stay out of the source ZIP. Generic relative paths replace
  workstation-specific benchmark command examples.
- Added issue/PR templates, Dependabot configuration, and pinned CI actions with
  read-only permissions and checkout credential persistence disabled. CI checks
  public source scope, dependency advisories, notices and secrets.
- Retained Apache-2.0/Cisco attribution and added a manifest for all **297**
  Cargo dependencies with **559** license/notice texts, plus Rust standard-library
  notices. Public third-party author names/copyright notices are intentionally
  preserved; they are not maintainer personal data.

## Validation

| Check | Result |
|---|---|
| Default Rust integration tests | 72 pass |
| Fresh build from extracted source ZIP | 72 tests pass in a separate build directory |
| DuckDB-feature integration tests | 74 pass |
| Embedded detection-pack fixtures | 106 pass across two packs |
| Separate model-record pack | 4 fixtures pass |
| Maintainer Python tests | 2 catalog-audit tests and 3 publication tests pass |
| Source privacy audit | No findings in the allowlisted publication tree |
| Gitleaks 8.30.1 | No findings in the clean source and Windows packages |
| Locked dependency advisory audit | 297 checked; zero current OSV findings |
| Windows release | Builds with static C runtime; imports only Windows system DLLs |
| Release binary privacy | No detected developer home/build prefixes in ASCII/UTF-16 |
| Enterprise smoke | JSON, JSONL, table, CSV and CycloneDX pass; app ID `0007` retained |
| Report integrity | CSV sidecar hash and official CycloneDX 1.6 schema pass |
| Public GitHub acquisition | Pinned Requests repository scanned without a token; zero findings |
| Fixed repository regression | 38/38 completed; findings and relationships unchanged |

The audited Windows executable is 28,931,584 bytes with SHA-256
`5afd58f166941258c0c50fa43d206ad5cd7bbd78eec74267aa9f2558cda04b51`.
The 733-file source archive has a separate per-file manifest and archive checksum.
The installed source and Git candidate list match that manifest exactly. Git
attributes preserve third-party notice bytes so their recorded hashes survive
checkout on different operating systems.

The fixed replay uses exactly the prior 34 repositories plus the four-repository
supplement. All reports still declare **partial coverage**. This rerun checks
regression behavior after dependency/privacy changes, not detection accuracy.
Cisco was not rerun during this audit; the [historical comparison](REAL_CODE_BENCHMARK.md)
retains its original binary hashes, pins, timing limits and qualifications.

Reviewed boundaries include token handling and redirect restrictions, bounded
archive acquisition, unsafe archive entries, explicit operator configuration,
indicator containment, output replacement/CSV formula escaping, pack validation,
and trusted-only DuckDB loading. Existing tests cover those boundaries. The
scanner still requires a stable local snapshot and is not an OS sandbox against
filesystem races. Inventory reports intentionally identify repositories/assets
and selected metadata; they are not automatically anonymized.

## Before public launch

Native Linux/macOS CI has not run locally. rustfmt and Clippy are unavailable in
the audit environment, so formatting/lint qualification remains open; an earlier
endpoint-protection block was not bypassed. No code-signing or independent
security/accuracy assessment is claimed. Optional DuckDB binary redistribution
also needs review of bundled native dependency notices.

Follow [publishing instructions](PUBLISHING.md), review the exact candidate files,
choose a public commit identity, enable GitHub private vulnerability reporting
and push protection where available, and require passing native CI. Repository
security settings cannot be configured until the destination repository exists.
Publish only the audited source/archive, not surrounding development artifacts.
