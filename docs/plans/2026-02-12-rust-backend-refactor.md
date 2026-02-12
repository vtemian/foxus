# Rust Backend Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the Foxus Rust backend to improve maintainability by splitting `commands.rs`, fixing the categorizer cache bug, extracting shared helpers, and adding named constants.

**Architecture:** Split the monolithic 1019-line `commands.rs` into feature-based modules (stats, focus, categories, rules). Extract shared validation and database access patterns into helpers. Fix the critical categorizer cache invalidation bug. Add constants for magic numbers.

**Tech Stack:** Rust, Tauri 2.0, rusqlite, serde

---

## Phase 1: Foundation - Constants and Shared Helpers

### Task 1: Add Time Constants Module

**Files:**
- Create: `src/tauri/src/constants.rs`
- Modify: `src/tauri/src/lib.rs:1-10`

**Step 1: Create the constants file**

```rust
// src/tauri/src/constants.rs

/// Seconds in one day (24 * 60 * 60)
pub const SECS_PER_DAY: i64 = 86400;

/// Maximum focus budget in minutes (24 hours)
pub const MAX_BUDGET_MINUTES: i32 = 24 * 60;

/// Maximum focus budget in seconds (24 hours)
pub const MAX_BUDGET_SECS: i32 = 24 * 60 * 60;

/// Maximum rule priority value
pub const MAX_RULE_PRIORITY: i32 = 1000;

/// Maximum category name length
pub const MAX_CATEGORY_NAME_LEN: usize = 100;

/// Maximum rule pattern length
pub const MAX_RULE_PATTERN_LEN: usize = 500;
```

**Step 2: Register module in lib.rs**

Add after line 1 in `src/tauri/src/lib.rs`:
```rust
pub mod constants;
```

**Step 3: Verify it compiles**

Run: `cd src/tauri && cargo check`
Expected: Compiles with no errors (warnings OK)

**Step 4: Commit**

```bash
git add src/tauri/src/constants.rs src/tauri/src/lib.rs
git commit -m "feat(rust): add constants module for magic numbers"
```

---

### Task 2: Add Database Helper Module

**Files:**
- Create: `src/tauri/src/db/helpers.rs`
- Modify: `src/tauri/src/db/mod.rs`

**Step 1: Create the helpers file**

```rust
// src/tauri/src/db/helpers.rs

use crate::db::Database;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Execute a database operation with proper lock handling and error mapping.
///
/// This reduces boilerplate in Tauri commands from ~10 lines to 1 line.
///
/// # Example
/// ```ignore
/// with_connection(&db, "load categories", |conn| {
///     Category::find_all(conn)
/// })
/// ```
pub fn with_connection<F, T>(
    db: &Arc<Mutex<Database>>,
    operation: &str,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&Connection) -> rusqlite::Result<T>,
{
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock for {}: {}", operation, e);
        format!("Failed to {}", operation)
    })?;

    f(db.connection()).map_err(|e| {
        log::error!("Failed to {}: {}", operation, e);
        format!("Failed to {}", operation)
    })
}

/// Execute a database operation, mapping specific SQLite errors to user-friendly messages.
pub fn with_connection_mapped<F, T, M>(
    db: &Arc<Mutex<Database>>,
    operation: &str,
    f: F,
    error_mapper: M,
) -> Result<T, String>
where
    F: FnOnce(&Connection) -> rusqlite::Result<T>,
    M: FnOnce(&str) -> String,
{
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock for {}: {}", operation, e);
        format!("Failed to {}", operation)
    })?;

    f(db.connection()).map_err(|e| {
        let err_str = e.to_string();
        log::error!("Failed to {}: {}", operation, err_str);
        error_mapper(&err_str)
    })
}
```

**Step 2: Export from db/mod.rs**

Add to `src/tauri/src/db/mod.rs` after line 1:
```rust
pub mod helpers;
pub use helpers::{with_connection, with_connection_mapped};
```

**Step 3: Verify it compiles**

Run: `cd src/tauri && cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/tauri/src/db/helpers.rs src/tauri/src/db/mod.rs
git commit -m "feat(rust): add database helper functions for commands"
```

---

### Task 3: Add Validation Module

**Files:**
- Create: `src/tauri/src/validation.rs`
- Modify: `src/tauri/src/lib.rs`

**Step 1: Create the validation file**

```rust
// src/tauri/src/validation.rs

use crate::constants::*;

/// Validate focus session budget in minutes.
/// Returns Ok(budget_secs) if valid, Err(message) if invalid.
pub fn validate_budget_minutes(budget_minutes: i32) -> Result<i32, String> {
    if budget_minutes <= 0 {
        return Err("Budget must be a positive number of minutes".to_string());
    }
    if budget_minutes > MAX_BUDGET_MINUTES {
        return Err(format!(
            "Budget cannot exceed {} minutes (24 hours)",
            MAX_BUDGET_MINUTES
        ));
    }
    Ok(budget_minutes * 60)
}

/// Validate focus schedule budget in seconds.
pub fn validate_budget_secs(budget_secs: i32) -> Result<(), String> {
    if budget_secs < 0 {
        return Err("Distraction budget cannot be negative".to_string());
    }
    if budget_secs > MAX_BUDGET_SECS {
        return Err("Distraction budget cannot exceed 24 hours".to_string());
    }
    Ok(())
}

/// Validate time format (HH:MM, 24-hour format).
pub fn validate_time_format(time: &str) -> Result<(), String> {
    if time.len() != 5 {
        return Err("Time must be in HH:MM format".to_string());
    }
    if &time[2..3] != ":" {
        return Err("Time must be in HH:MM format".to_string());
    }

    let hours: u32 = time[0..2]
        .parse()
        .map_err(|_| "Invalid hours in time".to_string())?;
    let minutes: u32 = time[3..5]
        .parse()
        .map_err(|_| "Invalid minutes in time".to_string())?;

    if hours >= 24 {
        return Err("Hours must be between 00 and 23".to_string());
    }
    if minutes >= 60 {
        return Err("Minutes must be between 00 and 59".to_string());
    }

    Ok(())
}

/// Validate days_of_week format (comma-separated day numbers 1-7).
pub fn validate_days_of_week(days: &str) -> Result<(), String> {
    if days.is_empty() {
        return Err("At least one day must be selected".to_string());
    }

    for part in days.split(',') {
        let day: u32 = part
            .trim()
            .parse()
            .map_err(|_| format!("Invalid day number: '{}'", part.trim()))?;

        if !(1..=7).contains(&day) {
            return Err(format!(
                "Day must be between 1 (Monday) and 7 (Sunday), got {}",
                day
            ));
        }
    }

    Ok(())
}

/// Validate productivity value (-1, 0, or 1).
pub fn validate_productivity(productivity: i32) -> Result<(), String> {
    if !(-1..=1).contains(&productivity) {
        return Err(
            "Productivity must be -1 (distracting), 0 (neutral), or 1 (productive)".to_string(),
        );
    }
    Ok(())
}

/// Validate category name.
pub fn validate_category_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Category name cannot be empty".to_string());
    }
    if name.len() > MAX_CATEGORY_NAME_LEN {
        return Err(format!(
            "Category name cannot exceed {} characters",
            MAX_CATEGORY_NAME_LEN
        ));
    }
    Ok(name)
}

/// Validate rule pattern.
pub fn validate_rule_pattern(pattern: &str) -> Result<&str, String> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err("Pattern cannot be empty".to_string());
    }
    if pattern.len() > MAX_RULE_PATTERN_LEN {
        return Err(format!(
            "Pattern cannot exceed {} characters",
            MAX_RULE_PATTERN_LEN
        ));
    }
    Ok(pattern)
}

/// Validate rule priority.
pub fn validate_rule_priority(priority: i32) -> Result<(), String> {
    if priority < 0 || priority > MAX_RULE_PRIORITY {
        return Err(format!(
            "Priority must be between 0 and {}",
            MAX_RULE_PRIORITY
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_budget_minutes_valid() {
        assert!(validate_budget_minutes(10).is_ok());
        assert_eq!(validate_budget_minutes(10).unwrap(), 600);
    }

    #[test]
    fn test_validate_budget_minutes_zero() {
        assert!(validate_budget_minutes(0).is_err());
    }

    #[test]
    fn test_validate_budget_minutes_negative() {
        assert!(validate_budget_minutes(-5).is_err());
    }

    #[test]
    fn test_validate_budget_minutes_too_large() {
        assert!(validate_budget_minutes(MAX_BUDGET_MINUTES + 1).is_err());
    }

    #[test]
    fn test_validate_time_format_valid() {
        assert!(validate_time_format("09:00").is_ok());
        assert!(validate_time_format("23:59").is_ok());
        assert!(validate_time_format("00:00").is_ok());
    }

    #[test]
    fn test_validate_time_format_invalid() {
        assert!(validate_time_format("9:00").is_err());
        assert!(validate_time_format("25:00").is_err());
        assert!(validate_time_format("12:60").is_err());
    }

    #[test]
    fn test_validate_days_of_week_valid() {
        assert!(validate_days_of_week("1,2,3").is_ok());
        assert!(validate_days_of_week("7").is_ok());
        assert!(validate_days_of_week("1,2,3,4,5,6,7").is_ok());
    }

    #[test]
    fn test_validate_days_of_week_invalid() {
        assert!(validate_days_of_week("").is_err());
        assert!(validate_days_of_week("0").is_err());
        assert!(validate_days_of_week("8").is_err());
    }
}
```

**Step 2: Register module in lib.rs**

Add after `pub mod constants;` in `src/tauri/src/lib.rs`:
```rust
pub mod validation;
```

**Step 3: Run the tests**

Run: `cd src/tauri && cargo test validation`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/tauri/src/validation.rs src/tauri/src/lib.rs
git commit -m "feat(rust): add validation module with tests"
```

---

## Phase 2: Fix Critical Bug - Categorizer Cache Invalidation

### Task 4: Fix Categorizer Cache Invalidation in Rule Commands

**Files:**
- Modify: `src/tauri/src/commands.rs:632-742` (rule commands)

**Step 1: Update create_rule to reload categorizer**

Find `create_rule` function (around line 632) and add categorizer parameter and reload:

```rust
#[tauri::command]
pub fn create_rule(
    db: State<Arc<Mutex<Database>>>,
    categorizer: State<Arc<Mutex<Categorizer>>>,  // ADD THIS
    pattern: String,
    match_type: String,
    category_id: i64,
    priority: i32,
) -> Result<RuleResponse, String> {
    // ... existing validation code ...

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to create rule".to_string()
    })?;
    let conn = db.connection();

    // ... existing category verification ...

    let rule = Rule::create(conn, pattern, match_type, category_id, priority).map_err(|e| {
        log::error!("Failed to create rule: {}", e);
        "Failed to create rule".to_string()
    })?;

    // Reload categorizer cache after creating rule
    let mut cat = categorizer.lock().map_err(|e| {
        log::error!("Failed to acquire categorizer lock: {}", e);
        "Failed to update categorizer".to_string()
    })?;
    cat.reload(conn).map_err(|e| {
        log::error!("Failed to reload categorizer: {}", e);
        "Failed to update categorizer".to_string()
    })?;

    Ok(RuleResponse::from(rule))
}
```

**Step 2: Update update_rule to reload categorizer**

Add same pattern to `update_rule` (around line 680).

**Step 3: Update delete_rule to reload categorizer**

Add same pattern to `delete_rule` (around line 727).

**Step 4: Add Categorizer import if missing**

Ensure `use crate::categorizer::Categorizer;` is at the top of commands.rs.

**Step 5: Verify it compiles**

Run: `cd src/tauri && cargo check`
Expected: Compiles with no errors

**Step 6: Commit**

```bash
git add src/tauri/src/commands.rs
git commit -m "fix(rust): reload categorizer cache after rule mutations

This fixes a bug where creating/updating/deleting rules via the UI
would not update the tracker's categorization until app restart."
```

---

## Phase 3: Split commands.rs into Feature Modules

### Task 5: Create Commands Module Directory Structure

**Files:**
- Create: `src/tauri/src/commands/mod.rs`
- Create: `src/tauri/src/commands/dtos.rs`

**Step 1: Create directory**

```bash
mkdir -p src/tauri/src/commands
```

**Step 2: Create DTOs file**

Extract all structs from commands.rs (lines 10-129) to `src/tauri/src/commands/dtos.rs`:

```rust
// src/tauri/src/commands/dtos.rs

use crate::models::{Category, FocusSchedule, Rule};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct StatsResponse {
    pub productive_secs: i32,
    pub neutral_secs: i32,
    pub distracting_secs: i32,
    pub top_apps: Vec<AppStat>,
}

#[derive(Serialize)]
pub struct DailyStats {
    pub date: i64,
    pub productive_secs: i32,
    pub neutral_secs: i32,
    pub distracting_secs: i32,
}

#[derive(Serialize)]
pub struct WeeklyStatsResponse {
    pub daily_stats: Vec<DailyStats>,
    pub total_productive_secs: i32,
    pub total_neutral_secs: i32,
    pub total_distracting_secs: i32,
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

#[derive(Serialize)]
pub struct FocusScheduleResponse {
    pub id: i64,
    pub days_of_week: String,
    pub start_time: String,
    pub end_time: String,
    pub distraction_budget_secs: i32,
    pub enabled: bool,
}

impl From<FocusSchedule> for FocusScheduleResponse {
    fn from(schedule: FocusSchedule) -> Self {
        Self {
            id: schedule.id.unwrap_or(0),
            days_of_week: schedule.days_of_week,
            start_time: schedule.start_time,
            end_time: schedule.end_time,
            distraction_budget_secs: schedule.distraction_budget,
            enabled: schedule.enabled,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateScheduleRequest {
    pub days_of_week: String,
    pub start_time: String,
    pub end_time: String,
    pub distraction_budget_secs: i32,
}

#[derive(Deserialize)]
pub struct UpdateScheduleRequest {
    pub id: i64,
    pub days_of_week: String,
    pub start_time: String,
    pub end_time: String,
    pub distraction_budget_secs: i32,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub productivity: i32,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            productivity: category.productivity,
        }
    }
}

#[derive(Serialize)]
pub struct RuleResponse {
    pub id: i64,
    pub pattern: String,
    pub match_type: String,
    pub category_id: i64,
    pub priority: i32,
}

impl From<Rule> for RuleResponse {
    fn from(rule: Rule) -> Self {
        Self {
            id: rule.id,
            pattern: rule.pattern,
            match_type: rule.match_type.as_str().to_string(),
            category_id: rule.category_id,
            priority: rule.priority,
        }
    }
}
```

**Step 3: Create mod.rs**

```rust
// src/tauri/src/commands/mod.rs

mod dtos;
pub mod stats;
pub mod focus;
pub mod categories;
pub mod rules;

pub use dtos::*;
pub use stats::*;
pub use focus::*;
pub use categories::*;
pub use rules::*;
```

**Step 4: Verify structure (won't compile yet)**

Run: `ls src/tauri/src/commands/`
Expected: `dtos.rs  mod.rs`

**Step 5: Commit**

```bash
git add src/tauri/src/commands/
git commit -m "chore(rust): create commands module structure with DTOs"
```

---

### Task 6: Extract Stats Commands

**Files:**
- Create: `src/tauri/src/commands/stats.rs`

**Step 1: Create stats.rs**

```rust
// src/tauri/src/commands/stats.rs

use crate::constants::SECS_PER_DAY;
use crate::db::Database;
use crate::models::{Activity, Category};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

use super::{AppStat, DailyStats, StatsResponse, WeeklyStatsResponse};

#[tauri::command]
pub fn get_today_stats(db: State<Arc<Mutex<Database>>>) -> Result<StatsResponse, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load statistics".to_string()
    })?;
    let conn = db.connection();

    let now = get_current_timestamp()?;
    let today_start = now - (now % SECS_PER_DAY);

    let categories = Category::find_all(conn).map_err(|e| {
        log::error!("Failed to load categories: {}", e);
        "Failed to load statistics".to_string()
    })?;

    let (productive_secs, neutral_secs, distracting_secs) =
        calculate_productivity_totals(conn, &categories, today_start, now)?;

    let top_apps = query_top_apps(conn, today_start, now, 5)?;

    Ok(StatsResponse {
        productive_secs,
        neutral_secs,
        distracting_secs,
        top_apps,
    })
}

#[tauri::command]
pub fn get_weekly_stats(db: State<Arc<Mutex<Database>>>) -> Result<WeeklyStatsResponse, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load statistics".to_string()
    })?;
    let conn = db.connection();

    let now = get_current_timestamp()?;
    let today_start = now - (now % SECS_PER_DAY);
    let week_start = today_start - (6 * SECS_PER_DAY);

    let categories = Category::find_all(conn).map_err(|e| {
        log::error!("Failed to load categories: {}", e);
        "Failed to load statistics".to_string()
    })?;

    let mut daily_stats = Vec::with_capacity(7);
    let mut total_productive_secs = 0;
    let mut total_neutral_secs = 0;
    let mut total_distracting_secs = 0;

    for day_offset in 0..7 {
        let day_start = week_start + (day_offset * SECS_PER_DAY);
        let day_end = day_start + SECS_PER_DAY;

        let (productive, neutral, distracting) =
            calculate_productivity_totals(conn, &categories, day_start, day_end)?;

        daily_stats.push(DailyStats {
            date: day_start,
            productive_secs: productive,
            neutral_secs: neutral,
            distracting_secs: distracting,
        });

        total_productive_secs += productive;
        total_neutral_secs += neutral;
        total_distracting_secs += distracting;
    }

    let top_apps = query_top_apps(conn, week_start, now, 10)?;

    Ok(WeeklyStatsResponse {
        daily_stats,
        total_productive_secs,
        total_neutral_secs,
        total_distracting_secs,
        top_apps,
    })
}

// Helper functions

fn get_current_timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            log::error!("System time error: {}", e);
            "Failed to load statistics".to_string()
        })
        .map(|d| d.as_secs() as i64)
}

fn calculate_productivity_totals(
    conn: &rusqlite::Connection,
    categories: &[Category],
    start: i64,
    end: i64,
) -> Result<(i32, i32, i32), String> {
    let totals = Activity::total_duration_by_category(conn, start, end).map_err(|e| {
        log::error!("Failed to load activity totals: {}", e);
        "Failed to load statistics".to_string()
    })?;

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

    Ok((productive_secs, neutral_secs, distracting_secs))
}

fn query_top_apps(
    conn: &rusqlite::Connection,
    start: i64,
    end: i64,
    limit: usize,
) -> Result<Vec<AppStat>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT app_name, SUM(duration_secs) as total, c.productivity
             FROM activities a
             LEFT JOIN categories c ON a.category_id = c.id
             WHERE a.timestamp >= ?1 AND a.timestamp < ?2 AND a.app_name IS NOT NULL
             GROUP BY app_name
             ORDER BY total DESC
             LIMIT ?3",
        )
        .map_err(|e| {
            log::error!("Failed to prepare top apps query: {}", e);
            "Failed to load statistics".to_string()
        })?;

    let top_apps: Vec<AppStat> = stmt
        .query_map(rusqlite::params![start, end, limit as i32], |row| {
            Ok(AppStat {
                name: row.get(0)?,
                duration_secs: row.get(1)?,
                productivity: row.get::<_, Option<i32>>(2)?.unwrap_or(0),
            })
        })
        .map_err(|e| {
            log::error!("Failed to query top apps: {}", e);
            "Failed to load statistics".to_string()
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(top_apps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_timestamp() {
        let ts = get_current_timestamp().unwrap();
        assert!(ts > 0);
    }
}
```

**Step 2: Verify it compiles**

Run: `cd src/tauri && cargo check`
Expected: May have errors - we'll fix in next steps

**Step 3: Commit**

```bash
git add src/tauri/src/commands/stats.rs
git commit -m "feat(rust): extract stats commands to separate module"
```

---

### Task 7: Extract Focus Commands

**Files:**
- Create: `src/tauri/src/commands/focus.rs`

**Step 1: Create focus.rs**

```rust
// src/tauri/src/commands/focus.rs

use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::FocusSchedule;
use crate::validation::{validate_budget_minutes, validate_budget_secs, validate_days_of_week, validate_time_format};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::{CreateScheduleRequest, FocusScheduleResponse, FocusStateResponse, UpdateScheduleRequest};

#[tauri::command]
pub fn get_focus_state(focus_manager: State<Arc<FocusManager>>) -> Result<FocusStateResponse, String> {
    let state = focus_manager.get_state().map_err(|e| {
        log::error!("Failed to get focus state: {}", e);
        "Failed to load focus state".to_string()
    })?;

    Ok(FocusStateResponse {
        active: state.active,
        budget_remaining: state.budget_remaining,
        session_duration_secs: state.session_duration_secs,
    })
}

#[tauri::command]
pub fn start_focus_session(
    focus_manager: State<Arc<FocusManager>>,
    budget_minutes: i32,
) -> Result<(), String> {
    let budget_secs = validate_budget_minutes(budget_minutes)?;

    focus_manager
        .start_session(budget_secs)
        .map_err(|_| "Failed to start focus session".to_string())?;
    Ok(())
}

#[tauri::command]
pub fn end_focus_session(focus_manager: State<Arc<FocusManager>>) -> Result<(), String> {
    focus_manager.end_session().map_err(|e| {
        log::error!("Failed to end focus session: {}", e);
        "Failed to end focus session".to_string()
    })?;
    Ok(())
}

#[tauri::command]
pub fn get_focus_schedules(db: State<Arc<Mutex<Database>>>) -> Result<Vec<FocusScheduleResponse>, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load schedules".to_string()
    })?;
    let conn = db.connection();

    let schedules = FocusSchedule::find_all(conn).map_err(|e| {
        log::error!("Failed to load schedules: {}", e);
        "Failed to load schedules".to_string()
    })?;

    Ok(schedules.into_iter().map(FocusScheduleResponse::from).collect())
}

#[tauri::command]
pub fn create_focus_schedule(
    db: State<Arc<Mutex<Database>>>,
    request: CreateScheduleRequest,
) -> Result<FocusScheduleResponse, String> {
    validate_schedule_request(&request.start_time, &request.end_time, &request.days_of_week, request.distraction_budget_secs)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to create schedule".to_string()
    })?;
    let conn = db.connection();

    let mut schedule = FocusSchedule::new(
        &request.days_of_week,
        &request.start_time,
        &request.end_time,
        request.distraction_budget_secs,
    );
    schedule.save(conn).map_err(|e| {
        log::error!("Failed to save schedule: {}", e);
        "Failed to create schedule".to_string()
    })?;

    Ok(FocusScheduleResponse::from(schedule))
}

#[tauri::command]
pub fn update_focus_schedule(
    db: State<Arc<Mutex<Database>>>,
    request: UpdateScheduleRequest,
) -> Result<FocusScheduleResponse, String> {
    validate_schedule_request(&request.start_time, &request.end_time, &request.days_of_week, request.distraction_budget_secs)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to update schedule".to_string()
    })?;
    let conn = db.connection();

    let existing = FocusSchedule::find_by_id(conn, request.id).map_err(|e| {
        log::error!("Failed to find schedule: {}", e);
        "Failed to update schedule".to_string()
    })?;

    if existing.is_none() {
        return Err("Schedule not found".to_string());
    }

    let schedule = FocusSchedule {
        id: Some(request.id),
        days_of_week: request.days_of_week,
        start_time: request.start_time,
        end_time: request.end_time,
        distraction_budget: request.distraction_budget_secs,
        enabled: request.enabled,
    };

    schedule.update(conn).map_err(|e| {
        log::error!("Failed to update schedule: {}", e);
        "Failed to update schedule".to_string()
    })?;

    Ok(FocusScheduleResponse::from(schedule))
}

#[tauri::command]
pub fn delete_focus_schedule(
    db: State<Arc<Mutex<Database>>>,
    id: i64,
) -> Result<bool, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to delete schedule".to_string()
    })?;
    let conn = db.connection();

    FocusSchedule::delete(conn, id).map_err(|e| {
        log::error!("Failed to delete schedule: {}", e);
        "Failed to delete schedule".to_string()
    })
}

#[tauri::command]
pub fn get_active_schedule(
    focus_manager: State<Arc<FocusManager>>,
) -> Result<Option<FocusScheduleResponse>, String> {
    let schedule = focus_manager.get_active_schedule().map_err(|e| {
        log::error!("Failed to get active schedule: {}", e);
        "Failed to get active schedule".to_string()
    })?;

    Ok(schedule.map(FocusScheduleResponse::from))
}

#[tauri::command]
pub fn check_focus_schedules(focus_manager: State<Arc<FocusManager>>) -> Result<(), String> {
    focus_manager.check_schedules().map_err(|e| {
        log::error!("Failed to check schedules: {}", e);
        "Failed to check schedules".to_string()
    })?;
    Ok(())
}

// Helper function to consolidate schedule validation
fn validate_schedule_request(
    start_time: &str,
    end_time: &str,
    days_of_week: &str,
    budget_secs: i32,
) -> Result<(), String> {
    validate_time_format(start_time)?;
    validate_time_format(end_time)?;
    validate_days_of_week(days_of_week)?;
    validate_budget_secs(budget_secs)?;
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/tauri/src/commands/focus.rs
git commit -m "feat(rust): extract focus commands to separate module"
```

---

### Task 8: Extract Category Commands

**Files:**
- Create: `src/tauri/src/commands/categories.rs`

**Step 1: Create categories.rs**

```rust
// src/tauri/src/commands/categories.rs

use crate::db::Database;
use crate::models::Category;
use crate::validation::{validate_category_name, validate_productivity};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::CategoryResponse;

#[tauri::command]
pub fn get_categories(db: State<Arc<Mutex<Database>>>) -> Result<Vec<CategoryResponse>, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load categories".to_string()
    })?;
    let conn = db.connection();

    let categories = Category::find_all(conn).map_err(|e| {
        log::error!("Failed to load categories: {}", e);
        "Failed to load categories".to_string()
    })?;

    Ok(categories.into_iter().map(CategoryResponse::from).collect())
}

#[tauri::command]
pub fn create_category(
    db: State<Arc<Mutex<Database>>>,
    name: String,
    productivity: i32,
) -> Result<CategoryResponse, String> {
    validate_productivity(productivity)?;
    let name = validate_category_name(&name)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to create category".to_string()
    })?;
    let conn = db.connection();

    let category = Category::create(conn, name, productivity).map_err(|e| {
        log::error!("Failed to create category: {}", e);
        if e.to_string().contains("UNIQUE constraint") {
            "A category with this name already exists".to_string()
        } else {
            "Failed to create category".to_string()
        }
    })?;

    Ok(CategoryResponse::from(category))
}

#[tauri::command]
pub fn update_category(
    db: State<Arc<Mutex<Database>>>,
    id: i64,
    name: String,
    productivity: i32,
) -> Result<bool, String> {
    validate_productivity(productivity)?;
    let name = validate_category_name(&name)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to update category".to_string()
    })?;
    let conn = db.connection();

    Category::update(conn, id, name, productivity).map_err(|e| {
        log::error!("Failed to update category: {}", e);
        if e.to_string().contains("UNIQUE constraint") {
            "A category with this name already exists".to_string()
        } else {
            "Failed to update category".to_string()
        }
    })
}

#[tauri::command]
pub fn delete_category(
    db: State<Arc<Mutex<Database>>>,
    id: i64,
) -> Result<bool, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to delete category".to_string()
    })?;
    let conn = db.connection();

    Category::delete(conn, id).map_err(|e| {
        log::error!("Failed to delete category: {}", e);
        if e.to_string().contains("FOREIGN KEY constraint") {
            "Cannot delete category: it is referenced by rules or activities".to_string()
        } else {
            "Failed to delete category".to_string()
        }
    })
}
```

**Step 2: Commit**

```bash
git add src/tauri/src/commands/categories.rs
git commit -m "feat(rust): extract category commands to separate module"
```

---

### Task 9: Extract Rule Commands

**Files:**
- Create: `src/tauri/src/commands/rules.rs`

**Step 1: Create rules.rs**

```rust
// src/tauri/src/commands/rules.rs

use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::models::{Category, MatchType, Rule};
use crate::validation::{validate_rule_pattern, validate_rule_priority};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::RuleResponse;

#[tauri::command]
pub fn get_rules(db: State<Arc<Mutex<Database>>>) -> Result<Vec<RuleResponse>, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load rules".to_string()
    })?;
    let conn = db.connection();

    let rules = Rule::find_all(conn).map_err(|e| {
        log::error!("Failed to load rules: {}", e);
        "Failed to load rules".to_string()
    })?;

    Ok(rules.into_iter().map(RuleResponse::from).collect())
}

#[tauri::command]
pub fn create_rule(
    db: State<Arc<Mutex<Database>>>,
    categorizer: State<Arc<Mutex<Categorizer>>>,
    pattern: String,
    match_type: String,
    category_id: i64,
    priority: i32,
) -> Result<RuleResponse, String> {
    let pattern = validate_rule_pattern(&pattern)?;
    let match_type = MatchType::from_str(&match_type)
        .ok_or_else(|| "Invalid match type: must be 'app', 'domain', or 'title'".to_string())?;
    validate_rule_priority(priority)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to create rule".to_string()
    })?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id)
        .map_err(|e| {
            log::error!("Failed to verify category: {}", e);
            "Failed to create rule".to_string()
        })?
        .is_none()
    {
        return Err("Category not found".to_string());
    }

    let rule = Rule::create(conn, pattern, match_type, category_id, priority).map_err(|e| {
        log::error!("Failed to create rule: {}", e);
        "Failed to create rule".to_string()
    })?;

    // Reload categorizer cache
    reload_categorizer(&categorizer, conn)?;

    Ok(RuleResponse::from(rule))
}

#[tauri::command]
pub fn update_rule(
    db: State<Arc<Mutex<Database>>>,
    categorizer: State<Arc<Mutex<Categorizer>>>,
    id: i64,
    pattern: String,
    match_type: String,
    category_id: i64,
    priority: i32,
) -> Result<bool, String> {
    let pattern = validate_rule_pattern(&pattern)?;
    let match_type = MatchType::from_str(&match_type)
        .ok_or_else(|| "Invalid match type: must be 'app', 'domain', or 'title'".to_string())?;
    validate_rule_priority(priority)?;

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to update rule".to_string()
    })?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id)
        .map_err(|e| {
            log::error!("Failed to verify category: {}", e);
            "Failed to update rule".to_string()
        })?
        .is_none()
    {
        return Err("Category not found".to_string());
    }

    let result = Rule::update(conn, id, pattern, match_type, category_id, priority).map_err(|e| {
        log::error!("Failed to update rule: {}", e);
        "Failed to update rule".to_string()
    })?;

    // Reload categorizer cache
    reload_categorizer(&categorizer, conn)?;

    Ok(result)
}

#[tauri::command]
pub fn delete_rule(
    db: State<Arc<Mutex<Database>>>,
    categorizer: State<Arc<Mutex<Categorizer>>>,
    id: i64,
) -> Result<bool, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to delete rule".to_string()
    })?;
    let conn = db.connection();

    let result = Rule::delete(conn, id).map_err(|e| {
        log::error!("Failed to delete rule: {}", e);
        "Failed to delete rule".to_string()
    })?;

    // Reload categorizer cache
    reload_categorizer(&categorizer, conn)?;

    Ok(result)
}

fn reload_categorizer(
    categorizer: &Arc<Mutex<Categorizer>>,
    conn: &rusqlite::Connection,
) -> Result<(), String> {
    let mut cat = categorizer.lock().map_err(|e| {
        log::error!("Failed to acquire categorizer lock: {}", e);
        "Failed to update categorizer".to_string()
    })?;
    cat.reload(conn).map_err(|e| {
        log::error!("Failed to reload categorizer: {}", e);
        "Failed to update categorizer".to_string()
    })?;
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/tauri/src/commands/rules.rs
git commit -m "feat(rust): extract rule commands with categorizer cache fix"
```

---

### Task 10: Update lib.rs to Use New Commands Module

**Files:**
- Modify: `src/tauri/src/lib.rs:1-10`
- Delete: `src/tauri/src/commands.rs` (old file)

**Step 1: Update lib.rs imports**

Replace the old `mod commands;` with:
```rust
mod commands;
```

The module system will now look for `commands/mod.rs` instead of `commands.rs`.

**Step 2: Verify it compiles**

Run: `cd src/tauri && cargo check`
Expected: Compiles with no errors

**Step 3: Run all tests**

Run: `cd src/tauri && cargo test`
Expected: All tests pass

**Step 4: Delete old commands.rs**

```bash
rm src/tauri/src/commands.rs
```

**Step 5: Commit**

```bash
git add src/tauri/src/lib.rs
git rm src/tauri/src/commands.rs
git commit -m "refactor(rust): switch to modular commands structure

BREAKING: commands.rs is now commands/ directory with:
- dtos.rs: Request/response types
- stats.rs: get_today_stats, get_weekly_stats
- focus.rs: Focus session and schedule commands
- categories.rs: Category CRUD
- rules.rs: Rule CRUD with categorizer cache invalidation"
```

---

## Phase 4: Verification

### Task 11: Full Integration Test

**Step 1: Clean build**

Run: `cd src/tauri && cargo clean && cargo build`
Expected: Builds successfully

**Step 2: Run all tests**

Run: `cd src/tauri && cargo test`
Expected: All tests pass

**Step 3: Run the app**

Run: `cargo tauri dev`
Expected: App launches, tray icon appears

**Step 4: Test rule changes**

1. Open the app
2. Create a new rule via the UI (if UI exists) or verify tracker still categorizes correctly
3. The categorizer cache should now reload automatically

**Step 5: Final commit**

```bash
git add -A
git commit -m "test: verify refactored commands module"
```

---

## Summary

### Files Created
| File | Purpose |
|------|---------|
| `src/tauri/src/constants.rs` | Named constants for magic numbers |
| `src/tauri/src/validation.rs` | Shared validation functions |
| `src/tauri/src/db/helpers.rs` | Database access helpers |
| `src/tauri/src/commands/mod.rs` | Commands module root |
| `src/tauri/src/commands/dtos.rs` | Request/response types |
| `src/tauri/src/commands/stats.rs` | Stats commands |
| `src/tauri/src/commands/focus.rs` | Focus commands |
| `src/tauri/src/commands/categories.rs` | Category commands |
| `src/tauri/src/commands/rules.rs` | Rule commands |

### Files Modified
| File | Change |
|------|--------|
| `src/tauri/src/lib.rs` | Added module declarations |
| `src/tauri/src/db/mod.rs` | Added helpers export |

### Files Deleted
| File | Reason |
|------|--------|
| `src/tauri/src/commands.rs` | Replaced by commands/ module |

### Key Improvements
1. **Fixed critical bug**: Categorizer cache now reloads after rule mutations
2. **Reduced commands.rs**: From 1019 lines to ~5 files of ~100-150 lines each
3. **DRY validation**: Consolidated into validation.rs
4. **Named constants**: No more magic numbers
5. **Better testability**: Smaller, focused modules
