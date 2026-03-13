//! Cron scheduler module for computing next run times.

use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;

use crate::types::CronSchedule;

/// Compute the next run time in milliseconds.
pub fn compute_next_run(schedule: &CronSchedule, now_ms: i64) -> Option<i64> {
    match schedule {
        CronSchedule::At { at_ms } => {
            // One-time execution
            if *at_ms > now_ms { Some(*at_ms) } else { None }
        }
        CronSchedule::Every { every_ms } => {
            // Recurring execution
            if *every_ms <= 0 {
                return None;
            }
            Some(now_ms + every_ms)
        }
        CronSchedule::Cron { expr, tz } => {
            // Cron expression based scheduling
            compute_cron_next_run(expr, tz.as_ref(), now_ms)
        }
    }
}

/// Compute next run time for a cron expression.
fn compute_cron_next_run(expr: &str, tz: Option<&String>, now_ms: i64) -> Option<i64> {
    // Parse the cron expression
    let schedule = match Schedule::from_str(expr) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // Get current time in the specified timezone
    let now_dt: DateTime<Tz> = if let Some(tz_name) = tz {
        let tz: Tz = match tz_name.parse() {
            Ok(tz) => tz,
            Err(_) => return None,
        };
        Utc.timestamp_millis_opt(now_ms).single()?.with_timezone(&tz)
    } else {
        Utc.timestamp_millis_opt(now_ms).single()?.with_timezone(&chrono_tz::UTC)
    };

    // Get the next scheduled time
    let next_dt = schedule.after(&now_dt).next()?;

    // Convert to milliseconds
    Some(next_dt.timestamp_millis())
}

/// Validate a schedule for job addition.
pub fn validate_schedule(schedule: &CronSchedule) -> Result<(), String> {
    match schedule {
        CronSchedule::Cron { expr, tz } => {
            // Validate cron expression
            if Schedule::from_str(expr).is_err() {
                return Err(format!("Invalid cron expression: {expr}"));
            }

            // Validate timezone if provided
            if let Some(tz_name) = tz
                && tz_name.parse::<Tz>().is_err()
            {
                return Err(format!("Unknown timezone: {tz_name}"));
            }
        }
        CronSchedule::At { at_ms } => {
            if *at_ms <= 0 {
                return Err("at_ms must be positive".to_string());
            }
        }
        CronSchedule::Every { every_ms } => {
            if *every_ms <= 0 {
                return Err("every_ms must be positive".to_string());
            }
        }
    }

    Ok(())
}

/// Check if a timezone is valid.
pub fn is_valid_timezone(tz: &str) -> bool {
    tz.parse::<Tz>().is_ok()
}

#[cfg(test)]
mod tests;
