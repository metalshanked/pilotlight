# Preparing a public repository

Pilot Light is an Apache-2.0 Rust application at version 0.1.0. Publishing source
does not establish production readiness. Keep the documented detection limits,
historical benchmark scope, upstream notices and dependency attribution intact.

## Audit the exact source tree

Use Rust 1.89+ and Python 3.11+. From the project root:

```sh
python scripts/public_source.py --check-git
python scripts/dependency-notices.py
python scripts/audit-dependencies.py
python scripts/test-public-source.py
python scripts/test-audit-catalog.py
cargo test --locked
cargo test --locked --features duckdb-catalog
```

Run Gitleaks 8.30.1 against the source directory with `gitleaks dir . --redact=100`.
The CI security job obtains the Linux executable from the official release and
verifies a pinned SHA-256. Once history exists, also run `gitleaks git . --redact=100`
before publishing previously private history. No scanner can prove the absence of
all secrets. Review the actual Git candidate file list, not just ignore rules.

`public_source.py` checks an explicit allowlist and rejects secret-like files,
developer home paths, credential-shaped strings, binary files and symlinks in
publication scope. `--check-git` also detects tracked/unignored files outside that
scope. Errors report locations/rules rather than matched values. `.gitignore`
excludes local environments, IDE state, credentials, generated reports, binaries
and raw DuckDB files. Ignoring a file does not remove it from existing Git history.

Create a fresh, deterministic source archive:

```sh
python scripts/public_source.py --check-git --output dist/pilotlight-source.zip
```

It contains only the audited source scope, with a separate per-file manifest and
archive SHA-256. It never copies `.git`, your virtual environment, build output,
local benchmark runs or the optional database. It refuses to overwrite an existing
archive. The embedded catalog is included and works offline.

Do not upload the surrounding working directory or historical raw benchmark
archives as part of the source release: those can contain machine paths and other
execution metadata. Historical benchmark reports refer to privately retained
development artifacts; those artifacts are not included in the public source ZIP.
The corpus pins and reproduction scripts remain public.

## Build a distributable executable

```sh
python scripts/build-release.py --target x86_64-pc-windows-msvc
python scripts/package-release.py --target x86_64-pc-windows-msvc --binary target/x86_64-pc-windows-msvc/release/pilotlight.exe --output artifacts
```

Use the appropriate Rust target on Linux/macOS; the native compiler and target
must already be installed. The wrapper remaps Rust and native compiler paths,
links the Windows C runtime statically, and checks the executable for local path
prefixes before declaring success. It uses controlled release flags. Normal
`cargo build` remains available for development. The packaging workflow includes
smoke tests, notices and checksums. Default packages do not embed DuckDB.

The local audit verifies Windows only. Linux/macOS native CI, rustfmt/Clippy
qualification and code-signing remain pending. A checksum is not a signature.

## GitHub owner steps

1. Create the desired GitHub repository; no owner or remote is hardcoded here.
2. Enable private vulnerability reporting and secret scanning/push protection
   where available. Configure branch protection and required CI checks.
3. Review `git status --short` and the publication audit, then stage only the
   reviewed source. Review `git diff --cached` before creating the first commit.
   Choose a public commit author identity and GitHub no-reply email if you do not
   want a personal email recorded in permanent public history. This preparation
   does not change your Git identity or global configuration.
4. Add your chosen remote and push after review. Run the native CI and portable
   binary workflows; fix failures before advertising those platforms as verified.

This preparation does not create commits, tags, remote repositories or releases.
The build workflow uploads Actions artifacts; it does not publish GitHub Releases.
