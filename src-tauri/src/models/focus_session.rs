use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct FocusSession {
    pub id: Option<i64>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub scheduled: bool,
    pub distraction_budget: i32,
    pub distraction_used: i32,
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before Unix epoch - this should never happen on properly configured systems")
        .as_secs() as i64
}

impl FocusSession {
    pub fn new(distraction_budget_secs: i32, scheduled: bool) -> Self {
        Self {
            id: None,
            started_at: current_timestamp(),
            ended_at: None,
            scheduled,
            distraction_budget: distraction_budget_secs,
            distraction_used: 0,
        }
    }

    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO focus_sessions (started_at, ended_at, scheduled, distraction_budget, distraction_used)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.started_at,
                self.ended_at,
                self.scheduled as i32,
                self.distraction_budget,
                self.distraction_used,
            ],
        )?;
        self.id = Some(conn.last_insert_rowid());
        Ok(())
    }

    pub fn find_active(conn: &Connection) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, started_at, ended_at, scheduled, distraction_budget, distraction_used
             FROM focus_sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: Some(row.get(0)?),
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                scheduled: row.get::<_, i32>(3)? != 0,
                distraction_budget: row.get(4)?,
                distraction_used: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Ends the focus session by setting the ended_at timestamp.
    /// Returns an error if the session has not been saved yet (id is None).
    pub fn end(&mut self, conn: &Connection) -> Result<()> {
        let id = self.id.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Cannot end unsaved session - call save() first".to_string())
        })?;

        let now = current_timestamp();
        self.ended_at = Some(now);

        conn.execute(
            "UPDATE focus_sessions SET ended_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;

        Ok(())
    }

    /// Adds distraction time to the session.
    /// Returns an error if the session has not been saved yet (id is None).
    pub fn add_distraction_time(&mut self, conn: &Connection, secs: i32) -> Result<()> {
        let id = self.id.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Cannot update unsaved session - call save() first".to_string())
        })?;

        self.distraction_used += secs;

        conn.execute(
            "UPDATE focus_sessions SET distraction_used = ?1 WHERE id = ?2",
            params![self.distraction_used, id],
        )?;

        Ok(())
    }

    pub fn budget_remaining(&self) -> i32 {
        (self.distraction_budget - self.distraction_used).max(0)
    }

    pub fn is_budget_exhausted(&self) -> bool {
        self.distraction_used >= self.distraction_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use tempfile::{tempdir, TempDir};

    fn setup_db() -> (Database, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        (db, dir)
    }

    #[test]
    fn test_new_creates_session_with_current_timestamp() {
        let session = FocusSession::new(600, false);

        assert!(session.id.is_none());
        assert!(session.started_at > 0);
        assert!(session.ended_at.is_none());
        assert!(!session.scheduled);
        assert_eq!(session.distraction_budget, 600);
        assert_eq!(session.distraction_used, 0);
    }

    #[test]
    fn test_new_creates_scheduled_session() {
        let session = FocusSession::new(300, true);

        assert!(session.scheduled);
        assert_eq!(session.distraction_budget, 300);
    }

    #[test]
    fn test_find_active_returns_none_when_no_sessions() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let found = FocusSession::find_active(conn).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_save_assigns_id() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        assert!(session.id.is_none());

        session.save(conn).unwrap();
        assert!(session.id.is_some());
    }

    #[test]
    fn test_create_and_find_active_session() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        assert!(FocusSession::find_active(conn).unwrap().is_none());

        let mut session = FocusSession::new(600, false);
        session.save(conn).unwrap();

        let found = FocusSession::find_active(conn).unwrap().unwrap();
        assert_eq!(found.id, session.id);
        assert_eq!(found.distraction_budget, 600);
        assert_eq!(found.started_at, session.started_at);
    }

    #[test]
    fn test_find_active_returns_most_recent() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session1 = FocusSession::new(300, false);
        session1.save(conn).unwrap();

        // Create a second session with different budget
        let mut session2 = FocusSession::new(600, true);
        session2.started_at = session1.started_at + 100; // Make it more recent
        session2.save(conn).unwrap();

        let found = FocusSession::find_active(conn).unwrap().unwrap();
        assert_eq!(found.id, session2.id);
        assert_eq!(found.distraction_budget, 600);
        assert!(found.scheduled);
    }

    #[test]
    fn test_end_session() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        session.save(conn).unwrap();

        session.end(conn).unwrap();

        assert!(session.ended_at.is_some());
        assert!(FocusSession::find_active(conn).unwrap().is_none());
    }

    #[test]
    fn test_end_session_persists_to_db() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        session.save(conn).unwrap();
        let session_id = session.id.unwrap();

        session.end(conn).unwrap();

        // Verify the ended_at was persisted by querying directly
        let ended_at: Option<i64> = conn.query_row(
            "SELECT ended_at FROM focus_sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0)
        ).unwrap();

        assert!(ended_at.is_some());
        assert_eq!(ended_at, session.ended_at);
    }

    #[test]
    fn test_add_distraction_time() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(300, false);
        session.save(conn).unwrap();

        assert_eq!(session.distraction_used, 0);

        session.add_distraction_time(conn, 50).unwrap();
        assert_eq!(session.distraction_used, 50);

        session.add_distraction_time(conn, 100).unwrap();
        assert_eq!(session.distraction_used, 150);
    }

    #[test]
    fn test_add_distraction_time_persists_to_db() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(300, false);
        session.save(conn).unwrap();
        let session_id = session.id.unwrap();

        session.add_distraction_time(conn, 75).unwrap();

        // Verify persisted to DB
        let used: i32 = conn.query_row(
            "SELECT distraction_used FROM focus_sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0)
        ).unwrap();

        assert_eq!(used, 75);
    }

    #[test]
    fn test_budget_remaining() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(300, false);
        session.save(conn).unwrap();

        assert_eq!(session.budget_remaining(), 300);

        session.add_distraction_time(conn, 100).unwrap();
        assert_eq!(session.budget_remaining(), 200);

        session.add_distraction_time(conn, 150).unwrap();
        assert_eq!(session.budget_remaining(), 50);

        session.add_distraction_time(conn, 50).unwrap();
        assert_eq!(session.budget_remaining(), 0);
    }

    #[test]
    fn test_budget_remaining_never_negative() {
        let mut session = FocusSession::new(100, false);
        session.distraction_used = 150; // Exceeds budget

        assert_eq!(session.budget_remaining(), 0);
    }

    #[test]
    fn test_is_budget_exhausted() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(300, false);
        session.save(conn).unwrap();

        assert!(!session.is_budget_exhausted());

        session.add_distraction_time(conn, 200).unwrap();
        assert!(!session.is_budget_exhausted());

        session.add_distraction_time(conn, 100).unwrap();
        assert!(session.is_budget_exhausted());
    }

    #[test]
    fn test_is_budget_exhausted_when_exceeded() {
        let mut session = FocusSession::new(100, false);
        session.distraction_used = 150; // Exceeds budget

        assert!(session.is_budget_exhausted());
    }

    #[test]
    fn test_scheduled_flag_persists() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut scheduled_session = FocusSession::new(600, true);
        scheduled_session.save(conn).unwrap();

        let found = FocusSession::find_active(conn).unwrap().unwrap();
        assert!(found.scheduled);
    }

    #[test]
    fn test_unscheduled_flag_persists() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut unscheduled_session = FocusSession::new(600, false);
        unscheduled_session.save(conn).unwrap();

        let found = FocusSession::find_active(conn).unwrap().unwrap();
        assert!(!found.scheduled);
    }

    #[test]
    fn test_end_unsaved_session_returns_error() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        // Don't save - session.id is None

        let result = session.end(conn);
        assert!(result.is_err(), "end() should fail on unsaved session");
    }

    #[test]
    fn test_add_distraction_time_unsaved_session_returns_error() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut session = FocusSession::new(600, false);
        // Don't save - session.id is None

        let result = session.add_distraction_time(conn, 50);
        assert!(result.is_err(), "add_distraction_time() should fail on unsaved session");
    }
}
