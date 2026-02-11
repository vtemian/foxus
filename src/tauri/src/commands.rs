use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::{Activity, Category, FocusSchedule, MatchType, Rule};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
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
pub struct DailyStats {
    /// Unix timestamp for the start of the day
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
    /// -1 = distracting, 0 = neutral, 1 = productive
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
    /// "app", "domain", or "title"
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

#[tauri::command]
pub fn get_today_stats(db: State<Arc<Mutex<Database>>>) -> Result<StatsResponse, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to load statistics".to_string()
    })?;
    let conn = db.connection();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            log::error!("System time error: {}", e);
            "Failed to load statistics".to_string()
        })?
        .as_secs() as i64;

    // Start of today (midnight)
    let today_start = now - (now % 86400);

    let categories = Category::find_all(conn).map_err(|e| {
        log::error!("Failed to load categories: {}", e);
        "Failed to load statistics".to_string()
    })?;
    let totals = Activity::total_duration_by_category(conn, today_start, now)
        .map_err(|e| {
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

    // Get top apps
    let mut stmt = conn.prepare(
        "SELECT app_name, SUM(duration_secs) as total, c.productivity
         FROM activities a
         LEFT JOIN categories c ON a.category_id = c.id
         WHERE a.timestamp >= ?1 AND a.timestamp < ?2 AND a.app_name IS NOT NULL
         GROUP BY app_name
         ORDER BY total DESC
         LIMIT 5"
    ).map_err(|e| {
        log::error!("Failed to prepare top apps query: {}", e);
        "Failed to load statistics".to_string()
    })?;

    let top_apps: Vec<AppStat> = stmt.query_map(rusqlite::params![today_start, now], |row| {
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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            log::error!("System time error: {}", e);
            "Failed to load statistics".to_string()
        })?
        .as_secs() as i64;

    // Start of today (midnight UTC)
    let today_start = now - (now % 86400);
    // Start of 7 days ago
    let week_start = today_start - (6 * 86400); // Include today, so 6 days back

    let categories = Category::find_all(conn).map_err(|e| {
        log::error!("Failed to load categories: {}", e);
        "Failed to load statistics".to_string()
    })?;

    // Calculate daily stats for the last 7 days
    let mut daily_stats = Vec::with_capacity(7);
    let mut total_productive_secs = 0;
    let mut total_neutral_secs = 0;
    let mut total_distracting_secs = 0;

    for day_offset in 0..7 {
        let day_start = week_start + (day_offset * 86400);
        let day_end = day_start + 86400;

        let (productive, neutral, distracting) = calculate_stats(
            conn, &categories, day_start, day_end
        ).map_err(|e| {
            log::error!("Failed to calculate daily stats: {}", e);
            "Failed to load statistics".to_string()
        })?;

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

    // Get top apps for the entire week
    let mut stmt = conn.prepare(
        "SELECT app_name, SUM(duration_secs) as total, c.productivity
         FROM activities a
         LEFT JOIN categories c ON a.category_id = c.id
         WHERE a.timestamp >= ?1 AND a.timestamp < ?2 AND a.app_name IS NOT NULL
         GROUP BY app_name
         ORDER BY total DESC
         LIMIT 10"
    ).map_err(|e| {
        log::error!("Failed to prepare top apps query: {}", e);
        "Failed to load statistics".to_string()
    })?;

    let top_apps: Vec<AppStat> = stmt.query_map(rusqlite::params![week_start, now], |row| {
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

    Ok(WeeklyStatsResponse {
        daily_stats,
        total_productive_secs,
        total_neutral_secs,
        total_distracting_secs,
        top_apps,
    })
}

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
    // Validate input: budget must be positive and reasonable (max 24 hours)
    if budget_minutes <= 0 {
        return Err("Budget must be a positive number of minutes".to_string());
    }
    const MAX_BUDGET_MINUTES: i32 = 24 * 60; // 24 hours
    if budget_minutes > MAX_BUDGET_MINUTES {
        return Err(format!(
            "Budget cannot exceed {} minutes (24 hours)",
            MAX_BUDGET_MINUTES
        ));
    }

    focus_manager
        .start_session(budget_minutes * 60)
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

// Focus schedule commands

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
    // Validate time format (HH:MM)
    validate_time_format(&request.start_time)?;
    validate_time_format(&request.end_time)?;

    // Validate days_of_week format
    validate_days_of_week(&request.days_of_week)?;

    // Validate budget
    if request.distraction_budget_secs < 0 {
        return Err("Distraction budget cannot be negative".to_string());
    }
    const MAX_BUDGET_SECS: i32 = 24 * 60 * 60;
    if request.distraction_budget_secs > MAX_BUDGET_SECS {
        return Err("Distraction budget cannot exceed 24 hours".to_string());
    }

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
    // Validate time format (HH:MM)
    validate_time_format(&request.start_time)?;
    validate_time_format(&request.end_time)?;

    // Validate days_of_week format
    validate_days_of_week(&request.days_of_week)?;

    // Validate budget
    if request.distraction_budget_secs < 0 {
        return Err("Distraction budget cannot be negative".to_string());
    }
    const MAX_BUDGET_SECS: i32 = 24 * 60 * 60;
    if request.distraction_budget_secs > MAX_BUDGET_SECS {
        return Err("Distraction budget cannot exceed 24 hours".to_string());
    }

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to update schedule".to_string()
    })?;
    let conn = db.connection();

    // Check if schedule exists
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

// Category management commands

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
    // Validate productivity value
    if !(-1..=1).contains(&productivity) {
        return Err("Productivity must be -1 (distracting), 0 (neutral), or 1 (productive)".to_string());
    }

    // Validate name
    let name = name.trim();
    if name.is_empty() {
        return Err("Category name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Category name cannot exceed 100 characters".to_string());
    }

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
    // Validate productivity value
    if !(-1..=1).contains(&productivity) {
        return Err("Productivity must be -1 (distracting), 0 (neutral), or 1 (productive)".to_string());
    }

    // Validate name
    let name = name.trim();
    if name.is_empty() {
        return Err("Category name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Category name cannot exceed 100 characters".to_string());
    }

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

// Rule management commands

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
    pattern: String,
    match_type: String,
    category_id: i64,
    priority: i32,
) -> Result<RuleResponse, String> {
    // Validate pattern
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err("Pattern cannot be empty".to_string());
    }
    if pattern.len() > 500 {
        return Err("Pattern cannot exceed 500 characters".to_string());
    }

    // Validate match_type
    let match_type = MatchType::from_str(&match_type)
        .ok_or_else(|| "Invalid match type: must be 'app', 'domain', or 'title'".to_string())?;

    // Validate priority
    if priority < 0 || priority > 1000 {
        return Err("Priority must be between 0 and 1000".to_string());
    }

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to create rule".to_string()
    })?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id).map_err(|e| {
        log::error!("Failed to verify category: {}", e);
        "Failed to create rule".to_string()
    })?.is_none() {
        return Err("Category not found".to_string());
    }

    let rule = Rule::create(conn, pattern, match_type, category_id, priority).map_err(|e| {
        log::error!("Failed to create rule: {}", e);
        "Failed to create rule".to_string()
    })?;

    Ok(RuleResponse::from(rule))
}

#[tauri::command]
pub fn update_rule(
    db: State<Arc<Mutex<Database>>>,
    id: i64,
    pattern: String,
    match_type: String,
    category_id: i64,
    priority: i32,
) -> Result<bool, String> {
    // Validate pattern
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err("Pattern cannot be empty".to_string());
    }
    if pattern.len() > 500 {
        return Err("Pattern cannot exceed 500 characters".to_string());
    }

    // Validate match_type
    let match_type = MatchType::from_str(&match_type)
        .ok_or_else(|| "Invalid match type: must be 'app', 'domain', or 'title'".to_string())?;

    // Validate priority
    if priority < 0 || priority > 1000 {
        return Err("Priority must be between 0 and 1000".to_string());
    }

    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to update rule".to_string()
    })?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id).map_err(|e| {
        log::error!("Failed to verify category: {}", e);
        "Failed to update rule".to_string()
    })?.is_none() {
        return Err("Category not found".to_string());
    }

    Rule::update(conn, id, pattern, match_type, category_id, priority).map_err(|e| {
        log::error!("Failed to update rule: {}", e);
        "Failed to update rule".to_string()
    })
}

#[tauri::command]
pub fn delete_rule(
    db: State<Arc<Mutex<Database>>>,
    id: i64,
) -> Result<bool, String> {
    let db = db.lock().map_err(|e| {
        log::error!("Failed to acquire database lock: {}", e);
        "Failed to delete rule".to_string()
    })?;
    let conn = db.connection();

    Rule::delete(conn, id).map_err(|e| {
        log::error!("Failed to delete rule: {}", e);
        "Failed to delete rule".to_string()
    })
}

// Helper functions for testing without Tauri State wrapper

/// Validate focus session budget input.
/// Returns Ok(budget_secs) if valid, Err(message) if invalid.
fn validate_budget(budget_minutes: i32) -> Result<i32, String> {
    if budget_minutes <= 0 {
        return Err("Budget must be a positive number of minutes".to_string());
    }
    const MAX_BUDGET_MINUTES: i32 = 24 * 60;
    if budget_minutes > MAX_BUDGET_MINUTES {
        return Err(format!(
            "Budget cannot exceed {} minutes (24 hours)",
            MAX_BUDGET_MINUTES
        ));
    }
    Ok(budget_minutes * 60)
}

/// Calculate stats from activities in a time range.
/// This is the core logic used by get_today_stats.
fn calculate_stats(
    conn: &Connection,
    categories: &[Category],
    start: i64,
    end: i64,
) -> rusqlite::Result<(i32, i32, i32)> {
    let totals = Activity::total_duration_by_category(conn, start, end)?;

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

/// Validate time format (HH:MM, 24-hour format).
fn validate_time_format(time: &str) -> Result<(), String> {
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
fn validate_days_of_week(days: &str) -> Result<(), String> {
    if days.is_empty() {
        return Err("At least one day must be selected".to_string());
    }

    for part in days.split(',') {
        let day: u32 = part
            .trim()
            .parse()
            .map_err(|_| format!("Invalid day number: '{}'", part.trim()))?;

        if !(1..=7).contains(&day) {
            return Err(format!("Day must be between 1 (Monday) and 7 (Sunday), got {}", day));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use tempfile::{tempdir, TempDir};

    fn setup_db() -> (Database, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        (db, dir)
    }

    // Budget validation tests

    #[test]
    fn test_validate_budget_rejects_zero() {
        let result = validate_budget(0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive"));
    }

    #[test]
    fn test_validate_budget_rejects_negative() {
        let result = validate_budget(-10);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("positive"));
    }

    #[test]
    fn test_validate_budget_accepts_one_minute() {
        let result = validate_budget(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 60); // 1 minute = 60 seconds
    }

    #[test]
    fn test_validate_budget_accepts_24_hours() {
        let result = validate_budget(24 * 60);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 24 * 60 * 60); // 24 hours in seconds
    }

    #[test]
    fn test_validate_budget_rejects_over_24_hours() {
        let result = validate_budget(24 * 60 + 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("24 hours"));
    }

    #[test]
    fn test_validate_budget_accepts_typical_value() {
        let result = validate_budget(60); // 1 hour
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3600);
    }

    // Stats calculation tests

    #[test]
    fn test_calculate_stats_with_no_activities() {
        let (db, _dir) = setup_db();
        let conn = db.connection();
        let categories = Category::find_all(conn).unwrap();

        let (productive, neutral, distracting) = calculate_stats(conn, &categories, 0, i64::MAX).unwrap();

        assert_eq!(productive, 0);
        assert_eq!(neutral, 0);
        assert_eq!(distracting, 0);
    }

    #[test]
    fn test_calculate_stats_aggregates_by_productivity() {
        let (db, _dir) = setup_db();
        let conn = db.connection();
        let categories = Category::find_all(conn).unwrap();

        let coding = categories.iter().find(|c| c.name == "Coding").unwrap();
        let communication = categories.iter().find(|c| c.name == "Communication").unwrap();
        let entertainment = categories.iter().find(|c| c.name == "Entertainment").unwrap();

        let now = 1700000000i64;

        // Add activities in each category
        let mut a1 = Activity::new(now, 100, "app", Some("VSCode"), None);
        a1.category_id = Some(coding.id);
        a1.save(conn).unwrap();

        let mut a2 = Activity::new(now + 100, 50, "app", Some("Slack"), None);
        a2.category_id = Some(communication.id);
        a2.save(conn).unwrap();

        let mut a3 = Activity::new(now + 200, 30, "app", Some("YouTube"), None);
        a3.category_id = Some(entertainment.id);
        a3.save(conn).unwrap();

        let (productive, neutral, distracting) = calculate_stats(
            conn, &categories, now - 10, now + 1000
        ).unwrap();

        assert_eq!(productive, 100); // Coding is productive (+1)
        assert_eq!(neutral, 50);     // Communication is neutral (0)
        assert_eq!(distracting, 30); // Entertainment is distracting (-1)
    }

    #[test]
    fn test_calculate_stats_respects_time_range() {
        let (db, _dir) = setup_db();
        let conn = db.connection();
        let categories = Category::find_all(conn).unwrap();

        let coding = categories.iter().find(|c| c.name == "Coding").unwrap();

        // Add activity at timestamp 1000
        let mut a1 = Activity::new(1000, 100, "app", Some("VSCode"), None);
        a1.category_id = Some(coding.id);
        a1.save(conn).unwrap();

        // Query range that excludes the activity
        let (productive, neutral, distracting) = calculate_stats(
            conn, &categories, 2000, 3000
        ).unwrap();

        assert_eq!(productive, 0);
        assert_eq!(neutral, 0);
        assert_eq!(distracting, 0);

        // Query range that includes the activity
        let (productive, _, _) = calculate_stats(
            conn, &categories, 500, 1500
        ).unwrap();

        assert_eq!(productive, 100);
    }

    // FocusManager integration tests for commands

    #[test]
    fn test_focus_session_lifecycle() {
        let (db, _dir) = setup_db();
        let db = Arc::new(Mutex::new(db));
        let manager = Arc::new(FocusManager::new(Arc::clone(&db)));

        // Initially inactive
        let state = manager.get_state().unwrap();
        assert!(!state.active);

        // Validate and start session
        let budget_secs = validate_budget(10).unwrap();
        manager.start_session(budget_secs).unwrap();

        // Now active with correct budget
        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert_eq!(state.budget_remaining, 600); // 10 minutes = 600 seconds

        // End session
        manager.end_session().unwrap();

        // Back to inactive
        let state = manager.get_state().unwrap();
        assert!(!state.active);
    }

    #[test]
    fn test_focus_session_replaces_existing() {
        let (db, _dir) = setup_db();
        let db = Arc::new(Mutex::new(db));
        let manager = Arc::new(FocusManager::new(Arc::clone(&db)));

        // Start first session with 10 minutes
        manager.start_session(600).unwrap();
        let state = manager.get_state().unwrap();
        assert_eq!(state.budget_remaining, 600);

        // Start second session with 5 minutes (should replace first)
        manager.start_session(300).unwrap();
        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert_eq!(state.budget_remaining, 300);
    }
}
