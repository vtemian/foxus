# Foxus: Project Rules

## What is Foxus?

Local-first productivity tracker for macOS and Linux with focus mode enforcement. Monitors active applications and browser activity, categorizes time as productive/neutral/distracting, and displays insights through a native menu bar interface with a Chrome extension for website blocking.

## Writing

- Never use em dashes. Use colons for definitions, commas or parentheses for asides.

## Engineering Principles

- DRY: extract shared patterns, no copy-paste
- YAGNI: no speculative features or unused abstractions
- Fail fast: validate inputs early, return/throw before the happy path
- Errors are values: structured `AppError` types with context, no bare `catch {}`
- Name by what it does in the domain, not how it's implemented
- Comments explain *why*, never temporal context or what changed

## Architecture

### Three Codebases

1. **Frontend** (`src/web/`) - React 18 + TypeScript + Tailwind 4 + Vite 5. See `src/web/CLAUDE.md`
2. **Backend** (`src/tauri/`) - Rust (Tauri 2.x). See `src/tauri/CLAUDE.md`
3. **Chrome Extension** (`extension/`) - Manifest V3, vanilla JS. Communicates with backend via native messaging

### Data Flow

1. Tracker polls active window every 5 seconds
2. Categorizer matches window against rules (app name, domain, title)
3. Activity recorded to SQLite with category and productivity level
4. Frontend queries stats via Tauri commands
5. Focus mode enforces distraction budgets, extension blocks sites

## Tooling

```bash
make install          # install all dependencies
make dev              # start Tauri dev (Vite auto-starts)
make kill-dev         # kill all dev processes
make check            # full pipeline (frontend quality gate + Rust lint + fmt)
make lint             # format check + clippy + frontend linting
make lint-rust        # clippy --all-targets -D warnings
make fmt              # format all code (frontend + Rust)
make test             # run all tests (Rust + frontend)
```

CI runs clippy, fmt check, ESLint, Biome, tsc, and tests on every push/PR to main (`.github/workflows/quality-gate.yml`).

## File Paths

- User data: `~/Library/Application Support/com.foxus.Foxus/` (macOS) or `~/.local/share/foxus/` (Linux)
- Database: `foxus.db` in user data dir
- Chrome extension: `extension/`
