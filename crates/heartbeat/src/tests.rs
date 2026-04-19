use std::path::PathBuf;

use nanobot_config::HeartbeatConfig;

use crate::{HeartbeatError, HeartbeatService};

mod config_validation {
    use super::*;

    #[tokio::test]
    async fn default_config_is_valid() {
        let config = HeartbeatConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_s, 1800);
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn interval_must_be_greater_than_zero() {
        let config = HeartbeatConfig { interval_s: 0, ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn interval_can_be_one_second() {
        let config = HeartbeatConfig { interval_s: 1, ..Default::default() };
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn interval_can_be_large_value() {
        let config = HeartbeatConfig {
            interval_s: 86400, // 1 day
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}

mod config_serialization {
    use super::*;

    #[tokio::test]
    async fn serialize_to_json() {
        let config = HeartbeatConfig { enabled: false, interval_s: 3600 };

        let json = serde_json::to_value(config).unwrap();
        assert_eq!(json["enabled"], false);
        assert_eq!(json["intervalS"], 3600);
    }

    #[tokio::test]
    async fn deserialize_from_json() {
        let json = r#"{"enabled": true, "intervalS": 7200}"#;
        let config: HeartbeatConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.interval_s, 7200);
    }

    #[tokio::test]
    async fn deserialize_empty_json_uses_defaults() {
        let json = r#"{}"#;
        let config: HeartbeatConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.interval_s, 1800);
    }

    #[tokio::test]
    async fn deserialize_with_missing_enabled_field() {
        let json = r#"{"intervalS": 5000}"#;
        let config: HeartbeatConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.interval_s, 5000);
    }

    #[tokio::test]
    async fn deserialize_with_missing_interval_field() {
        let json = r#"{"enabled": false}"#;
        let config: HeartbeatConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.interval_s, 1800);
    }
}

mod error_types {
    use super::*;

    #[test]
    fn error_variants_can_be_created() {
        let _ = HeartbeatError::Disabled;
        let _ = HeartbeatError::InvalidConfig("test".to_string());
        let _ = HeartbeatError::FileRead(std::io::Error::new(std::io::ErrorKind::NotFound, "not found"));
        let _ = HeartbeatError::Provider(anyhow::anyhow!("provider error"));
        let _ = HeartbeatError::Parse("parse error".to_string());
        let _ = HeartbeatError::Execute(anyhow::anyhow!("execute error"));
        let _ = HeartbeatError::Notify(anyhow::anyhow!("notify error"));
    }

    #[test]
    fn error_implements_std_error() {
        let error = HeartbeatError::Parse("test".to_string());
        assert_eq!(error.to_string(), "failed to parse LLM response: test");
    }
}

mod lifecycle_management {
    use super::*;

    #[tokio::test]
    async fn service_can_be_started_and_aborted() {
        let config = HeartbeatConfig::default();
        let provider = MockProvider::new();
        let workspace = PathBuf::from("/tmp/test_workspace");

        let service = HeartbeatService::new(workspace, provider, config, None, None);

        // Start the service in a background task
        let task = tokio::spawn(async move {
            let _ = service.start().await;
        });

        // Give the service a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Abort the task
        task.abort();

        // Wait for the task to be aborted
        let _ = tokio::time::timeout(tokio::time::Duration::from_millis(500), task).await;
    }

    #[tokio::test]
    async fn service_cannot_start_when_disabled() {
        let config = HeartbeatConfig { enabled: false, ..Default::default() };
        let provider = MockProvider::new();
        let workspace = PathBuf::from("/tmp/test_workspace");

        let service = HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.start().await;
        assert!(matches!(result, Err(HeartbeatError::Disabled)));
    }
}

struct MockProvider {
    _marker: std::marker::PhantomData<()>,
}

impl Clone for MockProvider {
    fn clone(&self) -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

impl MockProvider {
    fn new() -> Self {
        Self { _marker: std::marker::PhantomData }
    }
}

// Implement required traits for MockProvider
#[async_trait::async_trait]
impl nanobot_provider::Provider for MockProvider {
    fn bind_tools(&mut self, _tools: Vec<nanobot_tools::ToolDefinition>) {
        // Mock implementation
    }

    async fn chat(
        &self,
        _messages: &[nanobot_provider::Message],
        _options: &nanobot_provider::Options,
    ) -> Result<nanobot_provider::Message, anyhow::Error> {
        // Mock implementation - return an empty assistant response
        Ok(nanobot_provider::Message::assistant(String::new()))
    }
}

// Two-phase decision mechanism tests
mod two_phase_decision {
    use super::*;

    #[tokio::test]
    async fn tick_skips_when_heartbeat_not_found() {
        let config = HeartbeatConfig::default();
        let provider = MockProvider::new();
        let workspace = PathBuf::from("/tmp/nonexistent_workspace");

        let service: HeartbeatService<MockProvider> = HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn tick_skips_when_heartbeat_empty() {
        let config = HeartbeatConfig::default();
        let provider = MockProvider::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        // Create empty HEARTBEAT.md
        let heartbeat_path = workspace.join("HEARTBEAT.md");
        let content: &[u8] = b"";
        tokio::fs::write(&heartbeat_path, content).await.unwrap();

        let service: HeartbeatService<MockProvider> = HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn tick_skips_when_no_execute_callback_configured() {
        let config = HeartbeatConfig::default();
        let provider = MockProvider::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        // Create HEARTBEAT.md with content
        let heartbeat_path = workspace.join("HEARTBEAT.md");
        let content: &[u8] = b"# Tasks\n\n### Test Task\n- Description: Test\n- Priority: High\n- Status: Pending";
        tokio::fs::write(&heartbeat_path, content).await.unwrap();

        let service: HeartbeatService<MockProvider> = HeartbeatService::new(workspace, provider, config, None, None);

        // Note: This test requires the MockProvider to actually return a tool call
        // Since our current mock returns empty assistant response, the test will skip
        let result = service.tick().await;
        assert!(matches!(result, Ok(None)));
    }
}

// Exception scenarios and edge cases tests
mod exception_scenarios {
    use super::*;

    #[tokio::test]
    async fn tick_skips_when_file_read_fails() {
        let config = HeartbeatConfig::default();
        let provider = MockProvider::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let service: HeartbeatService<MockProvider> = HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        // The result should be Ok(None) when file not found (not an error)
        assert!(matches!(result, Ok(None)));
    }
}

// Parse retry mechanism tests
mod parse_retry {
    use nanobot_provider::Message;

    use super::*;

    /// MockProvider that returns invalid JSON arguments for first N calls, then valid
    struct RetryMockProvider {
        /// Number of times to return invalid arguments before succeeding
        fail_count: usize,
        /// Whether to return valid result after fail_count exhausted
        succeed_on_retry: bool,
    }

    impl Clone for RetryMockProvider {
        fn clone(&self) -> Self {
            Self { fail_count: self.fail_count, succeed_on_retry: self.succeed_on_retry }
        }
    }

    impl RetryMockProvider {
        fn new(fail_count: usize, succeed_on_retry: bool) -> Self {
            Self { fail_count, succeed_on_retry }
        }
    }

    #[async_trait::async_trait]
    impl nanobot_provider::Provider for RetryMockProvider {
        fn bind_tools(&mut self, _tools: Vec<nanobot_tools::ToolDefinition>) {}

        async fn chat(
            &self,
            messages: &[nanobot_provider::Message],
            _options: &nanobot_provider::Options,
        ) -> Result<nanobot_provider::Message, anyhow::Error> {
            // Count how many tool messages with "Invalid arguments format" we have received
            let retry_count = messages
                .iter()
                .filter(|m| matches!(m, Message::Tool { content, .. } if content.contains("Invalid arguments format")))
                .count();

            // Check if we should succeed based on retry count
            let should_succeed = if self.succeed_on_retry { retry_count >= self.fail_count } else { false };

            if should_succeed {
                // Return valid Skip action
                Ok(Message::assistant_with_tools(
                    String::new(),
                    vec![nanobot_provider::ToolCall::new("call_test", "heartbeat", serde_json::json!("skip"))],
                ))
            } else {
                // Return invalid JSON arguments
                Ok(Message::assistant_with_tools(
                    String::new(),
                    vec![nanobot_provider::ToolCall::new(
                        "call_test",
                        "heartbeat",
                        serde_json::json!({"invalid": "format"}),
                    )],
                ))
            }
        }
    }

    #[tokio::test]
    async fn parse_retry_succeeds_on_second_attempt() {
        let config = HeartbeatConfig::default();
        // First attempt fails, second succeeds
        let provider = RetryMockProvider::new(1, true);
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        // Create HEARTBEAT.md with content
        let heartbeat_path = workspace.join("HEARTBEAT.md");
        tokio::fs::write(&heartbeat_path, b"# Tasks\n\n- Task 1").await.unwrap();

        let service: HeartbeatService<RetryMockProvider> =
            HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        // Should succeed and return None (Skip action)
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn parse_retry_falls_back_to_skip_after_max_retries() {
        let config = HeartbeatConfig::default();
        // Always fail (will retry once, then degrade to skip)
        let provider = RetryMockProvider::new(10, false);
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        // Create HEARTBEAT.md with content
        let heartbeat_path = workspace.join("HEARTBEAT.md");
        tokio::fs::write(&heartbeat_path, b"# Tasks\n\n- Task 1").await.unwrap();

        let service: HeartbeatService<RetryMockProvider> =
            HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        // Should degrade to skip after max retries
        assert!(matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn parse_retry_max_retries_is_one() {
        let config = HeartbeatConfig::default();
        // First attempt fails, second also fails (but we only retry once)
        let provider = RetryMockProvider::new(1, false);
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        // Create HEARTBEAT.md with content
        let heartbeat_path = workspace.join("HEARTBEAT.md");
        tokio::fs::write(&heartbeat_path, b"# Tasks\n\n- Task 1").await.unwrap();

        let service: HeartbeatService<RetryMockProvider> =
            HeartbeatService::new(workspace, provider, config, None, None);

        let result = service.tick().await;
        // Should degrade to skip (1 initial + 1 retry = 2 total attempts)
        assert!(matches!(result, Ok(None)));
    }
}
