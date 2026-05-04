# Proof Evidence — #696 first slice: shared constants, struct dedup, EventEmitter trait stub

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/696-shared-constants-emitter-trait

## Requirement

Issue #696 tracks the game-loop triplication problem: `handle_system_command`,
`handle_game_input`, and related helpers are near-identical copies in
`parish-tauri/src/lib.rs` (then commands.rs), `parish-server/src/routes.rs`,
and `parish-cli/src/headless.rs`. This first slice extracts the lowest-hanging
fruit — constants, shared state structs, IPC payload re-exports, and the
`EventEmitter` trait stub — without touching the large game-loop functions that
are out of scope for this slice.

## What was moved

- **Constants** moved to `parish-core::ipc::handlers`:
  - `INFERENCE_FAILURE_MESSAGES` — Irish-themed fallback messages for failed NPC inference
  - `IDLE_MESSAGES` — atmospheric messages when no NPC is present
  - `NPC_REACTION_CONCURRENCY` — max concurrent NPC LLM calls (was `4` in both backends)
  - `REQUEST_ID` — `AtomicU64` with `SeqCst` ordering (shared process-wide counter)

- **Structs** moved to `parish-core::ipc::state`:
  - `ConversationRuntimeState` — local conversation transcript + inactivity tracking
  - `SaveState` — current save file metadata for status bar display
  - `UiConfigSnapshot` — UI config snapshot sent to frontend on boot

- **IPC payload re-exports** in `parish-tauri/src/events.rs` — now re-export
  `StreamTokenPayload`, `StreamTurnEndPayload`, `StreamEndPayload`,
  `TextLogPayload`, `NpcReactionPayload`, `LoadingPayload` from `parish-core`
  instead of duplicating them locally.

- **`EventEmitter` trait stub** added to `parish-core::ipc::event_emitter`:
  - Object-safe trait (`Send + Sync`, no generic methods)
  - Single method: `fn emit_event(&self, name: &str, payload: serde_json::Value)`
  - Not yet implemented by any backend (next slice)

- **`TextLogPayload` struct literal fixes** in `parish-tauri/src/commands.rs`:
  - 13 struct literals were missing `subtype: None` after the move unified the
    struct definition (the Tauri-local version lacked `subtype`; parish-core
    has it). Added `subtype: None` to each affected literal.

## cargo test -p parish-core

```
Running unittests src/lib.rs
Running tests/architecture_fitness.rs
Running tests/async_llm_integration.rs
Running tests/mod_artefact_malformed_input.rs
Running tests/wiring_parity.rs
Doc-tests parish_core
cargo test: 309 passed, 4 ignored (6 suites, 5.30s)
```

Architecture fitness test passes. All 309 tests pass.

## cargo clippy --workspace --all-targets -- -D warnings

```
cargo clippy: No issues found
```

## cargo fmt --check

```
(no output — all files formatted cleanly)
```

## cargo build --workspace

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.51s
```

All three binaries (`parish-cli`, `parish-server`, `parish-tauri`) compile cleanly.
