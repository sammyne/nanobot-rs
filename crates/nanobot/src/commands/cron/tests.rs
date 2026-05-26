//! Cron 命令测试

use super::*;

#[test]
fn format_time_valid_timestamp() {
    let ts = Some(1703500800000_i64); // 2023-12-25 08:00:00 UTC
    let result = format_time(ts);
    assert!(result.contains("2023-12-25"));
}

#[test]
fn format_time_none() {
    let result = format_time(None);
    assert_eq!(result, "-");
}
