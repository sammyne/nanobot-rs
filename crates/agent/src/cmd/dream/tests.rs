use super::*;
use crate::InboundMessage;
use crate::cmd::Command;

#[tokio::test]
async fn dream_cmd_returns_result() {
    let cmd = DreamCmd { result: "Dream completed: processed 5 entries, 2 files changed (MEMORY.md, USER.md)".into() };
    let msg = InboundMessage::new("test", "user", "chat", "/dream");
    let result = cmd.run(msg, "test:chat".into()).await.unwrap();
    assert!(result.contains("processed 5 entries"));
    assert!(result.contains("MEMORY.md"));
}

#[tokio::test]
async fn dream_log_cmd_returns_output() {
    let log = "Memory change history (last 10):\n\nabc1234 | 2026-01-15 | dream: process 5 entries";
    let cmd = DreamLogCmd { log_output: log.into() };
    let msg = InboundMessage::new("test", "user", "chat", "/dream-log");
    let result = cmd.run(msg, "test:chat".into()).await.unwrap();
    assert!(result.contains("abc1234"));
    assert!(result.contains("Memory change history"));
}

#[tokio::test]
async fn dream_restore_cmd_returns_output() {
    let cmd = DreamRestoreCmd { restore_output: "Memory restored to commit abc1234.".into() };
    let msg = InboundMessage::new("test", "user", "chat", "/dream-restore abc1234");
    let result = cmd.run(msg, "test:chat".into()).await.unwrap();
    assert!(result.contains("restored to commit abc1234"));
}
