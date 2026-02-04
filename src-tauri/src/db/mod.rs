pub mod schema;
pub mod migrations;

use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_opens() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let _db = Database::open(&db_path).unwrap();
        assert!(db_path.exists());
    }

    #[test]
    fn test_migrations_run() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(&db.connection()).unwrap();

        // Verify tables exist
        let count: i32 = db.connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='categories'",
                [],
                |row| row.get(0)
            ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_all_tables_created() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();

        // Verify all expected tables exist
        let expected_tables = ["categories", "rules", "activities", "focus_sessions", "focus_schedules"];
        for table in &expected_tables {
            let count: i32 = db.connection()
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0)
                ).unwrap();
            assert_eq!(count, 1, "Table {} should exist", table);
        }
    }

    #[test]
    fn test_default_categories_seeded() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        migrations::run(db.connection()).unwrap();

        // Verify default categories are seeded
        let count: i32 = db.connection()
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5, "Should have 5 default categories");

        // Verify specific categories exist
        let coding_exists: i32 = db.connection()
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE name='Coding' AND productivity=1",
                [],
                |row| row.get(0)
            ).unwrap();
        assert_eq!(coding_exists, 1, "Coding category should exist with productivity=1");
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();

        // Run migrations twice
        migrations::run(db.connection()).unwrap();
        migrations::run(db.connection()).unwrap();

        // Should still have only 5 categories (not duplicated)
        let count: i32 = db.connection()
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5, "Running migrations twice should not duplicate categories");
    }
}
