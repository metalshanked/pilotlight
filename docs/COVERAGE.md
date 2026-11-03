# Coverage in 0.1

Coverage means a specific, tested evidence mechanism. A catalog match is not a claim of complete SDK API coverage.

| Language | Parser | Import evidence | Recognized calls / aliases | Main limitations |
|---|---|---|---|---|
| Python | Tree-sitter | `import`, `from … import …` | Cataloged constructors/functions; aliases; direct client variables; literal model keywords; MCP and LangChain tool decorators, including bare decorators | Bounded named and namespace imports, re-exports and simple factory wrappers; no wildcard imports, whole-program types, dynamic imports or general wrapper flow |
| JavaScript | Tree-sitter | ES imports, CommonJS `require` evidence | ES default/named aliases and cataloged calls | CommonJS variables/destructuring and dynamic imports not resolved |
| TypeScript / TSX | Tree-sitter | ES imports | ES aliases, cataloged calls, object-literal model arguments | No typechecker, path aliases or re-export graph; type-only imports do not establish callable bindings |
| Java | Tree-sitter | Import declarations | Selected qualified MCP / LangChain4j / Spring builders | No Maven classpath, wildcard/type resolution or general method inference |
| Go | Tree-sitter | Import specifications and aliases | Curated declared package names, SDK constructors, conventional client/error result assignments and receiver operations | No module/package typechecking or receiver inference |
| Rust | Tree-sitter | Grouped/scoped/aliased use declarations | Cataloged constructors; curated prelude exports | Reviewed rmcp tool/server attributes are detected; general glob exports, custom macros, traits and method types remain unresolved |
| C# | Tree-sitter | Namespace and alias using directives | Cataloged constructors, static namespace factories, MCP tool attributes, typed target-new expressions, selected DI registrations and direct receiver operations | No Roslyn or general type inference |
| PHP | Tree-sitter | Namespace imports, aliases and grouped imports | Qualified cataloged constructors/static calls | No Composer autoloader graph or whole-program receiver types |
| Ruby | Tree-sitter | Literal `require` calls; explicitly cataloged constant roots | RubyLLM chat/embed factories, literal model keywords and direct ask/complete operations, including a direct factory-call chain | Constant roots carry `qualified_symbol` confidence, not verified load provenance; general fluent chains, dynamic requires and metaprogramming unresolved |
| Kotlin / Kotlin scripts | Tree-sitter | Qualified JVM imports and aliases | Selected Java SDK constructors/builders and local bindings | Wildcard imports, extension methods, compiler type information and general cross-file resolution unresolved |

Dependency manifests: package.json dependency sections; requirements*.txt direct package names; pyproject.toml project dependencies/optional groups and Poetry groups; Cargo dependency tables and package renames; go.mod requirements; Maven pom.xml; C# csproj PackageReference; literal Gemfile gems; common literal Gradle dependency declarations; Composer require/require-dev; PEP 735 dependency groups; NuGet packages.config and central Directory.Packages.props.

MCP configuration: JSON/JSONC files named `.mcp.json`, `mcp.json`, `claude_desktop_config.json`, with `mcpServers`, `servers`, or `mcp.servers`. Inventory includes declared server names only; command arguments, environment values, credentials, tool schemas and remote discovery are not collected. Equivalent fixed-name mcp.yaml/mcp.yml/mcp.toml files, dot-prefixed forms, mcp.jsonc and `.codex/config.toml` are supported. Entries require a command or URL. Names are configuration references, not proof of running servers.

Assets: dependency declarations/imports, recognized agents (including cataloged workflows), LLM clients and candidate calls, model references, MCP servers/clients, selected tool/embedding/vector-store/retriever/guardrail constructions, Feast feature stores, MLflow experiment-tracking and registration calls, and DVC filesystem constructions. OpenAI `client.decisions.create` is distinguished as a `decision_call` where the client binding resolves.

Default scope: .gitignore honored even for archive downloads; hidden paths included; `.git`, `.venv`, `venv`, `node_modules`, `target`, `vendor`, `dist`, `build`, `.idea`, `__pycache__` excluded. No symlink following. Read limit defaults to 2 MiB per file, with a maximum configurable limit of 64 MiB. At most 100,000 candidate files and bounded syntax traversal depth. Errors/limits are reported as diagnostics. No compiled binary, image, model-weight content, remote endpoint probing or runtime scanning in this release. Notebook format 4 supports isolated Python/JS/TS cells with explicit partial-coverage diagnostics for unresolved cross-cell state.

False positives remain possible: an installed package may be unused, an import may resolve to a local module with a catalog name, code may be unreachable, and lexical bindings may be ambiguous across scopes/control flow. `inferred_from_import` is deliberately not a confidence probability or runtime verification. A framework entry with no matching call symbols produces library/dependency evidence only.

Direct HTTP calls through generic clients, dynamically assembled endpoints/model names, environment-variable resolution and custom internal wrappers are not yet recognized without a cataloged import/symbol. These are important gaps for enterprise code and should be represented in the evaluation corpus.

The catalog covers 192 named framework/library entries across model SDKs, agent frameworks, RAG, vector stores, model runtimes, observability and guardrails. Run `pilotlight catalog list` for the exact snapshot. Broader coverage requires fixture-backed additions; do not describe it as “all known frameworks/languages.”

The catalog includes seven exact package mappings for Together AI, OpenRouter and Replicate across PyPI/npm. Together/OpenRouter include selected Python and JS/TS call mappings; Replicate currently provides dependency/import evidence only. OpenRouter context-manager aliases are not yet resolved. These additions expand existing language coverage; they do not add new language parsers.

## Catalog and format limits

Catalog 0.1 contains 192 entries, 329 exact package coordinates, 396 import prefixes and 199 explicit symbols, plus 21,114 exact Python API references from the seed. Deep Agents has Python and JS/TS factory coverage; ADK provides one reviewed Python remote-proxy constructor path. Model-client receiver operations require cataloged operation paths. Names like `files.create` no longer imply LLM usage. Catalog entries without operation mappings may have client/import evidence without call coverage.

Skills: SKILL.md YAML frontmatter with required name/description, conservative ASCII-name checks and directory-name agreement where the relative parent is available. Only identity is exported. YAML aliases are rejected. Skill validity does not imply safe or effective instructions.

Lockfiles: npm package-lock v1/v2/v3, Cargo v1–v4, uv v1, Poetry 1.0/1.1/2.0/2.1, pnpm 6/9, Composer, NuGet v1/v2. Package/version facts are separated from direct manifest declarations. Dependency scope is explicitly unspecified unless the adapter has evidence. Yarn/Bun/Go checksum graphs and some workspace/link/alias forms remain unsupported. Composer dependency recognition and the PHP syntax adapter both depend on reviewed package/import mappings.

Type coverage is relative to catalog-qualified lexical/config mechanisms. Known unsupported source extensions are tracked separately; no ignored-tree count or universal clean-repository claim is made. The default scan remains offline. See [implementation status](IMPLEMENTATION_STATUS.md) for the exact release boundaries.

Agent/MCP focus: see [the focused coverage matrix and evidence](AGENT_MCP_COVERAGE.md). `StateGraph` construction alone remains dependency evidence. MCP tools require a reviewed SDK receiver/attribute/decorator; a server construction does not itself establish tool registration.

The latest [enterprise-pattern coverage](ENTERPRISE_PATTERNS.md) adds inferred handwritten tool loops, Vercel configured loops, Anthropic tool runners, FastMCP proxy factories, C# DI and Rust rmcp attributes. The loop detector records bounded structural evidence, not runtime or whole-program proof.
