# Cross-platform binaries

One Rust codebase produces a separate executable for each OS/architecture. A Windows executable cannot run natively on Linux or macOS.

The `Build portable binaries` GitHub Actions workflow tests, builds, smoke-tests and packages these targets:

| Platform | Architecture | Rust target | Package |
|---|---|---|---|
| Linux | x64 | x86_64-unknown-linux-musl | tar.gz |
| Linux | ARM64 | aarch64-unknown-linux-musl | tar.gz |
| macOS | Intel | x86_64-apple-darwin | tar.gz |
| macOS | Apple Silicon | aarch64-apple-darwin | tar.gz |
| Windows | x64 | x86_64-pc-windows-msvc | zip |

Linux builds target musl to avoid binding the executable to the build runner's glibc version. Windows statically links the Visual C++ runtime. macOS binaries use system libraries. These portable packages use the default embedded JSON catalog; DuckDB builds remain an optional separate configuration.

Once the repository is hosted on GitHub, run the workflow manually or push a `v*` tag. Each successful job uploads an archive plus SHA-256 file to that Actions run for 30 days. These are build artifacts, not automatic public GitHub Releases. The workflow has read-only repository permissions and pins its actions to verified commits.

The archive contains the executable, license, documentation and demo. Users do not need Rust or Python. Python 3.11+ is used only by the maintainer packaging script. Tests and the demo run natively on each architecture before packaging.

Status as of 2026-10-10: Windows x64 has been built and tested locally. The Linux/macOS workflow is prepared but has not run; no Linux/macOS binaries are claimed available. There is no connected GitHub remote in this local project yet. macOS signing/notarization, Windows signing and release signatures remain future release work; a checksum provides integrity, not publisher identity.

For publication-safe builds and a source-only ZIP, use [the publication guide](PUBLISHING.md). Release builds remap Rust/native compiler paths and verify that local build prefixes are absent. The [prepublication audit](PREPUBLICATION_AUDIT.md) records the current Windows checks; historical benchmark hashes identify earlier binaries.
