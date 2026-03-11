//! Cron 命令测试

use super::*;

#[test]
fn test_format_time() {
    // 测试有效时间戳
    let ts = Some(1703500800000_i64); // 2023-12-25 08:00:00 UTC
    let result = format_time(ts);
    assert!(result.contains("2023-12-25"));

    // 测试空时间戳
    let result = format_time(None);
    assert_eq!(result, "-");
}

#[test]
fn test_format_schedule() {
    // Every
    let schedule = CronSchedule::Every { every_ms: 60000 };
    let result = format_schedule(&schedule);
    assert_eq!(result, "每 60 秒");

    // Cron without timezone
    let schedule = CronSchedule::Cron {
        expr: "0 0 9 * * *".to_string(),
        tz: None,
    };
    let result = format_schedule(&schedule);
    assert_eq!(result, "Cron: 0 0 9 * * *");

    // Cron with timezone
    let schedule = CronSchedule::Cron {
        expr: "0 0 9 * * *".to_string(),
        tz: Some("Asia/Shanghai".to_string()),
    };
    let result = format_schedule(&schedule);
    assert_eq!(result, "Cron: 0 0 9 * * * (Asia/Shanghai)");
}

#[test]
fn test_build_schedule_every() {
    let cmd = AddCmd {
        name: "test".to_string(),
        message: "test message".to_string(),
        every: Some(60),
        cron: None,
        at: None,
        tz: None,
    };

    let schedule = cmd.build_schedule().unwrap();
    assert!(matches!(schedule, CronSchedule::Every { every_ms: 60000 }));
}

#[test]
fn test_build_schedule_cron() {
    let cmd = AddCmd {
        name: "test".to_string(),
        message: "test message".to_string(),
        every: None,
        cron: Some("0 0 9 * * *".to_string()), // 6-field cron: sec min hour day month weekday
        at: None,
        tz: Some("Asia/Shanghai".to_string()),
    };

    let schedule = cmd.build_schedule().unwrap();
    match schedule {
        CronSchedule::Cron { expr, tz } => {
            assert_eq!(expr, "0 0 9 * * *");
            assert_eq!(tz, Some("Asia/Shanghai".to_string()));
        }
        _ => panic!("Expected CronSchedule::Cron"),
    }
}

#[test]
fn test_build_schedule_no_schedule() {
    let cmd = AddCmd {
        name: "test".to_string(),
        message: "test message".to_string(),
        every: None,
        cron: None,
        at: None,
        tz: None,
    };

    let result = cmd.build_schedule();
    assert!(result.is_err());
}

#[test]
fn test_build_schedule_multiple_schedules() {
    let cmd = AddCmd {
        name: "test".to_string(),
        message: "test message".to_string(),
        every: Some(60),
        cron: Some("0 9 * * *".to_string()),
        at: None,
        tz: None,
    };

    let result = cmd.build_schedule();
    assert!(result.is_err());
}

#[test]
fn test_build_schedule_tz_without_cron() {
    let cmd = AddCmd {
        name: "test".to_string(),
        message: "test message".to_string(),
        every: Some(60),
        cron: None,
        at: None,
        tz: Some("Asia/Shanghai".to_string()),
    };

    let result = cmd.build_schedule();
    assert!(result.is_err());
}
