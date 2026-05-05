Verdict: sufficient
Technical debt: clear

Slice 6 of #696 extracts `rebuild_inference` and `do_new_game` world-loading
into `parish-core::game_loop::inference` and `::save` respectively. Both server
and Tauri delegate to the shared helpers; each retains only the backend-specific
side effects (URL warning emit path, inference_client slot update for server).

Evidence confirms: 2232 tests pass, clippy reports no issues across the full
workspace, architecture fitness test continues to pass (parish-core does not
import axum/tauri/tower/wry/tao). The `InferenceSlots` grouping struct
correctly keeps the function under Clippy's 7-argument limit. The `save.rs`
pure function handles both game-mod and legacy-data-file paths, with soft NPC
load failure matching prior per-runtime behavior.

What remains un-extracted (`handle_system_command`, `do_save_game`, CLI
migration) is documented in `game_loop/mod.rs` with explicit rationale for
each, consistent with the prior slice's documentation pattern. No placeholder
markers, no unexplained `#[allow]` attributes, no dead code left behind.
