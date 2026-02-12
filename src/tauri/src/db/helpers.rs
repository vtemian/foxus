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
