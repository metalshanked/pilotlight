# Artifact taxonomy and selection contract

Current progress: [0.1 implementation status](IMPLEMENTATION_STATUS.md). This document retains the broader target contract; unimplemented requirements are not completion claims.

The 0.1 build implements selection and initial reporting; the broader production identity and coverage requirements below remain targets. Reviewed 2026-10-10. Pilot Light owns this contract independently. [Cisco's public type definitions](https://github.com/cisco-ai-defense/aibom/blob/8d7bec099d5ddcd24dbda13a9777f3cf11e2c325/aibom/src/aibom/models/enums.py) are a compatibility reference, not a claim of identical detection behavior.

## Selection

Proposed CLI:

```sh
pilotlight types list --format table
pilotlight scan . --types agent,agent_proxy,mcp_server,skill --format csv --output inventory.csv
pilotlight scan . --types all --exclude-types secret,other --coverage-output coverage.json
pilotlight explain --report inventory.json --asset ASSET_ID
```

Canonical values are singular snake_case. Reject unknown values with suggestions. Accept only unambiguous, documented aliases at the CLI boundary and serialize only canonical values. Reserve `agents` for the documented group `{agent, agent_proxy}` rather than treating it as a plural alias for `agent`; `mcp` means `{mcp_server, mcp_client, mcp_gateway}`, and `models` means `{model, model_artifact, model_endpoint, llm_endpoint}`. `all` cannot be mixed with specific types. Default is the full supported static inventory; credential-pattern scanning (`secret`) requires an explicit selection or `all`. No selection ever enables active probing or hosted inference.

Each type owns detector registrations and required prerequisites. Selecting MCP servers still permits import, manifest and binding analysis needed to find servers; it does not run unrelated training-data or credential detectors. Apply one normalized selection policy to every reporter. Never promote an import/dependency alone into an agent or server.

Reports contain the requested types, effective types, excluded types, scanner/catalog/config fingerprints, and per-type/per-language capabilities. Distinguish `complete_within_declared_scope`, `partial`, `unsupported`, `not_requested`, and `failed`. A type with zero findings is not automatically unsupported, and a registered type is not automatically supported in every language.

Reject a request with no executable detector before scanning. In mixed requests, return supported findings and explicit unsupported/partial coverage; `--strict-coverage` fails the quality gate. JSON always retains coverage. Terminal output shows it. CSV has a findings file and repository/type coverage sidecar, including repositories with zero findings. Silent omission is not allowed.

Top-level findings contain selected types only. Default relationships connect returned entities only. With an explicit `--include-context` option, supporting non-selected entities live in a separate context collection and are marked as context; they are never silently mixed into the requested inventory. All edge references must resolve. CSV relationship/context data uses separate files rather than overloading finding rows.

Proposed config follows the same policy:

```toml
[scan]
types = ["agent", "agent_proxy", "mcp_server", "skill"]
strict_coverage = true

[[indicators]]
name = "application_id"
path = ".company/.source/appinfo.yaml"
key = "appid"
required = true
```

Precedence: CLI overrides explicit operator config; embedded defaults apply last. Target-owned configuration may supply metadata only when the operator selects that source. It cannot enable network use, executable plugins or unbounded scanning.

## Thirty compatible core types

The table defines minimum production evidence, not current feature completion. Every detector also needs a versioned language/format capability entry, positive and negative fixtures, provenance and a diagnostic path for unresolved cases.

| Type | Evidence to implement | Boundary against false positives |
|---|---|---|
| `model` | Literal/model-config identifiers, local model registrations and resolved model references | An arbitrary string or SDK import is not a model identity |
| `llm_endpoint` | LLM route/client configuration with provider and normalized endpoint | Do not label every URL or OpenAI-compatible client as an OpenAI-hosted service |
| `model_endpoint` | Declared inference service or non-LLM serving route | Separate generic model serving from LLM inference |
| `agent` | Cataloged or structurally justified agent definition/construction | Package presence, class names containing Agent and comments are insufficient |
| `agent_proxy` | A local reference/client that invokes a remote agent, including A2A | Record the local proxy separately from the remote agent identity; no remote probing |
| `tool` | Registered tool definitions/decorators, schemas and bindings | A function named tool without AI/protocol context is insufficient |
| `mcp_server` | Declared MCP server implementation or configuration | Distinguish implementation from a client configuration referencing a remote server |
| `mcp_client` | MCP session/client construction and configured transport bindings | Generic HTTP/JSON-RPC clients are insufficient |
| `mcp_gateway` | Explicit MCP gateway/router configuration or recognized construction | Do not infer a gateway from a list of multiple servers |
| `embedding` | Embedding models, clients and configured embedding pipelines | Keep embedding calls distinct from chat calls |
| `vector_store` | Vector-store construction/configuration and contextual usage | A database client dependency alone remains a dependency |
| `dataset` | Dataset declarations, loader calls and manifest references | Do not classify every CSV/file as an AI dataset or read its contents by default |
| `retriever` | Retriever definitions/configurations and bindings | Retrieval evidence requires AI/search context, not a generic getter |
| `knowledge_base` | Named retrieval knowledge-base definitions or service references | Link to related stores/datasets without collapsing their identities |
| `feature_store` | Feature store/feature-view declarations and clients | Ordinary caches and key/value stores do not establish a feature store |
| `memory` | Agent/conversation memory components and state backends | Generic application memory/storage is insufficient |
| `prompt` | Prompt template/configuration definitions and recognized bindings | Export names/locations/hashes, not prompt text by default |
| `training_run` | Declared training job/run configuration or tracking-run construction | Static code describes an intended run, not proof a run happened |
| `hyperparameter` | Training/model parameters tied to an identified lifecycle component | Do not inventory every function argument or config number |
| `model_artifact` | Model-format headers/manifests and file metadata | Never deserialize executable pickle/model payloads to inventory them |
| `experiment_tracker` | Explicit experiment/tracking configuration or setup | Importing a tracking library does not prove an active tracking endpoint |
| `model_registry` | Model registry declarations and model-registration operations | Distinguish registry assets from package registries |
| `data_versioning` | Data-version manifests and contextual versioning integrations | A Git repository alone is not an AI data-versioning asset |
| `ml_pipeline` | Declared training/inference pipeline/workflow definitions | Generic CI pipelines need contextual AI evidence |
| `guardrail` | Guardrail policy/configuration and application bindings | Generic validation/safety-related names are insufficient |
| `skill` | Validated SKILL.md/frontmatter and supported skill/plugin manifests | Inventory metadata and references; never obey skill instructions or execute bundled scripts |
| `observability` | AI tracing/evaluation/telemetry integrations and configuration | Generic logging dependencies do not establish AI observability |
| `secret` | Credential references and optionally recognizable credential-shaped literals | Never output values or make validation calls; report redacted locations/categories only |
| `dependency` | Exact packages and resolved versions from manifests/lockfiles/imports | Separate declared, resolved, imported and transitive evidence |
| `other` | A reviewed rule that identifies an AI asset outside the taxonomy | Not a bucket for arbitrary unresolved symbols; unresolved evidence stays a diagnostic |

## Pilot Light extensions

Three additional selectable types preserve useful prototype concepts: `llm_client` (client construction), `llm_call` (statically identified generative invocation), and `decision_call` (bounded decision/classification invocation). None proves a runtime request. They require qualified call or configuration context; an SDK's file upload, administration or batch-list operation must not automatically become an LLM call.

This is a 33-type target taxonomy. Grouped aliases such as `agents`, `mcp` and `models` may expand to documented sets; store the expansion in reports and never reinterpret existing canonical names silently.

## Identity design

The greenfield 0.1 scanner emits canonical types directly. No legacy report conversion is required.

Separate an entity from its occurrences. Entity IDs include repository identity and normalized declaration/reference identity; finding IDs include commit, file, span, rule and occurrence. This avoids colliding identical paths across repositories while allowing controlled comparison between commits. Normalize local paths and record monorepo application boundaries. Do not merge two agents merely because they have the same display name.

## Required report fields

- Scan identity: repository identity, commit, dirty-tree state for local scans, scanner/schema/catalog/config versions and hashes, UTC time, scope and selection.
- Entity: canonical type, subtype, display name, framework/provider when justified, declaration/reference distinction, application ID, lifecycle role, static evidence status.
- Finding: file and exact span, rule/extractor ID, package/symbol/config evidence, resolution path, confidence category, unresolved reason where applicable. Numeric confidence is reserved for explicitly calibrated optional classifiers.
- Relationships: typed source/target references and their own evidence; no unsupported cross-repository runtime links.
- Coverage: attempted/succeeded/skipped/unsupported/failed counts by repository, language, type and reason; timeout, truncation and excluded-path accounting.
- Privacy: secret values, request bodies, prompt contents and arbitrary environment values excluded by default.

Tests must prove selection consistency across JSON/CSV/table/CycloneDX/SARIF; type aliases, contradictory options, all/none, unsupported requests, partial files, zero findings and filtered relationship integrity all need fixtures.
