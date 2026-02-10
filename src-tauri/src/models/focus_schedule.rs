use rusqlite::{Connection, Result, params};

/// A recurring focus schedule that can auto-start focus sessions.
#[derive(Debug, Clone)]
pub struct FocusSchedule {
    pub id: Option<i64>,
    /// Comma-separated day numbers (1=Monday, 7=Sunday). E.g., "1,2,3,4,5" for weekdays.
    pub days_of_week: String,
    /// Start time in HH:MM format (24-hour).
    pub start_time: String,
    /// End time in HH:MM format (24-hour).
    pub end_time: String,
    /// Distraction budget in seconds for auto-started sessions.
    pub distraction_budget: i32,
    /// Whether this schedule is enabled.
    pub enabled: bool,
}

impl FocusSchedule {
    /// Create a new focus schedule (not yet saved to database).
    pub fn new(
        days_of_week: &str,
        start_time: &str,
        end_time: &str,
        distraction_budget: i32,
    ) -> Self {
        Self {
            id: None,
            days_of_week: days_of_week.to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            distraction_budget,
            enabled: true,
        }
    }

    /// Save the schedule to the database.
    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO focus_schedules (days_of_week, start_time, end_time, distraction_budget, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.days_of_week,
                self.start_time,
                self.end_time,
                self.distraction_budget,
                self.enabled as i32,
            ],
        )?;
        self.id = Some(conn.last_insert_rowid());
        Ok(())
    }

    /// Update an existing schedule in the database.
    pub fn update(&self, conn: &Connection) -> Result<()> {
        let id = self.id.ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Cannot update unsaved schedule".to_string())
        })?;

        conn.execute(
            "UPDATE focus_schedules
             SET days_of_week = ?1, start_time = ?2, end_time = ?3,
                 distraction_budget = ?4, enabled = ?5
             WHERE id = ?6",
            params![
                self.days_of_week,
                self.start_time,
                self.end_time,
                self.distraction_budget,
                self.enabled as i32,
                id,
            ],
        )?;
        Ok(())
    }

    /// Find all schedules.
    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, days_of_week, start_time, end_time, distraction_budget, enabled
             FROM focus_schedules ORDER BY start_time"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: Some(row.get(0)?),
                days_of_week: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                distraction_budget: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
            })
        })?;

        rows.collect()
    }

    /// Find all enabled schedules.
    pub fn find_enabled(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, days_of_week, start_time, end_time, distraction_budget, enabled
             FROM focus_schedules WHERE enabled = 1 ORDER BY start_time"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: Some(row.get(0)?),
                days_of_week: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                distraction_budget: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
            })
        })?;

        rows.collect()
    }

    /// Find a schedule by ID.
    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, days_of_week, start_time, end_time, distraction_budget, enabled
             FROM focus_schedules WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: Some(row.get(0)?),
                days_of_week: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                distraction_budget: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a schedule from the database.
    pub fn delete(conn: &Connection, id: i64) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM focus_schedules WHERE id = ?1",
            params![id],
        )?;
        Ok(rows_affected > 0)
    }

    /// Check if this schedule applies to the given day of week (1=Monday, 7=Sunday).
    pub fn applies_to_day(&self, day: u32) -> bool {
        self.days_of_week
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .any(|d| d == day)
    }

    /// Check if the given time (HH:MM format) is within this schedule's time range.
    pub fn is_time_in_range(&self, time: &str) -> bool {
        time >= self.start_time.as_str() && time < self.end_time.as_str()
    }

    /// Check if this schedule is active at the given day and time.
    pub fn is_active_at(&self, day: u32, time: &str) -> bool {
        self.enabled && self.applies_to_day(day) && self.is_time_in_range(time)
    }

    /// Parse days_of_week string into a vector of day numbers.
    pub fn get_days(&self) -> Vec<u32> {
        self.days_of_week
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .collect()
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
    fn test_new_creates_schedule() {
        let schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);

        assert!(schedule.id.is_none());
        assert_eq!(schedule.days_of_week, "1,2,3,4,5");
        assert_eq!(schedule.start_time, "09:00");
        assert_eq!(schedule.end_time, "12:00");
        assert_eq!(schedule.distraction_budget, 600);
        assert!(schedule.enabled);
    }

    #[test]
    fn test_save_assigns_id() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        assert!(schedule.id.is_none());

        schedule.save(conn).unwrap();
        assert!(schedule.id.is_some());
    }

    #[test]
    fn test_find_all() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut s1 = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        s1.save(conn).unwrap();

        let mut s2 = FocusSchedule::new("1,3,5", "14:00", "17:00", 300);
        s2.save(conn).unwrap();

        let schedules = FocusSchedule::find_all(conn).unwrap();
        assert_eq!(schedules.len(), 2);
    }

    #[test]
    fn test_find_enabled() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut s1 = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        s1.save(conn).unwrap();

        let mut s2 = FocusSchedule::new("1,3,5", "14:00", "17:00", 300);
        s2.enabled = false;
        s2.save(conn).unwrap();

        let enabled = FocusSchedule::find_enabled(conn).unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].start_time, "09:00");
    }

    #[test]
    fn test_find_by_id() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        schedule.save(conn).unwrap();
        let id = schedule.id.unwrap();

        let found = FocusSchedule::find_by_id(conn, id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_time, "09:00");

        let not_found = FocusSchedule::find_by_id(conn, 99999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        schedule.save(conn).unwrap();
        let id = schedule.id.unwrap();

        schedule.start_time = "10:00".to_string();
        schedule.distraction_budget = 900;
        schedule.update(conn).unwrap();

        let found = FocusSchedule::find_by_id(conn, id).unwrap().unwrap();
        assert_eq!(found.start_time, "10:00");
        assert_eq!(found.distraction_budget, 900);
    }

    #[test]
    fn test_delete() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        schedule.save(conn).unwrap();
        let id = schedule.id.unwrap();

        let deleted = FocusSchedule::delete(conn, id).unwrap();
        assert!(deleted);

        let found = FocusSchedule::find_by_id(conn, id).unwrap();
        assert!(found.is_none());

        // Deleting non-existent should return false
        let deleted_again = FocusSchedule::delete(conn, id).unwrap();
        assert!(!deleted_again);
    }

    #[test]
    fn test_applies_to_day() {
        let schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);

        assert!(schedule.applies_to_day(1)); // Monday
        assert!(schedule.applies_to_day(5)); // Friday
        assert!(!schedule.applies_to_day(6)); // Saturday
        assert!(!schedule.applies_to_day(7)); // Sunday
    }

    #[test]
    fn test_is_time_in_range() {
        let schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);

        assert!(schedule.is_time_in_range("09:00"));
        assert!(schedule.is_time_in_range("10:30"));
        assert!(schedule.is_time_in_range("11:59"));
        assert!(!schedule.is_time_in_range("08:59"));
        assert!(!schedule.is_time_in_range("12:00")); // End time is exclusive
        assert!(!schedule.is_time_in_range("12:01"));
    }

    #[test]
    fn test_is_active_at() {
        let mut schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);

        // Active on Monday at 10:00
        assert!(schedule.is_active_at(1, "10:00"));

        // Not active on Saturday at 10:00
        assert!(!schedule.is_active_at(6, "10:00"));

        // Not active on Monday at 08:00
        assert!(!schedule.is_active_at(1, "08:00"));

        // Not active when disabled
        schedule.enabled = false;
        assert!(!schedule.is_active_at(1, "10:00"));
    }

    #[test]
    fn test_get_days() {
        let schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        let days = schedule.get_days();
        assert_eq!(days, vec![1, 2, 3, 4, 5]);

        let weekend = FocusSchedule::new("6,7", "10:00", "14:00", 300);
        let days = weekend.get_days();
        assert_eq!(days, vec![6, 7]);
    }

    #[test]
    fn test_update_unsaved_returns_error() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let schedule = FocusSchedule::new("1,2,3,4,5", "09:00", "12:00", 600);
        let result = schedule.update(conn);
        assert!(result.is_err());
    }
}
