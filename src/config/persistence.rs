//! Load and save the user config file.

use std::path::Path;

use super::settings::Settings;

/// Load settings from disk. Returns defaults if the file doesn't exist.
pub fn load(path: &Path) -> Settings {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}
