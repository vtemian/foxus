.PHONY: all install dev build clean check test test-rust test-frontend lint lint-rust lint-frontend fmt fmt-check fmt-frontend kill-dev help

# Default target
all: install check

# Install all dependencies
install:
	npm install
	cd src/tauri && cargo fetch

# Kill running dev processes
kill-dev:
	@-lsof -ti :1420 | xargs kill 2>/dev/null || true
	@-pkill -f "cargo tauri dev" 2>/dev/null || true
	@-pkill -9 -f "target/debug/foxus" 2>/dev/null || true
	@-pkill -9 -f "node.*vite" 2>/dev/null || true

# Run in development mode
dev: kill-dev
	cd src/tauri && cargo tauri dev

# Build for production
build:
	cd src/tauri && cargo tauri build

# ── Quality Gate ─────────────────────────────────────────────────────

# Full frontend quality gate (biome + eslint + tsc + tests)
check-frontend:
	npm run check

# Lint frontend (Biome + ESLint)
lint-frontend:
	npm run lint

# Format frontend
fmt-frontend:
	npm run format

# Rust quality gate (clippy with deny on all warnings)
lint-rust:
	cd src/tauri && cargo clippy --all-targets -- -D warnings

# Check Rust formatting
fmt-check:
	cd src/tauri && cargo fmt --all --check

# Full lint (format check + clippy + frontend linting)
lint: fmt-check lint-frontend lint-rust

# Run all checks (frontend quality gate + Rust lint + fmt check)
check: check-frontend lint-rust fmt-check

# ── Testing ──────────────────────────────────────────────────────────

# Run Rust tests
test-rust:
	cd src/tauri && cargo test

# Run frontend tests
test-frontend:
	npm test

# Run all tests
test: test-rust test-frontend

# ── Formatting ───────────────────────────────────────────────────────

# Format all code
fmt: fmt-frontend
	cd src/tauri && cargo fmt --all

# ── Cleanup ──────────────────────────────────────────────────────────

# Clean build artifacts
clean:
	rm -rf dist
	rm -rf src/tauri/target
	rm -rf node_modules

# ── Help ─────────────────────────────────────────────────────────────

help:
	@echo "Foxus Development Commands"
	@echo ""
	@echo "  make install        - Install all dependencies"
	@echo "  make dev            - Run in development mode"
	@echo "  make build          - Build for production"
	@echo "  make kill-dev       - Kill all dev processes"
	@echo ""
	@echo "  make check          - Full quality gate (frontend + Rust)"
	@echo "  make lint           - Full lint (format check + clippy + frontend)"
	@echo "  make lint-rust      - Clippy with deny on warnings"
	@echo "  make lint-frontend  - Biome + ESLint"
	@echo "  make fmt            - Format all code (frontend + Rust)"
	@echo "  make fmt-check      - Check formatting without changes"
	@echo ""
	@echo "  make test           - Run all tests (Rust + frontend)"
	@echo "  make test-rust      - Run Rust tests only"
	@echo "  make test-frontend  - Run frontend tests only"
	@echo ""
	@echo "  make clean          - Clean build artifacts"
