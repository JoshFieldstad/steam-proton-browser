# Steam Proton Browser

A fast, keyboard-driven TUI for browsing Steam and Proton compatibility layer folders. Think **k9s**, but for your Steam library.

Steam Proton Browser discovers your installed games, resolves opaque App IDs to human-readable names, and lets you drill straight into install directories, Proton prefixes, shader caches, workshop content, and more — then open any folder in your system file explorer with a single keypress.

## Features

- **Zero-config discovery** — auto-detects Steam installations, library folders, and Proton prefixes on Linux (native, Flatpak, Snap) and Steam Deck.
- **Drill-down navigation** — Library → Game → Folder Category → File Browser, with breadcrumb trail.
- **Vim-style keybindings** — `j`/`k` to navigate, `Enter` to drill in, `Esc` to go back, `/` to fuzzy filter.
- **Open anywhere** — press `o` to open a folder in your OS file explorer, `Enter` on a file to open it, `e` to edit in `$EDITOR`.
- **Width-responsive table** — adapts columns (Name, App ID, Size, Library, Last Played) to terminal width.
- **Fast** — instant startup with a local cache, pure local/offline operation.
- **Single binary** — no runtime dependencies.

## Screenshots

*Coming soon.*

## Installation

### From source

Requires **Rust 1.94+**.

```sh
git clone https://github.com/your-user/steam-proton-browser.git
cd steam-proton-browser
make install
```

This installs the binary to `~/.cargo/bin/steam-proton-browser`.

### Debug build

```sh
make build
make run
```

## Usage

```
steam-proton-browser [OPTIONS]

Options:
  --steam-path <PATH>   Override Steam installation path
  --refresh             Force a full rescan (ignore cache)
  -h, --help            Print help
  -V, --version         Print version
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Drill into selection / open file |
| `Esc` / `Backspace` | Go back |
| `o` | Open in system file explorer |
| `e` | Edit file in `$EDITOR` |
| `y` | Copy path to clipboard |
| `/` | Fuzzy filter |
| `s` | Cycle sort mode |
| `R` | Refresh / rescan |
| `?` | Toggle help |
| `q` | Quit |

### Navigation

```
Library View ──Enter──▶ Game Detail View ──Enter──▶ Folder Browser
     ◀──Esc──                  ◀──Esc──
```

## Folder Categories

When you select a game, Steam Proton Browser shows all existing folders for that title:

- **Install Directory** — the game's install location
- **Proton Prefix** — the compatdata directory (Wine/Proton virtual filesystem)
- **drive_c** — the virtual C: drive inside the Proton prefix
- **AppData** — the user's AppData inside the prefix
- **Shader Cache** — Vulkan/GL shader cache
- **Workshop Content** — Steam Workshop mods
- **Cloud Saves** — Steam cloud sync directory
- **Custom Compatibility Tools** — user-installed Proton/Wine builds

## Platform Support

| Platform | Status |
|----------|--------|
| Linux (native Steam) | ✅ Tested |
| Linux (Flatpak Steam) | 🔧 Untested, should work |
| Linux (Snap Steam) | 🔧 Untested, should work |
| Steam Deck | 🔧 Untested, should work |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions and contribution guidelines.

```sh
make setup   # Install pre-commit hooks (one-time)
make help    # Show all available targets
make build   # Debug build
make test    # Run tests
make lint    # Run clippy
make fmt     # Format code
```

## License

This project is licensed under the **BSD 3-Clause License** — see the [LICENSE](LICENSE) file for details.
