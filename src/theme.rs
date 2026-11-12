//! Colors, overridable from `config.json`:
//!
//! ```json
//! { "theme": { "accent": "magenta", "highlight_bg": "#3b4252" } }
//! ```
//!
//! Each value is a color name (`red`, `lightblue`, `darkgray`, ...), a hex
//! code (`#rrggbb`) or a 256-color index (`"42"`). Keys left out keep their
//! default.

use anyhow::{Context, Result, anyhow};
use ratatui::style::Color;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Active pane border, selected item text.
    pub accent: Color,
    /// Inactive borders and the status bar text.
    pub muted: Color,
    /// Book and chapter names.
    pub text: Color,
    /// Verse text.
    pub verse: Color,
    pub highlight_bg: Color,
    pub status_bg: Color,
    /// Search prompt and status messages.
    pub search: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            accent: Color::Cyan,
            muted: Color::DarkGray,
            text: Color::Rgb(171, 178, 191),
            verse: Color::Rgb(240, 240, 240),
            highlight_bg: Color::Rgb(40, 44, 52),
            status_bg: Color::Rgb(33, 37, 43),
            search: Color::Yellow,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ConfigFile {
    theme: ThemeFile,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct ThemeFile {
    accent: Option<String>,
    muted: Option<String>,
    text: Option<String>,
    verse: Option<String>,
    highlight_bg: Option<String>,
    status_bg: Option<String>,
    search: Option<String>,
}

/// Parse the contents of `config.json`. A typo in a key or color is an error
/// rather than being silently ignored, so the user learns why nothing changed.
pub fn parse(json: &str) -> Result<Theme> {
    let file: ConfigFile = serde_json::from_str(json).context("config.json")?;
    let t = file.theme;
    let mut theme = Theme::default();
    for (slot, value, name) in [
        (&mut theme.accent, t.accent, "accent"),
        (&mut theme.muted, t.muted, "muted"),
        (&mut theme.text, t.text, "text"),
        (&mut theme.verse, t.verse, "verse"),
        (&mut theme.highlight_bg, t.highlight_bg, "highlight_bg"),
        (&mut theme.status_bg, t.status_bg, "status_bg"),
        (&mut theme.search, t.search, "search"),
    ] {
        if let Some(value) = value {
            *slot = Color::from_str(&value)
                .map_err(|_| anyhow!("config.json: theme.{name}: unknown color '{value}'"))?;
        }
    }
    Ok(theme)
}

/// The theme from `config.json`, or the default plus a message to show the
/// user when the file exists but is broken.
pub fn load() -> (Theme, Option<String>) {
    let Some(path) = crate::store::config_path() else {
        return (Theme::default(), None);
    };
    match std::fs::read_to_string(&path) {
        Err(_) => (Theme::default(), None),
        Ok(json) => match parse(&json) {
            Ok(theme) => (theme, None),
            Err(err) => (Theme::default(), Some(format!("{err:#}"))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_the_default_theme() {
        assert_eq!(parse("{}").unwrap(), Theme::default());
    }

    #[test]
    fn names_and_hex_codes_override_single_keys() {
        let theme = parse(r##"{"theme": {"accent": "magenta", "verse": "#102030"}}"##).unwrap();
        assert_eq!(theme.accent, Color::Magenta);
        assert_eq!(theme.verse, Color::Rgb(0x10, 0x20, 0x30));
        assert_eq!(theme.muted, Theme::default().muted);
    }

    #[test]
    fn typos_are_reported() {
        let err = parse(r#"{"theme": {"acent": "red"}}"#).unwrap_err();
        assert!(format!("{err:#}").contains("acent"), "{err:#}");

        let err = parse(r#"{"theme": {"accent": "reddish"}}"#).unwrap_err();
        assert!(format!("{err:#}").contains("reddish"), "{err:#}");
    }
}
