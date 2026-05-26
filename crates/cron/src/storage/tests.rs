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

#[tokio::test]
async fn reload_if_changed_picks_up_external_modification() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");

    // 创建 storage 并添加一个 job
    let storage = CronStorage::load(path.clone()).await.unwrap();
    let job =
        CronJob::new("Original".to_string(), CronSchedule::Every { every_ms: 60000 }, CronPayload::default(), false);
    storage.add_job(job).await;
    storage.save().await.unwrap();

    // 等待确保 mtime 差异（文件系统精度通常 1 秒）
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // 外部修改 jobs.json（模拟 CLI 操作）
    let external_job =
        CronJob::new("External".to_string(), CronSchedule::Every { every_ms: 30000 }, CronPayload::default(), false);
    let external_store = CronStore { version: 1, jobs: vec![external_job] };
    let content = serde_json::to_string_pretty(&external_store).unwrap();
    tokio::fs::write(&path, content).await.unwrap();

    // reload_if_changed 应检测到变化并重载
    storage.reload_if_changed().await;

    let jobs = storage.list_jobs(true).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "External");
}

#[tokio::test]
async fn reload_if_changed_noop_when_unchanged() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");

    let storage = CronStorage::load(path).await.unwrap();
    let job = CronJob::new("Test".to_string(), CronSchedule::Every { every_ms: 60000 }, CronPayload::default(), false);
    storage.add_job(job).await;
    storage.save().await.unwrap();

    // 不修改文件，reload_if_changed 应为 noop
    storage.reload_if_changed().await;

    let jobs = storage.list_jobs(true).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name, "Test");
}
