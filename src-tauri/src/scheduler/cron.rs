//! Cron parsing and scheduling

use chrono::{DateTime, Utc, TimeZone};
use std::collections::HashMap;

/// Cron expression parser
#[derive(Debug, Clone)]
pub struct CronExpression {
    minute: CronField,
    hour: CronField,
    day_of_month: CronField,
    month: CronField,
    day_of_week: CronField,
}

#[derive(Debug, Clone)]
enum CronField {
    All,
    Exact(u32),
    List(Vec<u32>),
    Range(u32, u32),
    Step(u32, u32),
}

impl CronExpression {
    /// Parse a cron expression
    pub fn parse(expr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err("Cron expression must have 5 fields".to_string());
        }

        Ok(Self {
            minute: Self::parse_field(parts[0], 0, 59)?,
            hour: Self::parse_field(parts[1], 0, 23)?,
            day_of_month: Self::parse_field(parts[2], 1, 31)?,
            month: Self::parse_field(parts[3], 1, 12)?,
            day_of_week: Self::parse_field(parts[4], 0, 6)?,
        })
    }

    fn parse_field(s: &str, min: u32, max: u32) -> Result<CronField, String> {
        if s == "*" {
            return Ok(CronField::All);
        }

        if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid step expression: {}", s));
            }
            let step: u32 = parts[1].parse().map_err(|_| format!("Invalid step: {}", parts[1]))?;
            if step == 0 {
                return Err("Step cannot be zero".to_string());
            }
            return Ok(CronField::Step(min, step));
        }

        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid range: {}", s));
            }
            let start: u32 = parts[0].parse().map_err(|_| format!("Invalid range start: {}", parts[0]))?;
            let end: u32 = parts[1].parse().map_err(|_| format!("Invalid range end: {}", parts[1]))?;
            return Ok(CronField::Range(start, end));
        }

        if s.contains(',') {
            let values: Result<Vec<u32>, _> = s.split(',')
                .map(|v| v.parse().map_err(|_| format!("Invalid value: {}", v)))
                .collect();
            return Ok(CronField::List(values?));
        }

        let value: u32 = s.parse().map_err(|_| format!("Invalid value: {}", s))?;
        if value < min || value > max {
            return Err(format!("Value {} out of range [{}, {}]", value, min, max));
        }
        Ok(CronField::Exact(value))
    }

    /// Get the next scheduled time after the given time
    pub fn next_after(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // Simple implementation: check each minute for the next year
        let mut current = after.with_second(0).unwrap() + chrono::Duration::minutes(1);

        for _ in 0..525600 {
            if self.matches(&current) {
                return Some(current);
            }
            current = current + chrono::Duration::minutes(1);
        }

        None
    }

    fn matches(&self, dt: &DateTime<Utc>) -> bool {
        self.minute.matches(dt.minute() as u32) &&
        self.hour.matches(dt.hour() as u32) &&
        self.day_of_month.matches(dt.day() as u32) &&
        self.month.matches(dt.month() as u32) &&
        self.day_of_week.matches(dt.weekday().num_days_from_sunday())
    }
}

impl CronField {
    fn matches(&self, value: u32) -> bool {
        match self {
            CronField::All => true,
            CronField::Exact(v) => *v == value,
            CronField::List(v) => v.contains(&value),
            CronField::Range(start, end) => value >= *start && value <= *end,
            CronField::Step(start, step) => value >= *start && (value - start) % step == 0,
        }
    }
}

/// Human-readable schedule presets
pub fn parse_preset(preset: &str) -> Result<String, String> {
    match preset.to_lowercase().as_str() {
        "every_minute" => Ok("* * * * *".to_string()),
        "every_hour" => Ok("0 * * * *".to_string()),
        "every_day" | "daily" => Ok("0 0 * * *".to_string()),
        "every_week" | "weekly" => Ok("0 0 * * 0".to_string()),
        "every_month" | "monthly" => Ok("0 0 1 * *".to_string()),
        "workday_morning" => Ok("0 9 * * 1-5".to_string()),
        "workday_evening" => Ok("0 18 * * 1-5".to_string()),
        _ => Err(format!("Unknown preset: {}", preset)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cron() {
        let expr = CronExpression::parse("0 * * * *").unwrap();
        assert!(matches!(expr.minute, CronField::Exact(0)));
        assert!(matches!(expr.hour, CronField::All));
    }

    #[test]
    fn test_preset() {
        let expr = parse_preset("daily").unwrap();
        assert_eq!(expr, "0 0 * * *");
    }
}
