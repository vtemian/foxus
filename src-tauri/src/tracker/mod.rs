use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::models::Activity;
use crate::platform::{NativeTracker, PlatformTracker};
use log::{error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct TrackerConfig {
    pub poll_interval_secs: u64,
    pub idle_threshold_secs: u64,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 5,
            idle_threshold_secs: 120,
        }
    }
}

pub struct TrackerService {
    config: TrackerConfig,
    running: Arc<AtomicBool>,
    db: Arc<Mutex<Database>>,
    categorizer: Arc<Mutex<Categorizer>>,
    platform: NativeTracker,
}

impl TrackerService {
    pub fn new(db: Arc<Mutex<Database>>, categorizer: Arc<Mutex<Categorizer>>, config: TrackerConfig) -> Self {
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            db,
            categorizer,
            platform: NativeTracker::new(),
        }
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let db = Arc::clone(&self.db);
        let categorizer = Arc::clone(&self.categorizer);
        let poll_interval_secs = self.config.poll_interval_secs;
        let idle_threshold_secs = self.config.idle_threshold_secs;
        let platform = NativeTracker::new();

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let idle_secs = platform.get_idle_time_secs();

                if idle_secs < idle_threshold_secs {
                    if let Some(window) = platform.get_active_window() {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_else(|_| Duration::from_secs(0))
                            .as_secs() as i64;

                        let category_id = match categorizer.lock() {
                            Ok(cat) => cat.categorize_app(&window.app_name, Some(&window.window_title)),
                            Err(poisoned) => {
                                warn!("Categorizer mutex was poisoned, recovering");
                                poisoned.into_inner().categorize_app(&window.app_name, Some(&window.window_title))
                            }
                        };

                        let mut activity = Activity::new(
                            timestamp,
                            poll_interval_secs as i32,
                            "app",
                            Some(&window.app_name),
                            Some(&window.window_title),
                        );
                        activity.category_id = Some(category_id);

                        match db.lock() {
                            Ok(db_guard) => {
                                if let Err(e) = activity.save(db_guard.connection()) {
                                    error!("Failed to save activity: {}", e);
                                }
                            }
                            Err(poisoned) => {
                                warn!("Database mutex was poisoned, recovering");
                                if let Err(e) = activity.save(poisoned.into_inner().connection()) {
                                    error!("Failed to save activity after recovery: {}", e);
                                }
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_secs(poll_interval_secs));
            }
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use tempfile::{tempdir, TempDir};

    fn setup() -> (Arc<Mutex<Database>>, Arc<Mutex<Categorizer>>, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();

        let categorizer = Categorizer::new(db.connection()).unwrap();

        (Arc::new(Mutex::new(db)), Arc::new(Mutex::new(categorizer)), dir)
    }

    #[test]
    fn test_tracker_starts_and_stops() {
        let (db, categorizer, _dir) = setup();
        let config = TrackerConfig {
            poll_interval_secs: 1,
            idle_threshold_secs: 120,
        };

        let tracker = TrackerService::new(db, categorizer, config);

        assert!(!tracker.is_running());

        let handle = tracker.start();
        assert!(tracker.is_running());

        thread::sleep(Duration::from_millis(100));

        tracker.stop();
        handle.join().unwrap();

        assert!(!tracker.is_running());
    }

    /// Tests that the activity tracking and saving logic works correctly.
    /// This test directly exercises the save logic rather than relying on the
    /// threaded start() method, which depends on platform-specific window detection.
    #[test]
    fn test_tracker_saves_activities_to_db() {
        use crate::models::Activity;

        let (db, categorizer, _dir) = setup();

        // Simulate what the tracker does when it detects activity:
        // 1. Get timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;

        // 2. Categorize the app
        let category_id = {
            let cat = categorizer.lock().unwrap();
            cat.categorize_app("TestApp", Some("Test Window"))
        };

        // 3. Create and save activity
        let mut activity = Activity::new(
            timestamp,
            5, // poll_interval_secs
            "app",
            Some("TestApp"),
            Some("Test Window"),
        );
        activity.category_id = Some(category_id);

        {
            let db_guard = db.lock().unwrap();
            activity.save(db_guard.connection()).unwrap();
        }

        // 4. Verify activity was saved to the database
        let db_guard = db.lock().unwrap();
        let activities = Activity::find_in_range(
            db_guard.connection(),
            0,
            i64::MAX,
        )
        .unwrap();

        assert_eq!(activities.len(), 1, "Expected exactly one activity to be saved");

        let saved = &activities[0];
        assert_eq!(saved.source, "app");
        assert_eq!(saved.app_name, Some("TestApp".to_string()));
        assert_eq!(saved.window_title, Some("Test Window".to_string()));
        assert_eq!(saved.category_id, Some(category_id));
        assert_eq!(saved.duration_secs, 5);
    }
}
