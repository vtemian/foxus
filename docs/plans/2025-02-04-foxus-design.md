# Foxus Design Document

A local-first productivity tracker for macOS and Linux with focus mode enforcement.

## Overview

Foxus is a RescueTime-style productivity tracker that monitors active applications and browser activity, categorizes time as productive/neutral/distracting, and displays insights through a native menu bar interface. The core feature is **Focus Mode** - automatic blocking of distracting websites with a soft-block distraction budget system.

## Goals

- Track time spent on applications and websites automatically
- Categorize activity as productive, neutral, or distracting
- Provide daily/weekly productivity reports
- Enforce focus sessions with intelligent site blocking
- Privacy-first: all data stays local

## Non-Goals (YAGNI)

- Cloud sync or multi-device support
- Windows support (initially)
- Mobile apps
- Team/organization features
- Detailed website content analysis

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Foxus Desktop App                     │
│                    (Rust + Tauri)                        │
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
│               (HTML/CSS/JS popup UI)                    │
└─────────────────────────────────────────────────────────┘
         ▲
         │ Native Messaging
         ▼
┌─────────────────────────────────────────────────────────┐
│              Chrome Extension                            │
│  - Tracks active tab URL/title                          │
│  - Sends data to native host                            │
│  - Enforces blocking rules                              │
└─────────────────────────────────────────────────────────┘
```

### Key Components

- **Tracker Service**: Polls active window every 5 seconds, detects idle state
- **Categorizer Engine**: Maps apps/URLs to categories using rules + user overrides
- **Native Messaging Host**: Receives browser data from Chrome extension
- **SQLite Database**: Stores all activity data locally
- **Tray UI**: Quick stats view, settings, category management

## Data Model

### Core Tables

```sql
-- Activity records (one per tracking interval)
activities (
    id              INTEGER PRIMARY KEY,
    timestamp       INTEGER NOT NULL,     -- Unix timestamp
    duration_secs   INTEGER NOT NULL,     -- Usually 5 seconds per record
    source          TEXT NOT NULL,        -- 'app' or 'browser'
    app_name        TEXT,                 -- e.g., "Code", "Slack"
    window_title    TEXT,                 -- e.g., "main.rs - foxus"
    url             TEXT,                 -- For browser activity
    domain          TEXT,                 -- Extracted from URL
    category_id     INTEGER REFERENCES categories(id)
)

-- User-defined categories
categories (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,        -- e.g., "Coding", "Communication"
    productivity    INTEGER NOT NULL      -- -1 = distracting, 0 = neutral, 1 = productive
)

-- Rules for auto-categorization
rules (
    id              INTEGER PRIMARY KEY,
    pattern         TEXT NOT NULL,        -- Glob or regex for app/domain
    match_type      TEXT NOT NULL,        -- 'app', 'domain', 'title'
    category_id     INTEGER REFERENCES categories(id),
    priority        INTEGER DEFAULT 0     -- Higher = takes precedence
)

-- Focus session tracking
focus_sessions (
    id                  INTEGER PRIMARY KEY,
    started_at          INTEGER NOT NULL,
    ended_at            INTEGER,              -- NULL if active
    scheduled           BOOLEAN DEFAULT FALSE,
    distraction_budget  INTEGER NOT NULL,     -- Seconds allowed
    distraction_used    INTEGER DEFAULT 0     -- Seconds consumed
)

-- Recurring focus schedules
focus_schedules (
    id                  INTEGER PRIMARY KEY,
    days_of_week        TEXT NOT NULL,        -- e.g., "1,2,3,4,5" (Mon-Fri)
    start_time          TEXT NOT NULL,        -- e.g., "09:00"
    end_time            TEXT NOT NULL,        -- e.g., "12:00"
    distraction_budget  INTEGER NOT NULL,
    enabled             BOOLEAN DEFAULT TRUE
)

-- Sites to block (for Chrome extension)
blocked_sites (
    id              INTEGER PRIMARY KEY,
    domain          TEXT NOT NULL,
    schedule        TEXT                  -- JSON: when to block (null = always)
)
```

### Default Categories

Seeded on first run:

- **Coding** (productive): VSCode, terminal emulators, GitHub
- **Communication** (neutral): Slack, Discord, email clients
- **Entertainment** (distracting): YouTube, Netflix, Twitter/X, Reddit
- **Reference** (productive): Stack Overflow, documentation sites

### Storage Location

- macOS: `~/Library/Application Support/foxus/`
- Linux: `~/.local/share/foxus/`

## Focus Mode

Focus mode is the core feature. Everything else supports this.

### How It Works

1. **Activation**: Start manually from menu bar OR auto-start on schedule
2. **During session**: All sites categorized as "distracting" are blocked
3. **Soft block with budget**: User gets X minutes of "distraction allowance" per session
   - First visit to blocked site → warning page with remaining budget
   - Click "Use distraction time" → 30-second countdown, then access granted
   - Time on distracting sites counts against budget
   - Budget exhausted → hard block until session ends

### Block Page UI

```
┌─────────────────────────────────────────────┐
│                                             │
│     🎯 Focus Mode Active                    │
│                                             │
│     reddit.com is blocked                   │
│                                             │
│     Distraction budget: 12:34 remaining     │
│                                             │
│     ┌─────────────────────────────────┐    │
│     │   Use distraction time (30s)    │    │
│     └─────────────────────────────────┘    │
│                                             │
│     Session ends at 12:00 PM                │
│                                             │
└─────────────────────────────────────────────┘
```

## Chrome Extension

### Structure

```
foxus-extension/
├── manifest.json          # Manifest V3
├── background.js          # Service worker - native messaging, state management
├── content.js             # Injected into pages - detects active tab time
├── blocked.html           # The "you're in focus mode" interstitial
├── blocked.js             # Countdown timer, budget logic
└── popup/                 # Quick status when clicking extension icon
    ├── popup.html
    └── popup.js
```

### Native Messaging Protocol

```typescript
// Extension → Native Host
{ type: "activity", url: string, title: string, timestamp: number }
{ type: "request_state" }  // Get current focus mode status
{ type: "use_distraction_time" }  // User clicked "use budget"

// Native Host → Extension
{ type: "state", focusActive: boolean, budgetRemaining: number, blockedDomains: string[] }
{ type: "budget_updated", remaining: number }
{ type: "hard_blocked" }  // Budget exhausted
```

### Permissions Required

- `tabs` - read URLs and titles
- `webNavigation` - intercept navigation to blocked sites
- `nativeMessaging` - communicate with desktop app
- `storage` - cache state when native host unreachable

### Offline Behavior

If native host is unreachable, extension uses cached block list and last known focus state. Syncs when connection restored.

## Desktop Tracking

### Platform-Specific APIs

| Platform | Window Tracking | Idle Detection |
|----------|-----------------|----------------|
| macOS | `NSWorkspace` via `objc` crate | `CGEventSourceSecondsSinceLastEventType` |
| Linux (X11) | `xcb` | `XScreenSaverQueryInfo` |
| Linux (Wayland) | `wlr-foreign-toplevel` | `ext-idle-notify-v1` |

### Tracking Loop

```rust
loop {
    let active_window = platform::get_active_window();
    let is_idle = platform::get_idle_time() > IDLE_THRESHOLD;

    if is_idle {
        continue;  // Don't record, keep checking
    }

    let category = categorizer.categorize(&active_window);

    db.record_activity(Activity {
        timestamp: now(),
        duration_secs: POLL_INTERVAL,
        source: "app",
        app_name: active_window.app_name,
        window_title: active_window.title,
        category_id: category.id,
    });

    // Update focus mode distraction tracking
    if let Some(session) = db.active_focus_session() {
        if category.productivity < 0 {
            db.increment_distraction_used(session.id, POLL_INTERVAL);
        }
    }

    sleep(POLL_INTERVAL);
}
```

### Configuration

- **Polling interval**: 5 seconds (configurable)
- **Idle threshold**: 2 minutes (configurable)

## Menu Bar UI

### Normal Mode Popup

```
┌─────────────────────────────────────┐
│  Today                    ▼ Week    │
├─────────────────────────────────────┤
│                                     │
│  Productive      ████████░░  4h 12m │
│  Neutral         ███░░░░░░░  1h 30m │
│  Distracting     █░░░░░░░░░    28m  │
│                                     │
├─────────────────────────────────────┤
│  Top Apps                           │
│  ─────────────────────────────────  │
│  VSCode              2h 45m    🟢   │
│  Terminal            1h 12m    🟢   │
│  Slack                 58m    🟡   │
│  Chrome                43m    ⚪   │
│  Twitter               18m    🔴   │
│                                     │
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │    Start Focus Session          ││
│  └─────────────────────────────────┘│
│                                     │
│  ⚙️ Settings                        │
└─────────────────────────────────────┘
```

### Focus Mode Popup

```
┌─────────────────────────────────────┐
│  🎯 Focus Mode Active               │
├─────────────────────────────────────┤
│                                     │
│  Session: 1h 23m / 3h               │
│  ████████████░░░░░░░░               │
│                                     │
│  Distraction budget: 8:32 left      │
│  ██████░░░░░░░░░░░░░░               │
│                                     │
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │    End Focus Session            ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### Settings Panel

- Category management (add/edit/delete)
- App/domain rules (assign to categories)
- Focus schedules (create/edit recurring sessions)
- Default distraction budget
- Idle threshold
- Tracking interval

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop app | Rust + Tauri |
| Database | SQLite (via rusqlite) |
| Popup UI | HTML/CSS/JS (vanilla or Preact) |
| Chrome extension | Manifest V3, vanilla JS |
| macOS APIs | objc crate |
| Linux APIs | xcb, wayland-client |

## Error Handling

| Scenario | Handling |
|----------|----------|
| Native host crashes | Extension uses cached state, shows "reconnecting" |
| Extension not installed | Desktop tracks apps only, prompts to install extension |
| Database corrupted | Backup on startup, restore from backup if open fails |
| Focus schedule overlap | Later schedule takes precedence, warn in settings |
| Permission denied | Show setup guide on first run, check on startup |

## Platform Permissions

### macOS

- Accessibility permission required for window tracking
- Prompt user on first run with setup guide

### Linux

- X11: works out of the box
- Wayland: may need compositor-specific permissions

## Testing Strategy

| Layer | Approach |
|-------|----------|
| Tracker service | Unit tests with mocked platform APIs |
| Categorizer | Unit tests with fixture rules/apps |
| Native messaging | Integration tests with mock extension |
| Chrome extension | Jest + chrome-mock for unit tests |
| Focus mode logic | Unit tests for state machine |
| Full system | Manual E2E on macOS + Linux |

### MVP Test Coverage Focus

1. Focus mode state transitions
2. Distraction budget accounting
3. Block list sync between native ↔ extension
4. Activity categorization accuracy

## MVP Summary

- Rust + Tauri desktop app (macOS + Linux)
- Menu bar/system tray with quick stats popup
- Chrome extension with native messaging
- Local SQLite storage
- Focus mode with soft blocking + distraction budget
- Auto-categorization with user overrides
- Daily/weekly productivity reports
