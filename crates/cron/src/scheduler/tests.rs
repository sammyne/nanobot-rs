use chrono::Utc;

use super::*;
use crate::CronSchedule;

#[test]
fn compute_next_run_at() {
    let now = Utc::now().timestamp_millis();
    let schedule = CronSchedule::At { at_ms: now + 60000 };
    let next = schedule.compute_next_run(now);
    assert_eq!(next, Some(now + 60000));
}

#[test]
fn compute_next_run_at_past() {
    let now = Utc::now().timestamp_millis();
    let schedule = CronSchedule::At { at_ms: now - 60000 };
    let next = schedule.compute_next_run(now);
    assert_eq!(next, None);
}

#[test]
fn compute_next_run_every() {
    let now = Utc::now().timestamp_millis();
    let schedule = CronSchedule::Every { every_ms: 60000 };
    let next = schedule.compute_next_run(now);
    assert_eq!(next, Some(now + 60000));
}

#[test]
fn compute_next_run_cron() {
    let schedule = CronSchedule::Cron {
        expr: "0 * * * * *".to_string(), // Every minute
        tz: None,
    };
    let now = Utc::now().timestamp_millis();
    let next = schedule.compute_next_run(now);
    assert!(next.is_some());
    // Next run should be within the next minute
    let diff = next.unwrap() - now;
    assert!(diff > 0 && diff <= 60000);
}

#[test]
fn validate_schedule_valid() {
    let schedule = CronSchedule::Every { every_ms: 60000 };
    assert!(schedule.validate().is_ok());
}

#[test]
fn validate_schedule_invalid_every() {
    let schedule = CronSchedule::Every { every_ms: 0 };
    assert!(schedule.validate().is_err());
}

#[test]
fn validate_schedule_invalid_cron() {
    let schedule = CronSchedule::Cron { expr: "invalid".to_string(), tz: None };
    assert!(schedule.validate().is_err());
}

#[test]
fn validates_timezones_correctly() {
    assert!(is_valid_timezone("UTC"));
    assert!(is_valid_timezone("America/Vancouver"));
    assert!(!is_valid_timezone("Invalid/Timezone"));
}
