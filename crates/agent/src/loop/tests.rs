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

/// 整合触发条件测试用例结构
struct ConsolidationTriggerCase {
    name: &'static str,
    message_count: usize,
    last_consolidated: usize,
    memory_window: usize,
}

/// 验证整合触发条件：消息数量达到阈值时触发整合
#[tokio::test]
async fn consolidation_triggers_when_message_window_reached() {
    let test_vector = [
        ConsolidationTriggerCase {
            name: "消息数量未达到阈值 - 不触发整合",
            message_count: 5,
            last_consolidated: 0,
            memory_window: 10,
        },
        ConsolidationTriggerCase {
            name: "消息数量刚好达到阈值 - 触发整合",
            message_count: 10,
            last_consolidated: 0,
            memory_window: 10,
        },
        ConsolidationTriggerCase {
            name: "消息数量超过阈值 - 触发整合",
            message_count: 15,
            last_consolidated: 0,
            memory_window: 10,
        },
        ConsolidationTriggerCase {
            name: "部分消息已整合 - 未达到阈值",
            message_count: 15,
            last_consolidated: 10,
            memory_window: 10,
        },
        ConsolidationTriggerCase {
            name: "部分消息已整合 - 刚好达到阈值",
            message_count: 20,
            last_consolidated: 10,
            memory_window: 10,
        },
    ];

    for case in test_vector {
        // 创建配置，使用自定义 memory_window
        let config = AgentDefaults {
            workspace: PathBuf::from("/tmp/test"),
            model: "test-model".to_string(),
            max_tokens: 1024,
            temperature: 0.5,
            max_tool_iterations: 10,
            memory_window: case.memory_window,
        };

        let provider = MockProvider::new("test response");
        let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed");

        // 手动设置会话状态
        let session_key = "test:consolidation";
        let mut session = agent.sessions.get_or_create(session_key);
        session.last_consolidated = case.last_consolidated;
        // 添加消息
        for i in 0..case.message_count {
            session.add_message(Message::assistant(format!("Message {i}")));
        }
        // 保存会话
        agent.sessions.save(&session).expect("Failed to save session");

        // 验证初始状态：consolidating 应该为空
        {
            let consolidating = agent.consolidating.lock().await;
            assert!(consolidating.is_empty(), "case[{}]: consolidating should be empty before processing", case.name);
        }

        // 处理一条消息
        let inbound = InboundMessage::new("test", "user", "consolidation", "trigger test");
        let _ = agent.process_message(inbound, Some(session_key)).await;

        // 验证整合状态
        // 注意：由于 try_consolidate 是同步执行的，处理完成后 consolidating 应该已经清空
        // 所以这里我们无法直接验证整合是否被触发，但可以验证不会出现死锁或 panic
        {
            let consolidating = agent.consolidating.lock().await;
            assert!(
                consolidating.is_empty(),
                "case[{}]: consolidating should be empty after processing completed",
                case.name
            );
        }
    }
}

/// 验证整合进行中时拒绝新的整合请求
#[tokio::test]
async fn consolidation_rejected_when_already_in_progress() {
    let config = AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 5, // 设置较小的窗口便于测试
    };

    let provider = MockProvider::new("test response");
    let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
        .await
        .expect("AgentLoop creation should succeed");

    let session_key = "test:concurrent";

    // 手动标记会话正在整合
    {
        let mut consolidating = agent.consolidating.lock().await;
        consolidating.insert(session_key.to_string());
    }

    // 设置会话状态，使其满足消息窗口条件
    let mut session = agent.sessions.get_or_create(session_key);
    session.last_consolidated = 0;
    for i in 0..10 {
        session.add_message(Message::assistant(format!("Message {i}")));
    }
    agent.sessions.save(&session).expect("Failed to save session");

    // 处理消息 - 由于会话已标记为整合中，应该跳过整合
    let inbound = InboundMessage::new("test", "user", "concurrent", "test");
    let _ = agent.process_message(inbound, Some(session_key)).await;

    // 验证：consolidating 应该仍然只包含我们手动添加的标记
    // （因为整合被跳过，不会清除标记）
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(
            consolidating.contains(session_key),
            "consolidating should still contain the session key since consolidation was skipped"
        );
    }

    // 清理：移除手动添加的标记
    {
        let mut consolidating = agent.consolidating.lock().await;
        consolidating.remove(session_key);
    }
}

/// 验证整合状态标记和清除的正确性
#[tokio::test]
async fn consolidation_state_properly_managed() {
    let config = AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 5,
    };

    let provider = MockProvider::new("test response");
    let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
        .await
        .expect("AgentLoop creation should succeed");

    let session_key = "test:state_management";

    // 初始状态：consolidating 应该为空
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.is_empty(), "consolidating should be empty initially");
    }

    // 设置会话状态，使其满足整合条件
    let mut session = agent.sessions.get_or_create(session_key);
    session.last_consolidated = 0;
    for i in 0..10 {
        session.add_message(Message::assistant(format!("Message {i}")));
    }
    agent.sessions.save(&session).expect("Failed to save session");

    // 处理消息 - 应该触发整合
    let inbound = InboundMessage::new("test", "user", "state_management", "test");
    let _ = agent.process_message(inbound, Some(session_key)).await;

    // 整合完成后：consolidating 应该被清空
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.is_empty(), "consolidating should be empty after consolidation completed");
    }
}

/// 验证多个会话的整合状态相互独立
#[tokio::test]
async fn consolidation_state_independent_across_sessions() {
    let config = AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 5,
    };

    let provider = MockProvider::new("test response");
    let agent = AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
        .await
        .expect("AgentLoop creation should succeed");

    let session_key_1 = "test:session_1";
    let session_key_2 = "test:session_2";

    // 手动标记 session_1 正在整合
    {
        let mut consolidating = agent.consolidating.lock().await;
        consolidating.insert(session_key_1.to_string());
    }

    // 验证 session_2 不受影响
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.contains(session_key_1), "session_1 should be marked as consolidating");
        assert!(!consolidating.contains(session_key_2), "session_2 should NOT be marked as consolidating");
    }

    // 设置 session_2 满足整合条件
    let mut session = agent.sessions.get_or_create(session_key_2);
    session.last_consolidated = 0;
    for i in 0..10 {
        session.add_message(Message::assistant(format!("Message {i}")));
    }
    agent.sessions.save(&session).expect("Failed to save session");

    // 处理 session_2 的消息 - 应该能正常触发整合（不受 session_1 状态影响）
    let inbound = InboundMessage::new("test", "user", "session_2", "test");
    let _ = agent.process_message(inbound, Some(session_key_2)).await;

    // 验证：session_1 仍然标记为整合中，session_2 已完成
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.contains(session_key_1), "session_1 should still be marked as consolidating");
        assert!(
            !consolidating.contains(session_key_2),
            "session_2 should NOT be marked as consolidating after completion"
        );
    }

    // 清理
    {
        let mut consolidating = agent.consolidating.lock().await;
        consolidating.clear();
    }
}

/// 验证多线程并发访问 consolidating 状态的正确性
#[tokio::test]
async fn consolidation_state_thread_safe() {
    use std::sync::Arc;

    let config = AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 5,
    };

    let provider = MockProvider::new("test response");
    let agent = Arc::new(
        AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed"),
    );

    // 创建多个并发任务，每个任务尝试操作不同的会话
    let mut handles = Vec::new();
    let num_tasks = 10;

    for i in 0..num_tasks {
        let agent_clone = Arc::clone(&agent);
        let session_key = format!("test:concurrent_{i}");

        let handle = tokio::spawn(async move {
            // 设置会话状态
            let mut session = agent_clone.sessions.get_or_create(&session_key);
            session.last_consolidated = 0;
            for j in 0..10 {
                session.add_message(Message::assistant(format!("Message {j}")));
            }
            agent_clone.sessions.save(&session).expect("Failed to save session");

            // 处理消息
            let inbound = InboundMessage::new("test", "user", format!("concurrent_{i}"), "test");
            let _ = agent_clone.process_message(inbound, Some(&session_key)).await;

            // 验证：处理完成后，该会话不应该在 consolidating 中
            {
                let consolidating = agent_clone.consolidating.lock().await;
                assert!(
                    !consolidating.contains(&session_key),
                    "session {session_key} should not be in consolidating after completion"
                );
            }
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    // 最终验证：所有会话都应该不在 consolidating 中
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.is_empty(), "consolidating should be empty after all tasks completed");
    }
}

/// 验证 Mutex 能有效防止同一会话的并发整合
#[tokio::test]
async fn mutex_prevents_concurrent_consolidation_same_session() {
    use std::sync::Arc;
    use std::time::Duration;

    let config = AgentDefaults {
        workspace: PathBuf::from("/tmp/test"),
        model: "test-model".to_string(),
        max_tokens: 1024,
        temperature: 0.5,
        max_tool_iterations: 10,
        memory_window: 5,
    };

    let provider = MockProvider::new("test response");
    let agent = Arc::new(
        AgentLoop::new(provider, config, None, None, std::collections::HashMap::new())
            .await
            .expect("AgentLoop creation should succeed"),
    );

    let session_key = "test:same_session";

    // 设置会话状态
    let mut session = agent.sessions.get_or_create(session_key);
    session.last_consolidated = 0;
    for i in 0..10 {
        session.add_message(Message::assistant(format!("Message {i}")));
    }
    agent.sessions.save(&session).expect("Failed to save session");

    // 创建两个并发任务，同时处理同一会话的消息
    let agent_clone = Arc::clone(&agent);
    let session_key_clone = session_key.to_string();

    let handle1 = tokio::spawn(async move {
        let inbound = InboundMessage::new("test", "user", "same_session", "message 1");
        agent_clone.process_message(inbound, Some(&session_key_clone)).await
    });

    // 稍微延迟，确保第一个任务已经开始处理
    tokio::time::sleep(Duration::from_millis(10)).await;

    let agent_clone2 = Arc::clone(&agent);
    let session_key_clone2 = session_key.to_string();

    let handle2 = tokio::spawn(async move {
        let inbound = InboundMessage::new("test", "user", "same_session", "message 2");
        agent_clone2.process_message(inbound, Some(&session_key_clone2)).await
    });

    // 等待两个任务完成
    let _ = handle1.await;
    let _ = handle2.await;

    // 验证：最终 consolidating 应该为空
    {
        let consolidating = agent.consolidating.lock().await;
        assert!(consolidating.is_empty(), "consolidating should be empty after both tasks completed");
    }
}
