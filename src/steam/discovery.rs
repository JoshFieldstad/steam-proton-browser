//! Discover Steam installation root directories across platforms.

use std::path::PathBuf;

/// Discover all Steam installation roots on the current system.
/// Returns paths that exist on disk, in priority order.
pub fn discover_steam_roots() -> Vec<PathBuf> {
    let candidates = platform_candidates();
    candidates.into_iter().filter(|p| p.is_dir()).collect()
}

#[cfg(target_os = "linux")]
fn platform_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Native Steam
        paths.push(home.join(".local/share/Steam"));
        paths.push(home.join(".steam/steam"));

        // Flatpak
        paths.push(home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));

        // Snap
        paths.push(home.join("snap/steam/common/.local/share/Steam"));
    }

    // Deduplicate by canonical path (handles symlinks like ~/.steam/steam)
    dedup_by_canonical(paths)
}

/// Resolve symlinks and remove duplicate paths that point to the same location.
fn dedup_by_canonical(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = Vec::new();
    let mut result = Vec::new();

    for path in paths {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        if !seen.contains(&canonical) {
            seen.push(canonical);
            result.push(path);
        }
    }

    result
}

/// Given a Steam root, find the `libraryfolders.vdf` file and parse out additional
/// library folder paths.
pub fn discover_library_folders(steam_root: &std::path::Path) -> Vec<PathBuf> {
    let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");
    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => return vec![steam_root.to_path_buf()],
    };

    let doc = match super::acf::parse(&content) {
        Ok(d) => d,
        Err(_) => return vec![steam_root.to_path_buf()],
    };

    let mut folders = Vec::new();

    if let Some(lib_folders) = doc.get("libraryfolders").and_then(|v| v.as_map()) {
        // Keys are numeric indices: "0", "1", "2", ...
        let mut keys: Vec<&String> = lib_folders.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(entry) = lib_folders.get(key.as_str()) {
                if let Some(path_str) = entry.get_str("path") {
                    let path = PathBuf::from(path_str);
                    if path.is_dir() {
                        folders.push(path);
                    }
                }
            }
        }
    }

    if folders.is_empty() {
        folders.push(steam_root.to_path_buf());
    }

    folders
}
