// src/tauri/src/commands/rules.rs

use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::models::{Category, MatchType, Rule};
use crate::validation::{validate_rule_pattern, validate_rule_priority};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

use super::RuleResponse;

/// Helper to reload the categorizer cache after rule mutations.
/// This fixes a critical bug where creating/updating/deleting rules via the UI
/// would not update the tracker's categorization until app restart.
fn reload_categorizer(
    categorizer: &Arc<Mutex<Categorizer>>,
    conn: &Connection,
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

    // Reload categorizer cache after creating rule
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

    // Reload categorizer cache after updating rule
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

    // Reload categorizer cache after deleting rule
    reload_categorizer(&categorizer, conn)?;

    Ok(result)
}
