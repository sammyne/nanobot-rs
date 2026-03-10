use tempfile::tempdir;

use super::*;

#[tokio::test]
async fn cron_service_new() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = CronService::new(path, None);

    assert!(!service.is_running().await);
}

#[tokio::test]
async fn cron_service_start_stop() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = CronService::new(path, None);

    service.start().await.unwrap();
    assert!(service.is_running().await);

    service.stop().await;
    assert!(!service.is_running().await);
}

#[tokio::test]
async fn add_job() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = CronService::new(path, None);
    service.start().await.unwrap();

    let schedule = CronSchedule::Every { every_ms: 60000 };
    let job = service
        .add_job(
            "Test".to_string(),
            schedule,
            "Test message".to_string(),
            false,
            None,
            None,
            false,
        )
        .await
        .unwrap();

    assert_eq!(job.name, "Test");
    assert!(job.enabled);

    let jobs = service.list_jobs(false).await;
    assert_eq!(jobs.len(), 1);
}

#[tokio::test]
async fn remove_job() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = CronService::new(path, None);
    service.start().await.unwrap();

    let schedule = CronSchedule::Every { every_ms: 60000 };
    let job = service
        .add_job(
            "Test".to_string(),
            schedule,
            "Test message".to_string(),
            false,
            None,
            None,
            false,
        )
        .await
        .unwrap();

    assert!(service.remove_job(&job.id).await);
    assert!(!service.remove_job(&job.id).await);

    let jobs = service.list_jobs(true).await;
    assert!(jobs.is_empty());
}
