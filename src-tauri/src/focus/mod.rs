use crate::db::Database;
use crate::models::FocusSession;
use log::warn;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct FocusState {
    pub active: bool,
    pub budget_remaining: i32,
    pub blocked_domains: Vec<String>,
}

pub struct FocusManager {
    db: Arc<Mutex<Database>>,
}

impl FocusManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
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

        Ok(FocusState {
            active: session.is_some(),
            budget_remaining: session.map(|s| s.budget_remaining()).unwrap_or(0),
            blocked_domains,
        })
    }

    pub fn use_distraction_time(&self, secs: i32) -> rusqlite::Result<Option<i32>> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::models::{Category, Rule};
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
            Rule::create(conn, "reddit.com", "domain", entertainment.id, 10).unwrap();
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
}
