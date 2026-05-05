# Proof Evidence — #696 Slice 8: Wire SessionStore + Extract do_new_game

Evidence type: gameplay transcript

## Summary

This slice completes issue #696 by:

1. Moving `DbSessionStore` from `parish-server` to `parish-core::session_store`
2. Wiring `Arc<dyn SessionStore>` into Tauri's `AppState` and CLI's `App`
3. Extracting `do_new_game` to `parish-core::game_loop::save::do_new_game`
4. Extracting `do_save_game` to `parish-core::game_loop::save::do_save_game`
5. Updating server and Tauri to delegate to canonical impls

## File Line-Count Deltas (this slice)

| File | Insertions | Deletions | Net |
|---|---|---|---|
| `parish-core/src/session_store.rs` | +243 | -1 | **+242** (DbSessionStore moved here) |
| `parish-core/src/game_loop/save.rs` | +229 | -2 | **+227** (do_new_game + do_save_game added) |
| `parish-core/src/game_loop/mod.rs` | +97 | -68 | **+29** (doc rewrite) |
| `parish-cli/src/app.rs` | +10 | 0 | **+10** (session_store field + import) |
| `parish-cli/src/headless.rs` | +4 | 0 | **+4** (session_store wiring) |
| `parish-tauri/src/lib.rs` | +17 | -1 | **+16** (session_store field + construction) |
| `parish-tauri/src/commands.rs` | +138 | -208 | **-70** (do_new_game, do_save_game become stubs) |
| `parish-server/src/routes.rs` | +150 | -220 | **-70** (do_new_game_inner, do_save_game_inner become stubs) |
| `parish-server/src/session_store_impl.rs` | +234 | -390 | **-156** (DbSessionStore body removed, re-exported) |

**Net for runtime files (routes.rs + commands.rs + headless.rs):** -140 lines removed from runtimes.

## Canonical implementations

All canonical game-loop logic is now in `parish-core/src/game_loop/save.rs`:

- `load_fresh_world_and_npcs` — slice 6
- `do_save_game` — slice 7/8 (now canonical)
- `do_new_game` — slice 8 (new)

## SessionStore wiring

- **Server**: `AppState::session_store: Arc<dyn SessionStore>` (from #614, unchanged)
- **Tauri**: `AppState::session_store: Arc<dyn SessionStore>` (added this slice)
- **CLI**: `App::session_store: Arc<dyn SessionStore>` (added this slice)

`DbSessionStore` is now in `parish_core::session_store::DbSessionStore` with no dashmap dependency (uses `std::sync::RwLock<HashMap>`). Single-user runtimes pass `session_id = ""`.

## Verification

```
cargo build --workspace --all-targets  # clean
cargo clippy --workspace --all-targets -- -D warnings  # no issues
cargo test --workspace  # 2234 passed, 16 ignored
cargo test -p parish-core --test architecture_fitness  # 3 passed
```
