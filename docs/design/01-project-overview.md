# Steam Proton Browser — Project Overview

## Problem Statement

Navigating Steam's file structure — especially Proton compatibility layer prefixes, shader caches, and per-game configuration — is painful. Paths are deeply nested, use opaque numeric App IDs, and vary across platforms (Linux native, Steam Deck, Flatpak). Users who need to access these folders for modding, troubleshooting, or cleanup are forced to memorize or repeatedly look up paths like:

```
~/.steam/steam/steamapps/compatdata/<appid>/pfx/drive_c/users/steamuser/...
```

There is no first-class tool that maps **human-readable game titles** to the underlying folder hierarchy and lets you jump straight to the location you care about.

## Vision

**Steam Proton Browser** is a fast, keyboard-driven TUI application (in the spirit of **k9s**, **lazygit**, and **lazydocker**) that:

1. **Discovers** your local Steam library and enumerates installed titles.
2. **Resolves** each title's App ID to its on-disk folders (install dir, Proton prefix, shader cache, workshop content, cloud saves, etc.).
3. **Presents** a drill-down hierarchy: Library → Game → Folder Category → File Browser.
4. **Opens** any selected folder in the system's native file explorer with a single keypress.
5. Targets **Linux** — including native Steam, Flatpak, Snap, and Steam Deck.

## Goals

| # | Goal | Notes |
|---|------|-------|
| G1 | **Zero-config discovery** | Auto-detect Steam installation paths, library folders, and Proton prefixes without manual setup. |
| G2 | **Human-readable game list** | Map numeric App IDs → game names using `appmanifest_*.acf` files. |
| G3 | **Fast keyboard navigation** | Vim-style key bindings, fuzzy search/filter, breadcrumb trail. |
| G4 | **Open in file explorer** | One keypress to open any folder in the OS file manager (`xdg-open`). |
| G5 | **Linux-focused** | Linux (native + Flatpak + Snap Steam), Steam Deck. |
| G6 | **Minimal dependencies** | Single static binary, no runtime requirements. |
| G7 | **Extensible folder map** | Users can define custom folder categories or path patterns. |

## Non-Goals (v1)

- Cloud/remote Steam library browsing.
- File editing, renaming, or deletion within the TUI (read-only browser).
- Steam Web API integration (everything is local/offline).
- GUI (graphical) interface — TUI only for v1.

## Target Users

- **Linux gamers / Steam Deck users** who mod games, troubleshoot Proton issues, or manage disk space.
- **Power users** who prefer terminal workflows and keyboard-driven tools.
- **Developers** building tools on top of Steam's local data.

## Key Terminology

| Term | Meaning |
|------|---------|
| **App ID** | Steam's numeric identifier for a game/app (e.g., `292030` = The Witcher 3). |
| **compatdata** | Per-game Proton/Wine prefix directory containing the virtual Windows filesystem. |
| **pfx** | The Wine prefix root inside compatdata (`drive_c`, registry files, etc.). |
| **shadercache** | Per-game Vulkan/GL shader cache managed by Steam. |
| **Workshop** | Steam Workshop content directory for user-generated mods. |
| **Library Folder** | A Steam library location (Steam supports multiple library folders across drives). |
| **ACF file** | Valve's App Cache File format — a simple key-value manifest for each installed app. |
