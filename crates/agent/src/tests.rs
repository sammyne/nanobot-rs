//! Agent 库测试
//!
//! 按照表驱动测试规范编写的单元测试

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
        Self {
            response: response.into(),
            bound_tools: Vec::new(),
        }
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message]) -> anyhow::Result<Message> {
        Ok(Message::assistant(&self.response))
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        self.bound_tools = tools;
    }
}

/// 创建测试用 AgentDefaults
fn test_config() -> AgentDefaults {
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
fn test_subagent_manager<P: Provider + Clone + Send + Sync + 'static>(
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
        let config = test_config();
        let subagent_manager = test_subagent_manager(provider.clone());
        let agent = AgentLoop::new_direct(provider, config, None, Some(subagent_manager));

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
    let config = test_config();
    let subagent_manager = test_subagent_manager(provider.clone());
    let agent = AgentLoop::new_direct(provider, config, None, Some(subagent_manager));

    let result = agent
        .process_direct("", "cli:direct", None, None)
        .await
        .expect("empty message should be handled");

    assert_eq!(result, "OK");
}

/// 验证 config 方法返回正确的配置引用
#[test]
fn config_returns_correct_reference() {
    let provider = MockProvider::new("test");
    let config = test_config();
    let subagent_manager = test_subagent_manager(provider.clone());
    let agent = AgentLoop::new_direct(provider, config.clone(), None, Some(subagent_manager));

    let returned_config = agent.config();

    assert_eq!(returned_config.model, config.model, "model should match");
    assert_eq!(
        returned_config.max_tool_iterations, config.max_tool_iterations,
        "max_tool_iterations should match"
    );
    assert_eq!(returned_config.max_tokens, config.max_tokens, "max_tokens should match");
    assert_eq!(
        returned_config.temperature, config.temperature,
        "temperature should match"
    );
}

/// AgentDefaults 构造测试用例结构
struct DefaultsCase {
    name: &'static str,
    agent: AgentLoop<MockProvider>,
    expect_model: &'static str,
    expect_max_tokens: usize,
}

/// 验证 AgentLoop 能正确存储和使用不同的配置值
#[test]
fn agent_loop_uses_custom_config_values() {
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
    let subagent_manager1 = test_subagent_manager(provider1.clone());
    let provider2 = MockProvider::new("test");
    let subagent_manager2 = test_subagent_manager(provider2.clone());

    let test_vector = [
        DefaultsCase {
            name: "自定义配置 1",
            agent: AgentLoop::new_direct(provider1, custom_defaults1, None, Some(subagent_manager1)),
            expect_model: "custom-model-1",
            expect_max_tokens: 2048,
        },
        DefaultsCase {
            name: "自定义配置 2",
            agent: AgentLoop::new_direct(provider2, custom_defaults2, None, Some(subagent_manager2)),
            expect_model: "custom-model-2",
            expect_max_tokens: 4096,
        },
    ];

    for case in test_vector {
        let cfg = case.agent.config();
        assert_eq!(cfg.model, case.expect_model, "case[{}]: model mismatch", case.name);
        assert_eq!(
            cfg.max_tokens, case.expect_max_tokens,
            "case[{}]: max_tokens mismatch",
            case.name
        );
    }
}

/// 从顶级模块引入测试类型
use crate::InboundMessage;
/// 从顶级模块引入测试类型
use crate::OutboundMessage;

/// InboundMessage::new 构造测试
#[test]
fn inbound_message_construct() {
    let msg = InboundMessage::new("cli", "user123", "chat456", "Hello, world!");

    assert_eq!(msg.channel, "cli");
    assert_eq!(msg.sender_id, "user123");
    assert_eq!(msg.chat_id, "chat456");
    assert_eq!(msg.content, "Hello, world!");
    assert!(msg.metadata.is_empty());
}

/// 出站消息测试用例结构
struct OutboundCase {
    name: &'static str,
    content: &'static str,
    is_tool_hint: bool,
    #[allow(dead_code)]
    expect_is_progress: bool,
    expect_is_tool_hint: bool,
}

/// OutboundMessage::progress 构造测试
#[test]
fn outbound_message_progress_construct() {
    let test_vector = [
        OutboundCase {
            name: "普通进度消息",
            content: "thinking...",
            is_tool_hint: false,
            expect_is_progress: true,
            expect_is_tool_hint: false,
        },
        OutboundCase {
            name: "工具提示消息",
            content: "using tool...",
            is_tool_hint: true,
            expect_is_progress: true,
            expect_is_tool_hint: true,
        },
    ];

    for case in test_vector {
        let msg = OutboundMessage::progress(case.content, case.is_tool_hint);

        assert!(
            msg.is_progress(),
            "case[{}]: expected is_progress() to be true",
            case.name
        );
        assert_eq!(
            msg.is_tool_hint(),
            case.expect_is_tool_hint,
            "case[{}]: is_tool_hint() mismatch",
            case.name
        );
        assert_eq!(msg.content, case.content, "case[{}]: content mismatch", case.name);
    }
}

/// OutboundMessage::new 构造测试
#[test]
fn outbound_message_construct() {
    let msg = OutboundMessage::new("telegram", "chat789", "Response content");

    assert_eq!(msg.channel, "telegram");
    assert_eq!(msg.chat_id, "chat789");
    assert_eq!(msg.content, "Response content");
    assert!(msg.metadata.is_empty());
}

/// OutboundMessage 进度检测测试
#[test]
fn outbound_message_progress_detection() {
    // 进度消息
    let progress_msg = OutboundMessage::progress("thinking...", false);
    assert!(progress_msg.is_progress());

    // 普通消息
    let mut normal_msg = OutboundMessage::new("cli", "chat1", "normal");
    assert!(!normal_msg.is_progress());

    // 手动添加进度标记
    normal_msg
        .metadata
        .insert("_progress".to_string(), serde_json::json!(true));
    assert!(normal_msg.is_progress());
}

/// OutboundMessage 工具提示检测测试
#[test]
fn outbound_message_tool_hint_detection() {
    // 工具提示消息
    let tool_hint = OutboundMessage::progress("using tool...", true);
    assert!(tool_hint.is_tool_hint());

    // 普通进度消息
    let normal_progress = OutboundMessage::progress("thinking...", false);
    assert!(!normal_progress.is_tool_hint());

    // 普通消息
    let normal_msg = OutboundMessage::new("cli", "chat1", "normal");
    assert!(!normal_msg.is_tool_hint());
}
