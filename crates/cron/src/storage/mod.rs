//! Cron storage module for persisting jobs.

use std::path::PathBuf;
use std::time::SystemTime;

use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::types::{CronJob, CronStore};

/// Storage backend for cron jobs.
pub struct CronStorage {
    store_path: PathBuf,
    store: RwLock<CronStore>,
    /// 文件最后修改时间，用于检测外部修改（对齐 HKUDS/nanobot#1375）
    last_mtime: RwLock<SystemTime>,
}

impl CronStorage {
    /// Create a new storage instance and load data from disk.
    ///
    /// - 文件不存在：正常，创建空 store
    /// - 解析失败：warn + 空 store（降级运行）
    /// - 其他 IO 错误（权限等）：返回 Err
    pub async fn load(store_path: PathBuf) -> Result<Self, anyhow::Error> {
        let mut last_mtime = SystemTime::UNIX_EPOCH;

        let store = match tokio::fs::read_to_string(&store_path).await {
            Ok(content) => {
                // 记录文件 mtime
                if let Ok(meta) = tokio::fs::metadata(&store_path).await
                    && let Ok(mtime) = meta.modified()
                {
                    last_mtime = mtime;
                }
                match serde_json::from_str::<CronStore>(&content) {
                    Ok(store) => store,
                    Err(e) => {
                        warn!("Failed to parse cron store file: {e}, creating empty store");
                        CronStore::default()
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("Cron store file does not exist, creating empty store");
                CronStore::default()
            }
            Err(e) => return Err(e.into()),
        };

        Ok(CronStorage { store_path, store: RwLock::new(store), last_mtime: RwLock::new(last_mtime) })
    }

    /// Save jobs to disk.
    pub async fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.store_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)?;
        tokio::fs::write(&self.store_path, content).await?;

        // 更新 mtime
        if let Ok(meta) = tokio::fs::metadata(&self.store_path).await
            && let Ok(mtime) = meta.modified()
        {
            *self.last_mtime.write().await = mtime;
        }

        info!("Saved {} cron jobs to disk", store.jobs.len());
        Ok(())
    }

    /// 检测 jobs.json 是否被外部修改，如果是则重新加载。
    ///
    /// 通过比较文件 mtime 与上次记录的 mtime 判断。
    /// 任何错误仅记录 warn 日志，不影响现有内存数据。
    pub async fn reload_if_changed(&self) {
        let current_mtime = match tokio::fs::metadata(&self.store_path).await {
            Ok(meta) => match meta.modified() {
                Ok(mtime) => mtime,
                Err(_) => return,
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(_) => return,
        };

        if current_mtime == *self.last_mtime.read().await {
            return;
        }

        info!("Cron: jobs.json modified externally, reloading");

        let content = match tokio::fs::read_to_string(&self.store_path).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Cron: failed to read jobs.json for reload: {e}");
                return;
            }
        };

        match serde_json::from_str::<CronStore>(&content) {
            Ok(new_store) => {
                *self.store.write().await = new_store;
                *self.last_mtime.write().await = current_mtime;
            }
            Err(e) => {
                warn!("Cron: failed to parse jobs.json for reload: {e}");
            }
        }
    }

    /// Add a job to the store.
    pub async fn add_job(&self, job: CronJob) {
        self.store.write().await.jobs.push(job);
    }

    /// Remove a job from the store by ID.
    pub async fn remove_job(&self, job_id: &str) -> bool {
        let mut store = self.store.write().await;
        let before = store.jobs.len();
        store.jobs.retain(|j| j.id != job_id);
        store.jobs.len() < before
    }

    /// Get a job by ID.
    pub async fn get_job(&self, job_id: &str) -> Option<CronJob> {
        let store = self.store.read().await;
        store.jobs.iter().find(|j| j.id == job_id).cloned()
    }

    /// Update a job.
    pub async fn update_job(&self, job: CronJob) {
        let mut store = self.store.write().await;
        if let Some(existing) = store.jobs.iter_mut().find(|j| j.id == job.id) {
            *existing = job;
        }
    }

    /// List all jobs.
    pub async fn list_jobs(&self, include_disabled: bool) -> Vec<CronJob> {
        let store = self.store.read().await;
        let mut jobs: Vec<CronJob> = if include_disabled {
            store.jobs.clone()
        } else {
            store.jobs.iter().filter(|j| j.enabled).cloned().collect()
        };

        // Sort by next_run_at_ms (None values go to the end)
        jobs.sort_by(|a, b| match (a.state.next_run_at_ms, b.state.next_run_at_ms) {
            (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        jobs
    }

    /// Get the earliest next run time across all enabled jobs.
    pub async fn get_next_wake_ms(&self) -> Option<i64> {
        self.store
            .read()
            .await
            .jobs
            .iter()
            .filter(|j| j.enabled && j.state.next_run_at_ms.is_some())
            .filter_map(|j| j.state.next_run_at_ms)
            .min()
    }
}

#[cfg(test)]
mod tests;
