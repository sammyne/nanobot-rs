use super::*;

#[test]
fn test_schedule_serialization() {
    let schedule = CronSchedule::Every { every_ms: 60000 };
    let json = serde_json::to_string(&schedule).unwrap();
    assert!(json.contains("every"));
    assert!(json.contains("60000"));
}

#[test]
fn test_at_schedule_deserialization() {
    let json = r#"{"kind":"at","at_ms":1234567890}"#;
    let schedule: CronSchedule = serde_json::from_str(json).unwrap();
    match schedule {
        CronSchedule::At { at_ms } => assert_eq!(at_ms, 1234567890),
        _ => panic!("Expected At variant"),
    }
}

#[test]
fn test_cron_schedule_with_timezone() {
    let schedule = CronSchedule::Cron {
        expr: "0 9 * * *".to_string(),
        tz: Some("America/Vancouver".to_string()),
    };
    let json = serde_json::to_string(&schedule).unwrap();
    assert!(json.contains("cron"));
    assert!(json.contains("0 9 * * *"));
    assert!(json.contains("America/Vancouver"));
}

#[test]
fn test_job_creation() {
    let schedule = CronSchedule::Every { every_ms: 60000 };
    let payload = CronPayload {
        message: "Test message".to_string(),
        ..Default::default()
    };
    let job = CronJob::new("Test".to_string(), schedule, payload, false);

    assert!(!job.id.is_empty());
    assert_eq!(job.name, "Test");
    assert!(job.enabled);
    assert!(!job.delete_after_run);
}

#[test]
fn test_store_default() {
    let store = CronStore::default();
    assert_eq!(store.version, 1);
    assert!(store.jobs.is_empty());
}
