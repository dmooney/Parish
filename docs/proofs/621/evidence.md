Evidence type: gameplay transcript
Date: 2026-05-03
Branch: feat/621-otel-tracing-request-id

# Proof Evidence — #621: OTel tracing and request-ID middleware

## Acceptance criteria

1. OTel exporter gated behind `PARISH_OTEL_ENDPOINT` env var — no-op locally.
2. Request-ID middleware mints UUID per inbound HTTP request, attaches to extensions, echoes in `X-Request-Id` response header.
3. Standardized span fields documented in `docs/agent/tracing.md`.
4. Per-request metrics emitted via tracing events.
5. Feature-flag gate: `config.flags.is_disabled("otel-tracing")`, default-on.

## Dependency version conflict resolution

`cargo tree -i opentelemetry` showed two copies:
- `opentelemetry v0.28.0` pulled by `tracing-opentelemetry v0.29.0`
- `opentelemetry v0.29.1` pulled by `opentelemetry-otlp v0.29.0`

Fix: upgraded `tracing-opentelemetry` from `"0.29"` to `"0.30"` in `parish/Cargo.toml`.
`tracing-opentelemetry 0.30.x` requires `opentelemetry ^0.29.0`, matching `opentelemetry-otlp 0.29.x`.

After fix:

```
cargo tree -i opentelemetry
opentelemetry v0.29.1
├── opentelemetry-http v0.29.0
│   └── opentelemetry-otlp v0.29.0
├── opentelemetry_sdk v0.29.0
│   ├── opentelemetry-otlp v0.29.0
│   └── tracing-opentelemetry v0.30.0
└── [parish-server, parish-cli]
```

Single version. No more duplicate-crate trait-object errors.

## cargo check

```
cargo check -p parish -p parish-server
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

Exit code 0. No errors, no warnings.

## Test results

```
cargo test -p parish-server -p parish
cargo test: 503 passed, 2 ignored (16 suites)
```

All 503 tests pass. This includes:
- `tracing_setup` unit tests (env var gate, empty endpoint → None)
- `middleware` unit tests (cookie extraction, session key stability)
- `isolation` integration tests (now updated to pass `session_id` arg)
- `new_game_parity` integration tests (updated likewise)
- `security_headers`, `auth_guard`, `legal_routes` integration tests
- Full `parish-cli` harness and eval suites

## Feature flag verification

`middleware.rs:83`:
```rust
if global.template_config.flags.is_disabled("otel-tracing") {
    return next.run(req).await;
}
```

Uses `is_disabled` (not `is_enabled`) — default-on per CLAUDE.md rule #6.

## OTel env-var gate verification

`tracing_setup.rs::try_build_otel_provider`:
- Returns `None` when `PARISH_OTEL_ENDPOINT` is unset.
- Returns `None` when set to empty string.
- Returns `Some(SdkTracerProvider)` when a non-empty URL is provided.

Test `try_build_otel_provider_returns_none_when_endpoint_unset` and
`try_build_otel_provider_returns_none_for_empty_endpoint` both pass.

## X-Request-Id header echo

`middleware.rs` `request_id_layer`:
- UUID v4 generated per request via `uuid::Uuid::new_v4()`.
- Injected into request extensions as `RequestId`.
- Echoed in `X-Request-Id` response header via `HeaderValue::from_str`.

## Span fields

All standard fields listed in `docs/agent/tracing.md` are set by `request_id_layer`:
`request_id`, `route`, `method`, `status`, `latency_ms`, plus deferred fields
`session_id`, `account_id`, `model` recorded by downstream middleware/handlers.
