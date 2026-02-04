# Foxus Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a local-first productivity tracker with focus mode for macOS and Linux.

**Architecture:** Rust + Tauri desktop app with SQLite storage, Chrome extension communicating via native messaging. Menu bar UI shows stats and controls focus sessions. Focus mode blocks distracting sites with a soft-block distraction budget.

**Tech Stack:** Rust, Tauri 2.x, SQLite (rusqlite), Chrome Extension Manifest V3, HTML/CSS/JS for UI

---

## Phase 1: Project Setup + Core Data Layer

### Task 1: Initialize Tauri Project

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `package.json`
- Create: `src/index.html`

**Step 1: Install Tauri CLI**

Run: `cargo install tauri-cli`
Expected: Tauri CLI installed

**Step 2: Create Tauri project**

Run: `cargo tauri init`

When prompted:
- App name: `foxus`
- Window title: `Foxus`
- Web assets location: `../src`
- Dev server URL: leave empty
- Dev command: leave empty
- Build command: leave empty

Expected: `src-tauri/` directory created with Cargo.toml, tauri.conf.json, src/main.rs

**Step 3: Create minimal frontend**

Create `src/index.html`:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Foxus</title>
</head>
<body>
  <h1>Foxus</h1>
</body>
</html>
```

**Step 4: Verify build works**

Run: `cargo tauri build --debug`
Expected: Builds successfully, creates app bundle

**Step 5: Commit**

```bash
git add .
git commit -m "chore: initialize Tauri project"
```

---

### Task 2: Add SQLite Database Schema

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add rusqlite dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
directories = "5.0"
```

**Step 2: Write failing test for database initialization**

Create `src-tauri/src/db/mod.rs`:
```rust
pub mod schema;
pub mod migrations;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_opens() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        assert!(db_path.exists());
    }

    #[test]
    fn test_migrations_run() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(&db.connection()).unwrap();

        // Verify tables exist
        let count: i32 = db.connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='categories'",
                [],
                |row| row.get(0)
            ).unwrap();
        assert_eq!(count, 1);
    }
}
```

**Step 3: Add tempfile dev dependency**

Add to `src-tauri/Cargo.toml` under `[dev-dependencies]`:
```toml
tempfile = "3.10"
```

**Step 4: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_migrations_run`
Expected: FAIL - migrations module doesn't exist

**Step 5: Create schema module**

Create `src-tauri/src/db/schema.rs`:
```rust
pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    productivity INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS rules (
    id INTEGER PRIMARY KEY,
    pattern TEXT NOT NULL,
    match_type TEXT NOT NULL,
    category_id INTEGER REFERENCES categories(id),
    priority INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    duration_secs INTEGER NOT NULL,
    source TEXT NOT NULL,
    app_name TEXT,
    window_title TEXT,
    url TEXT,
    domain TEXT,
    category_id INTEGER REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS focus_sessions (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    scheduled INTEGER DEFAULT 0,
    distraction_budget INTEGER NOT NULL,
    distraction_used INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS focus_schedules (
    id INTEGER PRIMARY KEY,
    days_of_week TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    distraction_budget INTEGER NOT NULL,
    enabled INTEGER DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON activities(timestamp);
CREATE INDEX IF NOT EXISTS idx_activities_category ON activities(category_id);
CREATE INDEX IF NOT EXISTS idx_focus_sessions_active ON focus_sessions(ended_at) WHERE ended_at IS NULL;
"#;

pub const DEFAULT_CATEGORIES: &[(&str, i32)] = &[
    ("Coding", 1),
    ("Communication", 0),
    ("Entertainment", -1),
    ("Reference", 1),
    ("Uncategorized", 0),
];
```

**Step 6: Create migrations module**

Create `src-tauri/src/db/migrations.rs`:
```rust
use rusqlite::{Connection, Result};
use super::schema::{SCHEMA, DEFAULT_CATEGORIES};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    seed_default_categories(conn)?;
    Ok(())
}

fn seed_default_categories(conn: &Connection) -> Result<()> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0)
    )?;

    if count == 0 {
        for (name, productivity) in DEFAULT_CATEGORIES {
            conn.execute(
                "INSERT INTO categories (name, productivity) VALUES (?1, ?2)",
                [*name, &productivity.to_string()],
            )?;
        }
    }
    Ok(())
}
```

**Step 7: Run tests to verify they pass**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

**Step 8: Update main.rs to use db module**

Add to `src-tauri/src/main.rs` after other mod declarations:
```rust
mod db;
```

**Step 9: Commit**

```bash
git add .
git commit -m "feat: add SQLite database with schema and migrations"
```

---

### Task 3: Create Category and Activity Models

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/category.rs`
- Create: `src-tauri/src/models/activity.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Write failing test for Category model**

Create `src-tauri/src/models/mod.rs`:
```rust
pub mod category;
pub mod activity;

pub use category::Category;
pub use activity::Activity;
```

Create `src-tauri/src/models/category.rs`:
```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub productivity: i32,
}

impl Category {
    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, name, productivity FROM categories WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                productivity: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT id, name, productivity FROM categories ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                productivity: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    pub fn create(conn: &Connection, name: &str, productivity: i32) -> Result<Self> {
        conn.execute(
            "INSERT INTO categories (name, productivity) VALUES (?1, ?2)",
            params![name, productivity],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Self { id, name: name.to_string(), productivity })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use tempfile::tempdir;

    fn setup_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        db
    }

    #[test]
    fn test_find_all_returns_default_categories() {
        let db = setup_db();
        let categories = Category::find_all(db.connection()).unwrap();
        assert!(categories.len() >= 5);
        assert!(categories.iter().any(|c| c.name == "Coding"));
    }

    #[test]
    fn test_create_category() {
        let db = setup_db();
        let cat = Category::create(db.connection(), "Testing", 1).unwrap();
        assert_eq!(cat.name, "Testing");
        assert_eq!(cat.productivity, 1);

        let found = Category::find_by_id(db.connection(), cat.id).unwrap();
        assert_eq!(found, Some(cat));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_find_all`
Expected: FAIL - models module doesn't exist

**Step 3: Add models module to main.rs**

Add to `src-tauri/src/main.rs`:
```rust
mod models;
```

**Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test category`
Expected: All category tests PASS

**Step 5: Write Activity model with tests**

Create `src-tauri/src/models/activity.rs`:
```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: Option<i64>,
    pub timestamp: i64,
    pub duration_secs: i32,
    pub source: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub category_id: Option<i64>,
}

impl Activity {
    pub fn new(
        timestamp: i64,
        duration_secs: i32,
        source: &str,
        app_name: Option<&str>,
        window_title: Option<&str>,
    ) -> Self {
        Self {
            id: None,
            timestamp,
            duration_secs,
            source: source.to_string(),
            app_name: app_name.map(|s| s.to_string()),
            window_title: window_title.map(|s| s.to_string()),
            url: None,
            domain: None,
            category_id: None,
        }
    }

    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO activities (timestamp, duration_secs, source, app_name, window_title, url, domain, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.timestamp,
                self.duration_secs,
                self.source,
                self.app_name,
                self.window_title,
                self.url,
                self.domain,
                self.category_id,
            ],
        )?;
        self.id = Some(conn.last_insert_rowid());
        Ok(())
    }

    pub fn find_in_range(conn: &Connection, start: i64, end: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, duration_secs, source, app_name, window_title, url, domain, category_id
             FROM activities WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp"
        )?;

        let rows = stmt.query_map(params![start, end], |row| {
            Ok(Self {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                duration_secs: row.get(2)?,
                source: row.get(3)?,
                app_name: row.get(4)?,
                window_title: row.get(5)?,
                url: row.get(6)?,
                domain: row.get(7)?,
                category_id: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    pub fn total_duration_by_category(conn: &Connection, start: i64, end: i64) -> Result<Vec<(i64, i32)>> {
        let mut stmt = conn.prepare(
            "SELECT category_id, SUM(duration_secs) as total
             FROM activities
             WHERE timestamp >= ?1 AND timestamp < ?2 AND category_id IS NOT NULL
             GROUP BY category_id"
        )?;

        let rows = stmt.query_map(params![start, end], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use crate::models::Category;
    use tempfile::tempdir;

    fn setup_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        db
    }

    #[test]
    fn test_save_and_find_activity() {
        let db = setup_db();
        let now = 1700000000i64;

        let mut activity = Activity::new(now, 5, "app", Some("VSCode"), Some("main.rs"));
        activity.save(db.connection()).unwrap();

        let found = Activity::find_in_range(db.connection(), now - 10, now + 10).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].app_name, Some("VSCode".to_string()));
    }

    #[test]
    fn test_total_duration_by_category() {
        let db = setup_db();
        let conn = db.connection();
        let now = 1700000000i64;

        let categories = Category::find_all(conn).unwrap();
        let coding_id = categories.iter().find(|c| c.name == "Coding").unwrap().id;

        let mut a1 = Activity::new(now, 30, "app", Some("VSCode"), None);
        a1.category_id = Some(coding_id);
        a1.save(conn).unwrap();

        let mut a2 = Activity::new(now + 30, 20, "app", Some("VSCode"), None);
        a2.category_id = Some(coding_id);
        a2.save(conn).unwrap();

        let totals = Activity::total_duration_by_category(conn, now - 10, now + 100).unwrap();
        let coding_total = totals.iter().find(|(id, _)| *id == coding_id);
        assert_eq!(coding_total, Some(&(coding_id, 50)));
    }
}
```

**Step 6: Run all model tests**

Run: `cd src-tauri && cargo test models`
Expected: All tests PASS

**Step 7: Commit**

```bash
git add .
git commit -m "feat: add Category and Activity models"
```

---

### Task 4: Create Categorizer Engine

**Files:**
- Create: `src-tauri/src/categorizer/mod.rs`
- Create: `src-tauri/src/models/rule.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add Rule model**

Create `src-tauri/src/models/rule.rs`:
```rust
use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: i64,
    pub pattern: String,
    pub match_type: String,
    pub category_id: i64,
    pub priority: i32,
}

impl Rule {
    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, match_type, category_id, priority FROM rules ORDER BY priority DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: row.get(0)?,
                pattern: row.get(1)?,
                match_type: row.get(2)?,
                category_id: row.get(3)?,
                priority: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    pub fn create(conn: &Connection, pattern: &str, match_type: &str, category_id: i64, priority: i32) -> Result<Self> {
        conn.execute(
            "INSERT INTO rules (pattern, match_type, category_id, priority) VALUES (?1, ?2, ?3, ?4)",
            params![pattern, match_type, category_id, priority],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Self { id, pattern: pattern.to_string(), match_type: match_type.to_string(), category_id, priority })
    }
}
```

**Step 2: Update models/mod.rs**

```rust
pub mod category;
pub mod activity;
pub mod rule;

pub use category::Category;
pub use activity::Activity;
pub use rule::Rule;
```

**Step 3: Create Categorizer with tests**

Create `src-tauri/src/categorizer/mod.rs`:
```rust
use crate::models::{Category, Rule};
use rusqlite::Connection;

pub struct Categorizer {
    rules: Vec<(Rule, Category)>,
    default_category_id: i64,
}

impl Categorizer {
    pub fn new(conn: &Connection) -> rusqlite::Result<Self> {
        let rules = Rule::find_all(conn)?;
        let categories = Category::find_all(conn)?;

        let rules_with_categories: Vec<_> = rules
            .into_iter()
            .filter_map(|rule| {
                categories.iter()
                    .find(|c| c.id == rule.category_id)
                    .map(|c| (rule, c.clone()))
            })
            .collect();

        let default_category_id = categories
            .iter()
            .find(|c| c.name == "Uncategorized")
            .map(|c| c.id)
            .unwrap_or(1);

        Ok(Self {
            rules: rules_with_categories,
            default_category_id,
        })
    }

    pub fn categorize_app(&self, app_name: &str, window_title: Option<&str>) -> i64 {
        for (rule, _category) in &self.rules {
            let matches = match rule.match_type.as_str() {
                "app" => Self::pattern_matches(&rule.pattern, app_name),
                "title" => window_title.map(|t| Self::pattern_matches(&rule.pattern, t)).unwrap_or(false),
                _ => false,
            };

            if matches {
                return rule.category_id;
            }
        }

        self.default_category_id
    }

    pub fn categorize_url(&self, domain: &str) -> i64 {
        for (rule, _category) in &self.rules {
            if rule.match_type == "domain" && Self::pattern_matches(&rule.pattern, domain) {
                return rule.category_id;
            }
        }

        self.default_category_id
    }

    fn pattern_matches(pattern: &str, text: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        let text_lower = text.to_lowercase();

        if pattern_lower.contains('*') {
            let parts: Vec<&str> = pattern_lower.split('*').collect();
            let mut pos = 0;
            for part in parts {
                if part.is_empty() { continue; }
                if let Some(found) = text_lower[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            text_lower.contains(&pattern_lower)
        }
    }

    pub fn reload(&mut self, conn: &Connection) -> rusqlite::Result<()> {
        *self = Self::new(conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use tempfile::tempdir;

    fn setup_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        db
    }

    #[test]
    fn test_categorize_with_no_rules_returns_default() {
        let db = setup_db();
        let categorizer = Categorizer::new(db.connection()).unwrap();

        let category_id = categorizer.categorize_app("SomeApp", None);

        let uncategorized = Category::find_all(db.connection()).unwrap()
            .into_iter()
            .find(|c| c.name == "Uncategorized")
            .unwrap();

        assert_eq!(category_id, uncategorized.id);
    }

    #[test]
    fn test_categorize_app_with_rule() {
        let db = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        Rule::create(conn, "code", "app", coding.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();
        let category_id = categorizer.categorize_app("Visual Studio Code", None);

        assert_eq!(category_id, coding.id);
    }

    #[test]
    fn test_categorize_domain() {
        let db = setup_db();
        let conn = db.connection();

        let entertainment = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Entertainment")
            .unwrap();

        Rule::create(conn, "reddit.com", "domain", entertainment.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();
        let category_id = categorizer.categorize_url("reddit.com");

        assert_eq!(category_id, entertainment.id);
    }

    #[test]
    fn test_pattern_with_wildcard() {
        let db = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        Rule::create(conn, "*.github.*", "domain", coding.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();

        assert_eq!(categorizer.categorize_url("www.github.com"), coding.id);
        assert_eq!(categorizer.categorize_url("gist.github.io"), coding.id);
    }
}
```

**Step 4: Add categorizer module to main.rs**

```rust
mod categorizer;
```

**Step 5: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add .
git commit -m "feat: add Rule model and Categorizer engine"
```

---

## Phase 2: Desktop Tracking (macOS)

### Task 5: Create Platform Abstraction Layer

**Files:**
- Create: `src-tauri/src/platform/mod.rs`
- Create: `src-tauri/src/platform/types.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Define platform types**

Create `src-tauri/src/platform/types.rs`:
```rust
#[derive(Debug, Clone, Default)]
pub struct ActiveWindow {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: Option<String>,
}

pub trait PlatformTracker: Send + Sync {
    fn get_active_window(&self) -> Option<ActiveWindow>;
    fn get_idle_time_secs(&self) -> u64;
}
```

**Step 2: Create platform module**

Create `src-tauri/src/platform/mod.rs`:
```rust
pub mod types;

pub use types::{ActiveWindow, PlatformTracker};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub use macos::MacOSTracker as NativeTracker;

#[cfg(target_os = "linux")]
pub use linux::LinuxTracker as NativeTracker;

// Stub for development on other platforms
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct NativeTracker;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl PlatformTracker for NativeTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        Some(ActiveWindow {
            app_name: "TestApp".to_string(),
            window_title: "Test Window".to_string(),
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        0
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl NativeTracker {
    pub fn new() -> Self { Self }
}
```

**Step 3: Add platform module to main.rs**

```rust
mod platform;
```

**Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add platform abstraction layer"
```

---

### Task 6: Implement macOS Window Tracking

**Files:**
- Create: `src-tauri/src/platform/macos.rs`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add macOS dependencies**

Add to `src-tauri/Cargo.toml` under `[target.'cfg(target_os = "macos")'.dependencies]`:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSString", "NSArray", "NSDictionary"] }
objc2-app-kit = { version = "0.2", features = ["NSWorkspace", "NSRunningApplication"] }
core-graphics = "0.23"
```

**Step 2: Implement macOS tracker**

Create `src-tauri/src/platform/macos.rs`:
```rust
use super::types::{ActiveWindow, PlatformTracker};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSString;

pub struct MacOSTracker;

impl MacOSTracker {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformTracker for MacOSTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let app = workspace.frontmostApplication()?;

            let app_name = app
                .localizedName()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let bundle_id = app
                .bundleIdentifier()
                .map(|s| s.to_string());

            // Window title requires accessibility permissions
            // For now, use app name as window title fallback
            let window_title = get_window_title().unwrap_or_else(|| app_name.clone());

            Some(ActiveWindow {
                app_name,
                window_title,
                bundle_id,
            })
        }
    }

    fn get_idle_time_secs(&self) -> u64 {
        let source = match CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let idle_secs = source.seconds_since_last_event_type(
            core_graphics::event::CGEventType::Null
        );

        idle_secs.max(0.0) as u64
    }
}

fn get_window_title() -> Option<String> {
    // This requires accessibility permissions
    // Using AppleScript as a fallback that works without special permissions
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first window of (first application process whose frontmost is true)"#)
        .output()
        .ok()?;

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !title.is_empty() {
            return Some(title);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_window() {
        let tracker = MacOSTracker::new();
        // This test may fail in CI but should work locally
        if let Some(window) = tracker.get_active_window() {
            assert!(!window.app_name.is_empty());
            println!("Active: {} - {}", window.app_name, window.window_title);
        }
    }

    #[test]
    fn test_get_idle_time() {
        let tracker = MacOSTracker::new();
        let idle = tracker.get_idle_time_secs();
        // Should be a reasonable value (less than a day in seconds)
        assert!(idle < 86400);
    }
}
```

**Step 3: Verify it compiles on macOS**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Run tests manually (requires accessibility permissions)**

Run: `cd src-tauri && cargo test macos -- --nocapture`
Expected: Tests pass (may need to grant accessibility permissions)

**Step 5: Commit**

```bash
git add .
git commit -m "feat: implement macOS window tracking"
```

---

### Task 7: Create Tracker Service

**Files:**
- Create: `src-tauri/src/tracker/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create tracker service**

Create `src-tauri/src/tracker/mod.rs`:
```rust
use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::models::Activity;
use crate::platform::{NativeTracker, PlatformTracker};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct TrackerConfig {
    pub poll_interval_secs: u64,
    pub idle_threshold_secs: u64,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            idle_threshold_secs: 120,
        }
    }
}

pub struct TrackerService {
    config: TrackerConfig,
    running: Arc<AtomicBool>,
    db: Arc<Mutex<Database>>,
    categorizer: Arc<Mutex<Categorizer>>,
    platform: NativeTracker,
}

impl TrackerService {
    pub fn new(db: Arc<Mutex<Database>>, categorizer: Arc<Mutex<Categorizer>>, config: TrackerConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            db,
            categorizer,
            platform: NativeTracker::new(),
        }
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let db = Arc::clone(&self.db);
        let categorizer = Arc::clone(&self.categorizer);
        let config = TrackerConfig {
            poll_interval_secs: self.config.poll_interval_secs,
            idle_threshold_secs: self.config.idle_threshold_secs,
        };
        let platform = NativeTracker::new();

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let idle_secs = platform.get_idle_time_secs();

                if idle_secs < config.idle_threshold_secs {
                    if let Some(window) = platform.get_active_window() {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;

                        let category_id = {
                            let cat = categorizer.lock().unwrap();
                            cat.categorize_app(&window.app_name, Some(&window.window_title))
                        };

                        let mut activity = Activity::new(
                            timestamp,
                            config.poll_interval_secs as i32,
                            "app",
                            Some(&window.app_name),
                            Some(&window.window_title),
                        );
                        activity.category_id = Some(category_id);

                        if let Ok(db) = db.lock() {
                            let _ = activity.save(db.connection());
                        }
                    }
                }

                thread::sleep(Duration::from_secs(config.poll_interval_secs));
            }
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use tempfile::tempdir;

    fn setup() -> (Arc<Mutex<Database>>, Arc<Mutex<Categorizer>>) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();

        let categorizer = Categorizer::new(db.connection()).unwrap();

        (Arc::new(Mutex::new(db)), Arc::new(Mutex::new(categorizer)))
    }

    #[test]
    fn test_tracker_starts_and_stops() {
        let (db, categorizer) = setup();
        let config = TrackerConfig {
            poll_interval_secs: 1,
            idle_threshold_secs: 120,
        };

        let tracker = TrackerService::new(db, categorizer, config);

        assert!(!tracker.is_running());

        let handle = tracker.start();
        assert!(tracker.is_running());

        thread::sleep(Duration::from_millis(100));

        tracker.stop();
        handle.join().unwrap();

        assert!(!tracker.is_running());
    }
}
```

**Step 2: Add tracker module to main.rs**

```rust
mod tracker;
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test tracker`
Expected: Tests PASS

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add tracker service with polling loop"
```

---

## Phase 3: Focus Mode Engine

### Task 8: Create Focus Session Model

**Files:**
- Create: `src-tauri/src/models/focus_session.rs`
- Modify: `src-tauri/src/models/mod.rs`

**Step 1: Write failing test for FocusSession**

Create `src-tauri/src/models/focus_session.rs`:
```rust
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct FocusSession {
    pub id: Option<i64>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub scheduled: bool,
    pub distraction_budget: i32,
    pub distraction_used: i32,
}

impl FocusSession {
    pub fn new(distraction_budget_secs: i32, scheduled: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id: None,
            started_at: now,
            ended_at: None,
            scheduled,
            distraction_budget: distraction_budget_secs,
            distraction_used: 0,
        }
    }

    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO focus_sessions (started_at, ended_at, scheduled, distraction_budget, distraction_used)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.started_at,
                self.ended_at,
                self.scheduled as i32,
                self.distraction_budget,
                self.distraction_used,
            ],
        )?;
        self.id = Some(conn.last_insert_rowid());
        Ok(())
    }

    pub fn find_active(conn: &Connection) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, started_at, ended_at, scheduled, distraction_budget, distraction_used
             FROM focus_sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: Some(row.get(0)?),
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                scheduled: row.get::<_, i32>(3)? != 0,
                distraction_budget: row.get(4)?,
                distraction_used: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn end(&mut self, conn: &Connection) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.ended_at = Some(now);

        if let Some(id) = self.id {
            conn.execute(
                "UPDATE focus_sessions SET ended_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
        }

        Ok(())
    }

    pub fn add_distraction_time(&mut self, conn: &Connection, secs: i32) -> Result<()> {
        self.distraction_used += secs;

        if let Some(id) = self.id {
            conn.execute(
                "UPDATE focus_sessions SET distraction_used = ?1 WHERE id = ?2",
                params![self.distraction_used, id],
            )?;
        }

        Ok(())
    }

    pub fn budget_remaining(&self) -> i32 {
        (self.distraction_budget - self.distraction_used).max(0)
    }

    pub fn is_budget_exhausted(&self) -> bool {
        self.distraction_used >= self.distraction_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use tempfile::tempdir;

    fn setup_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        db
    }

    #[test]
    fn test_create_and_find_active_session() {
        let db = setup_db();
        let conn = db.connection();

        assert!(FocusSession::find_active(conn).unwrap().is_none());

        let mut session = FocusSession::new(600, false);
        session.save(conn).unwrap();

        let found = FocusSession::find_active(conn).unwrap().unwrap();
        assert_eq!(found.id, session.id);
        assert_eq!(found.distraction_budget, 600);
    }

    #[test]
    fn test_end_session() {
        let db = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        session.save(conn).unwrap();

        session.end(conn).unwrap();

        assert!(FocusSession::find_active(conn).unwrap().is_none());
    }

    #[test]
    fn test_distraction_budget() {
        let db = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(300, false);
        session.save(conn).unwrap();

        assert_eq!(session.budget_remaining(), 300);
        assert!(!session.is_budget_exhausted());

        session.add_distraction_time(conn, 100).unwrap();
        assert_eq!(session.budget_remaining(), 200);

        session.add_distraction_time(conn, 200).unwrap();
        assert_eq!(session.budget_remaining(), 0);
        assert!(session.is_budget_exhausted());
    }
}
```

**Step 2: Update models/mod.rs**

```rust
pub mod category;
pub mod activity;
pub mod rule;
pub mod focus_session;

pub use category::Category;
pub use activity::Activity;
pub use rule::Rule;
pub use focus_session::FocusSession;
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test focus_session`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add FocusSession model with budget tracking"
```

---

### Task 9: Create Focus Mode Manager

**Files:**
- Create: `src-tauri/src/focus/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create focus mode manager**

Create `src-tauri/src/focus/mod.rs`:
```rust
use crate::db::Database;
use crate::models::{Category, FocusSession};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct FocusState {
    pub active: bool,
    pub budget_remaining: i32,
    pub blocked_domains: Vec<String>,
}

pub struct FocusManager {
    db: Arc<Mutex<Database>>,
}

impl FocusManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    pub fn start_session(&self, distraction_budget_secs: i32) -> rusqlite::Result<FocusSession> {
        let db = self.db.lock().unwrap();
        let conn = db.connection();

        // End any existing active session
        if let Some(mut existing) = FocusSession::find_active(conn)? {
            existing.end(conn)?;
        }

        let mut session = FocusSession::new(distraction_budget_secs, false);
        session.save(conn)?;

        Ok(session)
    }

    pub fn end_session(&self) -> rusqlite::Result<Option<FocusSession>> {
        let db = self.db.lock().unwrap();
        let conn = db.connection();

        if let Some(mut session) = FocusSession::find_active(conn)? {
            session.end(conn)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub fn get_state(&self) -> rusqlite::Result<FocusState> {
        let db = self.db.lock().unwrap();
        let conn = db.connection();

        let session = FocusSession::find_active(conn)?;
        let blocked_domains = self.get_blocked_domains(conn)?;

        Ok(FocusState {
            active: session.is_some(),
            budget_remaining: session.map(|s| s.budget_remaining()).unwrap_or(0),
            blocked_domains,
        })
    }

    pub fn use_distraction_time(&self, secs: i32) -> rusqlite::Result<Option<i32>> {
        let db = self.db.lock().unwrap();
        let conn = db.connection();

        if let Some(mut session) = FocusSession::find_active(conn)? {
            session.add_distraction_time(conn, secs)?;
            Ok(Some(session.budget_remaining()))
        } else {
            Ok(None)
        }
    }

    pub fn is_domain_blocked(&self, domain: &str) -> rusqlite::Result<bool> {
        let state = self.get_state()?;

        if !state.active {
            return Ok(false);
        }

        Ok(state.blocked_domains.iter().any(|d| {
            domain.ends_with(d) || domain == d.trim_start_matches("*.")
        }))
    }

    fn get_blocked_domains(&self, conn: &Connection) -> rusqlite::Result<Vec<String>> {
        // Get domains from rules that map to distracting categories
        let mut stmt = conn.prepare(
            "SELECT r.pattern FROM rules r
             JOIN categories c ON r.category_id = c.id
             WHERE r.match_type = 'domain' AND c.productivity < 0"
        )?;

        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::models::Rule;
    use tempfile::tempdir;

    fn setup() -> Arc<Mutex<Database>> {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        Arc::new(Mutex::new(db))
    }

    #[test]
    fn test_start_and_end_session() {
        let db = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        let state = manager.get_state().unwrap();
        assert!(!state.active);

        let session = manager.start_session(600).unwrap();
        assert_eq!(session.distraction_budget, 600);

        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert_eq!(state.budget_remaining, 600);

        manager.end_session().unwrap();

        let state = manager.get_state().unwrap();
        assert!(!state.active);
    }

    #[test]
    fn test_use_distraction_time() {
        let db = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        manager.start_session(300).unwrap();

        let remaining = manager.use_distraction_time(100).unwrap().unwrap();
        assert_eq!(remaining, 200);

        let remaining = manager.use_distraction_time(200).unwrap().unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_blocked_domains() {
        let db = setup();

        // Add a rule for reddit as distracting
        {
            let db_lock = db.lock().unwrap();
            let conn = db_lock.connection();
            let entertainment = Category::find_all(conn).unwrap()
                .into_iter()
                .find(|c| c.name == "Entertainment")
                .unwrap();
            Rule::create(conn, "reddit.com", "domain", entertainment.id, 10).unwrap();
        }

        let manager = FocusManager::new(Arc::clone(&db));

        // Not blocked when focus mode is off
        assert!(!manager.is_domain_blocked("reddit.com").unwrap());

        manager.start_session(600).unwrap();

        // Blocked when focus mode is on
        assert!(manager.is_domain_blocked("reddit.com").unwrap());
        assert!(manager.is_domain_blocked("www.reddit.com").unwrap());

        // Other domains not blocked
        assert!(!manager.is_domain_blocked("github.com").unwrap());
    }
}
```

**Step 2: Add focus module to main.rs**

```rust
mod focus;
```

**Step 3: Run tests**

Run: `cd src-tauri && cargo test focus`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add FocusManager for session and blocking control"
```

---

## Phase 4: Chrome Extension

### Task 10: Create Extension Manifest and Structure

**Files:**
- Create: `extension/manifest.json`
- Create: `extension/background.js`
- Create: `extension/blocked.html`
- Create: `extension/blocked.js`
- Create: `extension/popup/popup.html`
- Create: `extension/popup/popup.js`

**Step 1: Create manifest.json**

Create `extension/manifest.json`:
```json
{
  "manifest_version": 3,
  "name": "Foxus",
  "version": "0.1.0",
  "description": "Focus mode and productivity tracking",
  "permissions": [
    "tabs",
    "webNavigation",
    "nativeMessaging",
    "storage"
  ],
  "host_permissions": [
    "<all_urls>"
  ],
  "background": {
    "service_worker": "background.js"
  },
  "action": {
    "default_popup": "popup/popup.html",
    "default_icon": {
      "16": "icons/icon16.png",
      "48": "icons/icon48.png",
      "128": "icons/icon128.png"
    }
  },
  "icons": {
    "16": "icons/icon16.png",
    "48": "icons/icon48.png",
    "128": "icons/icon128.png"
  }
}
```

**Step 2: Create background.js**

Create `extension/background.js`:
```javascript
const NATIVE_HOST = "com.foxus.native";

let focusState = {
  active: false,
  budgetRemaining: 0,
  blockedDomains: []
};

let nativePort = null;

function connectToNative() {
  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST);

    nativePort.onMessage.addListener((message) => {
      console.log("Native message:", message);
      if (message.type === "state") {
        focusState = {
          active: message.focusActive,
          budgetRemaining: message.budgetRemaining,
          blockedDomains: message.blockedDomains || []
        };
        chrome.storage.local.set({ focusState });
      } else if (message.type === "budget_updated") {
        focusState.budgetRemaining = message.remaining;
        chrome.storage.local.set({ focusState });
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log("Native host disconnected");
      nativePort = null;
      // Retry connection after delay
      setTimeout(connectToNative, 5000);
    });

    // Request initial state
    nativePort.postMessage({ type: "request_state" });
  } catch (e) {
    console.error("Failed to connect to native host:", e);
    // Load cached state
    chrome.storage.local.get(["focusState"], (result) => {
      if (result.focusState) {
        focusState = result.focusState;
      }
    });
  }
}

function sendActivity(url, title) {
  if (nativePort) {
    nativePort.postMessage({
      type: "activity",
      url,
      title,
      timestamp: Date.now()
    });
  }
}

function isDomainBlocked(url) {
  if (!focusState.active) return false;

  try {
    const domain = new URL(url).hostname;
    return focusState.blockedDomains.some(blocked => {
      if (blocked.startsWith("*.")) {
        return domain.endsWith(blocked.slice(1));
      }
      return domain === blocked || domain.endsWith("." + blocked);
    });
  } catch {
    return false;
  }
}

// Track tab changes
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  const tab = await chrome.tabs.get(activeInfo.tabId);
  if (tab.url) {
    sendActivity(tab.url, tab.title || "");
  }
});

// Track URL changes
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.url) {
    sendActivity(changeInfo.url, tab.title || "");
  }
});

// Block navigation to distracting sites
chrome.webNavigation.onBeforeNavigate.addListener((details) => {
  if (details.frameId !== 0) return; // Only main frame

  if (isDomainBlocked(details.url)) {
    const blockedUrl = chrome.runtime.getURL("blocked.html") +
      "?url=" + encodeURIComponent(details.url) +
      "&budget=" + focusState.budgetRemaining;

    chrome.tabs.update(details.tabId, { url: blockedUrl });
  }
});

// Handle messages from popup and blocked page
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === "get_state") {
    sendResponse(focusState);
  } else if (message.type === "use_distraction_time") {
    if (nativePort) {
      nativePort.postMessage({ type: "use_distraction_time" });
    }
    sendResponse({ success: true });
  }
  return true;
});

// Initialize
connectToNative();
```

**Step 3: Create blocked.html**

Create `extension/blocked.html`:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Site Blocked - Foxus</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      color: white;
    }
    .container {
      text-align: center;
      padding: 40px;
      max-width: 500px;
    }
    .icon { font-size: 64px; margin-bottom: 20px; }
    h1 { font-size: 28px; margin-bottom: 10px; }
    .domain { font-size: 18px; opacity: 0.9; margin-bottom: 30px; }
    .budget {
      background: rgba(255,255,255,0.2);
      padding: 20px;
      border-radius: 12px;
      margin-bottom: 30px;
    }
    .budget-label { font-size: 14px; opacity: 0.8; }
    .budget-time { font-size: 32px; font-weight: bold; }
    .btn {
      background: white;
      color: #667eea;
      border: none;
      padding: 16px 32px;
      font-size: 16px;
      border-radius: 8px;
      cursor: pointer;
      font-weight: 600;
      transition: transform 0.2s;
    }
    .btn:hover { transform: scale(1.05); }
    .btn:disabled {
      opacity: 0.5;
      cursor: not-allowed;
      transform: none;
    }
    .countdown {
      margin-top: 20px;
      font-size: 14px;
      opacity: 0.8;
    }
    .hard-blocked {
      background: rgba(255,0,0,0.2);
      padding: 20px;
      border-radius: 12px;
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="icon">🎯</div>
    <h1>Focus Mode Active</h1>
    <div class="domain" id="domain"></div>

    <div id="soft-block">
      <div class="budget">
        <div class="budget-label">Distraction budget remaining</div>
        <div class="budget-time" id="budget-time">--:--</div>
      </div>
      <button class="btn" id="use-budget-btn">Use distraction time</button>
      <div class="countdown" id="countdown" style="display: none;">
        Redirecting in <span id="countdown-time">30</span> seconds...
      </div>
    </div>

    <div id="hard-block" class="hard-blocked" style="display: none;">
      <p>Your distraction budget is exhausted.</p>
      <p style="margin-top: 10px;">Stay focused! You've got this.</p>
    </div>
  </div>
  <script src="blocked.js"></script>
</body>
</html>
```

**Step 4: Create blocked.js**

Create `extension/blocked.js`:
```javascript
const params = new URLSearchParams(window.location.search);
const blockedUrl = params.get("url");
const budget = parseInt(params.get("budget") || "0", 10);

document.getElementById("domain").textContent = blockedUrl ? new URL(blockedUrl).hostname : "Unknown site";

function formatTime(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

document.getElementById("budget-time").textContent = formatTime(budget);

if (budget <= 0) {
  document.getElementById("soft-block").style.display = "none";
  document.getElementById("hard-block").style.display = "block";
} else {
  const btn = document.getElementById("use-budget-btn");
  const countdown = document.getElementById("countdown");
  const countdownTime = document.getElementById("countdown-time");

  btn.addEventListener("click", () => {
    btn.disabled = true;
    countdown.style.display = "block";

    let remaining = 30;
    countdownTime.textContent = remaining;

    const interval = setInterval(() => {
      remaining--;
      countdownTime.textContent = remaining;

      if (remaining <= 0) {
        clearInterval(interval);
        chrome.runtime.sendMessage({ type: "use_distraction_time" }, () => {
          window.location.href = blockedUrl;
        });
      }
    }, 1000);
  });
}
```

**Step 5: Create popup**

Create `extension/popup/popup.html`:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      width: 280px;
      padding: 16px;
    }
    .header {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 16px;
    }
    .header h1 { font-size: 18px; }
    .status {
      padding: 12px;
      border-radius: 8px;
      margin-bottom: 12px;
    }
    .status.active {
      background: #e8f5e9;
      color: #2e7d32;
    }
    .status.inactive {
      background: #f5f5f5;
      color: #666;
    }
    .budget {
      font-size: 24px;
      font-weight: bold;
      margin-top: 8px;
    }
    .footer {
      font-size: 12px;
      color: #999;
      text-align: center;
    }
  </style>
</head>
<body>
  <div class="header">
    <span>🎯</span>
    <h1>Foxus</h1>
  </div>

  <div class="status" id="status">
    Loading...
  </div>

  <div class="footer">
    Manage in desktop app
  </div>

  <script src="popup.js"></script>
</body>
</html>
```

Create `extension/popup/popup.js`:
```javascript
function formatTime(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

chrome.runtime.sendMessage({ type: "get_state" }, (state) => {
  const statusEl = document.getElementById("status");

  if (state && state.active) {
    statusEl.className = "status active";
    statusEl.innerHTML = `
      <div>Focus Mode Active</div>
      <div class="budget">${formatTime(state.budgetRemaining)} remaining</div>
    `;
  } else {
    statusEl.className = "status inactive";
    statusEl.innerHTML = `<div>Focus Mode Off</div>`;
  }
});
```

**Step 6: Create placeholder icons**

Create `extension/icons/` directory and add placeholder icons (16x16, 48x48, 128x128 PNGs). For now, create simple text files as placeholders:

Run:
```bash
mkdir -p extension/icons
touch extension/icons/icon16.png extension/icons/icon48.png extension/icons/icon128.png
```

**Step 7: Commit**

```bash
git add extension/
git commit -m "feat: add Chrome extension with blocking and native messaging"
```

---

### Task 11: Create Native Messaging Host

**Files:**
- Create: `src-tauri/src/native_host/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create native host handler**

Create `src-tauri/src/native_host/mod.rs`:
```rust
use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::Activity;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    #[serde(rename = "activity")]
    Activity { url: String, title: String, timestamp: i64 },
    #[serde(rename = "request_state")]
    RequestState,
    #[serde(rename = "use_distraction_time")]
    UseDistractionTime,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutgoingMessage {
    #[serde(rename = "state")]
    State {
        #[serde(rename = "focusActive")]
        focus_active: bool,
        #[serde(rename = "budgetRemaining")]
        budget_remaining: i32,
        #[serde(rename = "blockedDomains")]
        blocked_domains: Vec<String>,
    },
    #[serde(rename = "budget_updated")]
    BudgetUpdated { remaining: i32 },
    #[serde(rename = "hard_blocked")]
    HardBlocked,
}

pub struct NativeHost {
    db: Arc<Mutex<Database>>,
    focus_manager: Arc<FocusManager>,
    categorizer: Arc<Mutex<Categorizer>>,
}

impl NativeHost {
    pub fn new(
        db: Arc<Mutex<Database>>,
        focus_manager: Arc<FocusManager>,
        categorizer: Arc<Mutex<Categorizer>>,
    ) -> Self {
        Self { db, focus_manager, categorizer }
    }

    pub fn run(&self) -> io::Result<()> {
        loop {
            let message = self.read_message()?;
            let response = self.handle_message(message);

            if let Some(resp) = response {
                self.write_message(&resp)?;
            }
        }
    }

    fn read_message(&self) -> io::Result<IncomingMessage> {
        let mut len_bytes = [0u8; 4];
        io::stdin().read_exact(&mut len_bytes)?;
        let len = u32::from_ne_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        io::stdin().read_exact(&mut buffer)?;

        serde_json::from_slice(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write_message(&self, message: &OutgoingMessage) -> io::Result<()> {
        let json = serde_json::to_vec(message)?;
        let len = json.len() as u32;

        io::stdout().write_all(&len.to_ne_bytes())?;
        io::stdout().write_all(&json)?;
        io::stdout().flush()?;

        Ok(())
    }

    fn handle_message(&self, message: IncomingMessage) -> Option<OutgoingMessage> {
        match message {
            IncomingMessage::Activity { url, title, timestamp } => {
                self.record_activity(&url, &title, timestamp);
                None
            }
            IncomingMessage::RequestState => {
                Some(self.get_state())
            }
            IncomingMessage::UseDistractionTime => {
                self.use_distraction_time()
            }
        }
    }

    fn record_activity(&self, url: &str, title: &str, _timestamp: i64) {
        let domain = extract_domain(url);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let category_id = {
            let cat = self.categorizer.lock().unwrap();
            cat.categorize_url(&domain)
        };

        let mut activity = Activity::new(timestamp, 5, "browser", None, Some(title));
        activity.url = Some(url.to_string());
        activity.domain = Some(domain);
        activity.category_id = Some(category_id);

        if let Ok(db) = self.db.lock() {
            let _ = activity.save(db.connection());
        }
    }

    fn get_state(&self) -> OutgoingMessage {
        match self.focus_manager.get_state() {
            Ok(state) => OutgoingMessage::State {
                focus_active: state.active,
                budget_remaining: state.budget_remaining,
                blocked_domains: state.blocked_domains,
            },
            Err(_) => OutgoingMessage::State {
                focus_active: false,
                budget_remaining: 0,
                blocked_domains: vec![],
            },
        }
    }

    fn use_distraction_time(&self) -> Option<OutgoingMessage> {
        match self.focus_manager.use_distraction_time(30) {
            Ok(Some(remaining)) => {
                if remaining <= 0 {
                    Some(OutgoingMessage::HardBlocked)
                } else {
                    Some(OutgoingMessage::BudgetUpdated { remaining })
                }
            }
            _ => None,
        }
    }
}

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.reddit.com/r/rust"), "www.reddit.com");
        assert_eq!(extract_domain("http://github.com"), "github.com");
        assert_eq!(extract_domain("https://docs.rs/tauri/latest"), "docs.rs");
    }
}
```

**Step 2: Add serde dependency**

Add to `src-tauri/Cargo.toml`:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Step 3: Add native_host module to main.rs**

```rust
mod native_host;
```

**Step 4: Run tests**

Run: `cd src-tauri && cargo test native_host`
Expected: Tests PASS

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add native messaging host for Chrome extension"
```

---

## Phase 5: Menu Bar UI

### Task 12: Set Up Tauri System Tray

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Enable tray feature in Cargo.toml**

Add to `src-tauri/Cargo.toml` features:
```toml
tauri = { version = "2", features = ["tray-icon"] }
```

**Step 2: Update tauri.conf.json**

Add tray configuration to `src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Foxus",
  "version": "0.1.0",
  "identifier": "com.foxus.app",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "withGlobalTauri": true,
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    },
    "windows": []
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

**Step 3: Update main.rs with tray setup**

Replace `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod categorizer;
mod db;
mod focus;
mod models;
mod native_host;
mod platform;
mod tracker;

use crate::categorizer::Categorizer;
use crate::db::{migrations, Database};
use crate::focus::FocusManager;
use crate::tracker::{TrackerConfig, TrackerService};
use directories::ProjectDirs;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    Manager, Runtime,
};

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from("com", "foxus", "Foxus")
        .expect("Could not determine project directories");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Could not create data directory");
    data_dir.join("foxus.db")
}

fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Foxus")
        .on_menu_event(|app, event| {
            if event.id == "quit" {
                app.exit(0);
            }
        })
        .build(app)?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db_path = get_db_path();
            let db = Database::open(&db_path).expect("Failed to open database");
            migrations::run(db.connection()).expect("Failed to run migrations");

            let db = Arc::new(Mutex::new(db));
            let categorizer = Arc::new(Mutex::new(
                Categorizer::new(db.lock().unwrap().connection()).unwrap()
            ));
            let focus_manager = Arc::new(FocusManager::new(Arc::clone(&db)));

            // Start tracker service
            let tracker = TrackerService::new(
                Arc::clone(&db),
                Arc::clone(&categorizer),
                TrackerConfig::default(),
            );
            tracker.start();

            // Store in app state
            app.manage(db);
            app.manage(categorizer);
            app.manage(focus_manager);

            // Setup tray
            setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Create placeholder icon**

Run:
```bash
mkdir -p src-tauri/icons
touch src-tauri/icons/icon.png
```

**Step 5: Verify it compiles**

Run: `cargo tauri build --debug`
Expected: Builds successfully

**Step 6: Commit**

```bash
git add .
git commit -m "feat: add system tray with basic menu"
```

---

### Task 13: Add Tauri Commands for UI

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Create Tauri commands**

Create `src-tauri/src/commands.rs`:
```rust
use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::{Activity, Category};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Serialize)]
pub struct StatsResponse {
    pub productive_secs: i32,
    pub neutral_secs: i32,
    pub distracting_secs: i32,
    pub top_apps: Vec<AppStat>,
}

#[derive(Serialize)]
pub struct AppStat {
    pub name: String,
    pub duration_secs: i32,
    pub productivity: i32,
}

#[derive(Serialize)]
pub struct FocusStateResponse {
    pub active: bool,
    pub budget_remaining: i32,
    pub session_duration_secs: Option<i64>,
}

#[tauri::command]
pub fn get_today_stats(db: State<Arc<Mutex<Database>>>) -> Result<StatsResponse, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let conn = db.connection();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Start of today (midnight)
    let today_start = now - (now % 86400);

    let categories = Category::find_all(conn).map_err(|e| e.to_string())?;
    let totals = Activity::total_duration_by_category(conn, today_start, now)
        .map_err(|e| e.to_string())?;

    let mut productive_secs = 0;
    let mut neutral_secs = 0;
    let mut distracting_secs = 0;

    for (cat_id, duration) in &totals {
        if let Some(cat) = categories.iter().find(|c| c.id == *cat_id) {
            match cat.productivity {
                1 => productive_secs += duration,
                0 => neutral_secs += duration,
                -1 => distracting_secs += duration,
                _ => {}
            }
        }
    }

    // Get top apps
    let mut stmt = conn.prepare(
        "SELECT app_name, SUM(duration_secs) as total, c.productivity
         FROM activities a
         LEFT JOIN categories c ON a.category_id = c.id
         WHERE a.timestamp >= ?1 AND a.app_name IS NOT NULL
         GROUP BY app_name
         ORDER BY total DESC
         LIMIT 5"
    ).map_err(|e| e.to_string())?;

    let top_apps: Vec<AppStat> = stmt.query_map([today_start], |row| {
        Ok(AppStat {
            name: row.get(0)?,
            duration_secs: row.get(1)?,
            productivity: row.get::<_, Option<i32>>(2)?.unwrap_or(0),
        })
    })
    .map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(StatsResponse {
        productive_secs,
        neutral_secs,
        distracting_secs,
        top_apps,
    })
}

#[tauri::command]
pub fn get_focus_state(focus_manager: State<Arc<FocusManager>>) -> Result<FocusStateResponse, String> {
    let state = focus_manager.get_state().map_err(|e| e.to_string())?;

    Ok(FocusStateResponse {
        active: state.active,
        budget_remaining: state.budget_remaining,
        session_duration_secs: None, // TODO: calculate from session start
    })
}

#[tauri::command]
pub fn start_focus_session(
    focus_manager: State<Arc<FocusManager>>,
    budget_minutes: i32,
) -> Result<(), String> {
    focus_manager
        .start_session(budget_minutes * 60)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn end_focus_session(focus_manager: State<Arc<FocusManager>>) -> Result<(), String> {
    focus_manager.end_session().map_err(|e| e.to_string())?;
    Ok(())
}
```

**Step 2: Register commands in main.rs**

Add to main.rs:
```rust
mod commands;

// In tauri::Builder::default(), add:
.invoke_handler(tauri::generate_handler![
    commands::get_today_stats,
    commands::get_focus_state,
    commands::start_focus_session,
    commands::end_focus_session,
])
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add .
git commit -m "feat: add Tauri commands for stats and focus control"
```

---

### Task 14: Create Popup UI

**Files:**
- Modify: `src/index.html`
- Create: `src/style.css`
- Create: `src/app.js`

**Step 1: Create popup HTML**

Replace `src/index.html`:
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Foxus</title>
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div class="container">
    <header>
      <h1>🎯 Foxus</h1>
      <select id="period">
        <option value="today">Today</option>
        <option value="week">This Week</option>
      </select>
    </header>

    <section id="stats-view">
      <div class="stats-bars">
        <div class="stat-row">
          <span class="label">Productive</span>
          <div class="bar-container">
            <div class="bar productive" id="bar-productive"></div>
          </div>
          <span class="time" id="time-productive">0h 0m</span>
        </div>
        <div class="stat-row">
          <span class="label">Neutral</span>
          <div class="bar-container">
            <div class="bar neutral" id="bar-neutral"></div>
          </div>
          <span class="time" id="time-neutral">0h 0m</span>
        </div>
        <div class="stat-row">
          <span class="label">Distracting</span>
          <div class="bar-container">
            <div class="bar distracting" id="bar-distracting"></div>
          </div>
          <span class="time" id="time-distracting">0h 0m</span>
        </div>
      </div>

      <div class="top-apps">
        <h3>Top Apps</h3>
        <ul id="app-list"></ul>
      </div>
    </section>

    <section id="focus-view" style="display: none;">
      <div class="focus-active">
        <h2>Focus Mode Active</h2>
        <div class="budget">
          <span class="budget-label">Budget remaining</span>
          <span class="budget-time" id="budget-time">0:00</span>
        </div>
      </div>
    </section>

    <footer>
      <button id="focus-btn" class="btn-focus">Start Focus Session</button>
    </footer>
  </div>
  <script src="app.js"></script>
</body>
</html>
```

**Step 2: Create CSS**

Create `src/style.css`:
```css
* { box-sizing: border-box; margin: 0; padding: 0; }

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: #f5f5f5;
  min-width: 300px;
}

.container {
  padding: 16px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

header h1 {
  font-size: 18px;
}

header select {
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #ddd;
}

.stats-bars {
  background: white;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 16px;
}

.stat-row {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.stat-row:last-child {
  margin-bottom: 0;
}

.label {
  width: 80px;
  font-size: 13px;
  color: #666;
}

.bar-container {
  flex: 1;
  height: 8px;
  background: #eee;
  border-radius: 4px;
  margin: 0 12px;
  overflow: hidden;
}

.bar {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.bar.productive { background: #4caf50; }
.bar.neutral { background: #ff9800; }
.bar.distracting { background: #f44336; }

.time {
  width: 60px;
  font-size: 13px;
  text-align: right;
  font-weight: 500;
}

.top-apps {
  background: white;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 16px;
}

.top-apps h3 {
  font-size: 14px;
  margin-bottom: 12px;
  color: #333;
}

.top-apps ul {
  list-style: none;
}

.top-apps li {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #eee;
  font-size: 13px;
}

.top-apps li:last-child {
  border-bottom: none;
}

.app-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.productivity-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.productivity-dot.productive { background: #4caf50; }
.productivity-dot.neutral { background: #ff9800; }
.productivity-dot.distracting { background: #f44336; }

footer {
  margin-top: 16px;
}

.btn-focus {
  width: 100%;
  padding: 12px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}

.btn-focus:hover {
  background: #5a6fd6;
}

.btn-focus.active {
  background: #f44336;
}

.focus-active {
  background: white;
  padding: 24px;
  border-radius: 8px;
  text-align: center;
}

.focus-active h2 {
  color: #4caf50;
  margin-bottom: 16px;
}

.budget {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.budget-label {
  font-size: 12px;
  color: #666;
}

.budget-time {
  font-size: 32px;
  font-weight: bold;
}
```

**Step 3: Create JavaScript**

Create `src/app.js`:
```javascript
const { invoke } = window.__TAURI__.core;

function formatTime(secs) {
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return `${hours}h ${mins}m`;
}

function formatBudget(secs) {
  const mins = Math.floor(secs / 60);
  const s = secs % 60;
  return `${mins}:${s.toString().padStart(2, "0")}`;
}

async function loadStats() {
  try {
    const stats = await invoke("get_today_stats");

    const total = stats.productive_secs + stats.neutral_secs + stats.distracting_secs;
    const maxPercent = total > 0 ? 100 : 0;

    document.getElementById("bar-productive").style.width =
      total > 0 ? `${(stats.productive_secs / total) * 100}%` : "0%";
    document.getElementById("bar-neutral").style.width =
      total > 0 ? `${(stats.neutral_secs / total) * 100}%` : "0%";
    document.getElementById("bar-distracting").style.width =
      total > 0 ? `${(stats.distracting_secs / total) * 100}%` : "0%";

    document.getElementById("time-productive").textContent = formatTime(stats.productive_secs);
    document.getElementById("time-neutral").textContent = formatTime(stats.neutral_secs);
    document.getElementById("time-distracting").textContent = formatTime(stats.distracting_secs);

    const appList = document.getElementById("app-list");
    appList.innerHTML = stats.top_apps.map(app => {
      const prodClass = app.productivity > 0 ? "productive" :
                        app.productivity < 0 ? "distracting" : "neutral";
      return `
        <li>
          <span class="app-name">
            <span class="productivity-dot ${prodClass}"></span>
            ${app.name}
          </span>
          <span>${formatTime(app.duration_secs)}</span>
        </li>
      `;
    }).join("");
  } catch (e) {
    console.error("Failed to load stats:", e);
  }
}

async function loadFocusState() {
  try {
    const state = await invoke("get_focus_state");
    const btn = document.getElementById("focus-btn");
    const statsView = document.getElementById("stats-view");
    const focusView = document.getElementById("focus-view");

    if (state.active) {
      statsView.style.display = "none";
      focusView.style.display = "block";
      btn.textContent = "End Focus Session";
      btn.classList.add("active");
      document.getElementById("budget-time").textContent = formatBudget(state.budget_remaining);
    } else {
      statsView.style.display = "block";
      focusView.style.display = "none";
      btn.textContent = "Start Focus Session";
      btn.classList.remove("active");
    }
  } catch (e) {
    console.error("Failed to load focus state:", e);
  }
}

document.getElementById("focus-btn").addEventListener("click", async () => {
  try {
    const state = await invoke("get_focus_state");
    if (state.active) {
      await invoke("end_focus_session");
    } else {
      await invoke("start_focus_session", { budgetMinutes: 10 });
    }
    loadFocusState();
    loadStats();
  } catch (e) {
    console.error("Failed to toggle focus:", e);
  }
});

// Initial load
loadStats();
loadFocusState();

// Refresh periodically
setInterval(() => {
  loadStats();
  loadFocusState();
}, 5000);
```

**Step 4: Verify it works**

Run: `cargo tauri dev`
Expected: App opens with tray icon, popup shows stats

**Step 5: Commit**

```bash
git add .
git commit -m "feat: add popup UI with stats and focus controls"
```

---

## Phase 6: Linux Support

### Task 15: Implement Linux Window Tracking

**Files:**
- Create: `src-tauri/src/platform/linux.rs`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Add Linux dependencies**

Add to `src-tauri/Cargo.toml`:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { version = "0.13", features = ["screensaver"] }
```

**Step 2: Implement Linux tracker**

Create `src-tauri/src/platform/linux.rs`:
```rust
use super::types::{ActiveWindow, PlatformTracker};
use x11rb::connection::Connection;
use x11rb::protocol::screensaver;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

pub struct LinuxTracker {
    conn: x11rb::rust_connection::RustConnection,
    root: Window,
}

impl LinuxTracker {
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        Self { conn, root }
    }

    fn get_atom(&self, name: &str) -> Option<u32> {
        self.conn
            .intern_atom(false, name.as_bytes())
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    }

    fn get_window_property(&self, window: Window, atom: u32) -> Option<String> {
        let reply = self.conn
            .get_property(false, window, atom, AtomEnum::ANY, 0, 1024)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.is_empty() {
            return None;
        }

        String::from_utf8(reply.value).ok()
    }

    fn get_active_window_id(&self) -> Option<Window> {
        let atom = self.get_atom("_NET_ACTIVE_WINDOW")?;
        let reply = self.conn
            .get_property(false, self.root, atom, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.len() >= 4 {
            Some(u32::from_ne_bytes([
                reply.value[0],
                reply.value[1],
                reply.value[2],
                reply.value[3],
            ]))
        } else {
            None
        }
    }
}

impl PlatformTracker for LinuxTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        let window_id = self.get_active_window_id()?;

        let name_atom = self.get_atom("_NET_WM_NAME")
            .or_else(|| Some(AtomEnum::WM_NAME.into()))?;

        let window_title = self.get_window_property(window_id, name_atom)
            .unwrap_or_else(|| "Unknown".to_string());

        let class_atom = AtomEnum::WM_CLASS.into();
        let app_name = self.get_window_property(window_id, class_atom)
            .map(|s| s.split('\0').next().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        Some(ActiveWindow {
            app_name,
            window_title,
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        let info = screensaver::query_info(&self.conn, self.root)
            .ok()
            .and_then(|cookie| cookie.reply().ok());

        info.map(|i| (i.ms_since_user_input / 1000) as u64).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires X11 display
    fn test_get_active_window() {
        let tracker = LinuxTracker::new();
        if let Some(window) = tracker.get_active_window() {
            println!("Active: {} - {}", window.app_name, window.window_title);
        }
    }
}
```

**Step 3: Verify it compiles (on Linux)**

Run: `cd src-tauri && cargo check --target x86_64-unknown-linux-gnu`
Expected: Compiles (may need cross-compilation setup)

**Step 4: Commit**

```bash
git add .
git commit -m "feat: implement Linux window tracking via X11"
```

---

## Final Task: Seed Default Rules

### Task 16: Add Default Categorization Rules

**Files:**
- Modify: `src-tauri/src/db/migrations.rs`

**Step 1: Update migrations to seed rules**

Add to `src-tauri/src/db/migrations.rs`:
```rust
pub const DEFAULT_RULES: &[(&str, &str, &str)] = &[
    // Coding (productive)
    ("code", "app", "Coding"),
    ("visual studio", "app", "Coding"),
    ("xcode", "app", "Coding"),
    ("intellij", "app", "Coding"),
    ("webstorm", "app", "Coding"),
    ("pycharm", "app", "Coding"),
    ("terminal", "app", "Coding"),
    ("iterm", "app", "Coding"),
    ("github.com", "domain", "Coding"),
    ("gitlab.com", "domain", "Coding"),
    ("stackoverflow.com", "domain", "Reference"),
    ("docs.rs", "domain", "Reference"),

    // Communication (neutral)
    ("slack", "app", "Communication"),
    ("discord", "app", "Communication"),
    ("mail", "app", "Communication"),
    ("outlook", "app", "Communication"),
    ("teams", "app", "Communication"),
    ("zoom", "app", "Communication"),

    // Entertainment (distracting)
    ("youtube.com", "domain", "Entertainment"),
    ("netflix.com", "domain", "Entertainment"),
    ("twitter.com", "domain", "Entertainment"),
    ("x.com", "domain", "Entertainment"),
    ("reddit.com", "domain", "Entertainment"),
    ("facebook.com", "domain", "Entertainment"),
    ("instagram.com", "domain", "Entertainment"),
    ("tiktok.com", "domain", "Entertainment"),
    ("twitch.tv", "domain", "Entertainment"),
];

fn seed_default_rules(conn: &Connection) -> Result<()> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM rules",
        [],
        |row| row.get(0)
    )?;

    if count > 0 {
        return Ok(());
    }

    let categories: std::collections::HashMap<String, i64> = conn
        .prepare("SELECT name, id FROM categories")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (pattern, match_type, category_name) in DEFAULT_RULES {
        if let Some(&category_id) = categories.get(*category_name) {
            conn.execute(
                "INSERT INTO rules (pattern, match_type, category_id, priority) VALUES (?1, ?2, ?3, 10)",
                [*pattern, *match_type, &category_id.to_string()],
            )?;
        }
    }

    Ok(())
}

// Update run() to call seed_default_rules
pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    seed_default_categories(conn)?;
    seed_default_rules(conn)?;
    Ok(())
}
```

**Step 2: Run tests**

Run: `cd src-tauri && cargo test`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add .
git commit -m "feat: seed default categorization rules"
```

---

## Summary

This plan covers the complete Foxus MVP:

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-4 | Project setup, database, models, categorizer |
| 2 | 5-7 | Platform abstraction, macOS tracking, tracker service |
| 3 | 8-9 | Focus session model, focus manager |
| 4 | 10-11 | Chrome extension, native messaging |
| 5 | 12-14 | Tauri tray, commands, popup UI |
| 6 | 15-16 | Linux support, default rules |

Each task follows TDD with:
- Failing test first
- Minimal implementation
- Verify pass
- Commit

Total: 16 tasks, ~50 commits
