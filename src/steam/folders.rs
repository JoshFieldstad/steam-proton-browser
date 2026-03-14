//! Resolve per-game folder categories to their on-disk paths.

use std::path::{Path, PathBuf};

/// A named folder category for a game.
#[derive(Debug, Clone)]
pub struct FolderEntry {
    pub label: String,
    pub path: PathBuf,
}

/// Resolve all relevant folder categories for a given app, returning only those that exist.
pub fn resolve_folders(
    app_id: u64,
    install_dir: &str,
    library_path: &Path,
    steam_roots: &[PathBuf],
) -> Vec<FolderEntry> {
    let steamapps = steamapps_dir(library_path);

    let mut entries = Vec::new();

    // Install directory
    let install_path = steamapps.join("common").join(install_dir);
    push_if_exists(&mut entries, "Install Directory", &install_path);

    // Proton prefix root
    let compat = steamapps.join("compatdata").join(app_id.to_string());
    push_if_exists(&mut entries, "Proton Prefix (compatdata)", &compat);

    // drive_c
    let drive_c = compat.join("pfx/drive_c");
    push_if_exists(&mut entries, "Proton Prefix — drive_c", &drive_c);

    // AppData (common save/config location under Proton)
    let appdata = drive_c.join("users/steamuser/AppData");
    push_if_exists(&mut entries, "Proton Prefix — AppData", &appdata);

    // Shader cache
    let shader = steamapps.join("shadercache").join(app_id.to_string());
    push_if_exists(&mut entries, "Shader Cache", &shader);

    // Workshop content
    let workshop = steamapps
        .join("workshop")
        .join("content")
        .join(app_id.to_string());
    push_if_exists(&mut entries, "Workshop Content", &workshop);

    // Cloud saves — search across all steam roots / userdata dirs
    for root in steam_roots {
        let userdata = root.join("userdata");
        if let Ok(users) = std::fs::read_dir(&userdata) {
            for user_entry in users.flatten() {
                let remote = user_entry.path().join(app_id.to_string()).join("remote");
                push_if_exists(&mut entries, "Cloud Saves", &remote);
            }
        }
    }

    // Custom compatibility tools (global, not per-app, but useful)
    for root in steam_roots {
        let compat_tools = root.join("compatibilitytools.d");
        push_if_exists(&mut entries, "Custom Compatibility Tools", &compat_tools);
    }

    entries
}

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

fn push_if_exists(entries: &mut Vec<FolderEntry>, label: &str, path: &Path) {
    if path.exists() {
        entries.push(FolderEntry {
            label: label.to_string(),
            path: path.to_path_buf(),
        });
    }
}
