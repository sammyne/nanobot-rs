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
    ) -> anyhow::Result<nanobot_provider::Message> {
        Ok(nanobot_provider::Message::assistant("ok"))
    }

    fn bind_tools(&mut self, _tools: Vec<nanobot_tools::ToolDefinition>) {}
}

/// 验证无 subagent_manager 时返回 "Stopped."
#[tokio::test]
async fn stop_without_manager() {
    let cmd = StopCmd::<MockProvider>::new(None);
    let msg = InboundMessage::new("cli", "user", "test", "/stop");
    let result = cmd.run(msg, "cli:test".to_string()).await;

    assert_eq!(result, Ok("Stopped.".to_string()));
}
