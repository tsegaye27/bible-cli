//! Files the reader keeps between runs: `state.json` (last position and
//! bookmarks, written by the app) and `config.json` (theme, written by the user).
//!
//! Both live in one directory:
//! - Windows: `%APPDATA%\bible-cli`
//! - elsewhere: `$XDG_CONFIG_HOME/bible-cli`, falling back to `~/.config/bible-cli`

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A verse, addressed by book number, chapter and verse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub book: u32,
    pub chapter: u32,
    pub verse: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub last: Option<Location>,
    pub bookmarks: Vec<Location>,
}

pub fn dir() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.map(|b| b.join("bible-cli"))
}

pub fn config_path() -> Option<PathBuf> {
    dir().map(|d| d.join("config.json"))
}

fn state_path() -> Option<PathBuf> {
    dir().map(|d| d.join("state.json"))
}

/// A missing or unreadable state file is a fresh start, not an error: losing
/// the reading position must never stop the reader from opening.
pub fn load_state() -> State {
    state_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Writes to a temporary file and renames it over the old one, so a crash
/// mid-write can't leave a truncated `state.json` and wipe the bookmarks.
pub fn save_state(state: &State) -> Result<()> {
    let path = state_path().ok_or_else(|| anyhow!("no home directory to save state in"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(state)?)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_and_tolerates_missing_fields() {
        let state = State {
            last: Some(Location {
                book: 19,
                chapter: 23,
                verse: 1,
            }),
            bookmarks: vec![Location {
                book: 43,
                chapter: 3,
                verse: 16,
            }],
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: State = serde_json::from_str(&json).unwrap();
        assert_eq!(back.last, state.last);
        assert_eq!(back.bookmarks, state.bookmarks);

        let empty: State = serde_json::from_str("{}").unwrap();
        assert!(empty.last.is_none() && empty.bookmarks.is_empty());
    }
}
