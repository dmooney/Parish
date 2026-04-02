# Color Themes

> Parent: [Architecture Overview](overview.md) | [Docs Index](../index.md)

## Overview

Parish uses a named **color theme** system where each theme provides a **light**
and **dark** palette variant. The engine selects the variant based on time of
day and applies seasonal/weather tinting on top. A visual **wipe transition**
animates the switch between light and dark.

This replaces the original system of 7 hardcoded time-of-day keyframe palettes
with smooth interpolation between them.

## Theme Data

Themes are defined in `themes.json` inside the mod directory (e.g.
`mods/kilteevan-1820/themes.json`). Each theme has:

| Field   | Type       | Description                              |
|---------|------------|------------------------------------------|
| `name`  | string     | Human-readable name (e.g. "Catppuccin")  |
| `slug`  | string     | Machine identifier (e.g. "catppuccin")   |
| `light` | Palette    | 7-color palette for daytime              |
| `dark`  | Palette    | 7-color palette for nighttime            |

Each palette has 7 semantic color slots as RGB arrays:

| Slot       | Purpose                                |
|------------|----------------------------------------|
| `bg`       | Main background                        |
| `fg`       | Primary text                           |
| `accent`   | Highlights, status bar                 |
| `panel_bg` | Sidebar/panel background offset        |
| `input_bg` | Text input field background            |
| `border`   | Border/separator lines                 |
| `muted`    | Secondary/subdued text                 |

### Included Themes (10)

| Theme          | Light Source      | Dark Source       |
|----------------|-------------------|-------------------|
| Parish Classic | Original morning  | Original night    |
| Catppuccin     | Latte             | Mocha             |
| Solarized      | Solarized Light   | Solarized Dark    |
| Gruvbox        | Gruvbox Light     | Gruvbox Dark      |
| Nord           | Snow Storm        | Polar Night       |
| Dracula        | Dracula Light     | Dracula           |
| Rose Pine      | Dawn              | Main              |
| Tokyo Night    | Day               | Storm             |
| One            | One Light         | One Dark          |
| Everforest     | Everforest Light  | Everforest Dark   |

## Light/Dark Selection

The engine uses a discrete hour-based switch, not gradual interpolation:

- **Light variant**: hours 6:00 – 18:59
- **Dark variant**: hours 19:00 – 5:59

Constants `LIGHT_START_HOUR` (6) and `DARK_START_HOUR` (19) in
`crates/parish-core/src/world/themes.rs`.

## Tinting Pipeline

After selecting the light or dark base palette, the existing tinting pipeline
runs on top:

1. **Season tinting** — subtle color shifts (spring: greener, autumn: warmer,
   winter: bluer, summer: golden).
2. **Weather tinting** — desaturation and brightness adjustments (fog, rain,
   storm, overcast).
3. **Contrast enforcement** — ensures minimum luminance difference between
   fg/bg and muted/bg text.

This preserves atmospheric variation within a theme.

## Architecture

### Rust Backend

```
themes.rs  ──>  ColorTheme { light, dark }
                    │
                    ├── palette_for_hour(hour) ──> &RawPalette
                    │
palette.rs  ──>  compute_themed_palette(base, season, weather, config)
                    │
                    ├── apply season tint
                    ├── apply weather tint
                    └── ensure contrast
                    │
ipc/handlers.rs ──> build_themed_palette(world, theme) ──> ThemePalette
                    │
                    └── ThemePalette { bg, fg, accent, ..., is_dark }
```

Key types and locations:

| Type / Function           | File                                         |
|---------------------------|----------------------------------------------|
| `ColorTheme`              | `crates/parish-core/src/world/themes.rs`     |
| `ThemeSet`                | `crates/parish-core/src/world/themes.rs`     |
| `is_dark_hour()`          | `crates/parish-core/src/world/themes.rs`     |
| `fallback_theme()`        | `crates/parish-core/src/world/themes.rs`     |
| `compute_themed_palette()`| `crates/parish-core/src/world/palette.rs`    |
| `build_themed_palette()`  | `crates/parish-core/src/ipc/handlers.rs`     |
| `ThemePalette.is_dark`    | `crates/parish-core/src/ipc/types.rs`        |

### Frontend (Svelte)

The `ThemePalette` sent over IPC includes an `is_dark: boolean` field. The
theme store (`ui/src/stores/theme.ts`) tracks the previous value and triggers
a wipe transition when it flips:

```
onThemeUpdate(palette)
    │
    ├── is_dark changed? ──> trigger wipe
    │       │
    │       ├── 0ms: add .theme-wipe overlay with new bg color
    │       ├── 300ms: apply all new CSS vars to :root
    │       └── 600ms: remove overlay
    │
    └── is_dark same? ──> apply CSS vars immediately
```

The wipe is a CSS `clip-path: inset()` animation defined in `ui/src/app.css`.

### Theme Configuration

The default theme is selected by slug in the mod's `ui.toml`:

```toml
[theme]
default_accent = "#c4a35a"
default_theme = "parish-classic"
```

The theme file path is declared in `mod.toml` via `files.themes`. At startup,
`GameMod::load()` reads `themes.json` and both the Tauri and axum backends
store:

- `active_theme: Mutex<ColorTheme>` — the currently selected theme (mutable
  at runtime via `/theme` command).
- `theme_set: ThemeSet` — all available themes (immutable after load).

### Runtime Theme Switching

Players can change themes at runtime using the `/theme` command:

| Command | Effect |
|---------|--------|
| `/theme` | Lists all available themes, marking the active one |
| `/theme <slug>` | Switches to the named theme immediately |

The command is parsed in `crates/parish-core/src/input/mod.rs` as
`Command::ShowTheme` or `Command::SetTheme(slug)`, and handled in both
`src-tauri/src/commands.rs` and `crates/parish-server/src/routes.rs`.

When a theme is changed, the next 500ms theme tick picks up the new
`ColorTheme` from the `Mutex` and broadcasts it. If the light/dark state
differs from the previous tick, the frontend triggers the wipe transition.

### Theme Tick

Both the Tauri desktop and axum web server emit a `theme-update` event every
500ms containing the current `ThemePalette`. The backend locks `active_theme`,
calls `build_themed_palette(world, &theme)` which re-evaluates the hour and
weather each tick.

## Adding a New Theme

1. Add a new entry to `mods/kilteevan-1820/themes.json` with `name`, `slug`,
   `light`, and `dark` palettes.
2. Optionally set `default_theme` in `ui.toml` to the new slug.
3. Run `cargo test` to verify the theme loads and has valid palettes.
4. Players can switch to the new theme at runtime with `/theme <slug>`.

## Backward Compatibility

- The legacy `compute_palette()` (keyframe interpolation) is still available
  and used by `build_theme()` for any code path that doesn't have a
  `ColorTheme` reference.
- The `From<RawPalette> for ThemePalette` impl defaults `is_dark` to `false`,
  so existing code that builds `ThemePalette` from raw palettes continues to
  work.
