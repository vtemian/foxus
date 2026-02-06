# Development Guide

This guide covers local development setup, building, and testing for Foxus.

## Prerequisites

### All Platforms

- [Rust](https://rustup.rs/) 1.70 or later
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli
```

### macOS

No additional dependencies required. The app uses native Cocoa APIs via the `objc2` crate.

**Note**: You'll need to grant Accessibility permissions for window tracking to work:
1. System Preferences → Security & Privacy → Privacy → Accessibility
2. Add your terminal or the Foxus app

### Linux (X11)

Install X11 development libraries:

```bash
# Debian/Ubuntu
sudo apt install libx11-dev libxss-dev libwebkit2gtk-4.1-dev libgtk-3-dev

# Fedora
sudo dnf install libX11-devel libXScrnSaver-devel webkit2gtk4.1-devel gtk3-devel

# Arch
sudo pacman -S libx11 libxss webkit2gtk-4.1 gtk3
```

## Project Structure

```
foxus/
├── src/                    # Frontend (HTML/CSS/JS)
│   ├── index.html
│   ├── app.js
│   └── style.css
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # App setup, Tauri integration
│   │   ├── commands.rs     # Tauri IPC commands
│   │   ├── db/             # Database layer
│   │   ├── models/         # Data models (Activity, Category, Rule, etc.)
│   │   ├── tracker/        # Activity tracking service
│   │   ├── categorizer/    # App/URL categorization engine
│   │   ├── focus/          # Focus session management
│   │   ├── platform/       # Platform-specific code (macOS, Linux)
│   │   └── native_host/    # Chrome native messaging host
│   ├── Cargo.toml
│   └── tauri.conf.json
├── extension/              # Chrome extension
│   ├── manifest.json
│   ├── background.js       # Service worker
│   ├── blocked.html/js     # Block page
│   └── popup/              # Extension popup
└── docs/                   # Design documents
```

## Building

### Development Build

```bash
cd src-tauri

# Run with hot-reload (recommended for development)
cargo tauri dev

# Or just build the Rust backend
cargo build
```

### Release Build

```bash
cd src-tauri
cargo tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Native Messaging Host

The native messaging host binary is built alongside the main app:

```bash
cd src-tauri
cargo build --bin foxus-native-host
```

## Testing

### Run All Tests

```bash
cd src-tauri
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test Module

```bash
# Test a specific module
cargo test tracker::tests
cargo test focus::tests
cargo test categorizer::tests

# Test a specific test
cargo test test_tracker_starts_and_stops
```

### Test Coverage by Module

| Module | What's Tested |
|--------|---------------|
| `db/` | Database creation, migrations, table schema |
| `models/` | CRUD operations, data integrity |
| `tracker/` | Start/stop, activity recording |
| `categorizer/` | Pattern matching, rule priority |
| `focus/` | Session lifecycle, budget tracking, domain blocking |

### Ignored Tests

Some tests require platform-specific features and are ignored by default:

```bash
# Run including ignored tests (requires GUI/permissions)
cargo test -- --ignored
```

## Linting

### Rust

```bash
cd src-tauri

# Check for warnings
cargo check

# Run clippy for more thorough linting
cargo clippy

# Auto-fix clippy suggestions
cargo clippy --fix
```

### Format Code

```bash
cargo fmt
```

## Chrome Extension Development

### Load Extension

1. Open `chrome://extensions` in Chrome
2. Enable "Developer mode" (top right)
3. Click "Load unpacked"
4. Select the `extension/` directory

### Reload After Changes

After modifying extension files, click the refresh icon on the extension card in `chrome://extensions`.

### View Logs

- **Background script**: Click "service worker" link on extension card → opens DevTools
- **Popup**: Right-click extension icon → Inspect popup
- **Blocked page**: Standard DevTools (F12)

### Native Messaging Setup

For the extension to communicate with the desktop app, you need to register the native messaging host:

#### macOS

```bash
# Create the manifest directory
mkdir -p ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts

# Create the manifest file
cat > ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.foxus.native.json << 'EOF'
{
  "name": "com.foxus.native",
  "description": "Foxus Native Messaging Host",
  "path": "/path/to/foxus/src-tauri/target/debug/foxus-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

Replace `/path/to/foxus` with your actual path and `YOUR_EXTENSION_ID` with the extension ID shown in `chrome://extensions`.

#### Linux

```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts

cat > ~/.config/google-chrome/NativeMessagingHosts/com.foxus.native.json << 'EOF'
{
  "name": "com.foxus.native",
  "description": "Foxus Native Messaging Host",
  "path": "/path/to/foxus/src-tauri/target/debug/foxus-native-host",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

## Database

### Location

- **macOS**: `~/Library/Application Support/com.foxus.Foxus/foxus.db`
- **Linux**: `~/.local/share/foxus/foxus.db`

### Inspect Database

```bash
# macOS
sqlite3 ~/Library/Application\ Support/com.foxus.Foxus/foxus.db

# Linux
sqlite3 ~/.local/share/foxus/foxus.db
```

### Useful Queries

```sql
-- View recent activities
SELECT * FROM activities ORDER BY timestamp DESC LIMIT 20;

-- View categories
SELECT * FROM categories;

-- View rules
SELECT * FROM rules ORDER BY priority DESC;

-- View active focus session
SELECT * FROM focus_sessions WHERE ended_at IS NULL;

-- Daily summary
SELECT
  c.name,
  c.productivity,
  SUM(a.duration_secs) / 60 as minutes
FROM activities a
JOIN categories c ON a.category_id = c.id
WHERE a.timestamp >= strftime('%s', 'now', 'start of day')
GROUP BY c.id
ORDER BY minutes DESC;
```

### Reset Database

Delete the database file to start fresh (categories and rules will be re-seeded):

```bash
# macOS
rm ~/Library/Application\ Support/com.foxus.Foxus/foxus.db

# Linux
rm ~/.local/share/foxus/foxus.db
```

## Debugging

### Enable Logging

Set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo tauri dev
RUST_LOG=foxus=trace cargo tauri dev
```

### Common Issues

#### "Accessibility permission denied" (macOS)

Grant permission in System Preferences → Security & Privacy → Privacy → Accessibility.

#### Extension shows "Native host disconnected"

1. Check that `foxus-native-host` is built: `cargo build --bin foxus-native-host`
2. Verify the native messaging manifest path is correct
3. Check the extension ID matches in the manifest

#### Window tracking not working (Linux)

Ensure you're running X11, not Wayland. Check with:

```bash
echo $XDG_SESSION_TYPE
```

## Architecture Notes

### Tracker Service

- Polls active window every 5 seconds (configurable)
- Detects idle time > 2 minutes (configurable) and pauses tracking
- Runs in a background thread, communicates via `Arc<Mutex<_>>`

### Categorizer

- Loads rules from database on startup
- Matches apps/URLs against patterns with priority ordering
- Supports wildcard patterns (`*slack*`, `*.github.com`)

### Focus Manager

- Manages focus session lifecycle (start/end)
- Tracks distraction budget usage
- Provides blocked domain list to extension

### Native Messaging Host

- Standalone binary for Chrome extension communication
- Uses stdin/stdout with length-prefixed JSON messages
- Shares database with main app
