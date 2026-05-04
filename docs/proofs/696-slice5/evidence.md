# Proof: #696 slice 5 — emit_npc_reactions extraction and cross-mode test

Evidence type: gameplay transcript

## Summary

Slice 5 of #696 extracts `emit_npc_reactions` from both `parish-server` and
`parish-tauri` into a shared `parish-core::game_loop::emit_npc_reactions`
function, and adds a cross-mode equivalence test (closes #734).

## What changed

### Files modified

| File | Before (main) | After (slice 5) | Delta |
|------|:---:|:---:|:---:|
| `parish/crates/parish-server/src/routes.rs` | 3469 | 3001 | **-468** |
| `parish/crates/parish-tauri/src/commands.rs` | 2998 | 2393 | **-605** |
| `parish/crates/parish-core/src/game_loop/reactions.rs` | 90 | 393 | +303 (new shared code + tests) |
| `parish/crates/parish-core/src/game_loop/mod.rs` | 69 | 118 | +49 (updated exports + docs) |

**Net line reduction across runtime files**: -1073 lines removed from server and Tauri
**New shared code**: +352 lines in parish-core (including tests and docs)

### New game_loop submodules / exports added in slice 5

- `parish_core::game_loop::emit_npc_reactions` — shared background-task reaction
  function accepting pre-captured NPC list, client, feature flag, emitter, and
  persist callback
- `parish_core::game_loop::PersistReactionFn` — type alias for the persist
  callback (`Arc<dyn Fn(String, String, String) + Send + Sync + 'static>`)

### What was NOT extracted (and why)

- **`rebuild_inference`** — depends on per-runtime `AppState` fields
  (`worker_handle`, `inference_log`, `inference_client`). A shared version
  requires a new `InferenceManager` trait; deferred.
- **`handle_system_command`** — requires runtime-specific handles (`app.exit(0)`,
  `event_bus`, `stdout`). Cannot be unified through `EventEmitter` alone without
  a richer side-effect protocol; deferred.
- **`do_save_game` / `do_new_game`** — `SessionStore` trait exists but is not
  wired to CLI or Tauri persistence paths; deferred.
- **`handle_movement` / `handle_game_input`** — already delegate to
  `parish_core::game_session::apply_movement`; remaining per-runtime code handles
  travel-encounter LLM enrichment with differing lock patterns; deferred.
- **CLI `emit_headless_npc_reactions`** — CLI `App` struct uses bare (non-Mutex)
  fields; the shared function needs `Arc<Mutex<T>>` semantics for write-back
  through the `persist` callback. Full CLI migration is deferred.

### Approach: why `persist: PersistReactionFn` instead of `Arc<Mutex<NpcManager>>`

Both `parish-server` and `parish-tauri` store `NpcManager` as `Mutex<NpcManager>`
inside `Arc<AppState>` — not individually arc-wrapped. This means neither runtime
can pass `Arc<Mutex<NpcManager>>` to the shared function without either:
1. Restructuring both AppState types (a ~2000-line change, wider than this slice), or
2. Cloning NpcManager (not possible; NpcManager is not Clone).

The `persist: Arc<dyn Fn(...)>` callback pattern avoids this: each runtime
closes over its own `Arc<AppState>` and performs the write inside a spawned task.
The shared function remains `Arc<AppState>`-free and backend-agnostic.

### CLI: why not migrated

The task description says the CLI migration to `Arc<Mutex<T>>` would touch
"hundreds of call sites throughout the CLI codebase." With 2293 lines in
`headless.rs` and a flat `App` struct with 50+ direct field accesses, the
migration is a dedicated slice (estimated 1 PR of the same size as this one).
The CLI's `emit_headless_npc_reactions` remains functionally identical to
the shared implementation and is explicitly documented as deferred.

## Cross-mode equivalence test

Test name: `game_loop::reactions::tests::cross_mode_equivalence_event_structure_is_correct`

Located in: `parish/crates/parish-core/src/game_loop/reactions.rs`

The test:
1. Drives `emit_npc_reactions` through a `RecordingEmitter` (mimicking all three
   runtime patterns — server broadcast, Tauri app.emit, CLI println).
2. Asserts that every emitted event has the correct name (`"npc-reaction"`) and
   payload structure (`message_id`, `emoji`, `source`).
3. Runs three parallel emitter instances to confirm structural parity.

## Build verification

```
cargo clippy --workspace --all-targets -- -D warnings  → 0 errors, 0 warnings
cargo build --workspace --all-targets                  → success
cargo test --package parish-core --test architecture_fitness → 3 passed
cargo test --package parish-core --lib game_loop::reactions  → 9 passed
cargo test --package parish-server --lib routes::tests::emit_npc_reactions → 2 passed
```
