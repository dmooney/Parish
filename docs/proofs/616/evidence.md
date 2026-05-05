Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/616-eventbus-trait

## Requirement

Replace bare `tokio::sync::broadcast` event bus in `parish-server` with an
`EventBus` trait + topic-aware subscriptions so future Redis/NATS impls can
drop in without touching emission call sites.

## Changes

- `parish-core/src/event_bus.rs` — new module: `Topic` enum, `EventBus` trait,
  `EventStream`, `BroadcastEventBus` (wraps the existing broadcast channel).
- `parish-core/src/lib.rs` — `pub mod event_bus` added.
- `parish-server/src/state.rs` — `AppState.event_bus` field changed from
  old inline `EventBus` struct to `BroadcastEventBus`; old struct removed;
  `ServerEvent` and `Topic` re-exported from `parish_core::event_bus`.
- `parish-server/src/ws.rs` — `handle_socket` subscribes via new
  `EventBus::subscribe(&[])` (firehose) and receives via `EventStream::recv()`.
- `parish-server/src/routes.rs`, `session.rs`, `editor_routes.rs` — all
  `emit(name, payload)` call sites migrated to
  `emit_named(Topic::Variant, name, payload)`.

## Architecture fitness

Command:

```sh
cargo test -p parish-core
```

Result:

```
cargo test: 314 passed, 4 ignored (6 suites, 5.36s)
```

Architecture fitness (`backend_agnostic_crates_do_not_pull_runtime_deps`,
`parish_cli_does_not_duplicate_parish_core_modules`, `no_orphaned_source_files`)
all pass. `parish-core` depends on no axum/tower/tauri crates.

## Full test suite

Command:

```sh
cargo test -p parish-server -p parish-core
```

Result:

```
cargo test (parish-core): 314 passed, 4 ignored (6 suites, 5.36s)
cargo test (parish-server): 199 passed (7 suites, 0.51s)
```

513 tests pass. No regressions. Wire-format invariant preserved: `ServerEvent`
still carries `event: String` and `payload: Value`; topic is server-side only.

## Topic variants covered

All 13 wire event names are mapped:

| Wire name | Topic |
|---|---|
| `text-log` | `TextLog` |
| `world-update` | `WorldUpdate` |
| `stream-token` | `InferenceToken` |
| `stream-end` | `InferenceToken` |
| `stream-turn-end` | `InferenceToken` |
| `travel-start` | `TravelStart` |
| `loading` | `Loading` |
| `npc-reaction` | `NpcReaction` |
| `toggle-full-map` | `UiControl` |
| `open-designer` | `UiControl` |
| `save-picker` | `UiControl` |
| `theme-switch` | `UiControl` |
| `tiles-switch` | `UiControl` |

`ClockTick` reserved for future use (no current emitter).
