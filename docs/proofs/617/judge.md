Verdict: sufficient
Technical debt: clear

PR #617 introduces the `InferenceClient` async trait, `ClientInferenceRequest`
envelope, `CachingInferenceClient` LRU decorator (default 500 entries),
and `MeteredInferenceClient` cost-metrics decorator in `parish-inference`.

Evidence:
- 205/205 new + existing `parish-inference` unit tests pass (cache hit/miss,
  metrics emission captured via `tracing_subscriber`, trait conformance for
  `AnyClientAdapter` and `MockClient`, `InferenceParams` hash/eq).
- 160+ `parish-server` tests pass including the `AppState` constructor test.
- `parish-core` architecture fitness (3 tests) confirms no axum/tauri deps
  leaked into `parish-inference`; wiring-parity suite (14 tests) passes.
- No `todo!`, `unimplemented!`, or placeholder markers in changed files.
- Streaming stays on `AnyClient`; only non-streaming completions go through
  the trait, making the cache exclusion structural rather than runtime-gated.
- `inference_client` field on `AppState` is `Option<Arc<dyn InferenceClient>>`,
  pluggable for a future Redis backend without touching handler code.
