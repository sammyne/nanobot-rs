use super::*;

/// 测试钉钉通道创建
#[tokio::test]
async fn dingtalk_creation() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: Vec::new(),
    };

    let dingtalk = DingTalk::new(config).await;
    assert!(dingtalk.is_ok());
}

/// 测试权限检查功能
#[tokio::test]
async fn permission_check() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: vec!["user1".to_string(), "user2".to_string()],
    };

    let dingtalk = DingTalk::new(config).await.unwrap();

    assert!(dingtalk.check_permission("user1"));
    assert!(dingtalk.check_permission("user2"));
    assert!(!dingtalk.check_permission("user3"));

    // 测试带分隔符的发送者 ID
    assert!(dingtalk.check_permission("user1|extra"));
    assert!(!dingtalk.check_permission("user3|extra"));
}
