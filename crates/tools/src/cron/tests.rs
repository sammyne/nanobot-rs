use tempfile::tempdir;

use super::*;

#[tokio::test]
async fn cron_tool_creation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    let tool = CronTool::new(service);

    assert_eq!(tool.name(), "cron");
    assert!(!tool.description().is_empty());
}

#[tokio::test]
async fn cron_tool_set_context() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    let tool = CronTool::new(service);

    tool.set_context("whatsapp".to_string(), "1234567890".to_string());

    assert_eq!(tool.get_channel(), "whatsapp");
    assert_eq!(tool.get_chat_id(), "1234567890");
}

#[tokio::test]
async fn cron_tool_add_without_context() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    service.start().await.unwrap();
    let tool = CronTool::new(service);

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "every_seconds": 60
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn cron_tool_list_empty() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    service.start().await.unwrap();
    let tool = CronTool::new(service);

    let params = serde_json::json!({
        "action": "list"
    });

    let result = tool.execute(params).await.unwrap();
    assert!(result.contains("No scheduled jobs"));
}

#[tokio::test]
async fn cron_tool_add_and_list() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    service.start().await.unwrap();
    let tool = CronTool::new(Arc::clone(&service));

    tool.set_context("whatsapp".to_string(), "1234567890".to_string());

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "every_seconds": 60
    });

    let result = tool.execute(params).await.unwrap();
    assert!(result.contains("Created job"));

    let params = serde_json::json!({
        "action": "list"
    });

    let result = tool.execute(params).await.unwrap();
    assert!(result.contains("Test reminder"));
}

#[tokio::test]
async fn cron_tool_invalid_tz() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cron.json");
    let service = Arc::new(CronService::new(path, None));
    service.start().await.unwrap();
    let tool = CronTool::new(service);

    tool.set_context("whatsapp".to_string(), "1234567890".to_string());

    let params = serde_json::json!({
        "action": "add",
        "message": "Test reminder",
        "every_seconds": 60,
        "tz": "Invalid/Timezone"
    });

    let result = tool.execute(params).await;
    assert!(result.is_err());
}
