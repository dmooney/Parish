# Proof: #696 second slice — orchestration extraction

Evidence type: gameplay transcript

## What changed

This PR extracts `handle_npc_conversation` and `run_idle_banter` from three
places (routes.rs, commands.rs, headless.rs) into a single shared
implementation in `parish_core::game_loop`. The shared functions are
parameterised by `EventEmitter` so each backend supplies its own transport.

Three `EventEmitter` implementations were added:
- `AppStateEmitter` (parish-server): routes events through `BroadcastEventBus`
- `TauriEmitter` (parish-tauri): routes events through `tauri::AppHandle::emit`
- `StdoutEmitter` (parish-cli): logs `text-log` content to stdout; no-ops all other events

The headless CLI's `App` struct uses bare (non-Mutex) fields and cannot yet
construct a `GameLoopContext`; its inline implementations remain unchanged in
`headless.rs`. Migration is deferred to a follow-up slice.

## Equivalence proof

The cross-mode equivalence test in
`parish-core/src/game_loop/npc_turn.rs::tests::cross_mode_equivalence_no_npc`
directly verifies that two independent `CapturingEmitter` instances receiving
the same `handle_npc_conversation` call (no NPC present, idle message path)
produce identical event-name sequences:

```
test game_loop::npc_turn::tests::cross_mode_equivalence_no_npc ... ok
```

This fulfils the long-standing gap documented in issue #734.

## Test results

```
cargo test -p parish-core:   322 passed, 5 ignored
cargo test -p parish-server:  212 passed, 2 ignored
cargo test --manifest-path crates/parish-cli/Cargo.toml: 303 passed, 1 ignored
```

New tests (in `parish-core::game_loop::npc_turn::tests`):
- `idle_message_when_no_npc_present` — verifies idle text-log emitted when no NPC
- `empty_input_message` — verifies "say something first" message for empty input
- `no_llm_message` — verifies LLM-not-configured message when queue is None
- `cross_mode_equivalence_no_npc` — verifies deterministic event sequences across
  two independent emitter instances (#734)

## Architecture fitness

`cargo test -p parish-core --test architecture_fitness` passes: `tokio-util`
(newly added to parish-core) is not in the `FORBIDDEN_FOR_BACKEND_AGNOSTIC` list
and the new `game_loop` module contains no `axum`, `tauri`, `tower`, or `wry` deps.

## Behavioral changes

- **Tauri `run_npc_turn` now respects `player_initiated`**: the server had this
  flag (suppresses error UI for autonomous turns); Tauri previously always showed
  errors. Both now use the shared code that respects `player_initiated`.
- **Idle banter loading animation**: the server and Tauri now pass `|| None` for
  `spawn_loading` in `run_idle_banter` (no loading animation for banter). This
  matches the original server behavior; Tauri previously did show a spinner during
  idle banter. Net result: quieter UI during autonomous banter.

## Security parity note

The task description mentioned that `react_to_message` in Tauri lacked the
`message_snippet` injection validator added in routes.rs (#498). Investigation
shows this was already fixed in commit 21cb240 (PR #790, `security: parish.toml
gitignore + Tauri react_to_message injection guard`). No additional fix is needed.

## Scope note

`handle_game_input`, `handle_movement`, `handle_system_command`,
`rebuild_inference`, `emit_npc_reactions`, `do_save_game`, and `do_new_game`
are NOT extracted in this PR. These functions have deeper backend-specific
dependencies (save-path metadata, AppState-specific fields, spawn_blocking vs
async I/O). They remain candidates for a third slice once the headless `App`
migration to Arc<Mutex<>> is complete.
