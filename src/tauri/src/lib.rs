pub mod categorizer;
mod commands;
pub mod constants;
pub mod db;
mod new_commands;
pub mod validation;
pub mod focus;
mod models;
pub mod native_host;
mod platform;
#[cfg(test)]
mod test_utils;
mod tracker;

use crate::categorizer::Categorizer;
use crate::db::{migrations, Database};
use crate::focus::FocusManager;
use crate::tracker::{TrackerConfig, TrackerService};
use directories::ProjectDirs;
use log::{error, warn};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

/// Holds the tracker thread handle for graceful shutdown
pub struct TrackerHandle(Mutex<Option<JoinHandle<()>>>);

/// Error type for Foxus initialization failures
#[derive(Debug)]
pub enum InitError {
    NoProjectDirs,
    DataDirCreation(std::io::Error),
    DatabaseOpen(rusqlite::Error),
    Migration(rusqlite::Error),
    Categorizer(rusqlite::Error),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::NoProjectDirs => write!(f, "Could not determine project directories"),
            InitError::DataDirCreation(e) => write!(f, "Could not create data directory: {}", e),
            InitError::DatabaseOpen(e) => write!(f, "Failed to open database: {}", e),
            InitError::Migration(e) => write!(f, "Failed to run database migrations: {}", e),
            InitError::Categorizer(e) => write!(f, "Failed to initialize categorizer: {}", e),
        }
    }
}

impl std::error::Error for InitError {}

fn get_db_path() -> Result<std::path::PathBuf, InitError> {
    let proj_dirs = ProjectDirs::from("com", "foxus", "Foxus")
        .ok_or(InitError::NoProjectDirs)?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir).map_err(InitError::DataDirCreation)?;
    Ok(data_dir.join("foxus.db"))
}

/// Lock a mutex, recovering from poisoning if necessary
fn safe_lock<'a, T>(mutex: &'a Mutex<T>, context: &str) -> std::sync::MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("{} mutex was poisoned, recovering", context);
            poisoned.into_inner()
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database with proper error handling
            let db_path = match get_db_path() {
                Ok(path) => path,
                Err(e) => {
                    error!("Foxus initialization failed: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };

            let db = match Database::open(&db_path) {
                Ok(db) => db,
                Err(e) => {
                    error!("Failed to open database: {}", e);
                    return Err(Box::new(InitError::DatabaseOpen(e)) as Box<dyn std::error::Error>);
                }
            };

            if let Err(e) = migrations::run(db.connection()) {
                error!("Failed to run migrations: {}", e);
                return Err(Box::new(InitError::Migration(e)) as Box<dyn std::error::Error>);
            }

            let db = Arc::new(Mutex::new(db));

            let categorizer = {
                let db_guard = safe_lock(&db, "Database");
                match Categorizer::new(db_guard.connection()) {
                    Ok(cat) => Arc::new(Mutex::new(cat)),
                    Err(e) => {
                        error!("Failed to initialize categorizer: {}", e);
                        return Err(Box::new(InitError::Categorizer(e)) as Box<dyn std::error::Error>);
                    }
                }
            };

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
            commands::get_weekly_stats,
            commands::get_focus_state,
            commands::start_focus_session,
            commands::end_focus_session,
            commands::get_focus_schedules,
            commands::create_focus_schedule,
            commands::update_focus_schedule,
            commands::delete_focus_schedule,
            commands::get_active_schedule,
            commands::check_focus_schedules,
            commands::get_categories,
            commands::create_category,
            commands::update_category,
            commands::delete_category,
            commands::get_rules,
            commands::create_rule,
            commands::update_rule,
            commands::delete_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
