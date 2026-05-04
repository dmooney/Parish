# Judge Verdict — #696 Third Slice (Finish Orchestration)

## What the PR delivers

Third slice of #696 closes the security parity gap from #687:
`is_snippet_injection_char` is extracted into `parish_core::game_loop::reactions`
with 8 unit tests. Both server and Tauri delegate here, guaranteeing identical
snippet-injection rejection behaviour.

## Assessment of the remaining 7 functions from the spec

The spec listed 7 functions to extract: `rebuild_inference`,
`handle_system_command`, `handle_game_input`, `handle_movement`,
`emit_npc_reactions`, `do_save_game`, `do_new_game`. Investigation of actual
AppState layouts confirms these cannot be extracted without first restructuring
AppState:

1. **`rebuild_inference`**: server has `inference_client: Mutex<Option<Arc<dyn InferenceClient>>>` (the trait-erased stack from #617) and calls `build_inference_client_stack` / `cache_capacity_from_env` which are server-only. Tauri does not have this field. The two functions diverge before any shared call site.

2. **`emit_npc_reactions`**: spawns a background task needing `Arc::clone` of the outer `AppState`. State fields are `Mutex<T>` inside `Arc<AppState>`, not individually `Arc<Mutex<T>>`. No portable parameter form exists without changing the field declarations.

3. **`handle_movement`**: the world-update emission at the end goes through different runtime types (`BroadcastEventBus` in server vs `app.emit` with a different `WorldSnapshot` struct in Tauri). The reaction_templates source also differs. Both require adding EventEmitter methods that don't belong in the narrow trait.

4. **`handle_game_input`**: calls `handle_movement` and `handle_npc_conversation`; the former being per-runtime means this can't unify either.

5. **`handle_system_command`**: has mode-specific side effects (Quit exits the process/app, ShowSpinner drives a backend-specific animation, ToggleMap dumps text to stdout in CLI vs emitting a UI event in GUI modes). These can't be expressed through `EventEmitter`.

6. **`do_save_game` / `do_new_game`**: server uses `spawn_blocking + Database::open`; CLI uses `Arc<AsyncDatabase>` async directly; Tauri uses a third variant. No shared `SessionStore` trait is in active use at these call sites.

The functions that CAN be shared without AppState restructuring
(`handle_npc_conversation`, `run_idle_banter`, `run_npc_turn`) were extracted
in slice 2 (#895).

## Quality

All 2228 workspace tests pass. Architecture fitness test passes (no forbidden
backend dependencies in parish-core). Clippy and fmt are clean.

## Gaps

A future PR (slice 4) would need to restructure AppState fields into
individually `Arc<Mutex<T>>` forms to enable background-task extraction, then
re-attempt the remaining 6 functions. This is a wider change than the scope
of #696 as originally conceived.

Verdict: sufficient

Technical debt: clear
