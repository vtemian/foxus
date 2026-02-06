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

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs =
        ProjectDirs::from("com", "foxus", "Foxus").expect("Could not determine project directories");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Could not create data directory");
    data_dir.join("foxus.db")
}

fn main() {
    let db_path = get_db_path();
    let db = Database::open(&db_path).expect("Failed to open database");
    migrations::run(db.connection()).expect("Failed to run migrations");

    let db = Arc::new(Mutex::new(db));
    let categorizer = Arc::new(Mutex::new(
        Categorizer::new(db.lock().unwrap().connection()).unwrap(),
    ));
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
