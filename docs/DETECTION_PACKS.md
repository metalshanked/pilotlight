# Detection packs

Pilot Light 0.1 loads declarative JSON packs. A pack adds verified package coordinates, import bindings, symbols, receiver operations and supported structural-pattern parameters. It cannot execute scripts, load native plugins, or run arbitrary SQL. Local scans remain offline.

Two packs are embedded in the executable and enabled by default:

- `pilotlight.agents`: 23 framework/SDK entry rules, including OpenAI Agents, LangChain/LangGraph, Deep Agents, CrewAI, AutoGen, Google ADK, Pydantic AI, Strands, Semantic Kernel, Vercel AI SDK and Mastra. Some families also contribute MCP roles.
- `pilotlight.mcp`: official MCP SDK mappings across supported languages, FastMCP, PHP MCP, LangChain MCP adapters, MCP-use and MCP-Go. Includes reviewed C# DI and Rust macro mappings.

These package existing reviewed coverage into independently inspectable definitions; 29 entry rules do not mean 29 completely covered APIs. The full catalog contains 195 entries. The packs include 106 inline fixtures. Three existing families (Microsoft Agent Framework, Haystack and LlamaIndex) were moved into pack ownership; they are not new catalog families. Package recognition alone is dependency evidence.

## Use a pack

```powershell
.\bin\pilotlight.exe packs list
.\bin\pilotlight.exe packs validate examples/company-pack.json
.\bin\pilotlight.exe packs test examples/company-pack.json
.\bin\pilotlight.exe scan C:\Code\MyApp --pack examples/company-pack.json --format csv --output inventory.csv
```

The company pack is deliberately fictional: replace its coordinates and import paths with a verified internal SDK. `--pack` is repeatable. It works with `scan`, `benchmark`, `types list`, `catalog list` and `catalog export`.

Operator configuration can reference packs relative to the configuration file:

```toml
[packs]
paths = ["company-pack.json"]
disable_defaults = false
```

Only an explicitly selected `--config` is read. Packs are never discovered or loaded from the repository being scanned. CLI pack paths resolve from the current directory and append to configuration paths.

To replace bundled definitions with an explicitly selected pack set:

```powershell
.\bin\pilotlight.exe scan C:\Code\MyApp --no-default-packs --pack packs/agents.json --pack packs/mcp.json
```

`--no-default-packs` removes the bundled agent/MCP framework definitions from the embedded baseline; other framework data and core config/skill adapters remain available. It is not a universal engine-disable switch. `--catalog` supplies a complete alternate baseline: bundled packs are not automatically added or removed from that document. Any pack provenance already serialized in that catalog is retained.

## Pin and install

```powershell
$sha = (Get-FileHash .\examples\company-pack.json -Algorithm SHA256).Hash.ToLowerInvariant()
.\bin\pilotlight.exe packs install .\examples\company-pack.json --sha256 $sha --output .\company-pinned.json
.\bin\pilotlight.exe scan C:\Code\MyApp --pack .\company-pinned.json
```

Install validates the pack and digest and atomically writes a new file; existing destinations are refused. It does not register or activate the file automatically. Obtain a publisher's expected digest through a trusted channel; calculating your own hash only pins the bytes you already have. Publisher signatures, a remote registry, dependency resolution, automatic updates and rollback management are not implemented.

## Authoring contract

Start from `examples/company-pack.json`. Required metadata: `schema_version`, stable `id`, numeric `version` and `minimum_engine_version` in MAJOR.MINOR.PATCH form, description, license, HTTPS sources, required engine capabilities, and rules. Rule IDs are unique within their pack. Each rule contains a typed catalog entry. Fixtures contain a language, source text and exact counts for the listed artifact types. `packs test` statically scans these sources against that pack in isolation; it never imports or executes them. Unlisted artifact types are not scored. A pack without fixtures can be validated but cannot pass `packs test`.

Generic mappings support package/import recognition, constructors and functions, exact receiver operations, language restrictions and subtypes. Reusable structural parameters include:

- `configured_loops`: callable → tools/count/stop argument names and qualified stop functions. Requires explicit multi-step configuration.
- `rust_attributes`: attribute → impl-method, trait-impl or flagged-impl target, plus required trait/flag where applicable.
- `di_registrations`: qualified extension methods on recognized .NET service collections. The extension namespace must be imported.

These mechanisms are data-driven. New languages, new control-flow analyses, arbitrary type inference and new protocol semantics still require engine work. Handwritten-loop correlation is a bounded engine mechanism; packs supply the model-call bindings it needs. Engine compatibility is enforced; SDK package-version constraints remain unimplemented.

Composition is additive and deterministic. Repeated identical facts are deduplicated; differing scalar classifications or ownership collisions fail rather than choosing the last input. Duplicate pack IDs fail. To revise a bundled pack with the same ID, disable defaults and explicitly supply the desired set. Arrays act as sets during merging, so authors must review language/import expansions carefully. Loading is transactional: invalid additions leave the original in-memory catalog unchanged.

Pack files are limited to 8 MiB, 1,000 entry rules and 500 fixtures, with 64 KiB per fixture. At most 32 external packs and an 8 MiB composed catalog are allowed. Unknown fields, unsupported schemas/capabilities and unsupported artifact types fail validation. Validation establishes schema/mechanism compatibility, not detection accuracy or publisher authenticity.

JSON and coverage sidecars record active pack IDs, versions and SHA-256 hashes. Attributed findings have `detection_rules`, including pack identity and entry-rule ID; CSV and CycloneDX retain these. A rule identifies a framework entry, not an individual AST predicate. Core-only findings may have no pack attribution. Application metadata remains separate.

## Maintenance and validation

For bundled frameworks, edit the pack definition and its fixtures. `scripts/sync-builtin-packs.py` updates the matching entries in the complete overlay/runtime snapshots while preserving seed API references. The remaining entries are maintained through the existing catalog process. Rebuild only to ship changed embedded defaults; operator-supplied pack changes require no engine rebuild. Never interpret raw seed rows as distinct framework detectors.

The regression suite proves custom Python/TypeScript mappings, configured loops, Rust attributes and C# registrations using a fictional SDK, as well as conflict handling, deterministic composition, checksums, explicit configuration paths, disabling defaults and report attribution. See [the Cisco comparison](PACK_BENCHMARK.md) for shared-source probes and limitations.

## Model records and future identifiers

The engine does not classify a class merely because it is named `ModelSpec`.

- `model_arguments` maps a qualified Python callable to a zero-based `position` and a list of `keywords`. A null position means keyword-only matching. The binding must resolve through imports and must not be shadowed. Exactly one selected argument must be a literal model identifier (1–160 ASCII letters/digits or `-_/.:`). This supports private and future identifiers without an allowlist. Bundled selectors cover Deep Agents' `deepagents_code.model_config.ModelSpec` and both public/internal Semantic Kernel `ModelSpec` paths.
- `model_identifiers` supplies exact, uniquely owned identifiers for conservative inference through locally declared Python dataclasses. A simple dataclass must have string fields including `model` or `model_name`; positional and keyword arguments are mapped to those fields. Names are irrelevant. The initial local-record allowlist contains eight reviewed OpenAI IDs; operators can add their own. This restriction does not apply to reviewed SDK selectors or existing model-call detection.

Use capabilities `model-arguments-v1` and `model-records-v1` respectively. See [the executable company example](../examples/model-record-pack.json): `pilotlight packs test examples/model-record-pack.json`. Adding its private constructor and model ID requires no Rust changes.

Local dataclass support permits frozen/slots and other reviewed literal options that preserve constructor order, and ordinary methods. Inheritance, custom constructors, `field(...)`, `ClassVar`, `InitVar`, keyword-only layouts, dynamic strings, splats and ambiguous argument mapping abstain. Generic cross-file dataclass inference and arbitrary helper methods remain unsupported. Reviewed `ModelSpec.parse`/`try_parse` bindings use the optional `require_provider_prefix` selector flag to require a nonempty `provider:model` literal; the full literal remains in the reference and evidence. Findings are model references, not proof of deployment or execution.

## Reviewed agent and MCP expansion

The expansion adds exact bindings for OpenAI Agents tools/MCP clients (Python and JS/TS, including core exports), Strands TypeScript, Microsoft Agent Framework Python/C#, Haystack agents, FastMCP Python clients and TypeScript servers/tools, MCP-use Python/TypeScript, MCP-Go, and LangChain MCP adapters in Python/JS/TS. Existing LlamaIndex agent bindings are now pack-owned. Pydantic AI covers current `MCPToolset` and reviewed v1 transport classes. LangChain's TypeScript pack covers v1 `MultiServerMCPClient` and v2 `MCPAdapter`. Those mappings do not imply SDK-version negotiation or complete coverage of either version.

MCP connection wrappers named `MCPServerStdio`, for example in OpenAI Agents and Pydantic AI, are classified as clients; class names alone do not determine their role. Each addition has source references and positive/negative fixtures. The evidence manifest is [catalog/pack-expansion-evidence.json](../catalog/pack-expansion-evidence.json). See [the fixed real-code replay](REAL_CODE_BENCHMARK.md) for observed gains and gaps.

Reviewed model selectors also resolve through bounded Python named re-exports and retain their resolution path. The graph does not prove wrapper argument forwarding: caller model values are not attributed through wrappers. Dataclass instances do not inherit constructor bindings. These boundaries have regression tests.
