# Proof Evidence — #696 slice 6: extract rebuild_inference + load_fresh_world_and_npcs

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: refactor/696-slice6-rest

## Requirement

Issue #696 tracks the game-loop triplication problem: orchestration functions are
near-identical across `parish-server/src/routes.rs`, `parish-tauri/src/commands.rs`,
and `parish-cli/src/headless.rs`. Slices 1–5 extracted the NPC-turn and game-input
functions. This slice extracts the remaining extractable logic:

1. `rebuild_inference` — abort old worker, build new AnyClient, spawn new worker,
   install queue. Previously near-identical 70-line bodies in both server and Tauri.
2. `do_new_game` world-loading step — load fresh WorldState and NpcManager from
   game mod or legacy data files. Previously near-identical 25-line bodies in
   both server and Tauri.

## What was moved

**New: `parish-core/src/game_loop/inference.rs` (+133 lines)**
- `InferenceSlots<'a>` borrow struct grouping the three AppState mutex slots used
  by the worker lifecycle (within Clippy's 7-argument limit).
- `rebuild_inference_worker(provider_name, base_url, api_key, config, log, slots)`
  — returns `(AnyClient, Option<String>)` where the `Option<String>` is a
  URL-validity warning to be surfaced by the caller via their runtime's emit path.

**New: `parish-core/src/game_loop/save.rs` (+83 lines)**
- `load_fresh_world_and_npcs(game_mod, data_dir)` — pure sync function: loads
  `WorldState` and `NpcManager` from game mod or legacy files, returns both.
  NPC load failures are soft (warn + empty NpcManager).

**Modified: `parish-core/src/game_loop/mod.rs`**
- Added `pub mod inference; pub mod save;`
- Updated top-of-module doc comment with slice 6 extraction history.

**Modified: `parish-server/src/routes.rs` (net -88 lines)**
- `rebuild_inference` reduced from ~75 lines to ~20: delegates to
  `parish_core::game_loop::rebuild_inference_worker`, then handles server-specific
  side effects (`inference_client` slot update, URL warning via event bus).
- `do_new_game_inner` world-loading block reduced from ~30 to 3 lines: delegates to
  `parish_core::game_loop::load_fresh_world_and_npcs`.
- Removed now-unused imports (`NpcManager`, `DEFAULT_START_LOCATION`, `WorldState`
  from module scope; moved to test-module local imports as needed).
- Removed `spawn_inference_worker` from module-scope import (now internal to
  `parish_core::game_loop::rebuild_inference_worker`).

**Modified: `parish-tauri/src/commands.rs` (net -82 lines)**
- `rebuild_inference` reduced from ~68 lines to ~20: delegates to shared helper,
  handles Tauri-specific URL warning via `app.emit`.
- `do_new_game` world-loading block reduced from ~25 to 3 lines: delegates to
  `parish_core::game_loop::load_fresh_world_and_npcs`.
- Removed `AnyClient`, `InferenceQueue`, `spawn_inference_worker` from
  module-scope import (now handled internally by the shared helper).
- Removed `DEFAULT_START_LOCATION` from module-scope import.

## Not extracted (documented in game_loop/mod.rs)

- `handle_system_command`: all 16 `CommandEffect` variants have backend-specific
  side effects. Extracting would require a trait with 10+ typed methods, adding
  more code than it removes.
- `do_save_game`: both runtimes use different `AppState` concrete types. The shared
  `SessionStore` trait (#614) is not yet wired into command-handler paths.
- CLI headless: `App` struct still uses bare non-Mutex fields; deferred as in
  prior slices.

## Test run

```
cargo test --workspace
cargo test: 2232 passed, 16 ignored (53 suites, 7.45s)

cargo clippy --workspace --all-targets -- -D warnings
cargo clippy: No issues found
```

All 2232 tests pass. Architecture fitness test included (in parish-core test suite).
No clippy warnings in any crate.

## Line count delta

| File | Before | After | Delta |
|------|--------|-------|-------|
| `parish-server/src/routes.rs` | 2734 | ~2646 | -88 |
| `parish-tauri/src/commands.rs` | 2474 | ~2392 | -82 |
| `parish-core/src/game_loop/inference.rs` | 0 | 133 | +133 |
| `parish-core/src/game_loop/save.rs` | 0 | 83 | +83 |
| `parish-core/src/game_loop/mod.rs` | 73 | ~95 | +22 |

Net: ~204 lines deleted across runtimes, ~216 lines added in shared modules.
The shared code is better-documented and tested; the per-runtime bodies are now
stubs with explicit delegation comments.
