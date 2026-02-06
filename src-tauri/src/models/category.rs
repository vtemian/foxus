use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub productivity: i32,
}

impl Category {
    /// Find a category by its ID.
    /// Currently used in tests; kept as part of the public API for future use.
    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
        let mut stmt = conn.prepare("SELECT id, name, productivity FROM categories WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                productivity: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT id, name, productivity FROM categories ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                productivity: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    /// Create a new category.
    /// Currently used in tests; kept as part of the public API for future use.
    #[allow(dead_code)]
    pub fn create(conn: &Connection, name: &str, productivity: i32) -> Result<Self> {
        conn.execute(
            "INSERT INTO categories (name, productivity) VALUES (?1, ?2)",
            params![name, productivity],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Self { id, name: name.to_string(), productivity })
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
    fn test_find_all_returns_default_categories() {
        let (db, _dir) = setup_db();
        let categories = Category::find_all(db.connection()).unwrap();
        assert!(categories.len() >= 5);
        assert!(categories.iter().any(|c| c.name == "Coding"));
    }

    #[test]
    fn test_create_category() {
        let (db, _dir) = setup_db();
        let cat = Category::create(db.connection(), "Testing", 1).unwrap();
        assert_eq!(cat.name, "Testing");
        assert_eq!(cat.productivity, 1);

        let found = Category::find_by_id(db.connection(), cat.id).unwrap();
        assert_eq!(found, Some(cat));
    }
}
