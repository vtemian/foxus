use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::models::Activity;
use crate::platform::{NativeTracker, PlatformTracker};
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
        let config = TrackerConfig {
            poll_interval_secs: self.config.poll_interval_secs,
            idle_threshold_secs: self.config.idle_threshold_secs,
        };
        let platform = NativeTracker::new();

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                let idle_secs = platform.get_idle_time_secs();

                if idle_secs < config.idle_threshold_secs {
                    if let Some(window) = platform.get_active_window() {
                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;

                        let category_id = {
                            let cat = categorizer.lock().unwrap();
                            cat.categorize_app(&window.app_name, Some(&window.window_title))
                        };

                        let mut activity = Activity::new(
                            timestamp,
                            config.poll_interval_secs as i32,
                            "app",
                            Some(&window.app_name),
                            Some(&window.window_title),
                        );
                        activity.category_id = Some(category_id);

                        if let Ok(db) = db.lock() {
                            let _ = activity.save(db.connection());
                        }
                    }
                }

                thread::sleep(Duration::from_secs(config.poll_interval_secs));
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
    use tempfile::tempdir;

    fn setup() -> (Arc<Mutex<Database>>, Arc<Mutex<Categorizer>>) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();

        let categorizer = Categorizer::new(db.connection()).unwrap();

        (Arc::new(Mutex::new(db)), Arc::new(Mutex::new(categorizer)))
    }

    #[test]
    fn test_tracker_starts_and_stops() {
        let (db, categorizer) = setup();
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
}
