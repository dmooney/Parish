# Proof Evidence — PR #897: improve ollama setup loading visibility

Evidence type: gameplay transcript
Date: 2026-05-04
Branch: claude/move-setup-to-gui-Z9Jvv

## Requirement

The Ollama first-run path could sit on opaque setup phases for a long time.
The player needed visible progress and entertaining status messages during
model downloads. This PR:

- Reworks the Tauri setup overlay with game title, themed activity log,
  color-cycling spinner, progress bar, percent display, download stats
- Adds frontend setup snapshot recovery for in-flight activity
- Adds streamed Ollama pull progress with aggregate artifact totals
- Documents force-redownload setting

## just check

Command:

```sh
just check
```

Result: All Rust tests pass (cargo test, cargo clippy, cargo fmt).
Architecture fitness and wiring parity gates are green.

## Frontend tests

Command:

```sh
npm exec vitest -- run src/components/SetupOverlay.test.ts
```

Result: SetupOverlay component tests pass including snapshot recovery,
progress bar updates, and error state rendering.

## cargo test (parish-tauri)

Command:

```sh
cargo test -p parish-tauri
```

Result: All parish-tauri tests pass including setup-status snapshot,
get_setup_snapshot command, and TauriProgress trait impl tests.

