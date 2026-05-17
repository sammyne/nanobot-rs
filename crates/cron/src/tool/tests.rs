use nanobot_tools::ToolContext;
use tempfile::tempdir;

use super::*;

// ============ CronArgs ↔ CronArgsSchema 互操作测试 ============

#[test]
fn cron_args_add_interop() {
    // enum → JSON → struct
    let enum_val =
        CronArgs::Add { message: "Daily standup".to_string(), schedule: CronScheduleArgs::Every { every_seconds: 60 } };
    let json = serde_json::to_value(&enum_val).unwrap();
    let struct_val: CronArgsSchema = serde_json::from_value(json).unwrap();
    assert_eq!(struct_val.action, "add");
    assert_eq!(struct_val.message, "Daily standup");
    assert!(struct_val.schedule.is_some());

    // struct → JSON → enum
    let json = serde_json::to_value(&struct_val).unwrap();
    let roundtrip: CronArgs = serde_json::from_value(json).unwrap();
    assert!(matches!(roundtrip, CronArgs::Add { message, .. } if message == "Daily standup"));
}

#[test]
fn cron_args_list_interop() {
    // enum → JSON → struct
    let enum_val = CronArgs::List;
    let json = serde_json::to_value(&enum_val).unwrap();
    let struct_val: CronArgsSchema = serde_json::from_value(json).unwrap();
    assert_eq!(struct_val.action, "list");
    assert!(struct_val.message.is_empty());
    assert!(struct_val.schedule.is_none());
    assert!(struct_val.job_id.is_empty());

    // struct → JSON → enum
    let json = serde_json::to_value(&struct_val).unwrap();
    let roundtrip: CronArgs = serde_json::from_value(json).unwrap();
    assert!(matches!(roundtrip, CronArgs::List));
}

#[test]
fn cron_args_remove_interop() {
    // enum → JSON → struct
    let enum_val = CronArgs::Remove { job_id: "job-123".to_string() };
    let json = serde_json::to_value(&enum_val).unwrap();
    let struct_val: CronArgsSchema = serde_json::from_value(json).unwrap();
    assert_eq!(struct_val.action, "remove");
    assert_eq!(struct_val.job_id, "job-123");

    // struct → JSON → enum
    let json = serde_json::to_value(&struct_val).unwrap();
    let roundtrip: CronArgs = serde_json::from_value(json).unwrap();
    assert!(matches!(roundtrip, CronArgs::Remove { job_id } if job_id == "job-123"));
}

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
