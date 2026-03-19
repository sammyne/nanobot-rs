use tempfile::tempdir;

use super::*;
use crate::types::{CronPayload, CronSchedule};

#[tokio::test]
async fn storage_new() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let storage = CronStorage::load(path).await.unwrap();

    let jobs = storage.list_jobs(true).await;
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn storage_save_and_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");

    let storage = CronStorage::load(path.clone()).await.unwrap();

    let job = CronJob::new("Test".to_string(), CronSchedule::Every { every_ms: 60000 }, CronPayload::default(), false);

    storage.add_job(job).await;
    storage.save().await.unwrap();

    // Load in a new storage instance
    let storage2 = CronStorage::load(path).await.unwrap();

    let jobs = storage2.list_jobs(true).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "Test");
}

#[tokio::test]
async fn storage_remove_job() {
    let storage = CronStorage::load(PathBuf::from("/tmp/test_cron.json")).await.unwrap();

    let job = CronJob::new("Test".to_string(), CronSchedule::Every { every_ms: 60000 }, CronPayload::default(), false);

    let job_id = job.id.clone();
    storage.add_job(job).await;

    assert!(storage.remove_job(&job_id).await);
    assert!(!storage.remove_job(&job_id).await);
}
