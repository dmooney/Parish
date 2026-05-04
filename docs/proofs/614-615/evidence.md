# Proof Evidence — Issues #614 and #615: SessionStore + IdentityStore traits

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/614-615-session-identity-stores

## Requirements

#614 — Introduce a `SessionStore` trait in `parish-core` so route handlers and
the autosave tick can depend on an abstract persistence seam instead of concrete
`AsyncDatabase` calls.  Methods: `load_latest_snapshot`, `save_snapshot`,
`list_branches`, `create_branch`, `load_branch`, `branch_log`, `acquire_save_lock`,
`release_save_lock`, `save_path`, `append_journal_event`, `read_journal`.

#615 — Split the `SessionRegistry` god struct into two focused traits in
`parish-core`: `IdentityStore` (OAuth account management) and `SessionRegistry`
(session bookkeeping).  Methods:
- `IdentityStore`: `lookup_by_provider`, `link_provider`, `get_account`, `create_account`.
- `SessionRegistry`: `lookup`, `register`, `touch`, `cleanup_stale`, `evict_idle`.

Both traits are backend-agnostic (no `axum`/`tauri` imports).  Default impls
(`DbSessionStore`, `SqliteIdentityStore`, `SqliteSessionRegistry`) live in
`parish-server/src/session_store_impl.rs` and back the same `sessions.db`
schema with no migration.

## Test Run — parish-core

Command:

```sh
cargo test -p parish-core
```

Result:

```
cargo test: 309 passed, 4 ignored (6 suites, 5.32s)
```

Architecture fitness test (`architecture_fitness.rs`) passes — no forbidden
`axum`/`tauri`/`tower*` imports in backend-agnostic crates, no orphaned files.

## Test Run — parish-server

Command:

```sh
cargo test -p parish-server
```

Result:

```
cargo test: 209 passed (7 suites, 0.51s)
```

New tests included:
- `SqliteIdentityStore` round-trip: `identity_store_link_and_lookup`, `identity_store_lookup_missing_returns_none`
- `SqliteSessionRegistry` round-trip: `session_registry_register_and_exists`, `session_registry_touch_updates_timestamp`
- `DbSessionStore` round-trip: `db_session_store_save_and_load_roundtrip`, `db_session_store_create_branch`, `db_session_store_journal_append_and_read`, `db_session_store_acquire_lock`
- `InMemorySessionStore` round-trip: `in_memory_session_store_roundtrip`, `in_memory_session_store_multiple_snapshots`

## Behaviour Impact

This is a pure trait-seam introduction.  No gameplay logic, no UI state, no
observable API behaviour changed.  The OAuth flow, session cookie lifecycle,
autosave tick, and all route handlers continue to use the same SQLite-backed
implementations.  `AppState.session_store: Arc<dyn SessionStore>` is plumbed
through but route handlers do not yet call methods on it — that migration is
deferred to follow-up PRs.

`GlobalState.sessions` keeps the concrete `SessionRegistry` struct for now;
migrating it to `Arc<dyn SessionRegistry>` is intentionally out of scope for
this PR.
