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

.PHONY: coverage
coverage: ## Run tests with code coverage summary
	cargo llvm-cov test --text

.PHONY: coverage-report
coverage-report: ## Generate coverage files for CI (codecov.json + coverage.txt)
	cargo llvm-cov test --codecov --output-path codecov.json
	cargo llvm-cov test --text --output-path coverage.txt

.PHONY: test-report
test-report: ## Generate JUnit XML test report
	cargo nextest run --profile ci

# ── Help ───────────────────────────────────────────────

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
