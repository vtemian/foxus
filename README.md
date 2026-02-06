# Foxus

A local-first productivity tracker for macOS and Linux with focus mode enforcement.

## Overview

Foxus is a RescueTime-style productivity tracker that monitors active applications and browser activity, categorizes time as productive/neutral/distracting, and displays insights through a native menu bar interface. The core feature is **Focus Mode** - automatic blocking of distracting websites with a soft-block distraction budget system.

## Features

- **Automatic Activity Tracking**: Monitors active applications and browser tabs
- **Smart Categorization**: Auto-categorizes apps and websites as productive, neutral, or distracting
- **Focus Mode**: Block distracting sites with a configurable "distraction budget"
- **Privacy-First**: All data stays local in SQLite - no cloud sync, no telemetry
- **Cross-Platform**: Works on macOS and Linux (X11)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Foxus Desktop App                    │
│                    (Rust + Tauri)                       │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │  Tracker    │  │  Categorizer │  │  Native Msg    │ │
│  │  Service    │  │  Engine      │  │  Host          │ │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         │                │                   │          │
│         └────────────────┼───────────────────┘          │
│                          ▼                              │
│                   ┌─────────────┐                       │
│                   │   SQLite    │                       │
│                   │   Database  │                       │
│                   └─────────────┘                       │
├─────────────────────────────────────────────────────────┤
│               Tauri Menu Bar / System Tray              │
└─────────────────────────────────────────────────────────┘
         ▲
         │ Native Messaging
         ▼
┌─────────────────────────────────────────────────────────┐
│              Chrome Extension                           │
│  - Tracks active tab URL/title                          │
│  - Enforces blocking rules                              │
└─────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- Platform-specific dependencies (see [Development Guide](DEVELOPMENT.md))

### Build & Run

```bash
# Clone the repository
git clone https://github.com/vtemian/foxus.git
cd foxus

# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
cd src-tauri
cargo tauri dev
```

### Install Chrome Extension

1. Open Chrome and navigate to `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked" and select the `extension/` directory

## How Focus Mode Works

1. **Start a session** from the menu bar or let it auto-start on schedule
2. **Distracting sites are blocked** when you try to visit them
3. **Soft block with budget**: You get a configurable amount of "distraction time"
   - First visit to a blocked site shows a warning with remaining budget
   - Click "Use distraction time" → 30-second countdown → access granted
   - Time on distracting sites counts against your budget
   - Budget exhausted → hard block until session ends

## Data Storage

All data is stored locally:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/com.foxus.Foxus/` |
| Linux | `~/.local/share/foxus/` |

## Default Categories

Seeded on first run:

- **Coding** (productive): VSCode, terminal emulators, GitHub
- **Communication** (neutral): Slack, Discord, email clients
- **Entertainment** (distracting): YouTube, Netflix, Twitter/X, Reddit
- **Reference** (productive): Stack Overflow, documentation sites

## Platform Permissions

### macOS

- **Accessibility permission** required for window tracking
- System will prompt on first run

### Linux

- **X11**: Works out of the box
- **Wayland**: Limited support (may need compositor-specific permissions)

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for local development setup and testing guide.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop app | Rust + Tauri 2.0 |
| Database | SQLite (rusqlite) |
| UI | HTML/CSS/JS |
| Chrome extension | Manifest V3 |
| macOS APIs | objc2 crate |
| Linux APIs | x11rb |

## License

MIT
