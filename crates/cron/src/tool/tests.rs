use nanobot_tools::ToolContext;
use tempfile::tempdir;

use super::*;

/// 创建测试用的 ToolContext
fn test_context() -> ToolContext {
    ToolContext::new("test-channel".to_string(), "12345".to_string())
}

/// 创建带指定 channel 和 chat_id 的 ToolContext
fn context_with(channel: &str, chat_id: &str) -> ToolContext {
    ToolContext::new(channel.to_string(), chat_id.to_string())
}

#[tokio::test]
async fn cron_tool_creation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path).await.unwrap());
    let tool = CronTool::new(service);

    assert_eq!(tool.name(), "cron");
    assert!(!tool.description().is_empty());
}

#[tokio::test]
async fn cron_tool_add_without_context() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path).await.unwrap());
    service.start().await;
    let tool = CronTool::new(service);
    // Use empty context to trigger the "no session context" error
    let ctx = ToolContext::new("".to_string(), "".to_string());

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "schedule": {
            "kind": "every",
            "every_seconds": 60
        }
    });

    let result = tool.execute(&ctx, params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn cron_tool_list_empty() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path).await.unwrap());
    service.start().await;
    let tool = CronTool::new(service);
    let ctx = test_context();

    let params = serde_json::json!({
        "action": "list"
    });

    let result = tool.execute(&ctx, params).await.unwrap();
    assert!(result.contains("No scheduled jobs"));
}

#[tokio::test]
async fn cron_tool_add_and_list() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path).await.unwrap());
    service.start().await;
    let tool = CronTool::new(Arc::clone(&service));
    let ctx = context_with("whatsapp", "1234567890");

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "schedule": {
            "kind": "every",
            "every_seconds": 60
        }
    });

    let result = tool.execute(&ctx, params).await.unwrap();
    assert!(result.contains("Created job"));

    let params = serde_json::json!({
        "action": "list"
    });

    let result = tool.execute(&ctx, params).await.unwrap();
    assert!(result.contains("Test reminder"));
}

#[tokio::test]
async fn cron_tool_invalid_tz() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path).await.unwrap());
    service.start().await;
    let tool = CronTool::new(service);
    let ctx = context_with("whatsapp", "1234567890");

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "schedule": {
            "kind": "cron",
            "expr": "0 * * * *",
            "tz": "Invalid/Timezone"
        }
    });

    let result = tool.execute(&ctx, params).await;
    assert!(result.is_err());
}
