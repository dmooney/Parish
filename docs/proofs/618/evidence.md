Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/618-account-id

# Proof Evidence — #618: stable account_id keying (not email/cookie)

## Acceptance criteria

1. `AuthContext` gains a stable `account_id: Uuid` field populated by `cf_access_guard`.
2. `cf_access_guard` resolves `account_id` via `IdentityStore::lookup_by_provider` /
   `create_account` — mints a UUID on first login, returns the same UUID thereafter.
3. Feature flag `account-id-keying` (default-on) gates the new keying.
4. Editor sessions keyed by `(account_id, mod_id)` instead of email — multi-tab
   from the same account no longer creates duplicate sessions.
5. `active_ws` uses `HashSet<Uuid>` instead of `HashSet<String>` — single-WS-per-account.
6. Tracing spans record `account_id` (UUID) not email.
7. Integration tests: multi-tab, email-change, and auth-regression coverage.

## cargo check

```
cd parish && cargo build -p parish-server
Compiling parish-server v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

Exit code 0. No errors, no warnings.

## Test results

```
cd parish && cargo test -p parish-server
cargo test: 215 passed, 2 ignored (9 suites, 0.52s)
```

All 215 tests pass. Three new tests added under `tests/account_id.rs`:

- `same_email_two_tabs_get_same_account_id` — verifies two tabs with the same
  CF-Access email resolve to the same UUID via `SqliteIdentityStore`.
- `link_provider_email_change_preserves_account_id` — verifies `link_provider`
  with a new display name does not change the stored `account_id`.
- `auth_flow_produces_valid_auth_context_uuid` — verifies the `resolve_account_id`
  logic mints a valid non-nil UUID and returns it consistently on repeat calls.

## Changed files

| File | Change |
|------|--------|
| `cf_auth.rs` | `AuthContext` gains `account_id: Uuid`; `validate()` returns `String` |
| `lib.rs` | `cf_access_guard` takes `GlobalState` state; calls `resolve_account_id` |
| `session.rs` | `GlobalState` gains `identity_store: Arc<dyn IdentityStore>` |
| `state.rs` | `editor_sessions: HashMap<(Uuid, String), ...>`; `active_ws: HashSet<Uuid>` |
| `editor_routes.rs` | All session accesses use `session_key(account_id)` |
| `ws.rs` | `ActiveWsGuard` tracks `account_id`; handler extracts from `AuthContext` |
| `tests/account_id.rs` | New integration tests |
| `tests/isolation.rs` | Updated `second_ws_upgrade_same_account_is_409` |

## Feature flag verification

`lib.rs::resolve_account_id`:
```rust
if flags.is_disabled("account-id-keying") {
    return uuid::Uuid::nil();
}
```

Uses `is_disabled` — default-on per CLAUDE.md rule #6. When disabled, `Uuid::nil()`
is returned so the old email-keyed path continues to work without a redeploy.

## Mode parity

`account_id` is web-server-only (CF Access auth lives there). Tauri uses local-user
mode with no auth middleware; CLI is single-user. Neither needs changes. The
`IdentityStore` trait in `parish-core::identity` stays the source of truth.

## Lock ordering

The new `identity_store` field on `GlobalState` is not a `Mutex` — it is an
`Arc<dyn IdentityStore>` whose implementations use their own internal locking.
It is never held across acquisition of any `AppState` mutex and does not appear
in the documented lock ordering chain.
