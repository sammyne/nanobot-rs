//! Cron type definitions.

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
        every_ms: i64,
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
        CronPayload {
            kind: default_payload_kind(),
            message: String::new(),
            deliver: false,
            channel: None,
            to: None,
        }
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
        CronStore {
            version: default_version(),
            jobs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
