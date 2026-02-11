use rusqlite::{Connection, Result};
use super::schema::{SCHEMA, DEFAULT_CATEGORIES};

pub const DEFAULT_RULES: &[(&str, &str, &str)] = &[
    // Coding (productive)
    ("code", "app", "Coding"),
    ("visual studio", "app", "Coding"),
    ("xcode", "app", "Coding"),
    ("intellij", "app", "Coding"),
    ("webstorm", "app", "Coding"),
    ("pycharm", "app", "Coding"),
    ("terminal", "app", "Coding"),
    ("iterm", "app", "Coding"),
    ("github.com", "domain", "Coding"),
    ("gitlab.com", "domain", "Coding"),
    ("stackoverflow.com", "domain", "Reference"),
    ("docs.rs", "domain", "Reference"),

    // Communication (neutral)
    ("slack", "app", "Communication"),
    ("discord", "app", "Communication"),
    ("mail", "app", "Communication"),
    ("outlook", "app", "Communication"),
    ("teams", "app", "Communication"),
    ("zoom", "app", "Communication"),

    // Entertainment (distracting)
    ("youtube.com", "domain", "Entertainment"),
    ("netflix.com", "domain", "Entertainment"),
    ("twitter.com", "domain", "Entertainment"),
    ("x.com", "domain", "Entertainment"),
    ("reddit.com", "domain", "Entertainment"),
    ("facebook.com", "domain", "Entertainment"),
    ("instagram.com", "domain", "Entertainment"),
    ("tiktok.com", "domain", "Entertainment"),
    ("twitch.tv", "domain", "Entertainment"),
];

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    seed_default_categories(conn)?;
    seed_default_rules(conn)?;
    Ok(())
}

fn seed_default_categories(conn: &Connection) -> Result<()> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0)
    )?;

    if count == 0 {
        for (name, productivity) in DEFAULT_CATEGORIES {
            conn.execute(
                "INSERT INTO categories (name, productivity) VALUES (?1, ?2)",
                [*name, &productivity.to_string()],
            )?;
        }
    }
    Ok(())
}

fn seed_default_rules(conn: &Connection) -> Result<()> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM rules",
        [],
        |row| row.get(0)
    )?;

    if count > 0 {
        return Ok(());
    }

    let categories: std::collections::HashMap<String, i64> = conn
        .prepare("SELECT name, id FROM categories")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    for (pattern, match_type, category_name) in DEFAULT_RULES {
        if let Some(&category_id) = categories.get(*category_name) {
            conn.execute(
                "INSERT INTO rules (pattern, match_type, category_id, priority) VALUES (?1, ?2, ?3, 10)",
                [*pattern, *match_type, &category_id.to_string()],
            )?;
        }
    }

    Ok(())
}
