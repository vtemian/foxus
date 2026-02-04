use rusqlite::{Connection, Result};
use super::schema::{SCHEMA, DEFAULT_CATEGORIES};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA)?;
    seed_default_categories(conn)?;
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
