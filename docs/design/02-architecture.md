# Steam Proton Browser — Architecture & Technology

## Technology Choice

### Language: **Rust**

Rust is the natural fit for this project:

- **Single static binary** — no runtime, no interpreter, trivial to distribute.
- **Cross-platform** — first-class support for Linux, macOS, Windows.
- **Rich TUI ecosystem** — `ratatui` (the successor to `tui-rs`) is mature and actively maintained.
- **Fast startup** — important for a utility you pop open, use, and close.
- **Strong community overlap** — Rust + Linux gaming communities share significant overlap (good for contributors).

### Key Crates

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework (layout, widgets, rendering). |
| `crossterm` | Cross-platform terminal backend (input, raw mode, colors). |
| `tokio` (optional) | Async runtime — only if we add file watchers or background scanning. |
| `serde` + `serde_json` | Parsing Steam's configuration files (some are JSON-like). |
| `dirs` / `directories` | Platform-specific standard directory resolution. |
| `open` | Cross-platform "open in file explorer" (`xdg-open`, `open`, `explorer.exe`). |
| `fuzzy-matcher` | Fuzzy search/filter for game titles. |
| `clap` | CLI argument parsing (config path overrides, steam path overrides). |
| `toml` | Serialize/deserialize the local cache and config files. |

## High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                   TUI Layer                         │
│  (ratatui + crossterm)                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────────┐ │
│  │ Game List│ │ Folder   │ │ File/Detail Preview  │ │
│  │ Panel    │ │ Category │ │ Panel                │ │
│  │          │ │ Panel    │ │                      │ │
│  └──────────┘ └──────────┘ └──────────────────────┘ │
│  ┌──────────────────────────────────────────────────┐│
│  │ Status Bar / Breadcrumb / Search                 ││
│  └──────────────────────────────────────────────────┘│
└────────────────────┬────────────────────────────────┘
                     │ Events (key input, resize)
                     ▼
┌─────────────────────────────────────────────────────┐
│                 App State                           │
│  ┌────────────┐ ┌───────────┐ ┌──────────────────┐  │
│  │ Navigation │ │ Filter /  │ │ Selection /      │  │
│  │ Stack      │ │ Search    │ │ Clipboard        │  │
│  └────────────┘ └───────────┘ └──────────────────┘  │
└────────────────────┬────────────────────────────────┘
                     │ Queries
                     ▼
┌─────────────────────────────────────────────────────┐
│              Steam Library Service                  │
│  ┌──────────────┐ ┌─────────────┐ ┌──────────────┐  │
│  │ Path         │ │ ACF Parser  │ │ Folder       │  │
│  │ Discovery    │ │ (manifests) │ │ Resolver     │  │
│  └──────────────┘ └─────────────┘ └──────────────┘  │
└────────────────────┬────────────────────────────────┘
                     │ Read / Write
                     ▼
┌─────────────────────────────────────────────────────┐
│                  Cache Layer                        │
│  ~/.cache/steam-proton-browser/cache.toml           │
│  Stores discovered Steam roots, library paths,      │
│  and app index for fast subsequent startups.         │
└────────────────────┬────────────────────────────────┘
                     │ Filesystem I/O
                     ▼
┌─────────────────────────────────────────────────────┐
│                 Operating System                    │
│  Steam dirs, Proton prefixes, file explorer launch  │
└─────────────────────────────────────────────────────┘
```

## Module Breakdown

### `steam` — Steam Library Service

Responsible for all interaction with Steam's on-disk layout.

- **`discovery`** — Locate Steam installation root(s) across platforms. Handle Flatpak paths, custom library folders defined in `libraryfolders.vdf`, and Windows registry lookups.
- **`acf`** — Parse Valve's ACF/VDF key-value format used in `appmanifest_*.acf` and `libraryfolders.vdf`.
- **`library`** — Enumerate installed apps, build the `AppId → GameInfo` index.
- **`folders`** — Given an App ID, resolve all relevant folder paths (install dir, compatdata, shadercache, workshop, etc.) and report which ones exist on disk.

### `tui` — Terminal UI

- **`app`** — Top-level application loop (event handling, state transitions, render dispatch).
- **`views`** — Individual view components (game list, folder list, file browser, help overlay).
- **`widgets`** — Reusable widget building blocks (filterable list, breadcrumb bar, status bar).
- **`keybindings`** — Keymap definitions and input routing.
- **`theme`** — Color scheme and styling constants.

### `cache` — Discovery Cache

Persists discovered Steam paths and library data so subsequent launches are near-instant.

- **`store`** — Read/write `cache.toml` from the platform cache directory (`~/.cache/steam-proton-browser/` on Linux, `~/Library/Caches/steam-proton-browser/` on macOS, `%LOCALAPPDATA%\steam-proton-browser\cache\` on Windows).
- **`types`** — `CacheFile` struct containing:
  - `version` — cache schema version for forward compatibility.
  - `last_updated` — timestamp of last full scan.
  - `steam_roots` — list of discovered Steam installation paths.
  - `library_folders` — list of library folder paths (from `libraryfolders.vdf`).
  - `apps` — vec of `CachedApp { app_id, name, install_dir, library_path, proton_version }` for every discovered title.
- **`invalidation`** — Logic to detect when the cache is stale (missing file, schema version mismatch, `libraryfolders.vdf` mtime changed, or user requests a manual refresh).

### `config` — User Configuration

- **`settings`** — Runtime settings (custom Steam paths, theme overrides, custom folder patterns).
- **`persistence`** — Load/save config from `~/.config/steam-proton-browser/config.toml`.

### `platform` — OS Abstractions

- **`explorer`** — Open a folder in the native file manager.
- **`paths`** — Platform-specific default paths for Steam, config dirs, etc.

## Data Flow

```
Startup
  │
  ├─ Try to load cache.toml
  │    ├─ Cache exists & valid?
  │    │    ├─ YES → load cached steam_roots, library_folders, apps
  │    │    │         Check libraryfolders.vdf mtime vs cache.last_updated
  │    │    │         If stale → trigger background rescan
  │    │    │         Otherwise → use cached data as-is
  │    │    └─ NO  → perform full discovery (below)
  │    │
  │    └─ Full Discovery:
  │         ├─ Discover Steam install path(s)
  │         ├─ Parse libraryfolders.vdf → list of library roots
  │         ├─ For each library root:
  │         │    ├─ Scan appmanifest_*.acf files
  │         │    └─ Build Vec<GameInfo> { app_id, name, install_dir, library_root }
  │         └─ Write results to cache.toml
  │
  ├─ Sort by name, populate initial UI state
  └─ Enter event loop
         │
         ├─ User selects a game → resolve folder categories for that app_id
         ├─ User selects a folder category → list contents / show path
         ├─ User presses Enter/O → open folder in system explorer
         ├─ User types "/" → enter fuzzy filter mode
         └─ User presses R → force full rescan & cache refresh
```

## Build & Distribution

| Target | Method |
|--------|--------|
| Linux (native) | `cargo build --release` — static binary via musl. |
| Linux (Flatpak Steam) | Same binary, different default paths. |
| Steam Deck | Same Linux binary, test on Deck's Game Mode terminal. |
| macOS | `cargo build --release` — universal binary (aarch64 + x86_64). |
| Windows | `cargo build --release` — MSVC or GNU target. |
| AUR / Homebrew / Cargo | Publish to package managers for easy install. |

## Testing Strategy

- **Unit tests** for ACF/VDF parsing with fixture files.
- **Unit tests** for path discovery logic with mock filesystem layouts.
- **Integration tests** with a synthetic Steam directory tree.
- **Manual testing** on Linux, Steam Deck, macOS, Windows.
- **CI** via GitHub Actions (Linux + Windows + macOS matrix).
