// src/tauri/src/new_commands/stats.rs

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
