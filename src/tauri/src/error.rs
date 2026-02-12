use thiserror::Error;

/// Application error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("'{name}' already exists")]
    AlreadyExists { name: String },

    #[error("{entity} not found")]
    NotFound { entity: &'static str },

    #[error("Invalid {field}: {reason}")]
    InvalidInput { field: &'static str, reason: String },

    #[error("Cannot delete: {reason}")]
    DeleteFailed { reason: String },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Lock poisoned")]
    LockPoisoned,

    #[error("{0}")]
    Internal(String),
}

// For Tauri command returns - converts AppError to String
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

/// Check if a rusqlite error is a UNIQUE constraint violation
pub fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(err, _)
        if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation)
}

/// Check if a rusqlite error is a FOREIGN KEY constraint violation
pub fn is_fk_violation(e: &rusqlite::Error) -> bool {
    e.to_string().contains("FOREIGN KEY constraint failed")
}
