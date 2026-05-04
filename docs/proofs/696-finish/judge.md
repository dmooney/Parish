# Judge Verdict — #696 Third Slice (Finish Orchestration)

## Review

The third slice of #696 addresses the `is_snippet_injection_char` security
parity gap documented in issue #687. The function previously existed as two
identical copies — one in `parish-server/src/routes.rs` and one in
`parish-tauri/src/commands.rs` — creating a maintenance risk where a fix to
one would not automatically propagate to the other.

### What the PR does

1. Adds `parish_core::game_loop::reactions` module containing the shared
   `is_snippet_injection_char` function with 8 unit tests covering all blocked
   character classes.

2. Replaces both per-runtime local definitions with `use` imports from the
   shared location, guaranteeing identical behaviour across runtimes.

3. Updates `parish_core::game_loop::mod.rs` with a documented explanation of
   why `handle_system_command`, `rebuild_inference`, `emit_npc_reactions`,
   `do_save_game`, `do_new_game`, `handle_movement`, and `handle_game_input`
   cannot be extracted at this architectural boundary without restructuring
   AppState.

### Assessment

The claim "7,500 → 2,500 lines across 3 files" in the issue spec assumed all 7
functions were portable. Investigation found that AppState stores state as
`Mutex<T>` fields inside `Arc<AppState>`, not as individually `Arc<Mutex<T>>`
fields, making background-task-spawning functions like `emit_npc_reactions`
impossible to extract without restructuring `AppState`. The functions that CAN
be extracted (`handle_npc_conversation`, `run_idle_banter`, `run_npc_turn`)
were extracted in slice 2. This slice completes the work by extracting the
one remaining portable function (`is_snippet_injection_char`) and documenting
the architectural constraints.

All 2,228 workspace tests pass. The architecture fitness test confirms
`parish-core` has no forbidden backend dependencies. Clippy and fmt are clean.

### Gaps and open items

- The headless CLI (`parish-cli`) still uses its own inline NPC orchestration.
  This is a pre-existing condition documented in the `mod.rs` comment since
  slice 2 and is outside scope for this PR.
- A future slice could restructure `AppState` to use individually
  `Arc<Mutex<T>>` fields, which would allow `emit_npc_reactions` to be
  extracted. This is a significant refactor beyond the scope of #696.

Verdict: sufficient

Technical debt: clear
