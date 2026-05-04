Verdict: sufficient
Technical debt: clear

PR is the first slice of #696 — pure structural refactor, no behavior changes.
Evidence documents that 309 parish-core tests pass (including architecture
fitness and wiring-parity gates), clippy is clean, and all three backends
compile. The moved constants, structs, and payload re-exports are byte-for-byte
equivalent to their originals; callers use re-exports from parish-core without
modification. The EventEmitter trait stub is additive-only (no callers yet).
No placeholder debt markers present in changed files.
