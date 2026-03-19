//! Cron storage module for persisting jobs.

use std::path::PathBuf;

use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::types::{CronJob, CronStore};

/// Storage backend for cron jobs.
pub struct CronStorage {
    store_path: PathBuf,
    store: RwLock<CronStore>,
}

impl CronStorage {
    /// Create a new storage instance and load data from disk.
    /// If the file doesn't exist or fails to load, an empty store is created.
    pub async fn load(store_path: PathBuf) -> Result<Self, anyhow::Error> {
        let store = match tokio::fs::read_to_string(&store_path).await {
            Ok(content) => match serde_json::from_str::<CronStore>(&content) {
                Ok(store) => store,
                Err(e) => {
                    warn!("Failed to parse cron store file: {}, creating empty store", e);
                    CronStore::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("Cron store file does not exist, creating empty store");
                CronStore::default()
            }
            Err(e) => {
                warn!("Failed to read cron store file: {}, creating empty store", e);
                CronStore::default()
            }
        };

        Ok(CronStorage { store_path, store: RwLock::new(store) })
    }

    /// Save jobs to disk.
    pub async fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.store_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let store = self.store.read().await;
        let content = serde_json::to_string_pretty(&*store)?;
        tokio::fs::write(&self.store_path, content).await?;

        info!("Saved {} cron jobs to disk", store.jobs.len());
        Ok(())
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
