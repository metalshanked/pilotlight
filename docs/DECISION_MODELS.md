# Optional fast decision models

Research checked 2026-10-10. No model weights were downloaded and no paid or hosted inference was run. The 0.1 scanner does not have a model backend enabled or implemented. These are candidates and an integration design, not measured improvements.

## Findings

The supplied [Reddit discussion](https://www.reddit.com/r/LLMDevs/comments/1wko2e5/i_reviewed_287_opensource_jev_projects_here_are/) is a discovery list of applications using Jev. Open-source applications do not establish that the underlying model is open-source or locally runnable. The supplied [Hugging Face community article](https://huggingface.co/blog/a2aprotocol/openai-decisions-api-vs-jev-a-practical-guide) describes a third-party API service; it is not official OpenAI documentation.

| Candidate | Verified primary-source description | Pilot Light assessment |
|---|---|---|
| [OpenAI Decisions](https://developers.openai.com/api/docs/guides/decisions) | Public beta using `gpt-6-luna` at `POST /v1/decisions`; predicate, choice and score answers. The October 6 release supersedes the community article's preview-only description. | Optional direct hosted provider, with explicit source-egress opt-in. The official “10x faster” comparison is not a benchmark of Pilot Light code classification. |
| [TypeSafe Jev](https://docs.typesafe.ai/introduction) | State plus Choice/Score/Noul questions returning typed values and probability distributions | Good API candidate for bounded disambiguation. Evaluate through the official provider. The linked SDK/apps do not demonstrate a redistributable local Jev checkpoint. |
| [Laya](https://huggingface.co/convaiinnovations/laya) | Apache-2.0 model card; encoder-based local checkpoints; runtime lists an ONNX option | First local candidate to test. Its published tasks are not a code-inventory benchmark; context limits, hardware and checkpoint choice materially affect speed/accuracy. |
| [Kev 4B](https://huggingface.co/jaredpalmer/kev-4b) | Qwen3.5-4B-Base with a LoRA adapter and decision pointer head; prefill-only inference; Apache-2.0 for adapter/head and base as stated in the card | Useful comparison for local classification; heavier than an encoder. Do not assume its reference runtime is directly compatible with an existing Rust inference engine. |
| [Tev1](https://huggingface.co/togethercomputer/Tev1-4B-experimental) | Experimental Qwen fine-tune retaining a next-token LM head; selects an option letter; card says the release license is still being finalized | Research comparison only until licensing/runtime are clarified. It is not the same architecture as a non-autoregressive decision model. |

## Where to put it

Static evidence extraction → deterministic matches → ambiguous candidate queue → optional bounded classifier → separate annotation.

Example question: “Does the provided function construct an agent, an LLM client, an MCP server, an ordinary service, or is there insufficient evidence?” Supply only the candidate's structural facts and a bounded source slice when the operator allows content. Include `unknown` and `insufficient_context` outcomes. Never silently erase strong deterministic findings because a classifier disagrees. Record disagreements separately for rule improvement.

Do not skip whole repositories based only on a model's low relevance score: that makes false negatives invisible. A decision model can prioritize expensive wrapper/graph analysis while the cheap deterministic baseline still runs everywhere. Model outputs cannot create arbitrary paths, issue shell commands, or invoke tools.

## Proposed provider contract

Input: schema version, candidate ID, language, bounded evidence, explicit choices and question/rubric version. Output: candidate ID, decision, finite probability distribution where supplied, abstention/refusal/error, exact provider/model/version, evidence hash and timing. Validate that values belong to the allowed set; reject nonfinite or out-of-range scores, missing IDs and malformed distributions. A numeric score is not automatically a calibrated probability.

Start with a local HTTP sidecar adapter to evaluate open models without increasing the core binary's dependencies. That is optional infrastructure, not a claim that a 4B model fits inside a tiny executable. Consider embedded ONNX or Candle only after operator compatibility, tokenizer behavior, quantization accuracy and runtime size are tested. Keep weights as separately pinned artifacts. Hosted adapters must be explicitly enabled and document which source data leaves the machine; private repository content must not go to an intermediary by default.

## Evaluation gate

Build a held-out, manually labeled code corpus with framework constructors, custom wrappers, type-only imports, test fixtures, comments, local lookalikes, config indirection and prompt-injection strings embedded in source. Split by repository/framework family to reduce leakage. Compare deterministic-only, decision-assisted and conventional LLM-assisted runs on identical inputs.

Measure per-kind precision/recall, abstention rate, calibration, p50/p95 end-to-end latency, token/compute cost, CPU/GPU memory, throughput under concurrency, and changes in false negatives. Include warm and cold startup and hosted network time. A constrained output can still be semantically wrong. Promote a backend only if it improves a declared accuracy/cost objective without degrading the baseline's recall beyond an agreed threshold.
