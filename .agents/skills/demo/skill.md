---
name: demo
description: Run the LLM auto-player (demo mode) to verify gameplay changes, surface bugs, and generate content. Use after implementing Tauri-facing features or NPC/world changes that the script harness cannot exercise.
disable-model-invocation: false
argument-hint: [turns] [pause_secs]
---

Run the LLM auto-player to prove a feature works in the live Tauri app.

## When to use demo mode

The script harness (`/prove`) covers headless CLI behavior. Demo mode covers the full Tauri stack:

- Tauri IPC commands (everything in `parish-tauri`)
- Streaming NPC dialogue in the UI
- Frontend store behavior (chat rendering, status updates)
- Features that depend on the real LLM generating content (NPC personality, Irish dialogue)
- Regression checks after touching `commands.rs`, `lib.rs`, or `+page.svelte`

## Running the demo

```sh
just demo [pause_secs] [max_turns]   # from repo root
# e.g.
just demo 2 5    # 5 turns, 2s between turns — fast smoke test
just demo 4 15   # 15 turns, 4s between turns — content generation run
just demo 3      # unlimited turns — sustained observation
```

Flags passed to the binary:
- `--demo` — enables demo mode
- `--demo-prompt mods/rundale/demo-prompt.txt` — wandering-stranger persona
- `--demo-pause <secs>` — seconds to wait between turns
- `--demo-max-turns <n>` — stop after n turns (omit for unlimited)

## Observing what happens

Demo mode has no persistent test output file. Capture logs:

```sh
RUST_LOG=info,parish_tauri_lib=debug just demo 2 5 > /tmp/demo.log 2>&1 &
# wait for turns to complete, then:
grep -E "chat \[|demo turn|WARN|ERROR" /tmp/demo.log
```

Key log lines to watch:

| Pattern | Meaning |
|---------|---------|
| `chat [player] input=...` | What the demo submitted as the player action |
| `chat [npc] speaker=... dialogue=...` | Full NPC response after streaming |
| `demo turn: LLM chose action=...` | Raw extracted action before submission |
| `chat source=system text=...` | Movement narration, system messages |
| `WARN Inference error: 429` | Rate-limit hit — NPC response will be blank |
| `WARN Tier 2 inference failed` | Background NPC simulation failed (separate) |
| `WARN World advanced during intent parse` | TOCTOU tick during intent LLM call (non-blocking) |

## What to verify

**After any NPC or dialogue change:**
- `chat [npc]` lines contain Irish-authentic speech — no anachronisms, no stage dialect
- Dialogue responds to what the player said, not to some prior turn
- NPC names are correct (`speaker=Peig Hannigan`, not a hallucinated name)

**After any Tauri IPC change:**
- `demo turn: LLM chose action` fires for each turn — if absent, `get_llm_player_action` is hanging or failing
- `chat [player]` appears after each action — `submit_input` reached the backend
- `chat [npc]` appears — NPC conversation completed and streamed

**After frontend changes (`+page.svelte`, stores):**
- Demo completes max_turns and stops (checks `demoEnabled` store)
- No `demo-player: waitForFalse timed out` warnings — streaming completes cleanly
- No multiple instances of the same message (store subscription leak)

**After world/movement changes:**
- `chat source=system` lines contain correct location names and travel times
- Player relocates when demo says `go to <location>`

## Common bugs demo surfaces that `/prove` misses

1. **Streaming never completes** — `waitForFalse` times out after 30s, logged as warning. Root cause is usually a missing `stream-end` event or frontend store not resetting.

2. **Thinking block leaking** — `chat [player]` shows multi-paragraph reasoning instead of a one-line action. Fix is in `extract_action_from_response` in `commands.rs`.

3. **Game stays paused** — demo auto-resumes (`/resume` before each turn) but if you see the game clock frozen in the UI, check `worldState.paused` in debug panel.

4. **NPC says nothing** — `chat [npc]` absent or empty. Check for `WARN Inference error: 429` (rate limit), or JSON parse failure (`dialogine` typo handled, but new variants may appear). Add `#[serde(alias = "...")]` to `NpcJsonResponse.dialogue` for new typos.

5. **Wrong action submitted** — `chat [player]` shows raw JSON or reasoning. The fill-in-the-blank technique (`{"action": "`) usually prevents this, but if the model ignores it, `extract_action_from_response` falls back to last-line stripping.

6. **Save picker appears** — a stale save lock from a killed demo session. Quit all game instances and restart. The demo still runs but game state is unrestored.

## Generating content

Run a long demo (20+ turns) to generate authentic NPC dialogue samples:

```sh
just demo 3 20 > /tmp/content.log 2>&1
grep "chat \[npc\]" /tmp/content.log | sed 's/.*dialogue=//' | sort
```

Good output: varied topics, Irish phrases, period-accurate references (1820, County Roscommon), NPCs referencing their location and the weather.

Red flags: NPCs repeating the same greeting, mentioning modern concepts, speaking in stage-Irish dialect ("begorrah"), or calling the player "Dave" before introduction.

## Demo does NOT replace

- `just check` — still required before every PR
- `/prove` — still required for headless gameplay features
- `/verify` — still required before push
- Playwright E2E tests — for UI layout and navigation

Demo mode is observational. It surfaces live behavior; it does not assert. If you find a bug, fix it and re-run.
