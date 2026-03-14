//! Cache layer — persist discovered Steam paths and app index for fast startup.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::steam::library::{GameInfo, Library};

const CACHE_VERSION: u32 = 1;

/// Serializable cache file contents.
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub version: u32,
    pub last_updated: DateTime<Utc>,
    pub steam_roots: Vec<CachedRoot>,
    pub library_folders: Vec<CachedLibraryFolder>,
    pub apps: Vec<GameInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedRoot {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedLibraryFolder {
    pub path: PathBuf,
}

impl CacheFile {
    pub fn into_library(self) -> Library {
        Library {
            steam_roots: self.steam_roots.into_iter().map(|r| r.path).collect(),
            library_folders: self.library_folders.into_iter().map(|f| f.path).collect(),
            games: self.apps,
        }
    }
}

/// Return the platform-appropriate cache file path.
pub fn cache_file_path() -> PathBuf {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("steam-proton-browser");
    cache_dir.join("cache.toml")
}

/// Try to load the cache from disk. Returns `None` if the file doesn't exist or can't be parsed.
pub fn load(path: &Path) -> Option<CacheFile> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

/// Check whether a loaded cache is still valid.
pub fn is_valid(cache: &CacheFile, steam_roots: &[PathBuf]) -> bool {
    // Schema version must match
    if cache.version != CACHE_VERSION {
        return false;
    }

    // Check if libraryfolders.vdf has been modified since last cache write
    for root in steam_roots {
        let vdf_path = root.join("steamapps/libraryfolders.vdf");
        if let Ok(metadata) = std::fs::metadata(&vdf_path) {
            if let Ok(modified) = metadata.modified() {
                let modified: DateTime<Utc> = modified.into();
                if modified > cache.last_updated {
                    return false;
                }
            }
        }
    }

    true
}

/// Write the cache to disk.
pub fn save(path: &Path, library: &Library, steam_roots: &[PathBuf]) -> Result<()> {
    let cache = CacheFile {
        version: CACHE_VERSION,
        last_updated: Utc::now(),
        steam_roots: steam_roots
            .iter()
            .map(|p| CachedRoot { path: p.clone() })
            .collect(),
        library_folders: library
            .library_folders
            .iter()
            .map(|p| CachedLibraryFolder { path: p.clone() })
            .collect(),
        apps: library.games.clone(),
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(&cache)?;
    std::fs::write(path, content)?;

    Ok(())
}
