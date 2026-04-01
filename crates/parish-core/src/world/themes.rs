//! Named color themes with light and dark variants.
//!
//! Themes are loaded from a JSON file (typically `themes.json` in the mod
//! directory). Each theme provides a light and dark [`RawPalette`], and the
//! engine picks one based on the current time of day.

use std::path::Path;

use serde::Deserialize;

use super::palette::{RawColor, RawPalette};
use crate::error::ParishError;

/// Hour at which the palette switches from dark to light (inclusive).
pub const LIGHT_START_HOUR: u32 = 6;

/// Hour at which the palette switches from light to dark (inclusive).
pub const DARK_START_HOUR: u32 = 19;

/// Returns `true` when the given hour falls in the "dark" range (19:00–05:59).
pub fn is_dark_hour(hour: u32) -> bool {
    !(LIGHT_START_HOUR..DARK_START_HOUR).contains(&hour)
}

// ---------------------------------------------------------------------------
// JSON schema for themes.json
// ---------------------------------------------------------------------------

/// JSON representation of a single RGB color as a 3-element array.
#[derive(Debug, Clone, Deserialize)]
struct JsonColor(u8, u8, u8);

impl From<JsonColor> for RawColor {
    fn from(c: JsonColor) -> Self {
        RawColor::new(c.0, c.1, c.2)
    }
}

/// JSON representation of a 7-slot palette.
#[derive(Debug, Clone, Deserialize)]
struct JsonPalette {
    bg: JsonColor,
    fg: JsonColor,
    accent: JsonColor,
    panel_bg: JsonColor,
    input_bg: JsonColor,
    border: JsonColor,
    muted: JsonColor,
}

impl From<JsonPalette> for RawPalette {
    fn from(p: JsonPalette) -> Self {
        RawPalette {
            bg: p.bg.into(),
            fg: p.fg.into(),
            accent: p.accent.into(),
            panel_bg: p.panel_bg.into(),
            input_bg: p.input_bg.into(),
            border: p.border.into(),
            muted: p.muted.into(),
        }
    }
}

/// JSON representation of a named theme with light/dark variants.
#[derive(Debug, Clone, Deserialize)]
struct JsonTheme {
    name: String,
    slug: String,
    light: JsonPalette,
    dark: JsonPalette,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A named color theme with light and dark palette variants.
#[derive(Debug, Clone)]
pub struct ColorTheme {
    /// Human-readable theme name (e.g. "Catppuccin").
    pub name: String,
    /// Machine-friendly slug (e.g. "catppuccin").
    pub slug: String,
    /// Palette used during daytime hours.
    pub light: RawPalette,
    /// Palette used during nighttime hours.
    pub dark: RawPalette,
}

impl ColorTheme {
    /// Returns the appropriate palette variant for the given hour.
    pub fn palette_for_hour(&self, hour: u32) -> &RawPalette {
        if is_dark_hour(hour) {
            &self.dark
        } else {
            &self.light
        }
    }
}

impl From<JsonTheme> for ColorTheme {
    fn from(jt: JsonTheme) -> Self {
        ColorTheme {
            name: jt.name,
            slug: jt.slug,
            light: jt.light.into(),
            dark: jt.dark.into(),
        }
    }
}

/// A collection of available color themes.
#[derive(Debug, Clone)]
pub struct ThemeSet {
    themes: Vec<ColorTheme>,
}

impl ThemeSet {
    /// Loads themes from a JSON file.
    pub fn load(path: &Path) -> Result<Self, ParishError> {
        let data = std::fs::read_to_string(path).map_err(|e| {
            ParishError::Config(format!(
                "Failed to read themes file {}: {}",
                path.display(),
                e
            ))
        })?;
        let json_themes: Vec<JsonTheme> = serde_json::from_str(&data).map_err(|e| {
            ParishError::Config(format!(
                "Failed to parse themes file {}: {}",
                path.display(),
                e
            ))
        })?;
        let themes = json_themes.into_iter().map(ColorTheme::from).collect();
        Ok(ThemeSet { themes })
    }

    /// Returns a theme by slug, or the first theme if not found.
    pub fn get(&self, slug: &str) -> &ColorTheme {
        self.themes
            .iter()
            .find(|t| t.slug == slug)
            .unwrap_or(&self.themes[0])
    }

    /// Returns all available theme names and slugs.
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.themes
            .iter()
            .map(|t| (t.name.as_str(), t.slug.as_str()))
            .collect()
    }

    /// Returns the number of themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Returns `true` if no themes are loaded.
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
}

impl Default for ThemeSet {
    /// Returns a single-theme set with the built-in Parish Classic theme.
    fn default() -> Self {
        ThemeSet {
            themes: vec![fallback_theme()],
        }
    }
}

/// The built-in "Parish Classic" theme matching the original hardcoded palettes.
pub fn fallback_theme() -> ColorTheme {
    ColorTheme {
        name: "Parish Classic".to_string(),
        slug: "parish-classic".to_string(),
        light: RawPalette {
            bg: RawColor::new(255, 245, 220),
            fg: RawColor::new(50, 35, 15),
            accent: RawColor::new(180, 130, 50),
            panel_bg: RawColor::new(250, 240, 215),
            input_bg: RawColor::new(245, 235, 210),
            border: RawColor::new(210, 190, 150),
            muted: RawColor::new(120, 100, 60),
        },
        dark: RawPalette {
            bg: RawColor::new(20, 25, 40),
            fg: RawColor::new(180, 180, 190),
            accent: RawColor::new(100, 110, 140),
            panel_bg: RawColor::new(25, 30, 48),
            input_bg: RawColor::new(30, 35, 55),
            border: RawColor::new(60, 65, 90),
            muted: RawColor::new(120, 120, 135),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dark_hour_boundaries() {
        // Dark hours: 19–23, 0–5
        assert!(is_dark_hour(0));
        assert!(is_dark_hour(5));
        assert!(!is_dark_hour(6));
        assert!(!is_dark_hour(12));
        assert!(!is_dark_hour(18));
        assert!(is_dark_hour(19));
        assert!(is_dark_hour(23));
    }

    #[test]
    fn fallback_theme_has_valid_palettes() {
        let theme = fallback_theme();
        assert_eq!(theme.slug, "parish-classic");
        assert_ne!(theme.light.bg, theme.dark.bg);
    }

    #[test]
    fn palette_for_hour_selects_correctly() {
        let theme = fallback_theme();
        assert_eq!(theme.palette_for_hour(12), &theme.light);
        assert_eq!(theme.palette_for_hour(22), &theme.dark);
        assert_eq!(theme.palette_for_hour(3), &theme.dark);
        assert_eq!(theme.palette_for_hour(6), &theme.light);
    }

    #[test]
    fn default_theme_set_has_one_theme() {
        let set = ThemeSet::default();
        assert_eq!(set.len(), 1);
        assert_eq!(set.get("parish-classic").slug, "parish-classic");
    }

    #[test]
    fn theme_set_get_fallback() {
        let set = ThemeSet::default();
        // Non-existent slug falls back to first
        assert_eq!(set.get("nonexistent").slug, "parish-classic");
    }

    #[test]
    fn load_themes_from_file() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("mods/kilteevan-1820/themes.json");
        if path.exists() {
            let set = ThemeSet::load(&path).unwrap();
            assert!(set.len() >= 10);
            assert_eq!(set.get("catppuccin").name, "Catppuccin");
            assert_eq!(set.get("gruvbox").name, "Gruvbox");
        }
    }

    #[test]
    fn theme_set_list() {
        let set = ThemeSet::default();
        let list = set.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], ("Parish Classic", "parish-classic"));
    }
}
