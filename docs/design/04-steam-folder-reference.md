# Steam Proton Browser — Steam Folder Structure Reference

This document maps out the Steam directory layout across platforms. This is the core domain knowledge the tool needs to navigate.

## Steam Installation Roots

### Linux (Native)

```
~/.local/share/Steam/
~/.steam/steam/            ← often a symlink to the above
```

### Linux (Flatpak)

```
~/.var/app/com.valvesoftware.Steam/.local/share/Steam/
```

### Linux (Snap)

```
~/snap/steam/common/.local/share/Steam/
```

### macOS

```
~/Library/Application Support/Steam/
```

### Windows

```
C:\Program Files (x86)\Steam\
```

## Key Paths Relative to Steam Root

### Library Folders Config

```
<steam_root>/steamapps/libraryfolders.vdf
```

This VDF file lists all library folders (including the default one). Each entry contains the path and a list of app IDs installed there.

### Per-Library Layout

Each library folder (including the default `<steam_root>/steamapps/`) has:

```
<library>/steamapps/
├── appmanifest_<appid>.acf        # One per installed app — name, size, state
├── common/
│   └── <game_install_dir>/        # The actual game files
├── compatdata/
│   └── <appid>/                   # Proton/Wine prefix for this app
│       ├── pfx/
│       │   ├── drive_c/           # Virtual C: drive
│       │   │   ├── Program Files/
│       │   │   ├── users/
│       │   │   │   └── steamuser/ # User profile (AppData, Documents, etc.)
│       │   │   └── windows/
│       │   ├── dosdevices/        # Drive letter symlinks
│       │   └── *.reg              # Wine registry files
│       ├── version                # Proton version used
│       └── config_info            # Prefix configuration
├── shadercache/
│   └── <appid>/                   # Vulkan/GL shader cache
│       ├── fozpipelinesv6/
│       ├── mesa_shader_cache_sf/
│       └── ...
├── workshop/
│   └── content/
│       └── <appid>/               # Workshop/mod content
└── downloading/
    └── <appid>/                   # In-progress downloads
```

### Steam Root-Level Paths

```
<steam_root>/
├── steamapps/                     # Default library (structure above)
├── userdata/
│   └── <userid>/
│       ├── <appid>/
│       │   └── remote/            # Cloud save data
│       └── config/
│           └── localconfig.vdf    # Per-user game config (launch options, etc.)
├── config/
│   ├── config.vdf                 # Global Steam config
│   └── loginusers.vdf             # Known user accounts
├── logs/                          # Steam client logs
├── shader_cache_temp_dir/         # Temporary shader compilation
└── compatibilitytools.d/          # Custom Proton/compatibility tool installs
    └── <custom_proton_version>/   # e.g., GE-Proton8-25
```

## Folder Categories for the TUI

Given an App ID, the tool should resolve and display these categories (only if the path exists on disk):

| Category | Path | Notes |
|----------|------|-------|
| **Install Directory** | `<library>/steamapps/common/<installdir>/` | `installdir` from the ACF manifest. |
| **Proton Prefix** | `<library>/steamapps/compatdata/<appid>/` | Top-level prefix dir. |
| **Proton Prefix — drive_c** | `<library>/steamapps/compatdata/<appid>/pfx/drive_c/` | The virtual Windows filesystem — most useful for modding. |
| **Proton Prefix — AppData** | `<library>/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser/AppData/` | Where many games store saves/config under Proton. |
| **Shader Cache** | `<library>/steamapps/shadercache/<appid>/` | Can be large; useful for cleanup. |
| **Workshop Content** | `<library>/steamapps/workshop/content/<appid>/` | Subscribed workshop items. |
| **Cloud Saves** | `<steam_root>/userdata/<userid>/<appid>/remote/` | Cloud-synced save data. |
| **Proton Logs** | `<steam_root>/steamapps/compatdata/<appid>/` | `steam-*.log` files in the prefix. |
| **Custom Compat Tools** | `<steam_root>/compatibilitytools.d/` | Global — not per-app, but useful to browse. |

## ACF Manifest Format

`appmanifest_<appid>.acf` is a VDF (Valve Data Format) file:

```vdf
"AppState"
{
    "appid"        "292030"
    "Universe"     "1"
    "name"         "The Witcher 3: Wild Hunt"
    "StateFlags"   "4"
    "installdir"   "The Witcher 3 Wild Hunt"
    "SizeOnDisk"   "48318214144"
    ...
}
```

Key fields:
- `appid` — numeric app identifier.
- `name` — human-readable game title.
- `installdir` — folder name under `steamapps/common/` (NOT the full path).
- `SizeOnDisk` — install size in bytes.

## VDF / libraryfolders.vdf Format

```vdf
"libraryfolders"
{
    "0"
    {
        "path"    "/home/user/.local/share/Steam"
        "label"   ""
        "apps"
        {
            "292030"   "48318214144"
            "1091500"  "70000000000"
        }
    }
    "1"
    {
        "path"    "/mnt/games/SteamLibrary"
        "label"   "Games Drive"
        "apps"
        {
            "1245620"  "50000000000"
        }
    }
}
```

## Cache File

The app maintains a local cache to avoid re-scanning Steam directories on every launch.

### Cache Location

| Platform | Path |
|----------|------|
| Linux | `~/.cache/steam-proton-browser/cache.toml` |
| macOS | `~/Library/Caches/steam-proton-browser/cache.toml` |
| Windows | `%LOCALAPPDATA%\steam-proton-browser\cache\cache.toml` |

### Cache Format

```toml
version = 1
last_updated = "2026-03-14T12:00:00Z"

[[steam_roots]]
path = "/home/user/.local/share/Steam"
source = "default"            # how it was discovered: default | flatpak | snap | registry | manual

[[steam_roots]]
path = "/home/user/.var/app/com.valvesoftware.Steam/.local/share/Steam"
source = "flatpak"

[[library_folders]]
path = "/home/user/.local/share/Steam/steamapps"
steam_root = "/home/user/.local/share/Steam"
label = ""

[[library_folders]]
path = "/mnt/games/SteamLibrary/steamapps"
steam_root = "/home/user/.local/share/Steam"
label = "Games Drive"

[[apps]]
app_id = 292030
name = "The Witcher 3: Wild Hunt"
install_dir = "The Witcher 3 Wild Hunt"
library_path = "/home/user/.local/share/Steam/steamapps"
size_on_disk = 48318214144
proton_version = "Proton Experimental"

[[apps]]
app_id = 1091500
name = "Cyberpunk 2077"
install_dir = "Cyberpunk 2077"
library_path = "/mnt/games/SteamLibrary/steamapps"
size_on_disk = 70000000000
proton_version = "GE-Proton8-25"
```

### Cache Invalidation

The cache is considered **stale** and triggers a rescan when any of these conditions are met:

1. `cache.toml` does not exist.
2. `version` field does not match the app's current cache schema version.
3. `libraryfolders.vdf` mtime is newer than `last_updated`.
4. User presses `R` in the TUI to force a manual refresh.
5. A CLI flag `--refresh` is passed at startup.

When the cache is stale, the app performs a full discovery scan and overwrites `cache.toml` with fresh data. When the cache is valid, startup skips filesystem scanning entirely and loads directly from the TOML file.

## Platform Detection Strategy

```
1. Check known default paths for current OS
2. On Linux, also check:
   a. Flatpak path (~/.var/app/com.valvesoftware.Steam/...)
   b. Snap path (~/snap/steam/...)
   c. Follow symlinks (~/.steam/steam → real path)
3. On Windows, check registry:
   HKCU\Software\Valve\Steam → SteamPath
4. Parse libraryfolders.vdf for additional library locations
5. Validate each path exists before adding to library list
6. Write all discovered paths to cache.toml
```
