# Related open-source projects

Reviewed 2026-10-10 from project-owned documentation. These are feature descriptions, not comparative accuracy or speed measurements.

| Project | Relevant scope | Implication for Pilot Light |
|---|---|---|
| [Cisco AIBOM](https://github.com/cisco-ai-defense/aibom) | Source/container/cloud inventory, 23 documented scanners, cross-reference resolution and an LLM-backed classification pipeline | Broad capability reference. Pilot Light focuses on the small, offline static CLI and transparent evidence first. No Cisco source was copied. |
| [Trusera AI-BOM](https://github.com/Trusera/ai-bom) | Python CLI with source, MCP configuration, notebook, model-file and cloud scanning; CSV and BOM exports; optional LLM enrichment and separate runtime SDKs | A direct functional alternative worth testing on the same labeled corpus. Framework counts alone do not measure detection quality. |
| [agent-bom](https://github.com/koda-ai-studio/agent-bom) | AI/MCP/package/cloud inventory, security findings, self-hosted control plane and runtime proxy/gateway options | Relevant for a later static-to-runtime integration; materially broader deployment scope than this initial executable. |
| [Syft AI catalogers](https://oss.anchore.com/docs/capabilities/ai/) | GGUF and safetensors model-file catalogers alongside conventional package SBOM generation | Potential complementary input for model artifacts and installed dependencies. This capability is distinct from agent construction and LLM call-site analysis. |

## What to improve first

Use the same pinned public repositories and manually labeled fixtures for all scanners. Record exact versions, modes, supported inputs, network use and failures. Count verified true/false positives and false negatives per asset kind and language. Do not interpret a tool reporting more findings as better recall without inspecting those findings.

Rust supports a portable executable and bounded concurrency, but extraction design dominates both accuracy and speed. Prioritize import/module resolution, wrapper tracing, notebooks, configuration formats and incremental invalidation. Keep the cheap catalog pass available without a hosted classifier. Add new data-driven mappings with fixtures; add new parsers or semantic behavior through versioned binary releases.

## Later runtime layer

Treat static declarations and runtime observations as separate evidence sources. A future collector could accept instrumented agent/model/MCP spans and gateway observations, then correlate them with repository URL, deployed commit/image digest, application ID and service identity. Record first/last observed time and deployment environment. Never convert a static dependency declaration into a claim that the model was called.

Runtime SDKs provide richer call/tool context but require instrumentation and can miss uninstrumented code. Network/proxy observations can discover outbound services, but encrypted or local inference and indirect routing limit model/agent attribution. Combining sources improves coverage; neither alone establishes a complete enterprise inventory. Runtime collection is not implemented in Pilot Light 0.1.
