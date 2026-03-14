//! Load and save the user config file.

use std::path::Path;

use anyhow::Result;

use super::settings::Settings;

/// Load settings from disk. Returns defaults if the file doesn't exist.
#[allow(dead_code)]
pub fn load(path: &Path) -> Settings {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

/// Save settings to disk.
#[allow(dead_code)]
pub fn save(path: &Path, settings: &Settings) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}
