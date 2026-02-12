use crate::db::Database;
use crate::error::AppError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Execute a database operation with proper lock handling.
///
/// # Example
/// ```ignore
/// with_connection(&db, |conn| Category::find_all(conn))?
/// ```
pub fn with_connection<F, T>(db: &Arc<Mutex<Database>>, f: F) -> Result<T, AppError>
where
    F: FnOnce(&Connection) -> rusqlite::Result<T>,
{
    let db = db.lock().map_err(|_| AppError::LockPoisoned)?;
    Ok(f(db.connection())?)
}
