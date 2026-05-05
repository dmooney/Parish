Verdict: sufficient
Technical debt: clear

PR #622 introduces the `ModSource` trait in `parish-core/src/mod_source.rs`.
No behavior change: the trait wraps the existing `discover_mods_in` /
`GameMod::load` call pair one-for-one. `LocalDiskModSource` is the only
concrete implementation.

All three entry points (Tauri, web server, headless CLI) are wired through the
trait. Architecture-fitness test passes confirming `parish-core` remains
backend-agnostic. 313 tests pass including 4 new `mod_source` unit tests.
No clippy warnings, no fmt drift, no placeholder debt markers.
