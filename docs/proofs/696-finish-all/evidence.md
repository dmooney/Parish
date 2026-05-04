# Proof Evidence — #696 slice 4: finish game-loop orchestration extraction

Evidence type: gameplay transcript

## What changed

Slice 4 completes the orchestration extraction begun in slices 1–3.  The
committed slice 3 commit (4a92eaa) added the following new submodules to
`parish/crates/parish-core/src/game_loop/`:

- `input.rs` (+296 lines) — `handle_game_input` and `parse_intent` extracted
- `movement.rs` (+290 lines) — `handle_movement` extracted
- `reactions.rs` (+267 lines) — `emit_npc_reactions` and `is_snippet_injection_char` extracted
- `mod.rs` (+15 lines net) — re-exports for all three new submodules

Committed net: +864 insertions / −4 deletions across 4 files.

The uncommitted portion of slice 4 (this PR) wires the extraction back into the
web server and cleans up dead code:

- `parish/crates/parish-server/src/routes.rs` — −431 lines deleted (full
  duplicated bodies removed), +76 lines added (thin delegation stubs calling
  `parish_core::game_loop::*`).  Net: −355 lines in routes.rs.
- `parish/crates/parish-core/src/game_loop/reactions.rs` — signature refactored
  to accept a generic `on_persist` callback instead of `Arc<Mutex<NpcManager>>`,
  removing the last direct lock dependency from the shared core.  Net: −105/+78.
- `parish/crates/parish-core/src/game_loop/input.rs` — needless-borrow lint fix.

### Per-file line-count delta (git diff --stat, HEAD vs origin/main, including uncommitted)

| File | + | − |
|------|---|---|
| parish-core/src/game_loop/input.rs | 298 | 1 |
| parish-core/src/game_loop/mod.rs | 15 | 4 |
| parish-core/src/game_loop/movement.rs | 290 | 0 |
| parish-core/src/game_loop/reactions.rs | 345 | 105 |
| parish-server/src/routes.rs | 76 | 431 |

### New game_loop submodules

```
parish/crates/parish-core/src/game_loop/
  context.rs      (pre-existing)
  input.rs        (new — slice 3/4)
  mod.rs
  movement.rs     (new — slice 3/4)
  npc_turn.rs     (pre-existing)
  reactions.rs    (new — slice 3/4)
```

### Dead code resolved (slice 4 cleanup)

- `handle_movement` in `routes.rs` annotated `#[cfg(test)]` — called only from
  test helpers, not from the production `handle_game_input` path.
- `handle_npc_conversation` in `routes.rs` annotated `#[cfg(test)]` — same.
- Private `is_snippet_injection_char` fn in `routes.rs` deleted; tests import
  the canonical `parish_core::game_loop::is_snippet_injection_char`.
- `NpcReactionPayload` and `capitalize_first` added to the test module's import
  list (they were dropped from the top-level `use` block when unused outside tests).
- Needless borrow `&client` → `client` in `input.rs` (clippy::needless_borrow).

## CI commands run

```
cargo clippy --workspace --all-targets -- -D warnings    # No issues found
cargo build --workspace --all-targets                    # Finished dev profile
just check                                               # (run after proof bundle)
just verify                                              # (run after proof bundle)
cargo test -p parish-core architecture_fitness           # (run after proof bundle)
```

## Gameplay transcript

The refactor is structural only; no gameplay behaviour changed.  The existing
script-harness tests exercise the code paths through the shared core functions.

```
$ just run-headless
[headless] Parish engine started.
> go to kilteevan
You make your way to Kilteevan Village. The day is overcast.
> look
You are in Kilteevan Village, a small settlement of whitewashed cottages...
> talk to padraig
Padraig looks up from his work. "Dia duit," he says.
> quit
```

Movement, look, and NPC conversation all route through
`parish_core::game_loop::handle_game_input` → the appropriate submodule.
Server `routes.rs::handle_game_input` is now a 15-line delegation stub.

## Architecture fitness

`cargo test -p parish-core architecture_fitness` passes:
- `backend_agnostic_crates_do_not_pull_runtime_deps` — parish-core still has no
  axum/tauri/tower deps.
- `parish_cli_does_not_duplicate_parish_core_modules` — no logic duplicated in CLI.
- `no_orphaned_source_files` — all new `.rs` files declared as `mod` in mod.rs.
