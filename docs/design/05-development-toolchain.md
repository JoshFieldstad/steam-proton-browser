# Steam Proton Browser — Development Toolchain & Repo Operations

## Required Toolchain

### Rust

Install via [rustup](https://rustup.rs/) (the official Rust toolchain manager):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

| Component | Minimum Version | Notes |
|-----------|----------------|-------|
| `rustc` | 1.75+ | Stable channel. MSRV (minimum supported Rust version) will be pinned in `Cargo.toml` via `rust-version`. |
| `cargo` | (bundled with rustc) | Build system and package manager. |
| `rustfmt` | (rustup component) | `rustup component add rustfmt` — enforced in CI. |
| `clippy` | (rustup component) | `rustup component add clippy` — enforced in CI. |

### Cross-Compilation Targets (optional)

```sh
# Linux static binary (musl)
rustup target add x86_64-unknown-linux-musl
```

### System Dependencies

| Platform | Packages | Purpose |
|----------|----------|---------|
| Linux (Debian/Ubuntu) | `build-essential`, `pkg-config` | C linker, build tools. |
| Linux (musl builds) | `musl-tools` | Static linking via musl. |

### Optional Dev Tools

| Tool | Install | Purpose |
|------|---------|---------|
| `cargo-watch` | `cargo install cargo-watch` | Auto-rebuild on file changes during development. |
| `cargo-nextest` | `cargo install cargo-nextest` | Faster test runner with better output. |
| `cargo-deny` | `cargo install cargo-deny` | Audit dependencies for licenses and vulnerabilities. |
| `cargo-release` | `cargo install cargo-release` | Automate version bumping and publishing. |

---

## Repo Operations — Makefile Constraint

**All common repo operations MUST be driven through the project `Makefile`.** Direct `cargo` commands are fine for ad-hoc exploration, but any operation that CI runs, documentation references, or a contributor needs to reproduce should have a `make` target.

### Rationale

- **Single source of truth** — contributors run `make build`, not a cargo command with flags they need to remember or look up.
- **Reproducibility** — CI and local dev use identical commands.
- **Abstraction** — if we add pre/post steps (codegen, asset copying, cache warming), the `make` target absorbs them without changing contributor workflows.
- **Discoverability** — `make help` lists every available operation.
- **Language-agnostic entry point** — even if parts of the project evolve beyond pure Rust (scripts, docs tooling, etc.), `make` stays the universal interface.

### Required Make Targets

Every one of these targets MUST exist in the root `Makefile`:

| Target | Action | Backing Command(s) |
|--------|--------|---------------------|
| `make build` | Compile debug build | `cargo build` |
| `make release` | Compile optimized release build | `cargo build --release` |
| `make run` | Build and run the app (debug) | `cargo run` |
| `make test` | Run all tests | `cargo nextest run` (fallback: `cargo test`) |
| `make lint` | Run clippy with strict warnings | `cargo clippy -- -D warnings` |
| `make fmt` | Format all source files | `cargo fmt` |
| `make fmt-check` | Check formatting (CI mode, no writes) | `cargo fmt -- --check` |
| `make check` | Type-check without producing binaries | `cargo check` |
| `make clean` | Remove build artifacts | `cargo clean` |
| `make audit` | Run dependency audit | `cargo deny check` |
| `make watch` | Rebuild on file changes | `cargo watch -x build` |
| `make install` | Install binary to `~/.cargo/bin` | `cargo install --path .` |
| `make ci` | Run the full CI pipeline locally | `make fmt-check lint test` |
| `make help` | Print all available targets | Auto-generated from target comments |

### Makefile Conventions

1. **Self-documenting** — each target must have a `## description` comment so `make help` can auto-generate a help listing:
   ```makefile
   build: ## Compile debug build
   	cargo build
   ```

2. **`help` as default target** — running bare `make` prints the help listing.

3. **No hidden flags** — all cargo flags, feature toggles, and environment variables used in a target must be visible in the `Makefile`, not buried in shell scripts or CI YAML.

4. **Phony targets** — all targets must be declared `.PHONY` since they don't produce files with matching names.

5. **Fail-fast** — use `set -e` or `&&` chaining in multi-command recipes so failures propagate immediately.

### Starter Makefile

```makefile
.DEFAULT_GOAL := help

# ── Build ──────────────────────────────────────────────

.PHONY: build
build: ## Compile debug build
	cargo build

.PHONY: release
release: ## Compile optimized release build
	cargo build --release

.PHONY: run
run: ## Build and run (debug)
	cargo run

.PHONY: install
install: ## Install binary to ~/.cargo/bin
	cargo install --path .

# ── Quality ────────────────────────────────────────────

.PHONY: test
test: ## Run all tests
	cargo nextest run 2>/dev/null || cargo test

.PHONY: lint
lint: ## Run clippy with strict warnings
	cargo clippy -- -D warnings

.PHONY: fmt
fmt: ## Format all source files
	cargo fmt

.PHONY: fmt-check
fmt-check: ## Check formatting (CI mode)
	cargo fmt -- --check

.PHONY: check
check: ## Type-check without producing binaries
	cargo check

.PHONY: audit
audit: ## Audit dependencies for vulnerabilities and license issues
	cargo deny check

# ── Dev ────────────────────────────────────────────────

.PHONY: watch
watch: ## Rebuild on file changes
	cargo watch -x build

# ── Maintenance ────────────────────────────────────────

.PHONY: clean
clean: ## Remove build artifacts
	cargo clean

# ── CI ─────────────────────────────────────────────────

.PHONY: ci
ci: fmt-check lint test ## Run full CI pipeline locally

# ── Help ───────────────────────────────────────────────

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
```

### Contributor Expectations

- **README and CONTRIBUTING docs** should reference `make` targets, never raw `cargo` commands.
- **CI workflows** (GitHub Actions) must call `make ci` rather than invoking `cargo` directly — this guarantees CI and local dev are always in sync.
- **New operations** (benchmarks, coverage, docs generation, etc.) must be added as `make` targets before being referenced anywhere.
