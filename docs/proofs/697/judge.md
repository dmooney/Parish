Verdict: sufficient
Technical debt: clear

This PR is a pure structural refactor of `NpcManager` (issue #697). No
gameplay behavior was added or changed — the same logic was moved from a
2,633-line god object into focused modules:

- `schedule.rs` — schedule resolution + tests
- `tier_assign.rs` — tier assignment + BFS + tests
- `banshee.rs` augmented — banshee tick loop + tests
- `tier4.rs` augmented — Tier 4 event application + tests
- `manager.rs` — coordinator with thin wrappers only

Evidence: 392 unit/integration tests pass, harness walkthrough exits 0 with
correct JSON output, architecture fitness gate passes (no orphaned files,
no runtime-dep leakage). The refactor reduced `manager.rs` from 2,633 to
1,082 lines. No gameplay regressions detected.
