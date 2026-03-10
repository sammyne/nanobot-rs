//! Cron service for managing and executing scheduled jobs.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

use crate::scheduler::{compute_next_run, validate_schedule};
use crate::storage::CronStorage;
use crate::types::{CronJob, CronPayload, CronSchedule};

/// Callback function type for job execution
pub type JobCallback =
    Arc<dyn Fn(CronJob) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync>;

/// Service for managing and executing scheduled jobs.
pub struct CronService {
    storage: Arc<CronStorage>,
    on_job: Option<JobCallback>,
    running: Arc<RwLock<bool>>,
    timer_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl CronService {
    /// Create a new cron service.
    pub fn new(store_path: PathBuf, on_job: Option<JobCallback>) -> Self {
        CronService {
            storage: Arc::new(CronStorage::new(store_path)),
            on_job,
            running: Arc::new(RwLock::new(false)),
            timer_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the cron service.
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Cron service is already running");
            return Ok(());
        }

        *running = true;
        drop(running);

        // Load persisted data
        self.storage.load().await?;

        // Recompute next run times for all enabled jobs
        self.recompute_next_runs().await;

        // Save updated state
        self.storage.save().await?;

        // Start the timer
        self.arm_timer().await;

        let job_count = self.storage.list_jobs(true).await.len();
        info!("Cron service started with {} jobs", job_count);

        Ok(())
    }

    /// Stop the cron service.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        let mut timer_task = self.timer_task.write().await;
        if let Some(task) = timer_task.take() {
            task.abort();
        }

        info!("Cron service stopped");
    }

    /// Check if the service is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Recompute next run times for all enabled jobs.
    async fn recompute_next_runs(&self) {
        let now = chrono::Utc::now().timestamp_millis();
        let mut jobs = self.storage.list_jobs(true).await;

        for job in &mut jobs {
            if job.enabled {
                job.state.next_run_at_ms = compute_next_run(&job.schedule, now);
                self.storage.update_job(job.clone()).await;
            }
        }
    }

    /// Schedule the next timer tick.
    async fn arm_timer(&self) {
        // Cancel existing timer
        let mut timer_task = self.timer_task.write().await;
        if let Some(task) = timer_task.take() {
            task.abort();
        }

        let running = Arc::clone(&self.running);
        let storage = Arc::clone(&self.storage);
        let on_job = self.on_job.clone();

        let task = tokio::spawn(async move {
            // Use a loop instead of recursion
            loop {
                // Check if still running
                if !*running.read().await {
                    break;
                }

                // Get next wake time
                let next_wake = match storage.get_next_wake_ms().await {
                    Some(t) => t,
                    None => {
                        // No jobs, wait a bit and check again
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };

                let now = chrono::Utc::now().timestamp_millis();
                let delay_ms = (next_wake - now).max(0) as u64;

                if delay_ms > 0 {
                    sleep(Duration::from_millis(delay_ms)).await;
                }

                // Check if still running after sleep
                if !*running.read().await {
                    break;
                }

                // Execute due jobs
                let start_now = chrono::Utc::now().timestamp_millis();
                let jobs = storage.list_jobs(true).await;

                let due_jobs: Vec<CronJob> = jobs
                    .into_iter()
                    .filter(|j| j.enabled && j.state.next_run_at_ms.map(|t| start_now >= t).unwrap_or(false))
                    .collect();

                for job in due_jobs {
                    Self::execute_job(storage.clone(), on_job.clone(), job).await;
                }

                if let Err(e) = storage.save().await {
                    error!("Failed to save cron store: {}", e);
                }
            }
        });

        *timer_task = Some(task);
    }

    /// Execute a single job.
    async fn execute_job(storage: Arc<CronStorage>, on_job: Option<JobCallback>, mut job: CronJob) {
        let start_ms = chrono::Utc::now().timestamp_millis();
        info!("Cron: executing job '{}' ({})", job.name, job.id);

        let result = if let Some(callback) = on_job {
            match callback(job.clone()).await {
                Ok(response) => {
                    info!("Cron: job '{}' completed: {}", job.name, response);
                    Ok(())
                }
                Err(e) => {
                    error!("Cron: job '{}' failed: {}", job.name, e);
                    Err(e)
                }
            }
        } else {
            info!("Cron: job '{}' completed (no callback)", job.name);
            Ok(())
        };

        job.state.last_run_at_ms = Some(start_ms);
        job.updated_at_ms = chrono::Utc::now().timestamp_millis();

        match result {
            Ok(()) => {
                job.state.last_status = Some("ok".to_string());
                job.state.last_error = None;
            }
            Err(e) => {
                job.state.last_status = Some("error".to_string());
                job.state.last_error = Some(e);
            }
        }

        // Handle one-shot jobs
        if matches!(job.schedule, CronSchedule::At { .. }) {
            if job.delete_after_run {
                storage.remove_job(&job.id).await;
                return;
            } else {
                job.enabled = false;
                job.state.next_run_at_ms = None;
            }
        } else {
            // Compute next run
            let now = chrono::Utc::now().timestamp_millis();
            job.state.next_run_at_ms = compute_next_run(&job.schedule, now);
        }

        storage.update_job(job).await;
    }

    // ========== Public API ==========

    /// List all jobs.
    pub async fn list_jobs(&self, include_disabled: bool) -> Vec<CronJob> {
        self.storage.list_jobs(include_disabled).await
    }

    /// Add a new job.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_job(
        &self,
        name: String,
        schedule: CronSchedule,
        message: String,
        deliver: bool,
        channel: Option<String>,
        to: Option<String>,
        delete_after_run: bool,
    ) -> Result<CronJob, String> {
        // Validate schedule
        validate_schedule(&schedule)?;

        let now = chrono::Utc::now().timestamp_millis();
        let next_run = compute_next_run(&schedule, now);

        let mut job = CronJob::new(
            name.clone(),
            schedule,
            CronPayload {
                kind: "agent_turn".to_string(),
                message,
                deliver,
                channel,
                to,
            },
            delete_after_run,
        );

        job.state.next_run_at_ms = next_run;
        job.created_at_ms = now;
        job.updated_at_ms = now;

        self.storage.add_job(job.clone()).await;

        if let Err(e) = self.storage.save().await {
            error!("Failed to save cron store: {}", e);
        }

        self.arm_timer().await;

        info!("Cron: added job '{}' ({})", name, job.id);
        Ok(job)
    }

    /// Remove a job by ID.
    pub async fn remove_job(&self, job_id: &str) -> bool {
        let removed = self.storage.remove_job(job_id).await;

        if removed {
            if let Err(e) = self.storage.save().await {
                error!("Failed to save cron store: {}", e);
            }
            self.arm_timer().await;
            info!("Cron: removed job {}", job_id);
        }

        removed
    }

    /// Enable or disable a job.
    pub async fn enable_job(&self, job_id: &str, enabled: bool) -> Option<CronJob> {
        let mut job = self.storage.get_job(job_id).await?;
        job.enabled = enabled;
        job.updated_at_ms = chrono::Utc::now().timestamp_millis();

        if enabled {
            job.state.next_run_at_ms = compute_next_run(&job.schedule, chrono::Utc::now().timestamp_millis());
        } else {
            job.state.next_run_at_ms = None;
        }

        self.storage.update_job(job.clone()).await;

        if let Err(e) = self.storage.save().await {
            error!("Failed to save cron store: {}", e);
        }

        self.arm_timer().await;
        Some(job)
    }

    /// Get service status.
    pub async fn status(&self) -> serde_json::Value {
        let job_count = self.storage.list_jobs(true).await.len();
        let next_wake = self.storage.get_next_wake_ms().await;

        serde_json::json!({
            "enabled": *self.running.read().await,
            "jobs": job_count,
            "next_wake_at_ms": next_wake,
        })
    }
}

#[cfg(test)]
mod tests;
