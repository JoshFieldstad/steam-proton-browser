# Contributing

## ⚠️ Disclaimer

This is a personal project. It is **not supported software**. There are no guarantees of:

- Timely/any responses to issues or pull requests
- Acceptance of any contributions
- Continued maintenance or development

Pull requests **may be completely ignored** without explanation. This is not meant to be a community project, if you personally have a desire to make it one I wish you the best and give my full emotional support within what is allowed by the license.

---

## Building from Source

### Prerequisites

- **Rust 1.85+** — install via [rustup](https://rustup.rs/)
- **GNU Make**- **pre-commit** — install via `pip install pre-commit`
### Build & Run

```sh
# First-time setup — install pre-commit hooks
make setup

# Debug build
make build

# Run the app
make run

# Optimized release build
make release

# Install to ~/.cargo/bin
make install
```

### All Make Targets

```sh
make help
```

| Target | Description |
|--------|-------------|
| `setup` | Install pre-commit hooks |
| `build` | Compile debug build |
| `release` | Compile optimized release build |
| `run` | Build and run (debug) |
| `install` | Install binary to `~/.cargo/bin` |
| `test` | Run all tests |
| `lint` | Run clippy with strict warnings |
| `fmt` | Format all source files |
| `fmt-check` | Check formatting (CI mode) |
| `check` | Type-check without producing binaries |
| `audit` | Audit dependencies for vulnerabilities |
| `licenses` | Regenerate THIRD-PARTY-LICENSES |
| `watch` | Rebuild on file changes |
| `clean` | Remove build artifacts |
| `ci` | Run the full CI check suite |

### Running Tests

```sh
make test
```

### Linting & Formatting

```sh
make lint    # clippy — must pass with zero warnings
make fmt     # auto-format with rustfmt
```

## Code Style

- All repo operations go through `make` targets — don't run raw `cargo` commands in CI or docs.
- Run `make setup` after cloning to install pre-commit hooks.
- `make ci` must pass before any PR is considered.
- No new warnings allowed — `make lint` enforces `-D warnings`.

## Project Structure

```
src/
├── main.rs                 # Entry point, CLI parsing, startup flow
├── cache.rs                # TOML-based path cache
├── steam/
│   ├── acf.rs              # VDF/ACF key-value parser
│   ├── discovery.rs        # Auto-detect Steam install paths
│   ├── library.rs          # Enumerate installed games
│   └── folders.rs          # Resolve per-game folder categories
├── tui/
│   ├── app.rs              # Main event loop, view state machine
│   ├── views.rs            # View rendering
│   ├── widgets.rs          # Reusable widget helpers
│   ├── keybindings.rs      # Key → Action mapping
│   └── theme.rs            # Color scheme
├── config/
│   ├── settings.rs         # User configuration struct
│   └── persistence.rs      # Config load/save
└── platform/
    ├── explorer.rs         # OS file explorer / editor integration
    └── paths.rs            # Platform-specific path helpers
```
