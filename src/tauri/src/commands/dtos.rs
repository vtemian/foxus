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
