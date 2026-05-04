# Judge verdict: #696 second slice

## Scope assessment

The PR extracts two of the ten orchestration functions listed in the issue
(`handle_npc_conversation` and `run_idle_banter`) into `parish_core::game_loop`,
parameterised by `EventEmitter`. The remaining eight functions are deferred.

This is a partial scope delivery, which the evidence.md acknowledges with a clear
scope note. The deferred functions have documented reasons (headless App migration
required, save-path metadata coupling). The partial scope does not constitute a
quality defect.

## Code quality

- New module `parish_core::game_loop` is clean, well-documented, and backend-agnostic.
- `GameLoopContext<'a>` uses references to existing Mutex fields; no restructuring
  of AppState was required. This is the correct design for this refactor stage.
- `AppStateEmitter`, `TauriEmitter`, `StdoutEmitter` are thin, single-purpose wrappers.
- Legacy function bodies in routes.rs and commands.rs are preserved with `#[allow(dead_code)]`
  for review continuity. These should be removed in a follow-up.
- Architecture fitness test passes: `tokio-util` addition is safe.

## Test coverage

- 4 new unit tests cover the main guard clauses (no NPC, empty input, no LLM, cross-mode equivalence).
- Cross-mode equivalence test directly addresses issue #734.
- All existing server tests (212) continue to pass, proving the delegation does not regress behavior.

## Behavioral impact

Two minor behavioral changes are documented:
1. Tauri's `run_npc_turn` now suppresses error UI for autonomous turns (aligning with server).
2. Idle banter no longer shows loading animation in Tauri (was an unintentional difference).

Both changes are improvements. Neither breaks a documented user spec.

## Security parity

The react_to_message validation gap mentioned in the issue was already addressed
in a prior PR. No security regression.

## What's missing vs issue spec

- Only 2 of 10 listed functions are extracted (the two longest/most duplicated).
- No `do_save_game`/`do_new_game` extraction.
- No full `emit_npc_reactions` extraction (documented reason: needs Arc ownership in background task).
- Headless CLI remains on inline implementation.

These are deferred, not forgotten. The PR does what it sets out to do for its stated scope.

Verdict: sufficient

The shared orchestration module is correct, well-tested, architecture-fitness-compliant,
and demonstrates the design that subsequent slices can follow. The deferred scope is
clearly documented with actionable next steps.

Technical debt: clear

The `#[allow(dead_code)]` legacy functions in routes.rs and commands.rs are the
only new debt introduced; they should be cleaned up in a follow-up once the PR
is reviewed and any additional tests pass.
