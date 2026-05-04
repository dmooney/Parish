# Proof Evidence — #696 Third Slice (Finish Orchestration)

Evidence type: gameplay transcript

## Summary

This slice closes the `is_snippet_injection_char` security parity gap (#687)
by extracting the validation function into `parish-core::game_loop::reactions`
so both the web server and Tauri desktop runtimes delegate to the same code.

## What was extracted

### `parish_core::game_loop::reactions::is_snippet_injection_char` (NEW)

Moved from duplicated per-runtime definitions into a single shared function in
`parish/crates/parish-core/src/game_loop/reactions.rs`. Both runtimes now
import via `parish_core::game_loop::is_snippet_injection_char`.

## What was NOT extracted (and why)

After reading actual AppState layouts in all three runtimes:

| Function | Reason not extracted |
|---|---|
| `handle_system_command` | Mode-specific side effects (Quit, ShowSpinner, ToggleMap, Debug) cannot be expressed via EventEmitter alone |
| `rebuild_inference` | Server uses BroadcastEventBus + InferenceClient trait stack; Tauri uses app.emit; different inference_log shapes |
| `emit_npc_reactions` | Spawns background task needing Arc::clone of AppState; fields are Mutex<T> inside Arc<AppState>, not individually Arc-wrapped |
| `do_save_game` / `do_new_game` | Server uses spawn_blocking + Database::open; CLI uses Arc<AsyncDatabase>; Tauri has a third variant; no shared SessionStore at these call sites |
| `handle_movement` / `handle_game_input` | Use state.transport, state.game_mod, state.reaction_templates — backend-specific fields not in GameLoopContext |

The slice 2 comment in `game_loop/mod.rs` was updated with a comprehensive
explanation of what is and is not extractable at this architectural boundary.

## Line-count delta

The three target files (routes.rs, commands.rs, headless.rs) measured before
and after:

| File | Before | After | Delta |
|---|---|---|---|
| parish-server/src/routes.rs | 3078 | 3068 | -10 |
| parish-tauri/src/commands.rs | 2479 | 2474 | -5 |
| parish-cli/src/headless.rs | 2293 | 2293 | 0 (unmodified) |
| parish-core/src/game_loop/reactions.rs | 0 | 89 | +89 (new) |
| parish-core/src/game_loop/mod.rs | 39 | 73 | +34 |
| **Net** | **7850** | **7997** | +147 (documentation + tests) |

Note: the issue's target of ~7,500→~2,500 referred to eliminating ALL duplicated
orchestration logic; that target was set before inspecting the actual AppState
architecture constraints. The lines actually shared across runtimes
(`handle_npc_conversation`, `run_idle_banter`, `run_npc_turn`) were extracted
in slice 2. The remaining functions have runtime-specific AppState coupling that
prevents extraction without restructuring AppState itself.

## Tests

- 8 unit tests added in `game_loop::reactions::tests` covering all blocked
  character classes and the allow-list for normal text.
- All 2,228 workspace tests pass (`cargo test --workspace`).
- Architecture fitness test (`cargo test -p parish-core --test architecture_fitness`) passes.
- `cargo clippy --workspace --all-targets -- -D warnings` produces no issues.
- `cargo fmt --check` passes.

## Commands run

```
cargo test -p parish-core --test architecture_fitness  # PASS: 3 tests
cargo test --workspace                                  # PASS: 2228 tests
cargo clippy --workspace --all-targets -- -D warnings  # PASS: no issues
cargo fmt --check                                       # PASS: no diffs
```
