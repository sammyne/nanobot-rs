//! Cron tool module for scheduling reminders and tasks.

mod scheduler;
mod service;
mod storage;

use std::sync::Arc;

use async_trait::async_trait;
pub use scheduler::*;
use schemars::JsonSchema;
use schemars::schema::SchemaObject;
use serde::{Deserialize, Serialize};
pub use service::*;
pub use storage::*;

use crate::core::{Tool, ToolError, ToolResult};
use crate::cron_types::CronSchedule;

/// Cron tool arguments for add operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronAddArgs {
    /// Action to perform (required)
    pub action: String,
    /// Reminder message (for add)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Interval in seconds (for recurring tasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub every_seconds: Option<i64>,
    /// Cron expression like '0 9 * * *' (for scheduled tasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_expr: Option<String>,
    /// IANA timezone for cron expressions (e.g. 'America/Vancouver')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    /// ISO datetime for one-time execution (e.g. '2026-02-12T10:30:00')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// Job ID (for remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

/// Tool to schedule reminders and recurring tasks.
pub struct CronTool {
    service: Arc<CronService>,
    channel: Arc<std::sync::RwLock<String>>,
    chat_id: Arc<std::sync::RwLock<String>>,
}

impl CronTool {
    /// Create a new CronTool instance.
    pub fn new(service: Arc<CronService>) -> Self {
        CronTool {
            service,
            channel: Arc::new(std::sync::RwLock::new(String::new())),
            chat_id: Arc::new(std::sync::RwLock::new(String::new())),
        }
    }

    /// Set the current session context for delivery.
    pub fn set_context(&self, channel: String, chat_id: String) {
        let mut ch = self.channel.write().unwrap();
        *ch = channel;
        let mut cid = self.chat_id.write().unwrap();
        *cid = chat_id;
    }

    /// Get current channel
    fn get_channel(&self) -> String {
        self.channel.read().unwrap().clone()
    }

    /// Get current chat_id
    fn get_chat_id(&self) -> String {
        self.chat_id.read().unwrap().clone()
    }

    /// Handle add action
    async fn handle_add(&self, args: &CronAddArgs) -> ToolResult {
        let message = match &args.message {
            Some(m) if !m.is_empty() => m.clone(),
            _ => return Err(ToolError::validation("message", "message is required for add")),
        };

        let channel = self.get_channel();
        let chat_id = self.get_chat_id();

        if channel.is_empty() || chat_id.is_empty() {
            return Err(ToolError::validation("context", "no session context (channel/chat_id)"));
        }

        // Validate timezone usage
        if args.tz.is_some() && args.cron_expr.is_none() {
            return Err(ToolError::validation("tz", "tz can only be used with cron_expr"));
        }

        // Validate timezone if provided
        if let Some(ref tz) = args.tz
            && !is_valid_timezone(tz)
        {
            return Err(ToolError::validation("tz", format!("unknown timezone '{tz}'")));
        }

        // Build schedule
        let (schedule, delete_after_run) = if let Some(every_seconds) = args.every_seconds {
            (
                CronSchedule::Every {
                    every_ms: every_seconds * 1000,
                },
                false,
            )
        } else if let Some(ref cron_expr) = args.cron_expr {
            (
                CronSchedule::Cron {
                    expr: cron_expr.clone(),
                    tz: args.tz.clone(),
                },
                false,
            )
        } else if let Some(ref at) = args.at {
            // Parse ISO datetime
            let dt = chrono::DateTime::parse_from_rfc3339(at)
                .map_err(|e| ToolError::validation("at", format!("invalid datetime format: {e}")))?;
            let at_ms = dt.timestamp_millis();
            (CronSchedule::At { at_ms }, true)
        } else {
            return Err(ToolError::validation(
                "schedule",
                "either every_seconds, cron_expr, or at is required",
            ));
        };

        // Add job
        let job = self
            .service
            .add_job(
                message.chars().take(30).collect(),
                schedule,
                message,
                true,
                Some(channel),
                Some(chat_id),
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
    async fn handle_remove(&self, job_id: &str) -> ToolResult {
        if job_id.is_empty() {
            return Err(ToolError::validation("job_id", "job_id is required for remove"));
        }

        if self.service.remove_job(job_id).await {
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
        "Schedule reminders and recurring tasks. Actions: add, list, remove."
    }

    fn parameters(&self) -> SchemaObject {
        let root_schema = schemars::schema_for!(CronAddArgs);
        root_schema.schema
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let args: CronAddArgs =
            serde_json::from_value(params).map_err(|e| ToolError::validation("params", e.to_string()))?;

        match args.action.as_str() {
            "add" => self.handle_add(&args).await,
            "list" => self.handle_list().await,
            "remove" => {
                let job_id = args.job_id.unwrap_or_default();
                self.handle_remove(&job_id).await
            }
            _ => Err(ToolError::validation(
                "action",
                format!("Unknown action: {}", args.action),
            )),
        }
    }
}

#[cfg(test)]
mod tests;
