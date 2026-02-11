use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: Option<i64>,
    pub timestamp: i64,
    pub duration_secs: i32,
    pub source: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub category_id: Option<i64>,
}

impl Activity {
    pub fn new(
        timestamp: i64,
        duration_secs: i32,
        source: &str,
        app_name: Option<&str>,
        window_title: Option<&str>,
    ) -> Self {
        Self {
            id: None,
            timestamp,
            duration_secs,
            source: source.to_string(),
            app_name: app_name.map(|s| s.to_string()),
            window_title: window_title.map(|s| s.to_string()),
            url: None,
            domain: None,
            category_id: None,
        }
    }

    pub fn save(&mut self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO activities (timestamp, duration_secs, source, app_name, window_title, url, domain, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.timestamp,
                self.duration_secs,
                self.source,
                self.app_name,
                self.window_title,
                self.url,
                self.domain,
                self.category_id,
            ],
        )?;
        self.id = Some(conn.last_insert_rowid());
        Ok(())
    }

    /// Find activities within a time range.
    /// Currently used in tests; kept as part of the public API for future use.
    #[allow(dead_code)]
    pub fn find_in_range(conn: &Connection, start: i64, end: i64) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, duration_secs, source, app_name, window_title, url, domain, category_id
             FROM activities WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp"
        )?;

        let rows = stmt.query_map(params![start, end], |row| {
            Ok(Self {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                duration_secs: row.get(2)?,
                source: row.get(3)?,
                app_name: row.get(4)?,
                window_title: row.get(5)?,
                url: row.get(6)?,
                domain: row.get(7)?,
                category_id: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    pub fn total_duration_by_category(conn: &Connection, start: i64, end: i64) -> Result<Vec<(i64, i32)>> {
        let mut stmt = conn.prepare(
            "SELECT category_id, SUM(duration_secs) as total
             FROM activities
             WHERE timestamp >= ?1 AND timestamp < ?2 AND category_id IS NOT NULL
             GROUP BY category_id"
        )?;

        let rows = stmt.query_map(params![start, end], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, migrations};
    use crate::models::Category;
    use tempfile::{tempdir, TempDir};

    fn setup_db() -> (Database, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();
        (db, dir)
    }

    #[test]
    fn test_save_and_find_activity() {
        let (db, _dir) = setup_db();
        let now = 1700000000i64;

        let mut activity = Activity::new(now, 5, "app", Some("VSCode"), Some("main.rs"));
        activity.save(db.connection()).unwrap();

        let found = Activity::find_in_range(db.connection(), now - 10, now + 10).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].app_name, Some("VSCode".to_string()));
    }

    #[test]
    fn test_total_duration_by_category() {
        let (db, _dir) = setup_db();
        let conn = db.connection();
        let now = 1700000000i64;

        let categories = Category::find_all(conn).unwrap();
        let coding_id = categories.iter().find(|c| c.name == "Coding").unwrap().id;

        let mut a1 = Activity::new(now, 30, "app", Some("VSCode"), None);
        a1.category_id = Some(coding_id);
        a1.save(conn).unwrap();

        let mut a2 = Activity::new(now + 30, 20, "app", Some("VSCode"), None);
        a2.category_id = Some(coding_id);
        a2.save(conn).unwrap();

        let totals = Activity::total_duration_by_category(conn, now - 10, now + 100).unwrap();
        let coding_total = totals.iter().find(|(id, _)| *id == coding_id);
        assert_eq!(coding_total, Some(&(coding_id, 50)));
    }
}
