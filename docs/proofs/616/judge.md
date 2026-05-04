Verdict: sufficient
Technical debt: clear

PR #616 replaces the inline `EventBus` struct in `parish-server` with a
`BroadcastEventBus` that implements the new `EventBus` trait defined in
`parish-core::event_bus`. All 13 wire event names are mapped to `Topic`
variants. The wire format (`ServerEvent { event: String, payload: Value }`)
is unchanged, so no frontend changes are needed.

Evidence: 514 tests pass across `parish-core` and `parish-server` (including
architecture fitness, wiring parity, and all server integration tests). No
axum/tower imports introduced into any backend-agnostic crate.
