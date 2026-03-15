//! Cache layer — persist discovered Steam paths and app index for fast startup.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::steam::library::{GameInfo, Library};

const CACHE_VERSION: u32 = 1;

/// Serializable cache file contents.
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub version: u32,
    /// Seconds since UNIX epoch.
    pub last_updated: u64,
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
        if let Ok(metadata) = std::fs::metadata(&vdf_path)
            && let Ok(modified) = metadata.modified()
        {
            let modified_secs = modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if modified_secs > cache.last_updated {
                return false;
            }
        }
    }

    true
}

/// Write the cache to disk.
pub fn save(
    path: &Path,
    library: &Library,
    steam_roots: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    let cache = CacheFile {
        version: CACHE_VERSION,
        last_updated: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::library::GameInfo;

    fn sample_library() -> Library {
        Library {
            steam_roots: vec![PathBuf::from("/home/user/.steam")],
            library_folders: vec![PathBuf::from("/home/user/.steam/steamapps")],
            games: vec![GameInfo {
                app_id: 292030,
                name: "The Witcher 3".to_string(),
                install_dir: "The Witcher 3 Wild Hunt".to_string(),
                library_path: PathBuf::from("/home/user/.steam"),
                size_on_disk: 48_000_000_000,
                proton_version: Some("Proton 9.0-4".to_string()),
                last_played: 1700000000,
            }],
        }
    }

    #[test]
    fn test_save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("cache.toml");
        let library = sample_library();
        let roots = vec![PathBuf::from("/home/user/.steam")];

        save(&cache_path, &library, &roots).unwrap();

        let loaded = load(&cache_path).expect("should load saved cache");
        assert_eq!(loaded.version, CACHE_VERSION);
        assert!(loaded.last_updated > 0);
        assert_eq!(loaded.apps.len(), 1);
        assert_eq!(loaded.apps[0].app_id, 292030);
        assert_eq!(loaded.apps[0].name, "The Witcher 3");
        assert_eq!(loaded.steam_roots.len(), 1);
    }

    #[test]
    fn test_into_library() {
        let cache = CacheFile {
            version: CACHE_VERSION,
            last_updated: 1700000000,
            steam_roots: vec![CachedRoot {
                path: PathBuf::from("/steam"),
            }],
            library_folders: vec![CachedLibraryFolder {
                path: PathBuf::from("/steam/steamapps"),
            }],
            apps: vec![],
        };
        let lib = cache.into_library();
        assert_eq!(lib.steam_roots, vec![PathBuf::from("/steam")]);
        assert_eq!(lib.library_folders, vec![PathBuf::from("/steam/steamapps")]);
        assert!(lib.games.is_empty());
    }

    #[test]
    fn test_is_valid_wrong_version() {
        let cache = CacheFile {
            version: CACHE_VERSION + 1,
            last_updated: u64::MAX,
            steam_roots: vec![],
            library_folders: vec![],
            apps: vec![],
        };
        assert!(!is_valid(&cache, &[]));
    }

    #[test]
    fn test_is_valid_correct_version_no_roots() {
        let cache = CacheFile {
            version: CACHE_VERSION,
            last_updated: u64::MAX,
            steam_roots: vec![],
            library_folders: vec![],
            apps: vec![],
        };
        // No roots to check means nothing invalidates
        assert!(is_valid(&cache, &[]));
    }

    #[test]
    fn test_load_nonexistent() {
        assert!(load(Path::new("/nonexistent/cache.toml")).is_none());
    }

    #[test]
    fn test_load_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "not valid { toml [").unwrap();
        assert!(load(&path).is_none());
    }
}
