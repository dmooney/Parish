Verdict: sufficient
Technical debt: clear

PR implementing #619 adds `Idempotency-Key` replay to `parish-server`.

Changes are confined to `parish-server` (middleware, session types) and
`docs/agent/` (architecture doc).  No Axum or server-specific code leaked
into leaf crates; the architecture fitness test continues to pass.

Evidence:

- 6 idempotency-specific tests covering the happy path (cache hit, handler
  executes once), distinct keys, missing key, TTL expiry, and per-session
  scoping.  All pass.
- Additional unit coverage for `IdempotencyKey` parsing and
  `CachedResponse` serialisation round-trip.
- 228 parish-server tests pass in total; no regressions from the merge of
  PRs #892 (admission-control) and #895 (NPC orchestration).
- Feature flag `idempotency-key` is default-on; kill-switch via
  `parish-flags.json` is verified by the flag-disabled branch in
  `idempotency_middleware`.
- `GlobalState::idempotency_cache` coexists correctly with
  `GlobalState::max_concurrent_sessions` (#892) — both fields are present
  and initialised in every `GlobalState` literal across the codebase.
