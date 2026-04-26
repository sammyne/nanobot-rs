//! Cron tool module for scheduling reminders and tasks.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use nanobot_tools::{Tool, ToolContext, ToolError, ToolResult};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

use crate::scheduler::is_valid_timezone;
use crate::service::CronService;
use crate::types::CronSchedule;

/// Lazy-initialized global schema for CronArgs
static CRON_PARAMETERS: LazyLock<Schema> = LazyLock::new(|| schemars::schema_for!(CronArgs));

/// Schedule definition for adding a cron job
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CronScheduleArgs {
    /// Interval-based scheduling: runs every N seconds after the job starts.
    ///
    /// This is NOT time-based. For "daily at 8 AM", use `Cron` with expr `"0 8 * * *"`.
    Every {
        /// Number of seconds between each execution.
        /// The interval is measured from when the job starts, not from a fixed clock time.
        every_seconds: u64,
    },
    /// Time-based scheduling using a cron expression.
    Cron {
        /// Cron expression in 6-field format (sec min hour day month weekday).
        ///
        /// Examples:
        /// - `"0 8 * * *"` — daily at 8:00 AM
        /// - `"0 9 * * 1-5"` — every weekday at 9:00 AM
        /// - `"0 */2 * * *"` — every 2 hours
        expr: String,
        /// IANA timezone name (e.g., "Asia/Shanghai", "America/Vancouver"). Uses UTC if omitted.
        #[serde(skip_serializing_if = "Option::is_none")]
        tz: Option<String>,
    },
    /// One-time execution at a specific datetime.
    At {
        /// ISO 8601 datetime string (e.g., "2024-01-01T09:00:00+08:00").
        at: String,
    },
}

/// Cron tool arguments for different actions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum CronArgs {
    /// Add a new scheduled job
    Add {
        /// Reminder message to display
        message: String,
        /// Schedule definition.
        ///
        /// Use `cron` for time-based scheduling (e.g., "daily at 8 AM").
        /// Use `every` only for interval-based scheduling (e.g., "every 60 seconds").
        schedule: CronScheduleArgs,
    },
    /// List all scheduled jobs
    List,
    /// Remove a scheduled job by ID
    Remove {
        /// Job ID to remove
        job_id: String,
    },
}

/// Tool to schedule reminders and recurring tasks.
pub struct CronTool {
    service: Arc<CronService>,
}

impl CronTool {
    /// Create a new CronTool instance.
    pub fn new(service: Arc<CronService>) -> Self {
        CronTool { service }
    }

    /// Handle add action
    async fn handle_add(
        &self,
        message: String,
        schedule: CronScheduleArgs,
        channel: &str,
        chat_id: &str,
    ) -> ToolResult {
        if channel.is_empty() || chat_id.is_empty() {
            return Err(ToolError::validation("context", "no session context (channel/chat_id)"));
        }

        // Build schedule
        let (schedule, delete_after_run) = match schedule {
            CronScheduleArgs::Every { every_seconds } => {
                if every_seconds == 0 {
                    return Err(ToolError::validation("every_seconds", "must be positive"));
                }
                (CronSchedule::Every { every_ms: every_seconds * 1000 }, false)
            }
            CronScheduleArgs::Cron { expr: cron_expr, tz } => {
                // Validate timezone if provided
                if let Some(tz_name) = &tz
                    && !is_valid_timezone(tz_name)
                {
                    return Err(ToolError::validation("tz", format!("unknown timezone '{tz_name}'")));
                }
                (CronSchedule::Cron { expr: cron_expr, tz }, false)
            }
            CronScheduleArgs::At { at } => {
                let dt = chrono::DateTime::parse_from_rfc3339(&at)
                    .map_err(|e| ToolError::validation("at", format!("invalid datetime format: {e}")))?;
                let at_ms = dt.timestamp_millis();
                (CronSchedule::At { at_ms }, true)
            }
        };

        // Add job
        let job = self
            .service
            .add_job(
                message.chars().take(30).collect(),
                schedule,
                message.clone(),
                true,
                Some(channel.to_string()),
                Some(chat_id.to_string()),
                delete_after_run,
            )
            .await
            .map_err(ToolError::execution)?;

        Ok(format!("Created job '{}' (id: {})", job.name, job.id))
    }

    /// Handle list action
    async fn handle_list(&self) -> ToolResult {
        let jobs = self.service.list_jobs(false).await;

        if jobs.is_empty() {
            return Ok("No scheduled jobs.".to_string());
        }

        let lines: Vec<String> = jobs
            .iter()
            .map(|j| {
                let kind = match &j.schedule {
                    CronSchedule::At { .. } => "at",
                    CronSchedule::Every { .. } => "every",
                    CronSchedule::Cron { .. } => "cron",
                };
                format!("- {} (id: {}, {})", j.name, j.id, kind)
            })
            .collect();

        Ok(format!("Scheduled jobs:\n{}", lines.join("\n")))
    }

    /// Handle remove action
    async fn handle_remove(&self, job_id: String) -> ToolResult {
        if self.service.remove_job(&job_id).await {
            Ok(format!("Removed job {job_id}"))
        } else {
            Ok(format!("Job {job_id} not found"))
        }
    }
}

#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }

    fn description(&self) -> &str {
        "Schedule reminders and recurring tasks. Actions:\n\
        - add: Create a new scheduled job. Use when user asks to 'remind me', 'schedule', 'add a task', or 'set up a recurring task'.\n\
        - list: Show all scheduled jobs. Use when user asks to 'list tasks', 'show my reminders', or 'what tasks do I have'.\n\
        - remove: Delete a job by ID. Use when user asks to 'delete', 'cancel', or 'remove' a scheduled task."
    }

    fn parameters(&self) -> Schema {
        CRON_PARAMETERS.clone()
    }

    async fn execute(&self, ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let args: CronArgs =
            serde_json::from_value(params).map_err(|e| ToolError::validation("params", e.to_string()))?;

        match args {
            CronArgs::Add { message, schedule } => self.handle_add(message, schedule, &ctx.channel, &ctx.chat_id).await,
            CronArgs::List => self.handle_list().await,
            CronArgs::Remove { job_id } => self.handle_remove(job_id).await,
        }
    }
}

#[cfg(test)]
mod tests;
