Verdict: sufficient
Technical debt: clear

PR #883 removes a duplicate `reqwest = { workspace = true }` key from
`parish-core/Cargo.toml`. No logic was changed — this is a one-line
manifest correction.

Evidence: `cargo metadata` resolves cleanly (0 errors), all 309
`parish-core` tests pass (architecture fitness, wiring parity, unit,
integration), no placeholder debt markers present.
