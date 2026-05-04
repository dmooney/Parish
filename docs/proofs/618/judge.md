Verdict: sufficient
Technical debt: clear

PR #618 adds a stable `account_id: Uuid` field to `AuthContext` and threads it through
all server-side keying points that previously used email strings.

All seven acceptance criteria are met:

1. `AuthContext` gains `account_id: Uuid` — populated by `cf_access_guard` via
   `resolve_account_id` which calls `IdentityStore::lookup_by_provider` / `create_account`.
2. First-login mints a UUID; subsequent logins return the same UUID from the
   `oauth_accounts` table — verified by `auth_flow_produces_valid_auth_context_uuid`.
3. `account-id-keying` flag gates the logic via `flags.is_disabled(...)` — default-on.
4. Editor sessions keyed by `(account_id, "")` — verified by updated isolation tests.
5. `active_ws` uses `HashSet<Uuid>` — verified by `second_ws_upgrade_same_account_is_409`.
6. `cf_access_guard` records `account_id.to_string()` on the tracing span.
7. Three integration tests in `tests/account_id.rs` cover multi-tab, email-change,
   and auth-regression scenarios.

Test suite: 215 tests passed, 0 failures. No placeholder debt markers found.
