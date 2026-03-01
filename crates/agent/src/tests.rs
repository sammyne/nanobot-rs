//! Agent 库测试

use super::*;
use async_trait::async_trait;
use nanobot_config::AgentDefaults;
use nanobot_provider::{Message, Provider};

/// Mock Provider 用于测试
struct MockProvider {
    /// 预设响应
    response: String,
}

impl MockProvider {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message]) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }
}

/// 创建测试用 AgentDefaults
fn test_config() -> AgentDefaults {
    AgentDefaults {
        workspace: "/tmp/test".to_string(),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 50,
    }
}

#[tokio::test]
async fn process_direct_returns_response() {
    let expected = "Hello, I am a test response";
    let provider = std::sync::Arc::new(MockProvider::new(expected));
    let config = test_config();
    let agent = AgentLoop::new(provider, config);

    let result = agent.process_direct("Hello").await.unwrap();

    assert_eq!(result, expected);
}

#[tokio::test]
async fn process_direct_empty_message() {
    let provider = std::sync::Arc::new(MockProvider::new("OK"));
    let config = test_config();
    let agent = AgentLoop::new(provider, config);

    let result = agent.process_direct("").await.unwrap();

    assert_eq!(result, "OK");
}

#[test]
fn config_returns_reference() {
    let provider = std::sync::Arc::new(MockProvider::new("test"));
    let config = test_config();
    let agent = AgentLoop::new(provider, config.clone());

    let returned_config = agent.config();

    assert_eq!(returned_config.model, config.model);
    assert_eq!(returned_config.max_tool_iterations, config.max_tool_iterations);
}
