//! Chrome Native Messaging Host for Foxus
//!
//! This binary runs as a standalone native messaging host for the Foxus Chrome extension.
//! It communicates via stdin/stdout using Chrome's native messaging protocol.

use directories::ProjectDirs;
use foxus_lib::{
    categorizer::Categorizer,
    db::{migrations, Database},
    focus::FocusManager,
    native_host::NativeHost,
};
use std::sync::{Arc, Mutex};

/// Get the database path, creating the data directory if needed.
fn get_db_path() -> Result<std::path::PathBuf, String> {
    let proj_dirs = ProjectDirs::from("com", "foxus", "Foxus")
        .ok_or_else(|| "Could not determine project directories".to_string())?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .map_err(|e| format!("Could not create data directory: {}", e))?;
    Ok(data_dir.join("foxus.db"))
}

/// Lock a mutex, recovering from poisoning if necessary.
fn safe_lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Warning: mutex was poisoned, recovering");
            poisoned.into_inner()
        }
    }
}

fn main() {
    // Initialize with proper error handling
    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Initialization error: {}", e);
            std::process::exit(1);
        }
    };

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = migrations::run(db.connection()) {
        eprintln!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }

    let db = Arc::new(Mutex::new(db));

    let categorizer = {
        let db_guard = safe_lock(&db);
        match Categorizer::new(db_guard.connection()) {
            Ok(cat) => Arc::new(Mutex::new(cat)),
            Err(e) => {
                eprintln!("Failed to initialize categorizer: {}", e);
                std::process::exit(1);
            }
        }
    };

    let focus_manager = Arc::new(FocusManager::new(Arc::clone(&db)));

    let host = NativeHost::new(db, focus_manager, categorizer);

    // Run the native host event loop
    // This will read from stdin and write to stdout until the connection is closed
    if let Err(e) = host.run() {
        // Only report unexpected errors; EOF is expected when Chrome closes the connection
        if e.kind() != std::io::ErrorKind::UnexpectedEof {
            eprintln!("Native host error: {}", e);
            std::process::exit(1);
        }
    }
}
