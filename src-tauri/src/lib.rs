mod categorizer;
mod db;
mod focus;
mod models;
mod native_host;
mod platform;
mod tracker;

use crate::categorizer::Categorizer;
use crate::db::{migrations, Database};
use crate::focus::FocusManager;
use crate::tracker::{TrackerConfig, TrackerService};
use directories::ProjectDirs;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn get_db_path() -> std::path::PathBuf {
    let proj_dirs = ProjectDirs::from("com", "foxus", "Foxus")
        .expect("Could not determine project directories");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).expect("Could not create data directory");
    data_dir.join("foxus.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db_path = get_db_path();
            let db = Database::open(&db_path).expect("Failed to open database");
            migrations::run(db.connection()).expect("Failed to run migrations");

            let db = Arc::new(Mutex::new(db));
            let categorizer = Arc::new(Mutex::new(
                Categorizer::new(db.lock().unwrap().connection()).unwrap(),
            ));
            let focus_manager = Arc::new(FocusManager::new(Arc::clone(&db)));

            // Start tracker service
            let tracker = TrackerService::new(
                Arc::clone(&db),
                Arc::clone(&categorizer),
                TrackerConfig::default(),
            );
            tracker.start();
            let tracker = Arc::new(tracker);

            // Store in app state
            app.manage(db);
            app.manage(categorizer);
            app.manage(focus_manager);
            app.manage(tracker);

            // Setup tray
            let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Foxus")
                .on_menu_event(|app, event| {
                    if event.id == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
