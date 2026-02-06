pub mod categorizer;
mod commands;
pub mod db;
pub mod focus;
mod models;
pub mod native_host;
mod platform;
mod tracker;

use crate::categorizer::Categorizer;
use crate::db::{migrations, Database};
use crate::focus::FocusManager;
use crate::tracker::{TrackerConfig, TrackerService};
use directories::ProjectDirs;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

/// Holds the tracker thread handle for graceful shutdown
pub struct TrackerHandle(Mutex<Option<JoinHandle<()>>>);

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
            let handle = tracker.start();
            let tracker = Arc::new(tracker);
            let tracker_handle = TrackerHandle(Mutex::new(Some(handle)));

            // Store in app state
            app.manage(db);
            app.manage(categorizer);
            app.manage(focus_manager);
            app.manage(tracker);
            app.manage(tracker_handle);

            // Setup tray
            let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Foxus")
                .on_menu_event(|app, event| {
                    if event.id == "quit" {
                        // Gracefully stop the tracker before exiting
                        if let Some(tracker) = app.try_state::<Arc<TrackerService>>() {
                            tracker.stop();
                        }
                        if let Some(handle_state) = app.try_state::<TrackerHandle>() {
                            if let Ok(mut guard) = handle_state.0.lock() {
                                if let Some(handle) = guard.take() {
                                    // Wait for tracker thread to finish (with timeout behavior from thread)
                                    let _ = handle.join();
                                }
                            }
                        }
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today_stats,
            commands::get_focus_state,
            commands::start_focus_session,
            commands::end_focus_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
