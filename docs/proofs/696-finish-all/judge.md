# Judge Verdict — #696 slice 4: finish game-loop orchestration extraction

## Review scope

Reviewed the full diff of `refactor/696-finish-all-orchestration` against
`origin/main`, encompassing slices 3 and 4 of issue #696.

## Structural assessment

**Extraction completeness.** `parish-core/src/game_loop/` now holds all shared
game-loop logic (`handle_game_input`, `handle_movement`, `handle_npc_conversation`,
`run_idle_banter`, `emit_npc_reactions`, `is_snippet_injection_char`).  The
`routes.rs` handler (`handle_game_input`) is a thin delegation stub (≤20 lines).
The Tauri command counterpart (`commands.rs`) was aligned in prior slices.

**Dead code.** No `dead_code` warnings remain after the slice 4 cleanup:
`handle_movement` and `handle_npc_conversation` in `routes.rs` are correctly
gated `#[cfg(test)]` because they are thin test shims, not production paths.
The private `is_snippet_injection_char` copy was deleted; tests import from
`parish_core::game_loop`.

**Mode parity.** Server and Tauri both delegate to the same
`parish_core::game_loop::*` functions.  The architecture-fitness test
`backend_agnostic_crates_do_not_pull_runtime_deps` prevents future drift.

**Behaviour preservation.** The `on_persist` callback refactor in
`reactions.rs` is a pure interface lift: the same lock-after-inference pattern
is preserved; the Mutex is acquired inside the callback exactly as before.

**Lint discipline.** `cargo clippy --workspace --all-targets -- -D warnings`
produces no errors or warnings.  The single `needless_borrow` was fixed rather
than suppressed with `#[allow]`.

## Concerns

None.  The refactor is mechanical: identical logic, relocated.

Verdict: sufficient

Technical debt: clear
