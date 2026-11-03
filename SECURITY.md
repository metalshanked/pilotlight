# Security

Pilot Light reads untrusted repositories but never installs target dependencies, executes target code or launches MCP servers. Local scans make no network requests. GitHub acquisition and explicit catalog updates use HTTPS. Reports omit raw source snippets, credential environment values and MCP command arguments by default; exported identifiers and explicitly configured indicator values can still be sensitive.

Scan a stable checkout/snapshot. The CLI is not an operating-system sandbox against a hostile process racing local filesystem mutations. Use resource limits when processing adversarial repositories. Treat externally supplied DuckDB files as trusted-publisher artifacts; use validated JSON for less trusted catalogs. A pinned checksum verifies bytes, not publisher identity.

Only the current 0.1 development line is maintained. It has not received an independent security assessment.

Use **Security → Report a vulnerability** on the GitHub repository for confidential reports when private vulnerability reporting is enabled. Include affected versions, a minimal synthetic reproduction, impact, and suggested mitigation. Do not include live credentials or proprietary repository contents. If the private reporting button is unavailable, request a private reporting channel from the repository maintainer without disclosing vulnerability details in a public issue. There is no response-time guarantee yet.

Repository owners must enable private vulnerability reporting before public launch; this local source tree cannot configure a GitHub repository that does not yet exist. See [publication steps](docs/PUBLISHING.md).

Inventory output is not anonymized: local source roots, public/private repository names, file paths, model IDs, symbol names, and explicitly configured indicator values identify the scanned application. Review and redact reports before sharing them. Keep reports outside the source checkout or inside ignored `reports/`. Parser diagnostics avoid source excerpts, and signed catalog URL errors omit URLs. The distribution build remaps local compiler paths; ordinary development builds do not promise this.
