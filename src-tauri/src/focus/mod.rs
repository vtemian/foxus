use crate::db::Database;
use crate::models::{FocusSchedule, FocusSession};
use log::{info, warn};
use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Minimum interval between use_distraction_time calls (rate limiting).
const DISTRACTION_TIME_RATE_LIMIT: Duration = Duration::from_secs(25);

#[derive(Debug, Clone)]
pub struct FocusState {
    pub active: bool,
    pub budget_remaining: i32,
    pub blocked_domains: Vec<String>,
    pub session_duration_secs: Option<i64>,
}

pub struct FocusManager {
    db: Arc<Mutex<Database>>,
    /// Timestamp of last use_distraction_time call for rate limiting.
    last_distraction_request: Mutex<Option<Instant>>,
}

impl FocusManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            last_distraction_request: Mutex::new(None),
        }
    }

    fn lock_db(&self) -> MutexGuard<'_, Database> {
        match self.db.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("FocusManager: database mutex was poisoned, recovering");
                poisoned.into_inner()
            }
        }
    }

    pub fn start_session(&self, distraction_budget_secs: i32) -> rusqlite::Result<FocusSession> {
        let db = self.lock_db();
        let conn = db.connection();

        // End any existing active session
        if let Some(mut existing) = FocusSession::find_active(conn)? {
            existing.end(conn)?;
        }

        let mut session = FocusSession::new(distraction_budget_secs, false);
        session.save(conn)?;

        Ok(session)
    }

    pub fn end_session(&self) -> rusqlite::Result<Option<FocusSession>> {
        let db = self.lock_db();
        let conn = db.connection();

        if let Some(mut session) = FocusSession::find_active(conn)? {
            session.end(conn)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub fn get_state(&self) -> rusqlite::Result<FocusState> {
        let db = self.lock_db();
        let conn = db.connection();

        let session = FocusSession::find_active(conn)?;
        let blocked_domains = self.get_blocked_domains(conn)?;

        let (active, budget_remaining, session_duration_secs) = match session {
            Some(s) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("System clock is before Unix epoch")
                    .as_secs() as i64;
                let duration = (now - s.started_at).max(0);
                (true, s.budget_remaining(), Some(duration))
            }
            None => (false, 0, None),
        };

        Ok(FocusState {
            active,
            budget_remaining,
            blocked_domains,
            session_duration_secs,
        })
    }

    /// Use distraction time from the current focus session's budget.
    ///
    /// Rate limited to prevent rapid calls from bypassing budget enforcement.
    /// Returns None if no active session or if rate limited.
    pub fn use_distraction_time(&self, secs: i32) -> rusqlite::Result<Option<i32>> {
        // Rate limiting: Check if enough time has passed since last request
        {
            let mut last_request = self.last_distraction_request.lock().unwrap_or_else(|p| p.into_inner());
            let now = Instant::now();

            if let Some(last) = *last_request {
                if now.duration_since(last) < DISTRACTION_TIME_RATE_LIMIT {
                    // Rate limited - return current budget without deducting
                    let db = self.lock_db();
                    let conn = db.connection();
                    if let Some(session) = FocusSession::find_active(conn)? {
                        return Ok(Some(session.budget_remaining()));
                    }
                    return Ok(None);
                }
            }

            *last_request = Some(now);
        }

        let db = self.lock_db();
        let conn = db.connection();

        if let Some(mut session) = FocusSession::find_active(conn)? {
            session.add_distraction_time(conn, secs)?;
            Ok(Some(session.budget_remaining()))
        } else {
            Ok(None)
        }
    }

    pub fn is_domain_blocked(&self, domain: &str) -> rusqlite::Result<bool> {
        let state = self.get_state()?;

        if !state.active {
            return Ok(false);
        }

        Ok(state.blocked_domains.iter().any(|d| {
            domain.ends_with(d) || domain == d.trim_start_matches("*.")
        }))
    }

    /// Reset rate limiting state. Used in tests to allow rapid calls.
    #[cfg(test)]
    pub fn reset_rate_limit(&self) {
        let mut last_request = self.last_distraction_request.lock().unwrap_or_else(|p| p.into_inner());
        *last_request = None;
    }

    fn get_blocked_domains(&self, conn: &Connection) -> rusqlite::Result<Vec<String>> {
        // Get domains from rules that map to distracting categories
        let mut stmt = conn.prepare(
            "SELECT r.pattern FROM rules r
             JOIN categories c ON r.category_id = c.id
             WHERE r.match_type = 'domain' AND c.productivity < 0"
        )?;

        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    /// Check schedules and auto-start/stop sessions as needed.
    ///
    /// Call this periodically (e.g., every minute) to enforce focus schedules.
    /// - If a schedule is active and no focus session exists, starts a scheduled session
    /// - If a scheduled session is active but no schedule matches, ends the session
    /// - Manual (non-scheduled) sessions are not affected by schedule checks
    pub fn check_schedules(&self) -> rusqlite::Result<()> {
        let db = self.lock_db();
        let conn = db.connection();

        let (day, time) = get_current_day_and_time();
        let active_schedule = self.find_active_schedule(conn, day, &time)?;
        let active_session = FocusSession::find_active(conn)?;

        match (active_schedule, active_session) {
            // Schedule active, no session -> start scheduled session
            (Some(schedule), None) => {
                info!(
                    "Starting scheduled focus session (schedule {}): {} budget secs",
                    schedule.id.unwrap_or(0),
                    schedule.distraction_budget
                );
                let mut session = FocusSession::new(schedule.distraction_budget, true);
                session.save(conn)?;
            }
            // No schedule active, scheduled session exists -> end it
            (None, Some(mut session)) if session.scheduled => {
                info!(
                    "Ending scheduled focus session {} - schedule no longer active",
                    session.id.unwrap_or(0)
                );
                session.end(conn)?;
            }
            // Schedule active with different budget, scheduled session exists -> update session
            (Some(schedule), Some(session)) if session.scheduled => {
                // Only end and restart if budget changed significantly (prevents churn)
                if (schedule.distraction_budget - session.distraction_budget).abs() > 60 {
                    info!(
                        "Schedule budget changed significantly, restarting session"
                    );
                    let mut session = session;
                    session.end(conn)?;
                    let mut new_session = FocusSession::new(schedule.distraction_budget, true);
                    new_session.save(conn)?;
                }
            }
            // Manual session active - don't interfere
            (_, Some(session)) if !session.scheduled => {
                // Leave manual sessions alone
            }
            // No schedule, no session - nothing to do
            _ => {}
        }

        Ok(())
    }

    /// Find the currently active schedule, if any.
    pub fn get_active_schedule(&self) -> rusqlite::Result<Option<FocusSchedule>> {
        let db = self.lock_db();
        let conn = db.connection();
        let (day, time) = get_current_day_and_time();
        self.find_active_schedule(conn, day, &time)
    }

    fn find_active_schedule(
        &self,
        conn: &Connection,
        day: u32,
        time: &str,
    ) -> rusqlite::Result<Option<FocusSchedule>> {
        let schedules = FocusSchedule::find_enabled(conn)?;

        Ok(schedules
            .into_iter()
            .find(|s| s.is_active_at(day, time)))
    }

    /// Start a scheduled focus session with the given budget.
    /// Unlike start_session, this marks the session as scheduled.
    pub fn start_scheduled_session(&self, distraction_budget_secs: i32) -> rusqlite::Result<FocusSession> {
        let db = self.lock_db();
        let conn = db.connection();

        // End any existing active session
        if let Some(mut existing) = FocusSession::find_active(conn)? {
            existing.end(conn)?;
        }

        let mut session = FocusSession::new(distraction_budget_secs, true);
        session.save(conn)?;

        Ok(session)
    }
}

/// Get the current day of week (1=Monday, 7=Sunday) and time (HH:MM format).
fn get_current_day_and_time() -> (u32, String) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();

    // Convert Unix timestamp to day of week and time
    // Unix epoch (Jan 1, 1970) was a Thursday (day 4 in ISO weekday)
    let days_since_epoch = now / 86400;
    let day_of_week = ((days_since_epoch + 3) % 7) + 1; // 1=Monday, 7=Sunday

    let seconds_today = now % 86400;
    let hours = (seconds_today / 3600) as u32;
    let minutes = ((seconds_today % 3600) / 60) as u32;

    (day_of_week as u32, format!("{:02}:{:02}", hours, minutes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::models::{Category, FocusSchedule, Rule, MatchType};
    use tempfile::{tempdir, TempDir};

    fn setup() -> (Arc<Mutex<Database>>, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        (Arc::new(Mutex::new(db)), dir)
    }

    #[test]
    fn test_start_and_end_session() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        let state = manager.get_state().unwrap();
        assert!(!state.active);

        let session = manager.start_session(600).unwrap();
        assert_eq!(session.distraction_budget, 600);

        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert_eq!(state.budget_remaining, 600);

        manager.end_session().unwrap();

        let state = manager.get_state().unwrap();
        assert!(!state.active);
    }

    #[test]
    fn test_use_distraction_time() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        manager.start_session(300).unwrap();

        let remaining = manager.use_distraction_time(100).unwrap().unwrap();
        assert_eq!(remaining, 200);

        // Reset rate limit to allow immediate second call in test
        manager.reset_rate_limit();

        let remaining = manager.use_distraction_time(200).unwrap().unwrap();
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_blocked_domains() {
        let (db, _dir) = setup();

        // Add a rule for reddit as distracting
        {
            let db_lock = db.lock().unwrap();
            let conn = db_lock.connection();
            let entertainment = Category::find_all(conn).unwrap()
                .into_iter()
                .find(|c| c.name == "Entertainment")
                .unwrap();
            Rule::create(conn, "reddit.com", MatchType::Domain, entertainment.id, 10).unwrap();
        }

        let manager = FocusManager::new(Arc::clone(&db));

        // Not blocked when focus mode is off
        assert!(!manager.is_domain_blocked("reddit.com").unwrap());

        manager.start_session(600).unwrap();

        // Blocked when focus mode is on
        assert!(manager.is_domain_blocked("reddit.com").unwrap());
        assert!(manager.is_domain_blocked("www.reddit.com").unwrap());

        // Other domains not blocked
        assert!(!manager.is_domain_blocked("github.com").unwrap());
    }

    #[test]
    fn test_session_duration_none_when_inactive() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        let state = manager.get_state().unwrap();
        assert!(!state.active);
        assert!(state.session_duration_secs.is_none());
    }

    #[test]
    fn test_session_duration_calculated_when_active() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        manager.start_session(600).unwrap();

        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert!(state.session_duration_secs.is_some());
        // Duration should be very small (close to 0) since we just started
        let duration = state.session_duration_secs.unwrap();
        assert!(duration >= 0, "Duration should be non-negative");
        assert!(duration < 5, "Duration should be small for a just-started session");
    }

    #[test]
    fn test_session_duration_none_after_end() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        manager.start_session(600).unwrap();
        let state = manager.get_state().unwrap();
        assert!(state.session_duration_secs.is_some());

        manager.end_session().unwrap();

        let state = manager.get_state().unwrap();
        assert!(!state.active);
        assert!(state.session_duration_secs.is_none());
    }

    #[test]
    fn test_start_scheduled_session() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        let session = manager.start_scheduled_session(600).unwrap();
        assert!(session.scheduled);
        assert_eq!(session.distraction_budget, 600);

        let state = manager.get_state().unwrap();
        assert!(state.active);
    }

    #[test]
    fn test_get_current_day_and_time_format() {
        let (day, time) = get_current_day_and_time();

        // Day should be 1-7
        assert!(day >= 1 && day <= 7, "Day should be between 1 and 7, got {}", day);

        // Time should be HH:MM format
        assert_eq!(time.len(), 5, "Time should be 5 characters (HH:MM), got {}", time);
        assert_eq!(&time[2..3], ":", "Time should have colon at position 2");

        let hours: u32 = time[0..2].parse().unwrap();
        let minutes: u32 = time[3..5].parse().unwrap();
        assert!(hours < 24, "Hours should be < 24, got {}", hours);
        assert!(minutes < 60, "Minutes should be < 60, got {}", minutes);
    }

    #[test]
    fn test_find_active_schedule_none_when_empty() {
        let (db, _dir) = setup();
        let manager = FocusManager::new(Arc::clone(&db));

        let schedule = manager.get_active_schedule().unwrap();
        assert!(schedule.is_none());
    }

    #[test]
    fn test_find_active_schedule_with_matching_schedule() {
        let (db, _dir) = setup();

        // Create a schedule that covers all days and times
        {
            let db_lock = db.lock().unwrap();
            let conn = db_lock.connection();
            let mut schedule = FocusSchedule::new("1,2,3,4,5,6,7", "00:00", "23:59", 600);
            schedule.save(conn).unwrap();
        }

        let manager = FocusManager::new(Arc::clone(&db));

        let schedule = manager.get_active_schedule().unwrap();
        assert!(schedule.is_some());
        assert_eq!(schedule.unwrap().distraction_budget, 600);
    }

    #[test]
    fn test_find_active_schedule_disabled_not_returned() {
        let (db, _dir) = setup();

        // Create a disabled schedule that covers all days and times
        {
            let db_lock = db.lock().unwrap();
            let conn = db_lock.connection();
            let mut schedule = FocusSchedule::new("1,2,3,4,5,6,7", "00:00", "23:59", 600);
            schedule.enabled = false;
            schedule.save(conn).unwrap();
        }

        let manager = FocusManager::new(Arc::clone(&db));

        let schedule = manager.get_active_schedule().unwrap();
        assert!(schedule.is_none());
    }

    #[test]
    fn test_check_schedules_does_not_affect_manual_session() {
        let (db, _dir) = setup();

        // Create a schedule that covers all days and times
        {
            let db_lock = db.lock().unwrap();
            let conn = db_lock.connection();
            let mut schedule = FocusSchedule::new("1,2,3,4,5,6,7", "00:00", "23:59", 300);
            schedule.save(conn).unwrap();
        }

        let manager = FocusManager::new(Arc::clone(&db));

        // Start a manual session with different budget
        manager.start_session(600).unwrap();

        // Check schedules should not affect manual session
        manager.check_schedules().unwrap();

        let state = manager.get_state().unwrap();
        assert!(state.active);
        assert_eq!(state.budget_remaining, 600); // Should still be 600, not 300
    }
}
