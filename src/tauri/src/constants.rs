// src/tauri/src/constants.rs

/// Seconds in one day (24 * 60 * 60)
pub const SECS_PER_DAY: i64 = 86400;

/// Maximum focus budget in minutes (24 hours)
pub const MAX_BUDGET_MINUTES: i32 = 24 * 60;

/// Maximum focus budget in seconds (24 hours)
pub const MAX_BUDGET_SECS: i32 = 24 * 60 * 60;

/// Maximum rule priority value
pub const MAX_RULE_PRIORITY: i32 = 1000;

/// Maximum category name length
pub const MAX_CATEGORY_NAME_LEN: usize = 100;

/// Maximum rule pattern length
pub const MAX_RULE_PATTERN_LEN: usize = 500;
