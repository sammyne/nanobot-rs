use std::time::Instant;

use nanobot_provider::Usage;

use super::*;
use crate::InboundMessage;
use crate::cmd::Command;

#[tokio::test]
async fn status_contains_version() {
    let cmd = StatusCmd {
        model: "test-model".to_string(),
        start_time: Instant::now(),
        last_usage: None,
        session_message_count: 0,
    };
    let msg = InboundMessage::new("test", "user", "chat", "/status");
    let result = cmd.run(msg, "test:chat".to_string()).await.unwrap();
    assert!(result.contains("nanobot v"));
}

#[tokio::test]
async fn status_contains_model() {
    let cmd = StatusCmd {
        model: "claude-sonnet-4-6".to_string(),
        start_time: Instant::now(),
        last_usage: None,
        session_message_count: 5,
    };
    let msg = InboundMessage::new("test", "user", "chat", "/status");
    let result = cmd.run(msg, "test:chat".to_string()).await.unwrap();
    assert!(result.contains("claude-sonnet-4-6"));
    assert!(result.contains("5"));
}

#[tokio::test]
async fn status_with_usage() {
    let cmd = StatusCmd {
        model: "test".to_string(),
        start_time: Instant::now(),
        last_usage: Some(Usage { input_tokens: 1234, output_tokens: 567 }),
        session_message_count: 0,
    };
    let msg = InboundMessage::new("test", "user", "chat", "/status");
    let result = cmd.run(msg, "test:chat".to_string()).await.unwrap();
    assert!(result.contains("1234"));
    assert!(result.contains("567"));
}

#[tokio::test]
async fn status_without_usage() {
    let cmd =
        StatusCmd { model: "test".to_string(), start_time: Instant::now(), last_usage: None, session_message_count: 0 };
    let msg = InboundMessage::new("test", "user", "chat", "/status");
    let result = cmd.run(msg, "test:chat".to_string()).await.unwrap();
    assert!(result.contains("N/A"));
}

#[test]
fn uptime_format() {
    assert_eq!(super::format_uptime(0), "0s");
    assert_eq!(super::format_uptime(59), "59s");
    assert_eq!(super::format_uptime(60), "1m 0s");
    assert_eq!(super::format_uptime(3661), "1h 1m 1s");
    assert_eq!(super::format_uptime(90061), "1d 1h 1m 1s");
}
