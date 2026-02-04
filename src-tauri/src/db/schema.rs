pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    productivity INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS rules (
    id INTEGER PRIMARY KEY,
    pattern TEXT NOT NULL,
    match_type TEXT NOT NULL,
    category_id INTEGER REFERENCES categories(id),
    priority INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    duration_secs INTEGER NOT NULL,
    source TEXT NOT NULL,
    app_name TEXT,
    window_title TEXT,
    url TEXT,
    domain TEXT,
    category_id INTEGER REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS focus_sessions (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    scheduled INTEGER DEFAULT 0,
    distraction_budget INTEGER NOT NULL,
    distraction_used INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS focus_schedules (
    id INTEGER PRIMARY KEY,
    days_of_week TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    distraction_budget INTEGER NOT NULL,
    enabled INTEGER DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON activities(timestamp);
CREATE INDEX IF NOT EXISTS idx_activities_category ON activities(category_id);
CREATE INDEX IF NOT EXISTS idx_focus_sessions_active ON focus_sessions(ended_at) WHERE ended_at IS NULL;
"#;

pub const DEFAULT_CATEGORIES: &[(&str, i32)] = &[
    ("Coding", 1),
    ("Communication", 0),
    ("Entertainment", -1),
    ("Reference", 1),
    ("Uncategorized", 0),
];
