use nanobot_provider::{Message, Options, ToolCall};

use super::*;

/// Mock provider that returns a pre-configured response.
#[derive(Clone)]
enum MockResponse {
    Ok(Message),
    Err(String),
}

#[derive(Clone)]
struct MockProvider {
    response: MockResponse,
}

impl MockProvider {
    fn with_tool_call(should_notify: bool, reason: &str) -> Self {
        let args = serde_json::json!({
            "should_notify": should_notify,
            "reason": reason,
        });
        let tc = ToolCall::new("call_1", "evaluate_notification", args);
        Self { response: MockResponse::Ok(Message::assistant_with_tools("", vec![tc])) }
    }

    fn with_error() -> Self {
        Self { response: MockResponse::Err("mock error".to_string()) }
    }

    fn with_no_tool_call() -> Self {
        Self { response: MockResponse::Ok(Message::assistant("Everything looks fine.")) }
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message], _options: &Options) -> anyhow::Result<Message> {
        match &self.response {
            MockResponse::Ok(msg) => Ok(msg.clone()),
            MockResponse::Err(e) => Err(anyhow::anyhow!("{e}")),
        }
    }

    fn bind_tools(&mut self, _tools: Vec<nanobot_tools::ToolDefinition>) {}
}

#[tokio::test]
async fn should_notify_true() {
    let provider = MockProvider::with_tool_call(true, "contains actionable info");
    assert!(evaluate_response(&provider, "Task completed with errors", "Check system health").await);
}

#[tokio::test]
async fn should_notify_false() {
    let provider = MockProvider::with_tool_call(false, "routine check, nothing new");
    assert!(!evaluate_response(&provider, "All systems normal", "Check system health").await);
}

#[tokio::test]
async fn fallback_on_provider_error() {
    let provider = MockProvider::with_error();
    assert!(evaluate_response(&provider, "response", "task").await);
}

#[tokio::test]
async fn fallback_on_no_tool_call() {
    let provider = MockProvider::with_no_tool_call();
    assert!(evaluate_response(&provider, "response", "task").await);
}
