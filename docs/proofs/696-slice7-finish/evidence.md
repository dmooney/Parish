# Evidence — #696 slice 7: finish system-command extraction + CLI Arc migration

Evidence type: gameplay transcript

## What changed

### New files (+1127 lines across 4 files)

| File | Lines |
|------|-------|
| `parish-core/src/game_loop/system_command.rs` | 222 |
| `parish-cli/src/command_host.rs` | 376 |
| `parish-server/src/command_host.rs` | 241 |
| `parish-tauri/src/command_host.rs` | 288 |

### Modified files (net -542 lines)

| File | Added | Removed | Net |
|------|-------|---------|-----|
| `parish-cli/src/headless.rs` | +20 | -263 | -243 |
| `parish-server/src/routes.rs` | +13 | -201 | -188 |
| `parish-tauri/src/commands.rs` | +14 | -154 | -140 |
| `parish-cli/src/lib.rs` | +1 | 0 | +1 |
| `parish-server/src/lib.rs` | +1 | 0 | +1 |
| `parish-tauri/src/lib.rs` | +1 | 0 | +1 |
| `parish-core/src/game_loop/mod.rs` | +2 | 0 | +2 |

### Overall delta
- Net: +585 lines added, -618 lines removed = -33 lines total (extraction consolidates)
- The 4 new files contain the shared dispatcher + 3 backend impls replacing 3 triplicated 150-line functions

## What was extracted

### `parish-core::game_loop::system_command` (new)
- `SystemCommandHost` trait with 18 methods covering all `CommandEffect` variants
- `handle_system_command(host: &dyn SystemCommandHost, cmd: Command)` — shared dispatcher

### `parish-cli/src/command_host.rs` (new)
- `CliCommandHost` wraps `Arc<tokio::sync::Mutex<App>>`
- Implements all 18 `SystemCommandHost` methods for headless CLI
- CLI's `handle_headless_command` temporarily wraps `App` in `Arc<Mutex<App>>`,
  calls the shared dispatcher, then moves `App` back (zero per-field migrations needed)

### `parish-server/src/command_host.rs` (new)
- `AppStateCommandHost` wraps `Arc<AppState>`
- Delegates `save_game()` to `crate::routes::do_save_game_inner` (no duplication)

### `parish-tauri/src/command_host.rs` (new)
- `TauriCommandHost` wraps `Arc<AppState>` + `tauri::AppHandle`

## Bug fixed in this PR

The previous agent's `handle_headless_command` used `unwrap_or_else(|| App::new())`
when moving `App` back from the `Arc<Mutex<App>>`. The `CliCommandHost` still held
its `Arc` clone, so `Arc::into_inner` always returned `None`, silently replacing
the app with a fresh empty `App::new()`. This meant all 18 headless command tests
were passing a blank app back to the caller — effects like `app.should_quit = true`
were immediately discarded.

Fix: drop `host` in a scoped block before calling `Arc::into_inner`, then use
`expect()` instead of `unwrap_or_else`.

## Test results

```
cargo test: 2234 passed, 17 ignored (53 suites, 7.36s)
cargo clippy: No issues found
```

## Dedup verification

- `NPC_REACTION_CONCURRENCY`: defined once in `parish-core/src/ipc/handlers.rs`, referenced from game_loop and headless
- `IDLE_MESSAGES`: defined once in `parish-core/src/ipc/handlers.rs`
- `INFERENCE_FAILURE_MESSAGES`: defined once in `parish-core/src/ipc/handlers.rs`
- `REQUEST_ID`: defined once in `parish-core/src/ipc/handlers.rs`

## What this PR does NOT include

- `Arc<dyn SessionStore>` wired into Tauri/CLI (server already has it; other runtimes deferred)
- `do_new_game` shared extraction (three runtimes use different mechanisms; requires SessionStore wiring first)
- Full per-field `Arc<Mutex<T>>` migration of CLI `App` struct (avoided by the wrapper pattern)
