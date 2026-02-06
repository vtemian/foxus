use rusqlite::{Connection, Result, params};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    App,
    Domain,
    Title,
}

impl MatchType {
    /// Convert to string representation for database storage.
    /// Used by Rule::create and in tests.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchType::App => "app",
            MatchType::Domain => "domain",
            MatchType::Title => "title",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "app" => Some(MatchType::App),
            "domain" => Some(MatchType::Domain),
            "title" => Some(MatchType::Title),
            _ => None,
        }
    }
}

/// A rule for categorizing activities based on patterns.
/// Fields `id` and `priority` are read from the database and used in tests;
/// kept as part of the public API for future use (e.g., rule editing UI).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Rule {
    pub id: i64,
    pub pattern: String,
    pub match_type: MatchType,
    pub category_id: i64,
    pub priority: i32,
}

impl Rule {
    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, match_type, category_id, priority FROM rules ORDER BY priority DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let match_type_str: String = row.get(2)?;
            let match_type = MatchType::from_str(&match_type_str)
                .unwrap_or(MatchType::App); // Default to App for unknown types
            Ok(Self {
                id: row.get(0)?,
                pattern: row.get(1)?,
                match_type,
                category_id: row.get(3)?,
                priority: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Create a new rule.
    /// Currently used in tests; kept as part of the public API for future use.
    #[allow(dead_code)]
    pub fn create(conn: &Connection, pattern: &str, match_type: MatchType, category_id: i64, priority: i32) -> Result<Self> {
        conn.execute(
            "INSERT INTO rules (pattern, match_type, category_id, priority) VALUES (?1, ?2, ?3, ?4)",
            params![pattern, match_type.as_str(), category_id, priority],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Self { id, pattern: pattern.to_string(), match_type, category_id, priority })
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
        assert!(rules.iter().any(|r| r.pattern == "code" && r.match_type == MatchType::App),
            "Should have 'code' app rule");
        assert!(rules.iter().any(|r| r.pattern == "youtube.com" && r.match_type == MatchType::Domain),
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

        let rule = Rule::create(conn, "code", MatchType::App, coding.id, 10).unwrap();

        assert_eq!(rule.pattern, "code");
        assert_eq!(rule.match_type, MatchType::App);
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
        Rule::create(conn, "low_priority", MatchType::App, coding.id, 5).unwrap();
        Rule::create(conn, "high_priority", MatchType::App, coding.id, 20).unwrap();

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

    #[test]
    fn test_match_type_as_str() {
        assert_eq!(MatchType::App.as_str(), "app");
        assert_eq!(MatchType::Domain.as_str(), "domain");
        assert_eq!(MatchType::Title.as_str(), "title");
    }

    #[test]
    fn test_match_type_from_str() {
        assert_eq!(MatchType::from_str("app"), Some(MatchType::App));
        assert_eq!(MatchType::from_str("domain"), Some(MatchType::Domain));
        assert_eq!(MatchType::from_str("title"), Some(MatchType::Title));
        assert_eq!(MatchType::from_str("invalid"), None);
        assert_eq!(MatchType::from_str(""), None);
    }

    #[test]
    fn test_match_type_roundtrip() {
        for mt in [MatchType::App, MatchType::Domain, MatchType::Title] {
            assert_eq!(MatchType::from_str(mt.as_str()), Some(mt));
        }
    }
}
