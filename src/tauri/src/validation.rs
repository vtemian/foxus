// src/tauri/src/validation.rs

use crate::constants::*;

/// Validate focus session budget in minutes.
/// Returns Ok(budget_secs) if valid, Err(message) if invalid.
pub fn validate_budget_minutes(budget_minutes: i32) -> Result<i32, String> {
    if budget_minutes <= 0 {
        return Err("Budget must be a positive number of minutes".to_string());
    }
    if budget_minutes > MAX_BUDGET_MINUTES {
        return Err(format!(
            "Budget cannot exceed {} minutes (24 hours)",
            MAX_BUDGET_MINUTES
        ));
    }
    Ok(budget_minutes * 60)
}

/// Validate focus schedule budget in seconds.
pub fn validate_budget_secs(budget_secs: i32) -> Result<(), String> {
    if budget_secs < 0 {
        return Err("Distraction budget cannot be negative".to_string());
    }
    if budget_secs > MAX_BUDGET_SECS {
        return Err("Distraction budget cannot exceed 24 hours".to_string());
    }
    Ok(())
}

/// Validate time format (HH:MM, 24-hour format).
pub fn validate_time_format(time: &str) -> Result<(), String> {
    if time.len() != 5 {
        return Err("Time must be in HH:MM format".to_string());
    }
    if &time[2..3] != ":" {
        return Err("Time must be in HH:MM format".to_string());
    }

    let hours: u32 = time[0..2]
        .parse()
        .map_err(|_| "Invalid hours in time".to_string())?;
    let minutes: u32 = time[3..5]
        .parse()
        .map_err(|_| "Invalid minutes in time".to_string())?;

    if hours >= 24 {
        return Err("Hours must be between 00 and 23".to_string());
    }
    if minutes >= 60 {
        return Err("Minutes must be between 00 and 59".to_string());
    }

    Ok(())
}

/// Validate days_of_week format (comma-separated day numbers 1-7).
pub fn validate_days_of_week(days: &str) -> Result<(), String> {
    if days.is_empty() {
        return Err("At least one day must be selected".to_string());
    }

    for part in days.split(',') {
        let day: u32 = part
            .trim()
            .parse()
            .map_err(|_| format!("Invalid day number: '{}'", part.trim()))?;

        if !(1..=7).contains(&day) {
            return Err(format!(
                "Day must be between 1 (Monday) and 7 (Sunday), got {}",
                day
            ));
        }
    }

    Ok(())
}

/// Validate productivity value (-1, 0, or 1).
pub fn validate_productivity(productivity: i32) -> Result<(), String> {
    if !(-1..=1).contains(&productivity) {
        return Err(
            "Productivity must be -1 (distracting), 0 (neutral), or 1 (productive)".to_string(),
        );
    }
    Ok(())
}

/// Validate category name.
pub fn validate_category_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Category name cannot be empty".to_string());
    }
    if name.len() > MAX_CATEGORY_NAME_LEN {
        return Err(format!(
            "Category name cannot exceed {} characters",
            MAX_CATEGORY_NAME_LEN
        ));
    }
    Ok(name)
}

/// Validate rule pattern.
pub fn validate_rule_pattern(pattern: &str) -> Result<&str, String> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err("Pattern cannot be empty".to_string());
    }
    if pattern.len() > MAX_RULE_PATTERN_LEN {
        return Err(format!(
            "Pattern cannot exceed {} characters",
            MAX_RULE_PATTERN_LEN
        ));
    }
    Ok(pattern)
}

/// Validate rule priority.
pub fn validate_rule_priority(priority: i32) -> Result<(), String> {
    if priority < 0 || priority > MAX_RULE_PRIORITY {
        return Err(format!(
            "Priority must be between 0 and {}",
            MAX_RULE_PRIORITY
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_budget_minutes_valid() {
        assert!(validate_budget_minutes(10).is_ok());
        assert_eq!(validate_budget_minutes(10).unwrap(), 600);
    }

    #[test]
    fn test_validate_budget_minutes_zero() {
        assert!(validate_budget_minutes(0).is_err());
    }

    #[test]
    fn test_validate_budget_minutes_negative() {
        assert!(validate_budget_minutes(-5).is_err());
    }

    #[test]
    fn test_validate_budget_minutes_too_large() {
        assert!(validate_budget_minutes(MAX_BUDGET_MINUTES + 1).is_err());
    }

    #[test]
    fn test_validate_time_format_valid() {
        assert!(validate_time_format("09:00").is_ok());
        assert!(validate_time_format("23:59").is_ok());
        assert!(validate_time_format("00:00").is_ok());
    }

    #[test]
    fn test_validate_time_format_invalid() {
        assert!(validate_time_format("9:00").is_err());
        assert!(validate_time_format("25:00").is_err());
        assert!(validate_time_format("12:60").is_err());
    }

    #[test]
    fn test_validate_days_of_week_valid() {
        assert!(validate_days_of_week("1,2,3").is_ok());
        assert!(validate_days_of_week("7").is_ok());
        assert!(validate_days_of_week("1,2,3,4,5,6,7").is_ok());
    }

    #[test]
    fn test_validate_days_of_week_invalid() {
        assert!(validate_days_of_week("").is_err());
        assert!(validate_days_of_week("0").is_err());
        assert!(validate_days_of_week("8").is_err());
    }
}
