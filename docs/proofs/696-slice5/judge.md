# Judge Verdict — #696 Slice 5

## Changes reviewed

- `emit_npc_reactions` extracted from `parish-server/src/routes.rs` and
  `parish-tauri/src/commands.rs` into `parish-core/src/game_loop/reactions.rs`
- Server and Tauri replaced with thin wrappers (pre-capture NPC list, resolve
  config, call shared function via `PersistReactionFn` callback)
- New cross-mode equivalence test in `game_loop/reactions::tests`
- Architecture fitness tests pass (no backend crate imports in parish-core)
- `mod.rs` docs updated with slice 5 rationale and deferred-work explanation

## Functional correctness

The extracted function preserves all invariants of the original:
- Pre-captured location prevents TOCTOU race (player moves between call and spawn)
- NPC_REACTION_CONCURRENCY semaphore preserved
- Panic/cancellation watcher task preserved
- Persist callback closes over `Arc<AppState>` and uses async locking — correct
- Event name `"npc-reaction"` preserved; both server `AppStateEmitter` and
  Tauri `TauriEmitter` route this correctly through `EventEmitter::emit_event`
- Server tests (`emit_npc_reactions_uses_precaptured_location`,
  `emit_npc_reactions_concurrent_batch_attributes_all_npcs`) still pass

## Scope decisions

The deferred items (`rebuild_inference`, `handle_system_command`, `do_save_game`,
`do_new_game`, CLI migration) are correctly documented with engineering rationale.
The scope choices are defensible and consistent with the task's own guidance on
partial work: "commit your last green state, push partial work."

## Architecture gate

`parish-core` does not import `axum`, `tauri`, `wry`, `tao`, or any
backend-specific crate. The `architecture_fitness` test confirms this.

## Code quality

- Clippy `-D warnings` passes clean
- `#[allow(clippy::too_many_arguments)]` justified by a comment referencing
  the distinct semantic roles of each parameter (per project convention)
- Unused imports cleaned up in server and Tauri
- Module doc comment updated to reflect slice 5 state

The extraction is functionally correct, test-covered, and builds clean.
The scope boundary is clearly documented with engineering rationale for each
deferred item. The cross-mode equivalence test provides structural parity
verification per #734.

Verdict: sufficient

No new TODO/FIXME comments introduced. Deferred work is tracked through
existing issues (#696 for remaining extraction, future slice for CLI migration).

Technical debt: clear
