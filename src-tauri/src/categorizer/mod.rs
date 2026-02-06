use crate::models::{Category, Rule};
use rusqlite::Connection;

pub struct Categorizer {
    rules: Vec<(Rule, Category)>,
    default_category_id: i64,
}

impl Categorizer {
    pub fn new(conn: &Connection) -> rusqlite::Result<Self> {
        let rules = Rule::find_all(conn)?;
        let categories = Category::find_all(conn)?;

        let rules_with_categories: Vec<_> = rules
            .into_iter()
            .filter_map(|rule| {
                categories.iter()
                    .find(|c| c.id == rule.category_id)
                    .map(|c| (rule, c.clone()))
            })
            .collect();

        let default_category_id = categories
            .iter()
            .find(|c| c.name == "Uncategorized")
            .map(|c| c.id)
            .unwrap_or(1);

        Ok(Self {
            rules: rules_with_categories,
            default_category_id,
        })
    }

    pub fn categorize_app(&self, app_name: &str, window_title: Option<&str>) -> i64 {
        for (rule, _category) in &self.rules {
            let matches = match rule.match_type.as_str() {
                "app" => Self::pattern_matches(&rule.pattern, app_name),
                "title" => window_title.map(|t| Self::pattern_matches(&rule.pattern, t)).unwrap_or(false),
                _ => false,
            };

            if matches {
                return rule.category_id;
            }
        }

        self.default_category_id
    }

    pub fn categorize_url(&self, domain: &str) -> i64 {
        for (rule, _category) in &self.rules {
            if rule.match_type == "domain" && Self::pattern_matches(&rule.pattern, domain) {
                return rule.category_id;
            }
        }

        self.default_category_id
    }

    fn pattern_matches(pattern: &str, text: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        let text_lower = text.to_lowercase();

        if pattern_lower.contains('*') {
            let parts: Vec<&str> = pattern_lower.split('*').collect();
            let mut pos = 0;
            for part in parts {
                if part.is_empty() { continue; }
                if let Some(found) = text_lower[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            text_lower.contains(&pattern_lower)
        }
    }

    pub fn reload(&mut self, conn: &Connection) -> rusqlite::Result<()> {
        *self = Self::new(conn)?;
        Ok(())
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
    fn test_categorize_with_no_rules_returns_default() {
        let (db, _dir) = setup_db();
        let categorizer = Categorizer::new(db.connection()).unwrap();

        let category_id = categorizer.categorize_app("SomeApp", None);

        let uncategorized = Category::find_all(db.connection()).unwrap()
            .into_iter()
            .find(|c| c.name == "Uncategorized")
            .unwrap();

        assert_eq!(category_id, uncategorized.id);
    }

    #[test]
    fn test_categorize_app_with_rule() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        Rule::create(conn, "code", "app", coding.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();
        let category_id = categorizer.categorize_app("Visual Studio Code", None);

        assert_eq!(category_id, coding.id);
    }

    #[test]
    fn test_categorize_domain() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let entertainment = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Entertainment")
            .unwrap();

        Rule::create(conn, "reddit.com", "domain", entertainment.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();
        let category_id = categorizer.categorize_url("reddit.com");

        assert_eq!(category_id, entertainment.id);
    }

    #[test]
    fn test_pattern_with_wildcard() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        Rule::create(conn, "*.github.*", "domain", coding.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();

        assert_eq!(categorizer.categorize_url("www.github.com"), coding.id);
        assert_eq!(categorizer.categorize_url("gist.github.io"), coding.id);
    }

    #[test]
    fn test_pattern_matches_case_insensitive() {
        assert!(Categorizer::pattern_matches("code", "Visual Studio CODE"));
        assert!(Categorizer::pattern_matches("CODE", "visual studio code"));
    }

    #[test]
    fn test_pattern_matches_wildcard() {
        assert!(Categorizer::pattern_matches("*.github.*", "www.github.com"));
        assert!(Categorizer::pattern_matches("*git*", "github.com"));
        assert!(!Categorizer::pattern_matches("*.github.*", "gitlab.com"));
    }

    #[test]
    fn test_categorize_app_with_window_title_rule() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();

        Rule::create(conn, "pull request", "title", coding.id, 10).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();

        // Should match based on window title
        let category_id = categorizer.categorize_app("Firefox", Some("Pull Request #123 - GitHub"));
        assert_eq!(category_id, coding.id);

        // Should not match when title doesn't match
        let uncategorized = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Uncategorized")
            .unwrap();
        let category_id = categorizer.categorize_app("Firefox", Some("YouTube - Videos"));
        assert_eq!(category_id, uncategorized.id);
    }

    #[test]
    fn test_reload_picks_up_new_rules() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let mut categorizer = Categorizer::new(conn).unwrap();

        let uncategorized = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Uncategorized")
            .unwrap();

        // "MyNewApp" doesn't match any default rules, should return uncategorized
        assert_eq!(categorizer.categorize_app("MyNewApp", None), uncategorized.id);

        // Add a rule for MyNewApp
        let coding = Category::find_all(conn).unwrap()
            .into_iter()
            .find(|c| c.name == "Coding")
            .unwrap();
        Rule::create(conn, "mynewapp", "app", coding.id, 10).unwrap();

        // Reload and check new rule is applied
        categorizer.reload(conn).unwrap();
        assert_eq!(categorizer.categorize_app("MyNewApp", None), coding.id);
    }

    #[test]
    fn test_higher_priority_rule_wins_when_both_match() {
        let (db, _dir) = setup_db();
        let conn = db.connection();

        let categories = Category::find_all(conn).unwrap();
        let coding = categories.iter().find(|c| c.name == "Coding").unwrap();
        let entertainment = categories.iter().find(|c| c.name == "Entertainment").unwrap();

        // Use app names that don't conflict with default rules
        // Lower priority rule matches broadly (any app containing "editor")
        Rule::create(conn, "editor", "app", entertainment.id, 5).unwrap();
        // Higher priority rule matches more specifically "fancy editor"
        Rule::create(conn, "fancy editor", "app", coding.id, 20).unwrap();

        let categorizer = Categorizer::new(conn).unwrap();

        // "Fancy Editor Pro" matches both "editor" and "fancy editor"
        // Should use the higher priority rule (priority 20 -> Coding)
        let category_id = categorizer.categorize_app("Fancy Editor Pro", None);
        assert_eq!(category_id, coding.id, "Higher priority rule should win");

        // "Simple Editor" only matches "editor" rule
        let category_id = categorizer.categorize_app("Simple Editor", None);
        assert_eq!(category_id, entertainment.id, "Should match the only applicable rule");
    }
}
