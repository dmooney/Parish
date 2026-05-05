# Judge Verdict — #696 Slice 8

## What was claimed

The slice claimed to:
1. Move `DbSessionStore` from `parish-server` to `parish-core` so all runtimes can use it
2. Wire `Arc<dyn SessionStore>` into Tauri `AppState` and CLI `App`
3. Extract `do_new_game` to `parish-core::game_loop::save`
4. Extract `do_save_game` to `parish-core::game_loop::save` (completing slice 7)
5. Reduce duplication in routes.rs, commands.rs, and session_store_impl.rs

## Verification

**Build:** `cargo build --workspace --all-targets` — clean, zero warnings.

**Lint:** `cargo clippy --workspace --all-targets -- -D warnings` — no issues.

**Tests:** `cargo test --workspace` — 2234 passed, 16 ignored. No regressions.

**Architecture gate:** `cargo test -p parish-core --test architecture_fitness` — 3 passed. `parish-core` does not import `axum`, `tauri`, `tower`, `wry`, or `tao`. `DbSessionStore` lives in `parish-core` and uses only `std` + `parish-persistence` — no forbidden imports.

## Scope assessment

**SessionStore wiring:** Server already had it. Tauri and CLI now have `session_store: Arc<dyn SessionStore>` in their state types. Wired with correct `DbSessionStore::new(saves_dir)` at startup. Single-user runtimes pass `session_id = ""` which resolves to flat `saves/` layout.

**do_new_game extraction:** Canonical impl is `parish_core::game_loop::do_new_game(NewGameParams)`. Server's `do_new_game_inner` is a 15-line delegation stub. Tauri's `do_new_game` is a 20-line delegation stub. CLI's `handle_headless_new_game` remains because it creates a branch on an existing `AsyncDatabase` rather than a fresh save file — a documented structural difference, not a deferral.

**do_save_game extraction:** Canonical impl is `parish_core::game_loop::do_save_game(...)`. Both `do_save_game_inner` (server) and `do_save_game` (Tauri) are delegation stubs of 8 lines each.

**DbSessionStore relocation:** Moved from `parish-server/src/session_store_impl.rs` to `parish-core/src/session_store.rs`. `DashMap` dependency replaced with `std::sync::RwLock<HashMap>` (no new crate dependency). `SqliteIdentityStore` and `SqliteSessionRegistry` remain in server (they use `rusqlite` directly and `is_valid_session_id` which is server-internal).

**game_loop/mod.rs doc:** All "deferred" and "future slice" language removed. The module comment now describes the final extracted state.

## Issues / caveats

- CLI `handle_headless_new_game` is not deduplicated. This is legitimate: the CLI branch strategy (create new branch on existing DB) differs from server/Tauri (create entirely new save file). Documented in evidence.md and in `game_loop/mod.rs` doc comment.
- `do_new_game` does NOT call `session_store.save_snapshot`. `new_save_path(saves_dir)` creates a new file (alphabetically next), while `DbSessionStore::ensure_db("")` returns the alphabetically-first existing file — a different file. Routing through SessionStore during new-game would have corrupted the previous save. The `session_store` field is wired into all three runtimes for future use by load/branch/journal paths; new-game file creation stays as `Database::open` directly.

Verdict: sufficient

Technical debt: clear
