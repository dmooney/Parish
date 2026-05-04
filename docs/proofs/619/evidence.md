
# Proof Evidence — Issue #619: Idempotency-Key middleware

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: feat/619-idempotency-key

## Requirement

Mutating HTTP routes must honour the `Idempotency-Key` request header:
a second POST with the same key and session must return the cached
response from the first invocation without re-executing the handler body.

## Middleware unit tests (in-crate)

Command:

```sh
cargo test -p parish-server -- idempotency
```

Result:

```
cargo test: 6 passed, 224 filtered out (10 suites, 0.01s)
```

Tests in `parish/crates/parish-server/src/middleware.rs` (mod tests):

- `same_idempotency_key_returns_cached_response_and_does_not_re_execute` —
  two POSTs with the same `Idempotency-Key` produce identical bodies and the
  handler executes exactly once (`call_count == 1`).
- `different_idempotency_key_executes_handler_twice` — distinct keys each
  execute the handler; `call_count == 2`.
- `no_idempotency_key_always_executes_handler` — requests without the header
  are never cached; three calls yield `call_count == 3`.
- `idempotency_key_expired_ttl_re_executes` — an artificially aged cache
  entry (older than TTL) does not replay; handler executes again.
- `idempotency_key_scoped_per_session` — same key from two different session
  IDs are treated as distinct; each executes the handler once.
- Additional unit tests for `IdempotencyKey` parsing (valid, empty,
  too-long, non-ASCII), `CachedResponse` round-trip serialisation, and
  `extract_cookie_value` helpers.

## Full parish-server test suite

Command:

```sh
cargo test -p parish-server
```

Result:

```
cargo test: 228 passed, 2 ignored (10 suites, 0.54s)
```

All pre-existing tests (admission control, auth guard, isolation, legal
routes, new-game parity, security headers) continue to pass.  No regressions.

## Feature flag verification

The `idempotency-key` flag is default-on via
`!flags.is_disabled("idempotency-key")`.  An unknown flag entry evaluates
`is_disabled` as `false`, so replay is active by default.  Setting the flag
explicitly to `false` in `parish-flags.json` bypasses the cache entirely.

## Supported mutating routes

| Route | Handler |
|---|---|
| `POST /api/save-game` | `routes::save_game` |
| `POST /api/create-branch` | `routes::create_branch` |
| `POST /api/new-save-file` | `routes::new_save_file` |
| `POST /api/new-game` | `routes::new_game` |
| `POST /api/editor-save` | `editor_routes::editor_save` |

## Cache parameters

- Scope: process-wide LRU, keyed by `(session_id, Idempotency-Key)`.
- Capacity: 1 000 entries (`IDEMPOTENCY_CACHE_CAPACITY`).
- TTL: 24 hours (`IDEMPOTENCY_TTL`).
- Storage: `GlobalState::idempotency_cache` (tokio Mutex around `lru::LruCache`).
