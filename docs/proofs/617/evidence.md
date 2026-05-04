# Proof Evidence — PR #617: InferenceClient trait, LRU cache, cost metrics

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/617-inference-client

## Requirement

Implement `InferenceClient` trait, request/response envelope, LRU response
cache, and structured cost metrics in `parish-inference`.

## What was implemented

1. **`InferenceClient` trait** (`parish/crates/parish-inference/src/inference_client.rs`):
   - `async fn complete(&self, req: ClientInferenceRequest) -> Result<ClientInferenceResponse, ParishError>`
   - `ClientInferenceRequest` envelope with `request_id: Uuid`, `session_id`, `account_id`, `model`, `prompt_hash`, `priority`, `params: InferenceParams`, `messages: Vec<Message>`
   - `InferenceParams` with `Hash + Eq` via IEEE 754 bit quantization for `f32`

2. **LRU response cache** (`CachingInferenceClient`):
   - Keyed by `(prompt_hash, model, params)`
   - Default capacity 500 entries (overridable via `PARISH_INFERENCE_CACHE_CAPACITY`)
   - Cache hit flagged on `ClientInferenceResponse.cache_hit`
   - Cache-disabled path: `build_inference_client_stack(..., false, _)` skips wrapper

3. **Cost metrics** (`MeteredInferenceClient`):
   - `tracing::info!` with target `parish_inference::metrics`
   - Fields: `request_id`, `session_id`, `account_id`, `model`, `latency_ms`, `cache_hit`, `tokens_in`, `tokens_out`
   - Matches PR #888 standardized span fields

4. **`AnyClientAdapter`**: wraps existing `AnyClient` to implement the new trait.

5. **`AppState` wiring**: `inference_client: Option<Arc<dyn InferenceClient>>` added to `parish-server/src/state.rs`, constructed via `build_inference_client_stack`.

## parish-inference tests

Command:

```sh
cargo test -p parish-inference
```

Result:

```
running 212 tests
test result: ok. 205 passed; 0 failed; 7 ignored

running 31 tests
test result: ok. 31 passed; 0 failed; 0 ignored

running 0 tests (doc-tests)
test result: ok. 0 passed; 0 failed; 0 ignored
```

New tests:
- `mock_client_complete_returns_response` — trait conformance
- `caching_client_returns_cached_on_second_call` — cache hit/miss
- `caching_client_misses_on_different_model` — cache key sensitivity
- `caching_client_misses_on_different_prompt_hash` — cache key sensitivity
- `caching_client_misses_on_different_params` — cache key sensitivity
- `metered_client_emits_tracing_event_on_success` — metrics emission captured via `tracing_subscriber`
- `any_client_adapter_implements_inference_client` — AnyClient adaptor
- `build_stack_cache_enabled` / `build_stack_cache_disabled` — factory helpers
- `inference_params_roundtrip`, `inference_params_none`, `inference_params_hash_eq`
- `hash_messages_deterministic`, `hash_messages_differs_on_content_change`

## parish-server tests

```
running 160 tests
test result: ok. 160 passed; 0 failed; 0 ignored

(plus 5 additional suites: 7, 26, 5, 2, 12 passed; 0 failed)
```

## parish-core tests (architecture fitness + wiring parity)

```
running 290 tests (unit)
test result: ok. 289 passed; 0 failed; 1 ignored

running 3 tests (architecture fitness)
test result: ok. 3 passed; 0 failed; 0 ignored

running 6 tests (integration)
test result: ok. 6 passed; 0 failed; 0 ignored

running 14 tests (wiring parity / misc)
test result: ok. 14 passed; 0 failed; 0 ignored
```

Architecture fitness confirms `parish-inference` has no axum/tauri deps.
Mode parity holds: `AnyClientAdapter` wraps the same `AnyClient` used by
Tauri, CLI, and web server.

## Feature flags

- `inference-client-trait`: default-on per CLAUDE.md §6
- `inference-response-cache`: default-on; `build_inference_client_stack(_, false, _)` is the off path
- `PARISH_INFERENCE_CACHE_CAPACITY`: env var override, default 500

## No placeholder debt markers

`agent-check.sh` debt scan: no `todo!`, `unimplemented!`, or placeholder
markers introduced.
