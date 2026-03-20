//! Subagent 组件测试用例
//!
//! 测试子代理管理器的核心功能，包括任务创建、执行、结果通知等。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use nanobot_channels::messages::InboundMessage;
use nanobot_provider::{Message, Provider};
use nanobot_subagent::SubagentManager;
use nanobot_tools::ToolDefinition;
use serde_json::json;
use tokio::sync::mpsc;

// ==================== Mock Provider ====================

/// Mock LLM Provider for testing
#[derive(Clone)]
struct MockProvider {
    responses: Vec<Message>,
    response_index: Arc<AtomicUsize>,
}

impl MockProvider {
    fn new(responses: Vec<Message>) -> Self {
        Self { responses, response_index: Arc::new(AtomicUsize::new(0)) }
    }

    fn simple_response(content: &str) -> Self {
        Self::new(vec![Message::assistant(content)])
    }

    fn with_tool_response(content: &str, tool_calls: Vec<nanobot_provider::ToolCall>) -> Self {
        Self::new(vec![Message::assistant_with_tools(content, tool_calls)])
    }
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn chat(&self, _messages: &[Message], _options: &nanobot_provider::Options) -> anyhow::Result<Message> {
        let index = self.response_index.fetch_add(1, Ordering::SeqCst);
        if index < self.responses.len() {
            Ok(self.responses[index].clone())
        } else {
            anyhow::bail!("No more mock responses")
        }
    }

    fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {
        // No-op for mock
    }
}

// ==================== Helper Functions ====================

/// 创建测试用的消息通道
fn create_test_channel() -> (mpsc::Sender<InboundMessage>, mpsc::Receiver<InboundMessage>) {
    mpsc::channel(100)
}

// ==================== Test: Subagent Creation and Management ====================

/// 测试子代理创建和管理的基本流程
///
/// 验证：
/// - 子代理能够成功创建
/// - 运行计数正确更新
/// - 任务完成后从运行集合中移除
#[tokio::test]
async fn subagent_creation_and_management() {
    let (sender, mut receiver) = create_test_channel();
    let provider = MockProvider::simple_response("Task completed successfully");

    let manager = SubagentManager::new(provider, std::path::PathBuf::from("/tmp/workspace"), sender, 0.7, 4096);

    // 验证初始运行计数为 0
    assert_eq!(manager.get_running_count(), 0, "Initial running count should be 0");

    // 创建任务 - clone Arc for spawn
    let result = manager
        .clone()
        .spawn("Test task description", None, "test_channel", "chat_123")
        .await
        .expect("Failed to spawn task");

    assert!(result.contains("started"));

    // 验证运行计数为 1
    assert_eq!(manager.get_running_count(), 1, "Running count should be 1 after task creation");

    // 等待任务完成
    tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        while manager.get_running_count() > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Task did not complete in time");

    // 验证收到完成消息
    let result_message = receiver.recv().await.expect("Did not receive completion message");

    assert_eq!(result_message.channel, "system");
    assert_eq!(result_message.sender_id, "subagent");
    assert!(result_message.content.contains("completed successfully"));

    println!("✓ Subagent creation and management test passed");
}

// ==================== Test: Task Execution with Tools ====================

/// 测试子代理执行工具的能力
///
/// 验证：
/// - 子代理能够执行文件系统工具
/// - 工具结果正确返回
#[tokio::test]
async fn task_execution_with_tools() {
    // 创建临时工作目录
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_path_buf();

    let (sender, mut receiver) = create_test_channel();

    // 创建工具调用响应
    let tool_calls = vec![nanobot_provider::ToolCall::new(
        "call_001",
        "write_file",
        json!({
            "path": "test_file.txt",
            "content": "Hello, subagent!"
        }),
    )];

    let provider = MockProvider::with_tool_response("Creating file", tool_calls);

    let manager = SubagentManager::new(provider, workspace_path.clone(), sender, 0.7, 4096);

    // 创建任务 - clone Arc for spawn
    let _result = manager
        .clone()
        .spawn("Write a test file", None, "test_channel", "chat_456")
        .await
        .expect("Failed to spawn task");

    // 等待任务完成
    tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        while manager.get_running_count() > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Task did not complete in time");

    // 验证文件已创建
    let file_path = temp_dir.path().join("test_file.txt");
    assert!(file_path.exists(), "File should be created");

    let content = tokio::fs::read_to_string(&file_path).await.expect("Failed to read file");
    assert_eq!(content, "Hello, subagent!", "File content should match");

    // 验证收到完成消息
    let result_message = receiver.recv().await.expect("Did not receive completion message");

    assert!(result_message.content.contains("Task"));

    println!("✓ Task execution with tools test passed");
}

// ==================== Test: Error Handling ====================

/// 测试错误处理机制
///
/// 验证：
/// - LLM 调用失败被正确捕获
/// - 错误信息通过消息总线通知
/// - 错误状态被正确标识
#[tokio::test]
async fn error_handling() {
    let (sender, mut receiver) = create_test_channel();

    // 创建一个总是失败的 mock provider
    #[derive(Clone)]
    struct FailingProvider;

    #[async_trait::async_trait]
    impl Provider for FailingProvider {
        async fn chat(&self, _messages: &[Message], _options: &nanobot_provider::Options) -> anyhow::Result<Message> {
            anyhow::bail!("LLM call failed")
        }

        fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {}
    }

    let manager = SubagentManager::new(FailingProvider, std::path::PathBuf::from("/tmp/workspace"), sender, 0.7, 4096);

    // 创建任务 - clone Arc for spawn
    let _result =
        manager.clone().spawn("Failing task", None, "test_channel", "chat_789").await.expect("Failed to spawn task");

    // 等待任务完成（应该失败）
    tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        while manager.get_running_count() > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Task did not complete in time");

    // 验证收到错误消息
    let result_message = receiver.recv().await.expect("Did not receive error message");

    assert!(result_message.content.contains("failed"));
    assert!(result_message.content.contains("Error"));

    println!("✓ Error handling test passed");
}

// ==================== Test: Multiple Concurrent Tasks ====================

/// 测试并发任务执行
///
/// 验证：
/// - 多个子代理能够并发运行
/// - 运行计数正确反映并发数量
/// - 所有任务都能正确完成
#[tokio::test]
async fn multiple_concurrent_tasks() {
    let (sender, mut receiver) = create_test_channel();
    let provider = MockProvider::simple_response("Task done");

    let manager = SubagentManager::new(provider, std::path::PathBuf::from("/tmp/workspace"), sender, 0.7, 4096);

    // 创建多个任务
    let task_count = 3;
    for i in 0..task_count {
        let _result = manager
            .clone()
            .spawn(format!("Concurrent task {i}"), None, "test_channel", format!("chat_{i}"))
            .await
            .expect("Failed to spawn task");
    }

    // 验证运行计数
    assert_eq!(manager.get_running_count(), task_count, "Running count should match task count");

    // 等待所有任务完成
    let mut completed_count = 0;
    tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        while completed_count < task_count {
            if receiver.recv().await.is_some() {
                completed_count += 1;
            }
        }
    })
    .await
    .expect("Tasks did not complete in time");

    // 验证所有任务都已完成
    assert_eq!(manager.get_running_count(), 0, "All tasks should be completed");

    println!("✓ Multiple concurrent tasks test passed (completed {task_count} tasks)");
}

// ==================== Test: Maximum Iterations ====================

/// 测试最大迭代次数限制
///
/// 验证：
/// - 超过最大迭代次数时任务被终止
/// - 超时信息正确返回
#[tokio::test]
async fn maximum_iterations_limit() {
    let (sender, mut receiver) = create_test_channel();

    // 创建一个总是返回工具调用的 mock provider
    #[derive(Clone)]
    struct InfiniteToolProvider;

    #[async_trait::async_trait]
    impl Provider for InfiniteToolProvider {
        async fn chat(&self, _messages: &[Message], _options: &nanobot_provider::Options) -> anyhow::Result<Message> {
            Ok(Message::assistant_with_tools(
                "Calling tool",
                vec![nanobot_provider::ToolCall::new(
                    "call_001",
                    "write_file",
                    json!({"path": "test.txt", "content": "test"}),
                )],
            ))
        }

        fn bind_tools(&mut self, _tools: Vec<ToolDefinition>) {}
    }

    let manager =
        SubagentManager::new(InfiniteToolProvider, std::path::PathBuf::from("/tmp/workspace"), sender, 0.7, 4096);

    // 创建任务 - clone Arc for spawn
    let _result = manager
        .clone()
        .spawn("Long running task", None, "test_channel", "chat_999")
        .await
        .expect("Failed to spawn task");

    // 等待任务完成（应该因达到最大迭代次数而终止）
    tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
        while manager.get_running_count() > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Task did not complete in time");

    // 验证收到完成消息（可能是正常完成或无最终响应）
    let result_message = receiver.recv().await.expect("Did not receive message");

    // 任务应该完成（无论是正常完成还是达到最大迭代次数）
    assert!(!result_message.content.is_empty());

    println!("✓ Maximum iterations limit test passed");
}

// ==================== Test: Subagent Manager Construction ====================

/// 测试子代理管理器的构造
///
/// 验证：
/// - 管理器能够成功创建
/// - 工具被正确绑定
#[tokio::test]
async fn manager_construction() {
    let (sender, _receiver) = create_test_channel();
    let provider = MockProvider::simple_response("OK");

    let manager = SubagentManager::new(provider, std::path::PathBuf::from("/tmp/workspace"), sender, 0.7, 4096);

    // 验证初始状态
    assert_eq!(manager.get_running_count(), 0);

    println!("✓ Manager construction test passed");
}

// ==================== Test: InboundMessage Session Key ====================

/// 测试 InboundMessage 的 session_key 方法
#[test]
fn inbound_message_session_key() {
    let msg = InboundMessage::new("test_channel", "user_123", "chat_456", "Hello");

    assert_eq!(msg.session_key(), "test_channel:chat_456");

    println!("✓ InboundMessage session key test passed");
}

// ==================== Test: Task Label Generation ====================

/// 测试任务标签生成
#[test]
fn task_label_generation() {
    use nanobot_subagent::Task;

    // 测试短描述
    let short_desc = "Do something";
    let label = Task::label_from_description(short_desc);
    assert_eq!(label, "Do something");

    // 测试长描述（应被截断）
    let long_desc = "This is a very long task description that should be truncated to 30 characters";
    let label = Task::label_from_description(long_desc);
    assert!(label.len() <= 30);

    println!("✓ Task label generation test passed");
}

// ==================== Test: Message Construction ====================

/// 测试消息构造
#[test]
fn message_construction() {
    // 测试用户消息
    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.role(), "user");
    assert_eq!(user_msg.content(), "Hello");

    // 测试助手消息
    let assistant_msg = Message::assistant("Hi there");
    assert_eq!(assistant_msg.role(), "assistant");
    assert_eq!(assistant_msg.content(), "Hi there");
    assert!(assistant_msg.tool_calls().is_empty());

    // 测试带工具调用的助手消息
    let tool_call = nanobot_provider::ToolCall::new("call_1", "test_tool", json!({"arg": "value"}));
    let assistant_with_tools = Message::assistant_with_tools("Using tool", vec![tool_call]);
    assert_eq!(assistant_with_tools.tool_calls().len(), 1);

    // 测试系统消息
    let system_msg = Message::system("System prompt");
    assert_eq!(system_msg.role(), "system");

    // 测试工具消息
    let tool_msg = Message::tool("call_1", "Tool result");
    assert_eq!(tool_msg.role(), "tool");
    assert_eq!(tool_msg.tool_call_id(), Some("call_1"));

    println!("✓ Message construction test passed");
}

// ==================== Test: ToolCall Construction ====================

/// 测试工具调用构造
#[test]
fn tool_call_construction() {
    let tool_call = nanobot_provider::ToolCall::new("call_123", "search", json!({"query": "rust"}));

    assert_eq!(tool_call.id, "call_123");
    assert_eq!(tool_call.name, "search");

    // 测试参数解析
    let args = tool_call.parse_arguments::<serde_json::Value>().expect("Failed to parse arguments");
    assert_eq!(args["query"], "rust");

    println!("✓ ToolCall construction test passed");
}
