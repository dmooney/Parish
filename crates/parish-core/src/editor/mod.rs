//! Parish Designer — editor support module.
//!
//! Backend for the GUI editor utility that lets game designers browse mods,
//! edit NPC and location data, validate cross-references, and inspect save
//! files. The editor operates on a **fresh in-memory copy loaded from disk**
//! and never touches the live gameplay [`GameMod`](crate::game_mod::GameMod)
//! or [`AppState`]; see `docs/design/designer-editor.md` for the full design.
//!
//! Unlike [`GameMod::load`](crate::game_mod::GameMod::load), which is
//! all-or-nothing, the editor loads each mod file independently via
//! [`mod_io::load_mod_snapshot`] so a broken `festivals.json` doesn't hide a
//! working `npcs.json`. Post-save revalidation uses
//! [`validate::validate_snapshot`], not `GameMod::load`, for the same reason.

pub mod format;
pub mod mod_io;
pub mod types;
pub mod validate;

pub use format::write_json_deterministic;
pub use mod_io::{list_mods, load_mod_snapshot};
pub use types::{
    EditorModSnapshot, ModSummary, ValidationIssue, ValidationReport, ValidationSeverity,
};
pub use validate::validate_snapshot;
