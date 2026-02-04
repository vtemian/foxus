use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: i64,
    pub pattern: String,
    pub match_type: String,
    pub category_id: i64,
    pub priority: i32,
}

impl Rule {
    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, match_type, category_id, priority FROM rules ORDER BY priority DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Self {
                id: row.get(0)?,
                pattern: row.get(1)?,
                match_type: row.get(2)?,
                category_id: row.get(3)?,
                priority: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    pub fn create(conn: &Connection, pattern: &str, match_type: &str, category_id: i64, priority: i32) -> Result<Self> {
        conn.execute(
            "INSERT INTO rules (pattern, match_type, category_id, priority) VALUES (?1, ?2, ?3, ?4)",
            params![pattern, match_type, category_id, priority],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Self { id, pattern: pattern.to_string(), match_type: match_type.to_string(), category_id, priority })
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
    fn test_find_all_returns_default_rules() {
        let (db, _dir) = setup_db();
        let rules = Rule::find_all(db.connection()).unwrap();
        // After migrations, we should have default rules seeded
        assert!(!rules.is_empty(), "Should have default rules after migrations");

        // Verify some specific default rules exist
        assert!(rules.iter().any(|r| r.pattern == "code" && r.match_type == "app"),
            "Should have 'code' app rule");
        assert!(rules.iter().any(|r| r.pattern == "youtube.com" && r.match_type == "domain"),
            "Should have 'youtube.com' domain rule");
    }

    #[test]
    fn test_create_rule() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        let rule = Rule::create(conn, "code", "app", coding.id, 10).unwrap();

        assert_eq!(rule.pattern, "code");
        assert_eq!(rule.match_type, "app");
        assert_eq!(rule.category_id, coding.id);
        assert_eq!(rule.priority, 10);
    }

    #[test]
    fn test_find_all_returns_rules_ordered_by_priority_desc() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        // Add rules with different priorities
        Rule::create(conn, "low_priority", "app", coding.id, 5).unwrap();
        Rule::create(conn, "high_priority", "app", coding.id, 20).unwrap();

        let rules = Rule::find_all(conn).unwrap();

        // Verify ordering: highest priority first
        // high_priority (20) should come before default rules (10) and low_priority (5)
        assert_eq!(rules[0].pattern, "high_priority", "Highest priority rule should be first");
        assert_eq!(rules[0].priority, 20);

        // low_priority should be last
        let last_rule = rules.last().unwrap();
        assert_eq!(last_rule.pattern, "low_priority", "Lowest priority rule should be last");
        assert_eq!(last_rule.priority, 5);
    }
}
