//! User configuration — settings and persistence.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Runtime settings loaded from config file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Additional Steam paths to search (beyond auto-detected ones).
    #[serde(default)]
    pub extra_steam_paths: Vec<PathBuf>,
}

/// Return the platform-appropriate config file path.
pub fn config_file_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-proton-browser");
    config_dir.join("config.toml")
}
