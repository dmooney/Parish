
# Proof Evidence — Issue #620: Admission control + per-process session budget

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: feat/620-admission-control

## Requirement

When the server reaches `max_concurrent_sessions`, new session creation must
be refused with `503 Service Unavailable` and a `Retry-After: 30` header.
Returning visitors (cookie present, session in memory or restorable from DB)
must always be admitted.

## Capacity check unit tests

Command:

```sh
cargo test -p parish-server -- tests::is_at_capacity tests::rejection_count tests::active_count
```

Result:

```
cargo test: 3 passed, 201 filtered out (8 suites, 0.00s)
```

Tests:
- `active_count_reflects_in_memory_sessions` — fresh registry reports 0.
- `is_at_capacity_returns_false_below_cap_true_at_cap` — below cap returns
  `false`, at/above cap returns `true` and increments `rejection_count`.
- `rejection_count_not_incremented_below_cap` — under-cap calls do not
  increment the counter.

## HTTP integration tests (cap=2)

Command:

```sh
cargo test -p parish-server --test admission_control
```

Result:

```
cargo test: 2 passed (1 suite, 0.02s)
```

Tests:
- `no_cap_allows_unlimited_sessions` — with `max_concurrent_sessions = None`,
  five consecutive new-visitor requests all succeed (no 503).
- `at_cap_new_visitor_gets_503_existing_visitor_allowed` — with cap=2:
  - Requests 1 and 2 succeed and set distinct `parish_sid` cookies.
  - `global.sessions.active_count() == 2` confirmed.
  - Request 3 (fresh visitor, no cookie) returns `503` with `Retry-After: 30`.
  - `rejection_count == 1` confirmed.
  - Request 4 (returning visitor with cookie from request 1) returns non-503.

## Full parish-server test suite

Command:

```sh
cargo test -p parish-server
```

Result:

```
cargo test: 208 passed (8 suites, 0.02s)
```

All pre-existing tests continue to pass. No regressions.

## Feature flag verification

The `admission-control` flag is default-on via `!is_disabled("admission-control")`.
A missing flag entry (empty `FeatureFlags`) evaluates `is_disabled` as `false`
(unknown = not disabled), so the cap is active by default.
Setting the flag explicitly to `false` in `parish-flags.json` sets
`max_concurrent_sessions = None`, disabling the check entirely.
