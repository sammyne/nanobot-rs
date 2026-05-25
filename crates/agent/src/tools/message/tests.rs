use std::path::PathBuf;

use serde_json::json;
use tokio::sync::mpsc;

use super::*;

fn test_tool(workspace: &std::path::Path) -> (MessageTool, mpsc::Receiver<OutboundMessage>) {
    let (tx, rx) = mpsc::channel(100);
    let tool = MessageTool::new(tx, workspace.to_path_buf(), true);
    (tool, rx)
}

fn test_ctx() -> ToolContext {
    ToolContext::new("feishu", "chat_123")
}

#[tokio::test]
async fn send_basic_message() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, mut rx) = test_tool(dir.path());

    let result = tool.execute(&test_ctx(), json!({"content": "hello"})).await;
    assert!(result.is_ok());

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.content, "hello");
    assert_eq!(msg.channel, "feishu");
    assert_eq!(msg.chat_id, "chat_123");
    assert!(msg.media.is_empty());
}

#[tokio::test]
async fn default_channel_and_chat_id_from_context() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, mut rx) = test_tool(dir.path());

    tool.execute(&test_ctx(), json!({"content": "test"})).await.unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.channel, "feishu");
    assert_eq!(msg.chat_id, "chat_123");
}

#[tokio::test]
async fn custom_channel_and_chat_id() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, mut rx) = test_tool(dir.path());

    tool.execute(&test_ctx(), json!({"content": "cross", "channel": "dingtalk", "chat_id": "group_456"}))
        .await
        .unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.channel, "dingtalk");
    assert_eq!(msg.chat_id, "group_456");
}

#[tokio::test]
async fn media_local_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("image.png"), b"fake png").unwrap();

    let (tool, mut rx) = test_tool(dir.path());

    tool.execute(&test_ctx(), json!({"content": "see image", "media": ["image.png"]})).await.unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.media.len(), 1);
    assert!(msg.media[0].contains("image.png"));
}

#[tokio::test]
async fn media_url_passthrough() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, mut rx) = test_tool(dir.path());

    tool.execute(&test_ctx(), json!({"content": "link", "media": ["https://example.com/img.png"]})).await.unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.media, vec!["https://example.com/img.png"]);
}

#[tokio::test]
async fn media_file_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, _rx) = test_tool(dir.path());

    let result = tool.execute(&test_ctx(), json!({"content": "oops", "media": ["nonexistent.png"]})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn media_outside_workspace_rejected() {
    let dir = tempfile::tempdir().unwrap();
    // 创建 workspace 外的文件
    let outside = tempfile::tempdir().unwrap();
    let outside_file = outside.path().join("secret.txt");
    std::fs::write(&outside_file, b"secret").unwrap();

    let (tool, _rx) = test_tool(dir.path());

    let result =
        tool.execute(&test_ctx(), json!({"content": "steal", "media": [outside_file.display().to_string()]})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn missing_content_param_fails() {
    let dir = tempfile::tempdir().unwrap();
    let (tool, _rx) = test_tool(dir.path());

    let result = tool.execute(&test_ctx(), json!({})).await;
    assert!(result.is_err());
}
