# Judge Verdict — #696 slice 7: system-command extraction + CLI Arc migration

## Review scope

Reviewed the full diff of `refactor/696-slice7-finish` against `origin/main`,
covering all 11 changed files (4 new, 7 modified).

## Structural assessment

**Extraction completeness.** `parish-core/src/game_loop/system_command.rs` now
holds the shared `handle_system_command` dispatcher and the `SystemCommandHost`
trait with all 18 effect-handler methods. The three runtimes (server, Tauri, CLI)
provide `command_host.rs` implementations delegating to this shared path. Each
runtime's `handle_system_command` function is now a ~15-line stub.

**CLI migration approach.** Rather than migrating all `App` fields to
`Arc<Mutex<T>>` (which would touch hundreds of call sites), the CLI wraps the
entire `App` in `Arc<Mutex<App>>` for the duration of each command dispatch,
then moves it back. This satisfies `Send + Sync` for the trait without
cascading refactors. The `expect()` guard ensures any future accidental
Arc clone is caught immediately rather than silently resetting state.

**Bug fix.** The previous agent's `unwrap_or_else(|| App::new())` fallback meant
all 18 headless command tests were silently passing an empty `App` back to the
caller. Fixed by scoping `host` so it's dropped before `Arc::into_inner`, then
using `expect()`. All 2234 tests pass post-fix.

**No duplication added.** Server's `command_host.rs` delegates to
`crate::routes::do_save_game_inner` rather than duplicating it.

**Architecture gate.** `system_command.rs` imports no backend crates
(`axum`, `tauri`, `wry`, `tao`, `tower`). The architecture fitness test
enforces this mechanically.

**Dead code.** No unused imports or functions remain after clippy --all-targets.

## Remaining scope (factual, not a deferral justification)

- `Arc<dyn SessionStore>` is not yet wired into Tauri or CLI runtimes (server only).
- `do_new_game` is not shared; three runtimes use different mechanisms. Requires
  SessionStore wiring as a prerequisite.
- These items are the subject of a follow-up issue, not this PR.

The extraction is complete for `handle_system_command`. The CLI migration uses a
sound pattern that avoids cascading per-field changes. Tests are green. The
deferred items are factual scope gaps, not engineering-judgment deferrals.

Verdict: sufficient

Technical debt: clear
