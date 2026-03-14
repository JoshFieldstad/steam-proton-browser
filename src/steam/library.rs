//! Steam library scanning — enumerate installed games from appmanifest files.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Information about a single installed Steam game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub app_id: u64,
    pub name: String,
    pub install_dir: String,
    pub library_path: PathBuf,
    pub size_on_disk: u64,
    pub proton_version: Option<String>,
    /// Unix timestamp of last play time (0 if never played).
    pub last_played: u64,
}

/// Prefixes/substrings in app names that indicate Steam runtime/tool entries
/// rather than actual games.
const RUNTIME_PREFIXES: &[&str] = &[
    "Proton ",
    "Proton-",
    "Proton Experimental",
    "Steam Linux Runtime",
    "Steamworks ",
    "Steam Runtime",
    "SteamVR",
];

impl GameInfo {
    /// Returns true if this entry is a Proton runtime, Steam redistributable,
    /// or other non-game tool entry.
    pub fn is_runtime(&self) -> bool {
        RUNTIME_PREFIXES.iter().any(|p| self.name.starts_with(p))
    }
}

/// The full scanned library — all games across all library folders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub steam_roots: Vec<PathBuf>,
    pub library_folders: Vec<PathBuf>,
    pub games: Vec<GameInfo>,
}

/// Scan all library folders reachable from the given Steam roots and build a `Library`.
pub fn scan_libraries(steam_roots: &[PathBuf]) -> Result<Library> {
    let mut all_library_folders = Vec::new();
    for root in steam_roots {
        let folders = super::discovery::discover_library_folders(root);
        for folder in folders {
            if !all_library_folders.contains(&folder) {
                all_library_folders.push(folder);
            }
        }
    }

    let mut games = Vec::new();
    for lib_folder in &all_library_folders {
        let steamapps = steamapps_dir(lib_folder);
        if !steamapps.is_dir() {
            continue;
        }

        let manifests = match std::fs::read_dir(&steamapps) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in manifests.flatten() {
            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                if let Ok(game) = parse_appmanifest(&path, lib_folder) {
                    games.push(game);
                }
            }
        }
    }

    Ok(Library {
        steam_roots: steam_roots.to_vec(),
        library_folders: all_library_folders,
        games,
    })
}

/// Given a library folder, return the steamapps directory.
/// If the path already ends with "steamapps", return it as-is.
fn steamapps_dir(lib_folder: &Path) -> PathBuf {
    if lib_folder
        .file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("steamapps"))
    {
        lib_folder.to_path_buf()
    } else {
        lib_folder.join("steamapps")
    }
}

/// Parse a single `appmanifest_<appid>.acf` file into a `GameInfo`.
fn parse_appmanifest(path: &Path, library_path: &Path) -> Result<GameInfo> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let doc = super::acf::parse(&content).with_context(|| format!("parsing {}", path.display()))?;

    let app_state = doc
        .get("AppState")
        .with_context(|| format!("missing AppState in {}", path.display()))?;

    let app_id: u64 = app_state
        .get_str("appid")
        .with_context(|| "missing appid")?
        .parse()
        .with_context(|| "invalid appid")?;

    let name = app_state.get_str("name").unwrap_or("Unknown").to_string();

    let install_dir = app_state.get_str("installdir").unwrap_or("").to_string();

    let size_on_disk: u64 = app_state
        .get_str("SizeOnDisk")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let last_played: u64 = app_state
        .get_str("LastPlayed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Try to detect Proton version from compatdata
    let steamapps = steamapps_dir(library_path);
    let proton_version = detect_proton_version(&steamapps, app_id);

    Ok(GameInfo {
        app_id,
        name,
        install_dir,
        library_path: library_path.to_path_buf(),
        size_on_disk,
        proton_version,
        last_played,
    })
}

/// Try to read the Proton version from compatdata/<appid>/version.
fn detect_proton_version(steamapps: &Path, app_id: u64) -> Option<String> {
    let version_file = steamapps
        .join("compatdata")
        .join(app_id.to_string())
        .join("version");

    std::fs::read_to_string(version_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
