//! Agent 库测试
//!
//! 按照表驱动测试规范编写的单元测试

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
        let provider = std::sync::Arc::new(MockProvider::new(case.expected_response));
        let config = test_config();
        let agent = AgentLoop::new_direct(provider, config);

        let result = agent
            .process_direct(case.input, case.session_id)
            .await
            .unwrap_or_else(|e| panic!("case[{}]: process_direct failed: {}", case.name, e));

        assert_eq!(
            result, case.expected_response,
            "case[{}]: response mismatch",
            case.name
        );
    }
}

/// 验证 process_direct 能正确处理空消息输入
#[tokio::test]
async fn process_direct_handles_empty_message() {
    let provider = std::sync::Arc::new(MockProvider::new("OK"));
    let config = test_config();
    let agent = AgentLoop::new_direct(provider, config);

    let result = agent
        .process_direct("", None::<&str>)
        .await
        .expect("empty message should be handled");

    assert_eq!(result, "OK");
}

/// 验证 config 方法返回正确的配置引用
#[test]
fn config_returns_correct_reference() {
    let provider = std::sync::Arc::new(MockProvider::new("test"));
    let config = test_config();
    let agent = AgentLoop::new_direct(provider, config.clone());

    let returned_config = agent.config();

    assert_eq!(returned_config.model, config.model, "model should match");
    assert_eq!(
        returned_config.max_tool_iterations, config.max_tool_iterations,
        "max_tool_iterations should match"
    );
    assert_eq!(
        returned_config.max_tokens, config.max_tokens,
        "max_tokens should match"
    );
    assert_eq!(
        returned_config.temperature, config.temperature,
        "temperature should match"
    );
}

/// AgentDefaults 构造测试用例结构
struct DefaultsCase {
    name: &'static str,
    agent: AgentLoop,
    expect_model: &'static str,
    expect_max_tokens: usize,
}

/// 验证 AgentLoop 能正确存储和使用不同的配置值
#[test]
fn agent_loop_uses_custom_config_values() {
    let custom_defaults1 = AgentDefaults {
        workspace: "/tmp/test1".to_string(),
        model: "custom-model-1".to_string(),
        max_tokens: 2048,
        temperature: 0.7,
        max_tool_iterations: 20,
        memory_window: 100,
    };

    let custom_defaults2 = AgentDefaults {
        workspace: "/tmp/test2".to_string(),
        model: "custom-model-2".to_string(),
        max_tokens: 4096,
        temperature: 0.3,
        max_tool_iterations: 5,
        memory_window: 25,
    };

    let test_vector = [
        DefaultsCase {
            name: "自定义配置 1",
            agent: AgentLoop::new_direct(
                std::sync::Arc::new(MockProvider::new("test")),
                custom_defaults1,
            ),
            expect_model: "custom-model-1",
            expect_max_tokens: 2048,
        },
        DefaultsCase {
            name: "自定义配置 2",
            agent: AgentLoop::new_direct(
                std::sync::Arc::new(MockProvider::new("test")),
                custom_defaults2,
            ),
            expect_model: "custom-model-2",
            expect_max_tokens: 4096,
        },
    ];

    for case in test_vector {
        let cfg = case.agent.config();
        assert_eq!(
            cfg.model, case.expect_model,
            "case[{}]: model mismatch",
            case.name
        );
        assert_eq!(
            cfg.max_tokens, case.expect_max_tokens,
            "case[{}]: max_tokens mismatch",
            case.name
        );
    }
}

/// 从 bus 模块引入测试类型
use crate::bus::{InboundMessage, OutboundMessage};

/// 入站消息测试用例结构
struct InboundCase {
    name: &'static str,
    session_id: &'static str,
    content: &'static str,
    expect: Option<(&'static str, &'static str, &'static str)>,
}

/// InboundMessage::from_session_id 解析测试
#[test]
fn inbound_message_from_session_id_parsing() {
    let test_vector = [
        InboundCase {
            name: "标准格式 cli:direct",
            session_id: "cli:direct",
            content: "hello",
            expect: Some(("cli", "user", "direct")),
        },
        InboundCase {
            name: "多冒号格式 telegram:123:456",
            session_id: "telegram:123:456",
            content: "hello",
            expect: Some(("telegram", "user", "123:456")),
        },
        InboundCase {
            name: "空 session_id",
            session_id: "",
            content: "hello",
            expect: None,
        },
        InboundCase {
            name: "缺少冒号的 session_id",
            session_id: "cli",
            content: "hello",
            expect: None,
        },
    ];

    for case in test_vector {
        let result = InboundMessage::from_session_id(case.session_id, case.content);

        match (result, case.expect) {
            (Some((ch, sender, id)), Some((exp_ch, exp_sender, exp_id))) => {
                assert_eq!(ch, exp_ch, "case[{}]: channel mismatch", case.name);
                assert_eq!(
                    sender, exp_sender,
                    "case[{}]: sender_id mismatch",
                    case.name
                );
                assert_eq!(id, exp_id, "case[{}]: chat_id mismatch", case.name);
            }
            (None, None) => {}
            (Some(_), None) => {
                panic!("case[{}]: expected None but got Some", case.name);
            }
            (None, Some(_)) => {
                panic!("case[{}]: expected Some but got None", case.name);
            }
        }
    }
}

/// InboundMessage::new 构造测试
#[test]
fn inbound_message_construct() {
    let msg = InboundMessage::new("cli", "user123", "chat456", "Hello, world!");

    assert_eq!(msg.channel, "cli");
    assert_eq!(msg.sender_id, "user123");
    assert_eq!(msg.chat_id, "chat456");
    assert_eq!(msg.content, "Hello, world!");
    assert!(msg.metadata.is_none());
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
        assert_eq!(
            msg.content, case.content,
            "case[{}]: content mismatch",
            case.name
        );
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

/// 消息序列化和反序列化测试
#[test]
fn message_serialization_roundtrip() {
    let msg = OutboundMessage::progress("test progress", false);
    let json_str = serde_json::to_string(&msg).expect("serialization should succeed");

    let deserialized: OutboundMessage =
        serde_json::from_str(&json_str).expect("deserialization should succeed");

    assert_eq!(deserialized.content, msg.content);
    assert!(deserialized.is_progress());
}
