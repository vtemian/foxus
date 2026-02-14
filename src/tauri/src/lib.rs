pub mod categorizer;
mod commands;
pub mod constants;
pub mod db;
pub mod error;
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
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    webview::WebviewWindowBuilder,
    AppHandle, Manager, RunEvent, Wry,
};

/// Holds the tracker thread handle for graceful shutdown
pub struct TrackerHandle(Mutex<Option<JoinHandle<()>>>);

/// Holds the tray icon for dynamic menu updates
pub struct TrayHandle(Mutex<Option<TrayIcon<Wry>>>);

/// Build the tray menu based on current focus state
fn build_tray_menu(app: &AppHandle) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let focus_active = if let Some(focus_manager) = app.try_state::<Arc<FocusManager>>() {
        focus_manager.get_state().map(|s| s.active).unwrap_or(false)
    } else {
        false
    };

    let open = MenuItem::with_id(app, "open", "Open Foxus", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    if focus_active {
        let end_focus = MenuItem::with_id(app, "end_focus", "End Focus Session", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
        Ok(Menu::with_items(app, &[&open, &separator, &end_focus, &separator, &quit])?)
    } else {
        let focus_10 = MenuItem::with_id(app, "focus_10", "Start Focus (10 min)", true, None::<&str>)?;
        let focus_25 = MenuItem::with_id(app, "focus_25", "Start Focus (25 min)", true, None::<&str>)?;
        let focus_60 = MenuItem::with_id(app, "focus_60", "Start Focus (1 hour)", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
        Ok(Menu::with_items(app, &[&open, &separator, &focus_10, &focus_25, &focus_60, &separator, &quit])?)
    }
}

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

            // Create main window at startup (hidden)
            let _main_window = WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
                .title("Foxus")
                .inner_size(420.0, 600.0)
                .resizable(true)
                .visible(false)
                .center()
                .build()?;

            // Setup tray with initial menu
            let open = MenuItem::with_id(app, "open", "Open Foxus", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let focus_10 = MenuItem::with_id(app, "focus_10", "Start Focus (10 min)", true, None::<&str>)?;
            let focus_25 = MenuItem::with_id(app, "focus_25", "Start Focus (25 min)", true, None::<&str>)?;
            let focus_60 = MenuItem::with_id(app, "focus_60", "Start Focus (1 hour)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Foxus", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &separator, &focus_10, &focus_25, &focus_60, &separator, &quit])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("Foxus")
                .on_menu_event(|app, event| {
                    let event_id = event.id.0.as_str();

                    // Handle focus session actions
                    if event_id.starts_with("focus_") || event_id == "end_focus" {
                        if let Some(focus_manager) = app.try_state::<Arc<FocusManager>>() {
                            let result = match event_id {
                                "focus_10" => focus_manager.start_session(10 * 60).map(|_| ()),
                                "focus_25" => focus_manager.start_session(25 * 60).map(|_| ()),
                                "focus_60" => focus_manager.start_session(60 * 60).map(|_| ()),
                                "end_focus" => focus_manager.end_session().map(|_| ()),
                                _ => Ok(()),
                            };

                            if let Err(e) = result {
                                error!("Failed to handle focus action: {}", e);
                            }

                            // Update tray menu to reflect new focus state
                            if let Some(tray_handle) = app.try_state::<TrayHandle>() {
                                if let Ok(guard) = tray_handle.0.lock() {
                                    if let Some(tray) = guard.as_ref() {
                                        match build_tray_menu(app) {
                                            Ok(new_menu) => {
                                                if let Err(e) = tray.set_menu(Some(new_menu)) {
                                                    error!("Failed to update tray menu: {}", e);
                                                }
                                            }
                                            Err(e) => error!("Failed to build tray menu: {}", e),
                                        }
                                    }
                                }
                            }
                        }
                    } else if event_id == "open" {
                        // Show the main window
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    } else if event_id == "quit" {
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

            // Store tray handle for dynamic menu updates
            app.manage(TrayHandle(Mutex::new(Some(tray))));

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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Prevent the app from exiting when all windows are closed
            // This is essential for tray-only apps
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
