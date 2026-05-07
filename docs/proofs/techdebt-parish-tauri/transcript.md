Evidence type: gameplay transcript

## Summary

Technical debt cleanup for the `parish-tauri` crate. Four items resolved, one deferred.

### Changes

**TD-001 (P1) - Command registry sync:**
- Added `get_demo_config`, `get_demo_context`, `get_llm_player_action` to `EXPECTED_COMMANDS` in `src/command_registry.rs`
- Updated `EXPECTED_COUNT` from 29 to 32 in `tests/command_registry.rs`
- Added compile-time symbol imports for 3 demo commands

**TD-002 (P2) - Save implementation duplication:**
- Deleted `do_save_game_inner` (52 lines) from `src/command_host.rs`
- `TauriCommandHost::save_game` now delegates to `commands::do_save_game` -> `parish_core::game_loop::do_save_game`
- Removed unused imports: `Database`, `new_save_path`, `GameSnapshot`

**TD-004 (P1) - Weak tests:**
- Added `get_world_snapshot_inner_returns_start_location` unit test
- Updated stale doc comments in `tests/command_logic.rs` (28 -> 32, deferred 25 -> 29)

**TD-005 (P3) - Snapshot helper duplication:**
- 5 manual `snapshot_from_world + compute_name_hints` call sites refactored to use `get_world_snapshot_inner`
- Removed unused imports from `command_host.rs`

**TD-003 (P2) - Deferred:** run() complexity. No dead code to prune; requires extraction work.

### Files modified

- `parish/crates/parish-tauri/src/command_registry.rs`
- `parish/crates/parish-tauri/src/commands.rs` (TD-005 x4, TD-004 test, pub(crate))
- `parish/crates/parish-tauri/src/command_host.rs` (TD-002, TD-005)
- `parish/crates/parish-tauri/src/lib.rs` (none, TD-003 deferred)
- `parish/crates/parish-tauri/tests/command_registry.rs`
- `parish/crates/parish-tauri/tests/command_logic.rs`
- `parish/crates/parish-tauri/TODO.md`

### Test output

```
running 43 tests (unit, src/lib.rs)
... all ok
running 13 tests (command_logic.rs)
... all ok
running 3 tests (command_registry.rs)
... all ok
running 17 tests (input_validation.rs)
... all ok
result: 76 passed; 0 failed
```

### Clippy output

```
cargo clippy -p parish-tauri -- -D warnings
Finished dev profile — no warnings
```
