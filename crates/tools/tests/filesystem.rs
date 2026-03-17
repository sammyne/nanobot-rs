//! 文件系统工具集成测试
//!
//! 测试 ReadFileTool、WriteFileTool、EditFileTool、ListDirTool 的功能。

use nanobot_tools::{EditFileTool, ListDirTool, ReadFileTool, Tool, ToolContext, ToolResult, WriteFileTool};
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

// ==================== ReadFileTool 测试 ====================

/// 正常读取文件
#[tokio::test]
async fn read_file_success() {
    let temp_dir = setup();
    let tool = ReadFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let test_content = "Hello, World!\nThis is a test file.";
    let ctx = test_context();

    tokio::fs::write(temp_dir.path().join("test.txt"), test_content).await.unwrap();

    let result: ToolResult = tool.execute(&ctx, json!({"path": "test.txt"})).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_content);
}

/// 读取不存在的文件
#[tokio::test]
async fn read_file_not_found() {
    let temp_dir = setup();
    let tool = ReadFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    let result: ToolResult = tool.execute(&ctx, json!({"path": "nonexistent.txt"})).await;

    assert!(result.is_err());
}

/// 路径指向目录而非文件
#[tokio::test]
async fn read_file_is_directory() {
    let temp_dir = setup();
    let tool = ReadFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    tokio::fs::create_dir(temp_dir.path().join("subdir")).await.unwrap();

    let result: ToolResult = tool.execute(&ctx, json!({"path": "subdir"})).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不是文件"));
}

// ==================== WriteFileTool 测试 ====================

/// 正常写入文件
#[tokio::test]
async fn write_file_success() {
    let temp_dir = setup();
    let tool = WriteFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let content = "Test content for writing.";
    let ctx = test_context();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "path": "output.txt",
                "content": content
            }),
        )
        .await;

    assert!(result.is_ok());

    let written = tokio::fs::read_to_string(temp_dir.path().join("output.txt")).await.unwrap();
    assert_eq!(written, content);
}

/// 自动创建父目录
#[tokio::test]
async fn write_file_create_parent_dirs() {
    let temp_dir = setup();
    let tool = WriteFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "path": "deep/nested/path/file.txt",
                "content": "nested content"
            }),
        )
        .await;

    assert!(result.is_ok());

    let path = temp_dir.path().join("deep/nested/path/file.txt");
    assert!(path.exists());
}

// ==================== EditFileTool 测试 ====================

/// 正常编辑文件
#[tokio::test]
async fn edit_file_success() {
    let temp_dir = setup();
    let tool = EditFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let original = "line1\nline2\nline3";
    let ctx = test_context();

    tokio::fs::write(temp_dir.path().join("edit.txt"), original).await.unwrap();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "path": "edit.txt",
                "old_text": "line2",
                "new_text": "modified_line2"
            }),
        )
        .await;

    assert!(result.is_ok());

    let content = tokio::fs::read_to_string(temp_dir.path().join("edit.txt")).await.unwrap();
    assert_eq!(content, "line1\nmodified_line2\nline3");
}

/// old_text 不匹配
#[tokio::test]
async fn edit_file_no_match() {
    let temp_dir = setup();
    let tool = EditFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    tokio::fs::write(temp_dir.path().join("edit.txt"), "some content").await.unwrap();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "path": "edit.txt",
                "old_text": "nonexistent",
                "new_text": "replacement"
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("未找到匹配"));
}

/// 多处匹配
#[tokio::test]
async fn edit_file_multiple_matches() {
    let temp_dir = setup();
    let tool = EditFileTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    tokio::fs::write(temp_dir.path().join("edit.txt"), "abc abc abc").await.unwrap();

    let result: ToolResult = tool
        .execute(
            &ctx,
            json!({
                "path": "edit.txt",
                "old_text": "abc",
                "new_text": "xyz"
            }),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("处匹配"));
}

// ==================== ListDirTool 测试 ====================

/// 正常列出目录
#[tokio::test]
async fn list_dir_success() {
    let temp_dir = setup();
    let tool = ListDirTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    tokio::fs::write(temp_dir.path().join("file1.txt"), "").await.unwrap();
    tokio::fs::create_dir(temp_dir.path().join("subdir")).await.unwrap();

    let result: ToolResult = tool.execute(&ctx, json!({"path": "."})).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("file1.txt"));
    assert!(output.contains("subdir"));
}

/// 列出不存在的目录
#[tokio::test]
async fn list_dir_not_found() {
    let temp_dir = setup();
    let tool = ListDirTool::new(temp_dir.path().to_str().unwrap(), None::<&str>);
    let ctx = test_context();

    let result: ToolResult = tool.execute(&ctx, json!({"path": "nonexistent_dir"})).await;

    assert!(result.is_err());
}
