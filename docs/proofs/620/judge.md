Verdict: sufficient
Technical debt: clear

PR implementing #620 adds admission control to `parish-server`.

Changes are confined to `parish-server` (middleware, session registry) and
`parish-config` (SessionConfig field). The architecture fitness test (which
forbids Axum deps from leaking into shared crates) still passes because no
Axum code was added to leaf crates.

Evidence:
- 3 unit tests covering `SessionRegistry::is_at_capacity` and `rejection_count`.
- 2 integration tests: unlimited mode passes all requests; cap=2 mode blocks
  the third new visitor (503 + Retry-After: 30) while allowing the returning
  visitor (existing cookie) through.
- All 208 parish-server tests pass; no regressions.
- Feature flag `admission-control` is default-on; kill-switch via
  `parish-flags.json` works correctly via `is_disabled`.
- `PARISH_MAX_SESSIONS` env var overrides the TOML/default cap at runtime.
- Documentation added to `docs/agent/architecture.md` under "Session capacity".
