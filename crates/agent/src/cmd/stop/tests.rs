use std::path::PathBuf;

use super::*;
use crate::InboundMessage;
use crate::cmd::Command;

/// Mock Provider for stop command tests
#[derive(Clone)]
struct MockProvider;

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn chat(
        &self,
        _messages: &[nanobot_provider::Message],
        _options: &nanobot_provider::Options,
    ) -> anyhow::Result<nanobot_provider::MeteredMessage> {
        Ok(nanobot_provider::Message::assistant("ok").into())
    }

    fn bind_tools(&mut self, _tools: Vec<nanobot_tools::ToolDefinition>) {}
}

/// 验证无运行中任务时返回 "Stopped."
#[tokio::test]
async fn stop_with_no_running_tasks() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let manager = SubagentManager::new(MockProvider, PathBuf::from("/tmp/test"), tx, 0.7, 4096);
    let cmd = StopCmd::new(manager);
    let msg = InboundMessage::new("cli", "user", "test", "/stop");
    let result = cmd.run(msg, "cli:test".to_string()).await;

    assert_eq!(result, Ok("Stopped.".to_string()));
}
