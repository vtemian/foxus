use crate::db::{with_connection, Database};
use crate::error::{is_fk_violation, is_unique_violation, AppError};
use crate::models::Category;
use crate::validation::{validate_category_name, validate_productivity};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::CategoryResponse;

#[tauri::command]
pub fn get_categories(db: State<Arc<Mutex<Database>>>) -> Result<Vec<CategoryResponse>, String> {
    let categories = with_connection(&db, |conn| Category::find_all(conn))?;
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

    let category = with_connection(&db, |conn| Category::create(conn, name, productivity))
        .map_err(|e| match &e {
            AppError::Database(db_err) if is_unique_violation(db_err) => {
                AppError::AlreadyExists { name: name.into() }
            }
            _ => e,
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

    let result = with_connection(&db, |conn| Category::update(conn, id, name, productivity))
        .map_err(|e| match &e {
            AppError::Database(db_err) if is_unique_violation(db_err) => {
                AppError::AlreadyExists { name: name.into() }
            }
            _ => e,
        })?;

    Ok(result)
}

#[tauri::command]
pub fn delete_category(db: State<Arc<Mutex<Database>>>, id: i64) -> Result<bool, String> {
    let result = with_connection(&db, |conn| Category::delete(conn, id)).map_err(|e| match &e {
        AppError::Database(db_err) if is_fk_violation(db_err) => AppError::DeleteFailed {
            reason: "category is used by rules or activities".into(),
        },
        _ => e,
    })?;

    Ok(result)
}
