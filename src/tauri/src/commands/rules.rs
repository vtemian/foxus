use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::error::AppError;
use crate::models::{Category, MatchType, Rule};
use crate::validation::{validate_rule_pattern, validate_rule_priority};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::State;

use super::RuleResponse;

/// Reload categorizer cache after rule mutations.
fn reload_categorizer(categorizer: &Arc<Mutex<Categorizer>>, conn: &Connection) -> Result<(), String> {
    let mut cat = categorizer.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    cat.reload(conn).map_err(|e| AppError::from(e).to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_rules(db: State<Arc<Mutex<Database>>>) -> Result<Vec<RuleResponse>, String> {
    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let rules = Rule::find_all(db.connection()).map_err(|e| AppError::from(e).to_string())?;
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
    let match_type = MatchType::from_str(&match_type).ok_or_else(|| {
        AppError::InvalidInput {
            field: "match_type",
            reason: "must be 'app', 'domain', or 'title'".into(),
        }
        .to_string()
    })?;
    validate_rule_priority(priority)?;

    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id)
        .map_err(|e| AppError::from(e).to_string())?
        .is_none()
    {
        return Err(AppError::NotFound { entity: "Category" }.to_string());
    }

    let rule = Rule::create(conn, pattern, match_type, category_id, priority)
        .map_err(|e| AppError::from(e).to_string())?;

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
    let match_type = MatchType::from_str(&match_type).ok_or_else(|| {
        AppError::InvalidInput {
            field: "match_type",
            reason: "must be 'app', 'domain', or 'title'".into(),
        }
        .to_string()
    })?;
    validate_rule_priority(priority)?;

    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let conn = db.connection();

    // Verify category exists
    if Category::find_by_id(conn, category_id)
        .map_err(|e| AppError::from(e).to_string())?
        .is_none()
    {
        return Err(AppError::NotFound { entity: "Category" }.to_string());
    }

    let result = Rule::update(conn, id, pattern, match_type, category_id, priority)
        .map_err(|e| AppError::from(e).to_string())?;

    reload_categorizer(&categorizer, conn)?;

    Ok(result)
}

#[tauri::command]
pub fn delete_rule(
    db: State<Arc<Mutex<Database>>>,
    categorizer: State<Arc<Mutex<Categorizer>>>,
    id: i64,
) -> Result<bool, String> {
    let db = db.lock().map_err(|_| AppError::LockPoisoned.to_string())?;
    let conn = db.connection();

    let result = Rule::delete(conn, id).map_err(|e| AppError::from(e).to_string())?;

    reload_categorizer(&categorizer, conn)?;

    Ok(result)
}
