use crate::constants::*;
use crate::error::AppError;

/// Validate focus session budget in minutes.
/// Returns Ok(budget_secs) if valid.
pub fn validate_budget_minutes(budget_minutes: i32) -> Result<i32, AppError> {
    if budget_minutes <= 0 {
        return Err(AppError::InvalidInput {
            field: "budget_minutes",
            reason: "must be positive".into(),
        });
    }
    if budget_minutes > MAX_BUDGET_MINUTES {
        return Err(AppError::InvalidInput {
            field: "budget_minutes",
            reason: format!("cannot exceed {} minutes", MAX_BUDGET_MINUTES),
        });
    }
    Ok(budget_minutes * 60)
}

/// Validate focus schedule budget in seconds.
pub fn validate_budget_secs(budget_secs: i32) -> Result<(), AppError> {
    if budget_secs < 0 {
        return Err(AppError::InvalidInput {
            field: "distraction_budget",
            reason: "cannot be negative".into(),
        });
    }
    if budget_secs > MAX_BUDGET_SECS {
        return Err(AppError::InvalidInput {
            field: "distraction_budget",
            reason: "cannot exceed 24 hours".into(),
        });
    }
    Ok(())
}

/// Validate time format (HH:MM, 24-hour format).
pub fn validate_time_format(time: &str) -> Result<(), AppError> {
    let err = |reason: &str| AppError::InvalidInput {
        field: "time",
        reason: reason.into(),
    };

    if time.len() != 5 || &time[2..3] != ":" {
        return Err(err("must be in HH:MM format"));
    }

    let hours: u32 = time[0..2].parse().map_err(|_| err("invalid hours"))?;
    let minutes: u32 = time[3..5].parse().map_err(|_| err("invalid minutes"))?;

    if hours >= 24 {
        return Err(err("hours must be 00-23"));
    }
    if minutes >= 60 {
        return Err(err("minutes must be 00-59"));
    }

    Ok(())
}

/// Validate days_of_week format (comma-separated day numbers 1-7).
pub fn validate_days_of_week(days: &str) -> Result<(), AppError> {
    if days.is_empty() {
        return Err(AppError::InvalidInput {
            field: "days_of_week",
            reason: "at least one day required".into(),
        });
    }

    for part in days.split(',') {
        let day: u32 = part.trim().parse().map_err(|_| AppError::InvalidInput {
            field: "days_of_week",
            reason: format!("invalid day: '{}'", part.trim()),
        })?;

        if !(1..=7).contains(&day) {
            return Err(AppError::InvalidInput {
                field: "days_of_week",
                reason: format!("day must be 1-7, got {}", day),
            });
        }
    }

    Ok(())
}

/// Validate productivity value (-1, 0, or 1).
pub fn validate_productivity(productivity: i32) -> Result<(), AppError> {
    if !(-1..=1).contains(&productivity) {
        return Err(AppError::InvalidInput {
            field: "productivity",
            reason: "must be -1, 0, or 1".into(),
        });
    }
    Ok(())
}

/// Validate category name.
pub fn validate_category_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::InvalidInput {
            field: "name",
            reason: "cannot be empty".into(),
        });
    }
    if name.len() > MAX_CATEGORY_NAME_LEN {
        return Err(AppError::InvalidInput {
            field: "name",
            reason: format!("cannot exceed {} characters", MAX_CATEGORY_NAME_LEN),
        });
    }
    Ok(name)
}

/// Validate rule pattern.
pub fn validate_rule_pattern(pattern: &str) -> Result<&str, AppError> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err(AppError::InvalidInput {
            field: "pattern",
            reason: "cannot be empty".into(),
        });
    }
    if pattern.len() > MAX_RULE_PATTERN_LEN {
        return Err(AppError::InvalidInput {
            field: "pattern",
            reason: format!("cannot exceed {} characters", MAX_RULE_PATTERN_LEN),
        });
    }
    Ok(pattern)
}

/// Validate rule priority.
pub fn validate_rule_priority(priority: i32) -> Result<(), AppError> {
    if !(0..=MAX_RULE_PRIORITY).contains(&priority) {
        return Err(AppError::InvalidInput {
            field: "priority",
            reason: format!("must be 0-{}", MAX_RULE_PRIORITY),
        });
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
