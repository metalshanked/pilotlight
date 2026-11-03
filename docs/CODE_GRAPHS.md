# Code graph options for Pilot Light

Research checked 2026-10-10. These are architecture recommendations, not benchmark results.

| Option | Value for AI inventory | Decision |
|---|---|---|
| [Tree-sitter](https://tree-sitter.github.io/tree-sitter/using-parsers/) | Structural extraction across languages without requiring a project build | Embedded now. Syntax trees do not provide type or cross-file symbol resolution automatically. |
| [ast-grep](https://github.com/ast-grep/ast-grep) | Rust structural patterns built on Tree-sitter; approachable declarative rule authoring | Evaluate as a rule-authoring layer. Avoid shipping both CLIs or parsing every file twice. |
| [SCIP](https://github.com/scip-code/scip) | Shared representation for symbols, occurrences and relationships produced by language indexers | Future optional import of an existing SCIP index; do not require external indexers for a normal scan. |
| [Stack Graphs](https://github.com/github/stack-graphs) | Incremental cross-file name resolution with language-specific rules | Useful design reference, but GitHub has archived the project and states it is no longer maintained. No core dependency planned. |
| [Joern](https://github.com/joernio/joern) | Code property graphs combining structural and flow relationships | Optional research/offline enrichment, too much operational complexity for the first single-binary scanner. |

## Graph work that should improve recall

Implemented in 0.1: bounded Python named and namespace imports/re-exports, relative module paths and simple synchronous single-return factory wrappers, driven by the active catalog. Findings retain resolution chains. This handles a useful subset of internal wrappers; JS/TS module graphs, constant propagation, richer control flow and type information remain open. Cycles, ambiguous paths and unsupported wrappers abstain with partial-coverage diagnostics.

Resolve a local wrapper such as `company_ai.make_agent` through imports/re-exports to a known factory. Follow model identifiers from a configuration constant to a recognized call. Connect an agent to its tool definitions. Distinguish a local `Agent` class from a framework's `Agent`. These need binding/data-flow evidence; embeddings alone do not establish them.

Recommended order: module index → import/alias resolution → constant propagation → bounded wrapper traversal → optional SCIP type information. Implement positive and negative fixtures before each expansion. Preserve unresolved edges rather than pretending every dynamic value is known. A graph database server is unnecessary for the first implementation; compact maps and adjacency lists suffice. DuckDB can persist extracted facts for analytics without becoming a mandatory graph traversal engine.

The development assistant's codebase-memory MCP is a development aid, not a Pilot Light runtime dependency. Indexing the pre-existing local project was declined by automatic approval review due to possible source transmission; implementation proceeded using local source inspection. No source was exported through that attempted index operation.
