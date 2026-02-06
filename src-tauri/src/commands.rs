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
