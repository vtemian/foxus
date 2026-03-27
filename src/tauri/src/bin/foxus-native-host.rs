//! Chrome Native Messaging Host for Foxus
//!
//! This binary runs as a standalone native messaging host for the Foxus Chrome extension.
//! It communicates via stdin/stdout using Chrome's native messaging protocol.

// Native messaging host uses stderr for logging because stdout is reserved
// for the Chrome Native Messaging protocol (length-prefixed JSON).
#![expect(
    clippy::print_stderr,
    reason = "Native messaging host uses stderr for logging because stdout is reserved for the Chrome protocol"
)]

use foxus_lib::{
    categorizer::Categorizer,
    db::{migrations, Database},
    focus::FocusManager,
    get_db_path,
    native_host::NativeHost,
    safe_lock,
};
use std::sync::{Arc, Mutex};

fn main() {
    // Initialize with proper error handling
    let db_path = match get_db_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Initialization error: {e}");
            std::process::exit(1);
        }
    };

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = migrations::run(db.connection()) {
        eprintln!("Failed to run migrations: {e}");
        std::process::exit(1);
    }

    let db = Arc::new(Mutex::new(db));

    let categorizer = {
        let db_guard = safe_lock(&db, "Database");
        match Categorizer::new(db_guard.connection()) {
            Ok(cat) => Arc::new(Mutex::new(cat)),
            Err(e) => {
                eprintln!("Failed to initialize categorizer: {e}");
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
            eprintln!("Native host error: {e}");
            std::process::exit(1);
        }
    }
}
