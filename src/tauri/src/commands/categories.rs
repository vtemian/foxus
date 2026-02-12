// src/tauri/src/commands/categories.rs

use crate::db::{with_connection, with_connection_mapped, Database};
use crate::models::Category;
use crate::validation::{validate_category_name, validate_productivity};
use std::sync::{Arc, Mutex};
use tauri::State;

use super::CategoryResponse;

#[tauri::command]
pub fn get_categories(db: State<Arc<Mutex<Database>>>) -> Result<Vec<CategoryResponse>, String> {
    let categories = with_connection(&db, "load categories", |conn| Category::find_all(conn))?;
    Ok(categories
        .into_iter()
        .map(CategoryResponse::from)
        .collect())
}

#[tauri::command]
pub fn create_category(
    db: State<Arc<Mutex<Database>>>,
    name: String,
    productivity: i32,
) -> Result<CategoryResponse, String> {
    validate_productivity(productivity)?;
    let name = validate_category_name(&name)?;

    let category = with_connection_mapped(
        &db,
        "create category",
        |conn| Category::create(conn, name, productivity),
        |err| {
            if err.contains("UNIQUE constraint") {
                "A category with this name already exists".to_string()
            } else {
                "Failed to create category".to_string()
            }
        },
    )?;

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

    with_connection_mapped(
        &db,
        "update category",
        |conn| Category::update(conn, id, name, productivity),
        |err| {
            if err.contains("UNIQUE constraint") {
                "A category with this name already exists".to_string()
            } else {
                "Failed to update category".to_string()
            }
        },
    )
}

#[tauri::command]
pub fn delete_category(db: State<Arc<Mutex<Database>>>, id: i64) -> Result<bool, String> {
    with_connection_mapped(
        &db,
        "delete category",
        |conn| Category::delete(conn, id),
        |err| {
            if err.contains("FOREIGN KEY constraint") {
                "Cannot delete category: it is referenced by rules or activities".to_string()
            } else {
                "Failed to delete category".to_string()
            }
        },
    )
}
