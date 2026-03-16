//! AgentLoop 模块测试
//!
//! 按照 AGENTS.md 规范编写的表驱动测试

use std::path::PathBuf;

use async_trait::async_trait;
use nanobot_config::AgentDefaults;
use nanobot_provider::{Message, Provider};
use nanobot_subagent::SubagentManager;
use nanobot_tools::ToolDefinition;

use super::*;

/// Mock Provider 用于测试
#[derive(Clone)]
struct MockProvider {
    /// 预设响应
    response: String,
    /// 绑定的工具列表
    bound_tools: Vec<ToolDefinition>,
}

impl MockProvider {
    fn new(response: impl Into<String>) -> Self {
        Self { response: response.into(), bound_tools: Vec::new() }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message], _options: &nanobot_provider::Options) -> anyhow::Result<Message> {
        Ok(Message::assistant(&self.response))
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        self.bound_tools = tools;
    }
}

/// 创建测试用 AgentDefaults
fn mock_config() -> AgentDefaults {
    AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 50,
    }
}

/// 创建测试用 SubagentManager
fn mock_subagent_manager<P: Provider + Clone + Send + Sync + 'static>(
    provider: P,
) -> std::sync::Arc<SubagentManager<P>> {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    SubagentManager::new(provider.clone(), PathBuf::from("/tmp/test"), tx, 0.5, 1024)
}

/// process_direct 测试用例结构
struct ProcessDirectCase {
    name: &'static str,
    input: &'static str,
    session_id: Option<&'static str>,
    expected_response: &'static str,
}

/// 验证 process_direct 方法返回预期的 Provider 响应
#[tokio::test]
async fn process_direct_returns_expected_response() {
    let test_vector = [
        ProcessDirectCase {
            name: "标准消息处理",
            input: "Hello",
            session_id: None,
            expected_response: "Hello, I am a test response",
        },
        ProcessDirectCase {
            name: "带 session_id 的消息处理",
            input: "Hello",
            session_id: Some("cli:direct"),
            expected_response: "Hello, I am a test response",
        },
    ];

    for case in test_vector {
        let provider = MockProvider::new(case.expected_response);
        let config = mock_config();
        let subagent_manager = mock_subagent_manager(provider.clone());
        let agent = AgentLoop::new(provider, config, None, Some(subagent_manager), std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed");

        let session_key = case.session_id.unwrap_or("cli:direct");
        let result = agent
            .process_direct(case.input, session_key, None, None)
            .await
            .unwrap_or_else(|e| panic!("case[{}]: process_direct failed: {}", case.name, e));

        assert_eq!(result, case.expected_response, "case[{}]: response mismatch", case.name);
    }
}

/// 验证 process_direct 能正确处理空消息输入
#[tokio::test]
async fn process_direct_handles_empty_message() {
    let provider = MockProvider::new("OK");
    let config = mock_config();
    let subagent_manager = mock_subagent_manager(provider.clone());
    let agent = AgentLoop::new(provider, config, None, Some(subagent_manager), std::collections::HashMap::new())
        .await
        .expect("AgentLoop creation should succeed");

    let result = agent.process_direct("", "cli:direct", None, None).await.expect("empty message should be handled");

    assert_eq!(result, "OK");
}

/// 验证 config 方法返回正确的配置引用
#[tokio::test]
async fn config_returns_correct_reference() {
    let provider = MockProvider::new("test");
    let config = mock_config();
    let subagent_manager = mock_subagent_manager(provider.clone());
    let agent =
        AgentLoop::new(provider, config.clone(), None, Some(subagent_manager), std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed");

    let returned_config = agent.config();

    assert_eq!(returned_config.model, config.model, "model should match");
    assert_eq!(returned_config.max_tool_iterations, config.max_tool_iterations, "max_tool_iterations should match");
    assert_eq!(returned_config.max_tokens, config.max_tokens, "max_tokens should match");
    assert_eq!(returned_config.temperature, config.temperature, "temperature should match");
}

/// AgentDefaults 构造测试用例结构
struct DefaultsCase {
    name: &'static str,
    agent: AgentLoop<MockProvider>,
    expect_model: &'static str,
    expect_max_tokens: usize,
}

/// 验证 AgentLoop 能正确存储和使用不同的配置值
#[tokio::test]
async fn agent_loop_uses_custom_config_values() {
    let custom_defaults1 = AgentDefaults {
        workspace: PathBuf::from("/tmp/test1"),
        model: "custom-model-1".to_string(),
        max_tokens: 2048,
        temperature: 0.7,
        max_tool_iterations: 20,
        memory_window: 100,
    };

    let custom_defaults2 = AgentDefaults {
        workspace: PathBuf::from("/tmp/test2"),
        model: "custom-model-2".to_string(),
        max_tokens: 4096,
        temperature: 0.3,
        max_tool_iterations: 5,
        memory_window: 25,
    };

    let provider1 = MockProvider::new("test");
    let subagent_manager1 = mock_subagent_manager(provider1.clone());
    let provider2 = MockProvider::new("test");
    let subagent_manager2 = mock_subagent_manager(provider2.clone());

    let test_vector = [
        DefaultsCase {
            name: "自定义配置 1",
            agent: AgentLoop::new(
                provider1,
                custom_defaults1,
                None,
                Some(subagent_manager1),
                std::collections::HashMap::new(),
            )
            .await
            .expect("AgentLoop creation should succeed"),
            expect_model: "custom-model-1",
            expect_max_tokens: 2048,
        },
        DefaultsCase {
            name: "自定义配置 2",
            agent: AgentLoop::new(
                provider2,
                custom_defaults2,
                None,
                Some(subagent_manager2),
                std::collections::HashMap::new(),
            )
            .await
            .expect("AgentLoop creation should succeed"),
            expect_model: "custom-model-2",
            expect_max_tokens: 4096,
        },
    ];

    for case in test_vector {
        let cfg = case.agent.config();
        assert_eq!(cfg.model, case.expect_model, "case[{}]: model mismatch", case.name);
        assert_eq!(cfg.max_tokens, case.expect_max_tokens, "case[{}]: max_tokens mismatch", case.name);
    }
}

/// 系统消息路由测试用例结构
struct SystemMessageCase {
    name: &'static str,
    chat_id: &'static str,
    expected_response: &'static str,
    expect_channel: &'static str,
    expect_chat_id: &'static str,
}

/// 验证 channel=system 的消息能被正确路由到目标通道
#[tokio::test]
async fn process_message_routes_system_message_correctly() {
    let test_vector = [
        SystemMessageCase {
            name: "系统消息 - 标准格式 telegram:12345",
            chat_id: "telegram:12345",
            expected_response: "System message processed",
            expect_channel: "telegram",
            expect_chat_id: "12345",
        },
        SystemMessageCase {
            name: "系统消息 - 无分隔符使用默认 cli",
            chat_id: "simple_id",
            expected_response: "System message processed",
            expect_channel: "cli",
            expect_chat_id: "simple_id",
        },
        SystemMessageCase {
            name: "系统消息 - 多个冒号只分割第一个",
            chat_id: "wechat:group:456",
            expected_response: "System message processed",
            expect_channel: "wechat",
            expect_chat_id: "group:456",
        },
    ];

    for case in test_vector {
        let provider = MockProvider::new(case.expected_response);
        let config = mock_config();
        let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed");

        // 构造系统消息
        let inbound = InboundMessage::new("system", "scheduler", case.chat_id, "Test content");

        // 调用 process_message
        let outbound = agent.process_message(inbound, None).await;

        // 验证路由信息
        assert_eq!(outbound.channel, case.expect_channel, "case[{}]: channel mismatch", case.name);
        assert_eq!(outbound.chat_id, case.expect_chat_id, "case[{}]: chat_id mismatch", case.name);
        assert_eq!(outbound.content, case.expected_response, "case[{}]: content mismatch", case.name);
    }
}

/// 验证非系统消息保持原有路由信息不变
#[tokio::test]
async fn process_message_preserves_non_system_routing() {
    let provider = MockProvider::new("Normal message response");
    let config = mock_config();
    let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
        .await
        .expect("AgentLoop creation should succeed");

    // 构造普通消息
    let inbound = InboundMessage::new("cli", "user", "chat123", "Hello");

    // 调用 process_message
    let outbound = agent.process_message(inbound, None).await;

    // 验证路由信息保持不变
    assert_eq!(outbound.channel, "cli", "channel should be unchanged");
    assert_eq!(outbound.chat_id, "chat123", "chat_id should be unchanged");
}
