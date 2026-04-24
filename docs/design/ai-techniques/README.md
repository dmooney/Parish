# SOTA AI Techniques — Brainstorm

A menu of state-of-the-art AI/ML techniques we could incorporate into Rundale.
Each entry is a short design note pitched against what the engine already has
(see `docs/adr/002-cognitive-lod-tiers.md`, `docs/adr/005-ollama-local-inference.md`,
`crates/parish-npc/`, `crates/parish-inference/`).

These are brainstorm notes, not committed plans. Each technique should graduate
to an ADR + feature flag (`crates/parish-config/src/flags.rs`) before shipping.

## Baseline (what we already have)

- 4-tier cognitive LOD (Tier 1 player dialogue → Tier 4 rules).
- Local Ollama + optional cloud routing per category (ADR-005, ADR-013, ADR-017).
- Priority-lane inference queue (`crates/parish-inference/src/lib.rs`).
- Keyword-based memory: 20-entry short-term ring + 50-entry long-term
  (`crates/parish-npc/src/memory.rs`).
- Structured JSON output with `---` separator (ADR-008).
- Anachronism / prompt-injection defense (`crates/parish-npc/src/anachronism.rs`,
  ADR-010).
- Hand-authored NPCs / world / schedules (`mods/rundale/`).

## Gap map (where SOTA would move the needle)

| Gap | Today | Opportunity |
| --- | --- | --- |
| Retrieval | Keyword overlap | Semantic embeddings + RAG |
| Memory consolidation | Importance threshold promotion | Reflection, summarisation agents |
| Dialogue reliability | Post-hoc JSON parse | Constrained decoding (grammar) |
| Dialogue quality | Single-shot generation | Self-refine / critic passes |
| NPC agency | Schedule + mood deltas | ReAct-style tool-using agents |
| Local perf | Sequential Ollama calls | Prompt cache, speculative decoding, batching |
| Voice | Hand-written prompts | LoRA-tuned period dialect model |
| Player adaptation | Static persona prompt | Online player modelling |
| Social spread | Probabilistic rules | Graph diffusion + theory-of-mind |
| Multimodal | Text only | TTS, ASR, diffusion portraits |
| Evaluation | Unit tests + /prove | LLM-as-judge regression harness |

## Topic notes

1. [`01-semantic-memory-and-rag.md`](01-semantic-memory-and-rag.md) — embeddings,
   hybrid retrieval, reflection, MemGPT-style paging.
2. [`02-structured-generation.md`](02-structured-generation.md) — GBNF / JSON
   schema constrained decoding, grammar-guided output.
3. [`03-dialogue-quality-loops.md`](03-dialogue-quality-loops.md) — self-refine,
   reflexion, LLM-as-judge, rejection sampling.
4. [`04-agent-planning-and-tools.md`](04-agent-planning-and-tools.md) — ReAct,
   function calling for world queries, tree-of-thought for goals.
5. [`05-inference-performance.md`](05-inference-performance.md) — prompt / KV
   cache reuse, speculative decoding, continuous batching, LoRA adapters.
6. [`06-personalization-and-learning.md`](06-personalization-and-learning.md) —
   player modelling, DPO from thumbs-up dialogue, online preference learning.
7. [`07-social-simulation.md`](07-social-simulation.md) — theory-of-mind beliefs,
   multi-agent debate for Tier 2, graph diffusion for gossip.
8. [`08-multimodal.md`](08-multimodal.md) — Whisper ASR, TTS with per-NPC voice,
   diffusion portraits and ambient art.
9. [`09-evaluation-and-safety.md`](09-evaluation-and-safety.md) — LLM-as-judge
   harness, red-team suite, calibrated abstention.

## Prioritisation heuristic

Rank each technique by:

- **Player-visible impact** (does it change the feel of a conversation?).
- **Implementation cost in Parish** (does it fit the crate boundary?).
- **Local-first compatibility** (can it run under Ollama, or does it need cloud?).
- **Mode parity** (ADR rule: CLI / web / Tauri must agree).

High-impact + low-cost + local-first should ship first: semantic memory and
constrained JSON decoding are the obvious starting points.
