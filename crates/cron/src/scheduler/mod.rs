//! Cron scheduler module for computing next run times.

use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use cron::Schedule;

/// Compute next run time for a cron expression.
///
/// This is used internally by `CronSchedule::compute_next_run`.
pub fn compute_cron_next_run(expr: &str, tz: Option<&String>, now_ms: i64) -> Option<i64> {
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

/// Check if a timezone is valid.
pub fn is_valid_timezone(tz: &str) -> bool {
    tz.parse::<Tz>().is_ok()
}

#[cfg(test)]
mod tests;
