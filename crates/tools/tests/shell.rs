//! ShellTool 集成测试
//!
//! 测试 Shell 工具的命令执行、安全拦截和超时处理。

use nanobot_tools::{ShellTool, Tool, ToolContext, ToolResult};
use serde_json::json;
use tempfile::TempDir;

/// 准备临时工作环境
fn setup() -> TempDir {
    TempDir::new().unwrap()
}

/// 创建测试用的 ToolContext
fn test_context() -> ToolContext {
    ToolContext::new("test-channel".to_string(), "12345".to_string())
}

/// 正常执行命令
#[tokio::test]
async fn shell_echo_success() {
    let temp_dir = setup();
    let tool = ShellTool::new(temp_dir.path().to_str().unwrap());
    let ctx = test_context();

    let result: ToolResult = tool.execute(&ctx, json!({"command": "echo hello"})).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("hello"));
}

/// 危险命令拦截
#[tokio::test]
async fn shell_dangerous_command_blocked() {
    let temp_dir = setup();
    let tool = ShellTool::new(temp_dir.path().to_str().unwrap());
    let ctx = test_context();

    let result: ToolResult = tool.execute(&ctx, json!({"command": "rm -rf /"})).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("拒绝"));
}

/// 超时处理
#[tokio::test]
async fn shell_timeout() {
    let temp_dir = setup();
    let tool = ShellTool::new(temp_dir.path().to_str().unwrap()).with_timeout(1);
    let ctx = test_context();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "command": "sleep 10",
                "timeout_ms": 100
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("超时"));
}
