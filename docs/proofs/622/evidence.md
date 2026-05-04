Evidence type: gameplay transcript
Date: 2026-05-03
Branch: feat/622-modsource-trait

# Proof Evidence — PR #622: ModSource trait

## Requirement

Introduce a `ModSource` trait in `parish-core` that abstracts mod-content loading
from the local disk so future S3/HTTP implementations drop in without touching
call sites. Provide `LocalDiskModSource` as the sole concrete implementation.
Wire the trait through all three entry points (Tauri, web server, headless CLI).
No behavior change.

## Build

Command:

```sh
cargo build -p parish-core
cargo build -p parish-server
cargo build -p parish
```

Result: all three builds completed with exit code 0. No errors or warnings.

## cargo fmt

Command:

```sh
cargo fmt --check
```

Result: exit code 0. All files pass formatting check.

## cargo clippy

Command:

```sh
cargo clippy --all-targets
```

Result: `cargo clippy: No issues found`. Exit code 0.

## parish-core tests

Command:

```sh
cargo test -p parish-core
```

Result:

```
cargo test: 313 passed, 5 ignored (6 suites, 5.33s)
```

313 tests pass including:
- architecture_fitness (no backend-agnostic crate has runtime deps)
- wiring_parity
- mod_source (4 new tests: list_mods_returns_setting_first,
  load_mod_returns_error_for_unknown_id, list_mods_no_mods_root_returns_error,
  mod_source_is_dyn_compatible)
- all pre-existing game_mod tests (37 tests covering GameMod::load, discover_mods, etc.)

## Behavior verification

The `ModSource` trait wraps the existing `discover_mods_in` / `GameMod::load`
functions one-for-one. The three entry points use:

- `parish-server`: async `load_setting_mod_via_source()` which calls
  `LocalDiskModSource::new().list_mods().await.load_mod().await`
- `parish-tauri` (sync): `load_setting_mod_sync()` which calls
  `LocalDiskModSource::new()`, `discover_mods_in()`, `GameMod::load()` directly
- `parish-cli`: `load_setting_mod_sync()` for auto-detect; explicit `--game-mod`
  path still calls `GameMod::load()` directly, bypassing discovery

All failure paths (no mods directory, multiple setting mods, malformed mod.toml)
return `None` / `Option<GameMod>` with tracing::warn, identical to the pre-trait
code path.
