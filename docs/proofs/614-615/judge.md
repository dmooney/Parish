Verdict: sufficient
Technical debt: clear

This PR is a pure trait-seam refactor (issues #614, #615). No gameplay
behaviour was added or changed.

Two new backend-agnostic traits land in `parish-core`:

- `session_store::SessionStore` — abstract per-session persistence (#614).
- `identity::IdentityStore` + `identity::SessionRegistry` — split OAuth
  account management from session bookkeeping (#615).

Default SQLite implementations (`DbSessionStore`, `SqliteIdentityStore`,
`SqliteSessionRegistry`) live in `parish-server/src/session_store_impl.rs`
and back the same `sessions.db` schema; no migration is required.

Evidence: 309 parish-core tests pass (including the architecture fitness gate),
209 parish-server tests pass (including new round-trip tests for all three
implementations and an in-memory backend proving the trait is genuinely
backend-agnostic).

`GlobalState.sessions` intentionally keeps the concrete `SessionRegistry`
struct; migrating it to `Arc<dyn SessionRegistry>` is deferred to follow-up
PRs.  The PR body records this decision explicitly.
