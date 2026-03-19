//! Cron type definitions.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Schedule definition for a cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CronSchedule {
    /// One-time execution at a specific timestamp
    At {
        /// Timestamp in milliseconds
        at_ms: i64,
    },
    /// Recurring execution at fixed intervals
    Every {
        /// Interval in milliseconds
        every_ms: u64,
    },
    /// Cron expression based scheduling
    Cron {
        /// Cron expression (e.g. "0 9 * * *")
        expr: String,
        /// IANA timezone for cron expressions (e.g. "America/Vancouver")
        #[serde(skip_serializing_if = "Option::is_none")]
        tz: Option<String>,
    },
}

impl Default for CronSchedule {
    fn default() -> Self {
        CronSchedule::Every { every_ms: 60000 } // Default to 1 minute
    }
}

impl CronSchedule {
    /// Compute the next run time in milliseconds.
    pub fn compute_next_run(&self, now_ms: i64) -> Option<i64> {
        match self {
            CronSchedule::At { at_ms } => {
                // One-time execution
                if *at_ms > now_ms { Some(*at_ms) } else { None }
            }
            CronSchedule::Every { every_ms } => {
                // Recurring execution
                if *every_ms == 0 {
                    return None;
                }
                Some(now_ms + (*every_ms as i64))
            }
            CronSchedule::Cron { expr, tz } => {
                // Cron expression based scheduling - delegate to scheduler module
                crate::scheduler::compute_cron_next_run(expr, tz.as_ref(), now_ms)
            }
        }
    }

    /// Validate a schedule for job addition.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            CronSchedule::Cron { expr, tz } => {
                // Validate cron expression
                if cron::Schedule::from_str(expr).is_err() {
                    return Err(format!("Invalid cron expression: {expr}"));
                }

                // Validate timezone if provided
                if let Some(tz_name) = tz
                    && tz_name.parse::<chrono_tz::Tz>().is_err()
                {
                    return Err(format!("Unknown timezone: {tz_name}"));
                }
            }
            CronSchedule::At { at_ms } => {
                if *at_ms <= 0 {
                    return Err("at_ms must be positive".to_string());
                }
            }
            CronSchedule::Every { every_ms } => {
                if *every_ms == 0 {
                    return Err("every_ms must be positive".to_string());
                }
            }
        }

        Ok(())
    }
}

/// What to do when the job runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronPayload {
    /// Kind of payload
    #[serde(default = "default_payload_kind")]
    pub kind: String,
    /// Message content
    #[serde(default)]
    pub message: String,
    /// Whether to deliver response to channel
    #[serde(default)]
    pub deliver: bool,
    /// Channel to deliver to (e.g. "whatsapp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Recipient identifier (e.g. phone number)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

fn default_payload_kind() -> String {
    "agent_turn".to_string()
}

impl Default for CronPayload {
    fn default() -> Self {
        CronPayload { kind: default_payload_kind(), message: String::new(), deliver: false, channel: None, to: None }
    }
}

/// Runtime state of a job.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CronJobState {
    /// Next execution timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run_at_ms: Option<i64>,
    /// Last execution timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at_ms: Option<i64>,
    /// Last execution status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_status: Option<String>,
    /// Last error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// A scheduled job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    /// Unique job identifier
    pub id: String,
    /// Job name (truncated message)
    pub name: String,
    /// Whether the job is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Schedule definition
    #[serde(default)]
    pub schedule: CronSchedule,
    /// Payload to execute
    #[serde(default)]
    pub payload: CronPayload,
    /// Runtime state
    #[serde(default)]
    pub state: CronJobState,
    /// Creation timestamp in milliseconds
    #[serde(default)]
    pub created_at_ms: i64,
    /// Last update timestamp in milliseconds
    #[serde(default)]
    pub updated_at_ms: i64,
    /// Whether to delete after one-time execution
    #[serde(default)]
    pub delete_after_run: bool,
}

fn default_enabled() -> bool {
    true
}

impl CronJob {
    /// Create a new job with a generated UUID
    pub fn new(name: String, schedule: CronSchedule, payload: CronPayload, delete_after_run: bool) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        CronJob {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            enabled: true,
            schedule,
            payload,
            state: CronJobState::default(),
            created_at_ms: now,
            updated_at_ms: now,
            delete_after_run,
        }
    }
}

/// Persistent store for cron jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronStore {
    /// Store version
    #[serde(default = "default_version")]
    pub version: i32,
    /// List of jobs
    #[serde(default)]
    pub jobs: Vec<CronJob>,
}

fn default_version() -> i32 {
    1
}

impl Default for CronStore {
    fn default() -> Self {
        CronStore { version: default_version(), jobs: Vec::new() }
    }
}

#[cfg(test)]
mod tests;
