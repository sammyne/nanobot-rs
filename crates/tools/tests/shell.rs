//! ShellTool 集成测试
//!
//! 测试 Shell 工具的命令执行、安全拦截和超时处理。

use std::path::PathBuf;

use nanobot_tools::{ExecTool, ExecToolOptions, Tool, ToolContext, ToolResult};
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
    let tool = ExecTool::new(ExecToolOptions { workspace: Some(PathBuf::from(temp_dir.path())), ..Default::default() });
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
    let tool = ExecTool::new(ExecToolOptions { workspace: Some(PathBuf::from(temp_dir.path())), ..Default::default() });
    let ctx = test_context();

    let result: ToolResult = tool.execute(&ctx, json!({"command": "rm -rf /"})).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("拒绝"));
}

/// 超时处理
#[tokio::test]
async fn shell_timeout() {
    let temp_dir = setup();
    let tool = ExecTool::new(ExecToolOptions {
        workspace: Some(PathBuf::from(temp_dir.path())),
        timeout: 1,
        ..Default::default()
    });
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

/// restrict_to_workspace 限制工作空间
#[tokio::test]
async fn shell_restrict_to_workspace_blocks_path_traversal() {
    let temp_dir = setup();
    let tool = ExecTool::new(ExecToolOptions {
        workspace: Some(PathBuf::from(temp_dir.path())),
        restrict_to_workspace: true,
        ..Default::default()
    });
    let ctx = test_context();

    // 尝试访问工作空间外的路径
    let result: ToolResult = tool.execute(&ctx, json!({"command": "cat /etc/passwd"})).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("拒绝") || error_msg.contains("路径"));
}

/// restrict_to_workspace 允许工作空间内操作
#[tokio::test]
async fn shell_restrict_to_workspace_allows_inside() {
    let temp_dir = setup();
    let workspace = PathBuf::from(temp_dir.path());
    let tool = ExecTool::new(ExecToolOptions {
        workspace: Some(workspace.clone()),
        restrict_to_workspace: true,
        ..Default::default()
    });
    let ctx = test_context();

    // 在工作空间内创建文件
    let test_file = workspace.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    // 读取工作空间内的文件应该成功
    let result: ToolResult = tool.execute(&ctx, json!({"command": format!("cat {}", test_file.display())})).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("test content"));
}

/// exec.timeout 配置影响命令超时
#[tokio::test]
async fn shell_exec_timeout_config() {
    let temp_dir = setup();
    // 设置超时为 1 秒
    let tool = ExecTool::new(ExecToolOptions {
        workspace: Some(PathBuf::from(temp_dir.path())),
        timeout: 1,
        ..Default::default()
    });
    let ctx = test_context();

    // 执行一个会超时的命令（不指定 timeout_ms，使用配置的 timeout）
    let result: ToolResult = tool.execute(&ctx, json!({"command": "sleep 5"})).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("超时"));
}

/// exec.pathAppend 配置影响 PATH 环境变量
#[tokio::test]
async fn shell_path_append_config() {
    let temp_dir = setup();

    // 创建一个自定义路径目录
    let custom_bin = temp_dir.path().join("custom_bin");
    std::fs::create_dir(&custom_bin).unwrap();

    // 在自定义目录中创建一个脚本
    let script_path = custom_bin.join("mytestcmd");
    #[cfg(unix)]
    {
        std::fs::write(&script_path, "#!/bin/sh\necho 'custom_path_works'").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        std::fs::write(custom_bin.join("mytestcmd.bat"), "@echo custom_path_works").unwrap();
    }

    let tool = ExecTool::new(ExecToolOptions {
        workspace: Some(PathBuf::from(temp_dir.path())),
        path_append: custom_bin.display().to_string(),
        ..Default::default()
    });
    let ctx = test_context();

    // 执行自定义命令（应该能从 PATH 中找到）
    let result: ToolResult = tool.execute(&ctx, json!({"command": "mytestcmd"})).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("custom_path_works"));
}

/// ToolRegistry 集成测试：使用 ToolsConfig 配置
#[tokio::test]
async fn tool_registry_with_tools_config() {
    use nanobot_config::ExecToolConfig;
    use nanobot_tools::ToolRegistry;

    let temp_dir = setup();
    let workspace = PathBuf::from(temp_dir.path());

    // 创建 ExecToolConfig
    let exec_config = ExecToolConfig { timeout: 30, path_append: String::new() };
    let restrict_to_workspace = true;

    // 使用配置创建 ToolRegistry
    let registry = ToolRegistry::new(workspace.clone(), exec_config, restrict_to_workspace);

    // 验证 shell 工具已注册
    assert!(registry.contains("shell"));

    // 执行命令验证配置生效
    let ctx = test_context();
    let result = registry.execute(&ctx, "shell", json!({"command": "echo test"})).await;

    assert!(result.is_ok());
}
