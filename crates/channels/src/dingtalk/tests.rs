use tokio::sync::mpsc;

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

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await;
    assert!(dingtalk.is_ok());
}

/// 测试权限检查功能 - 精确匹配
#[tokio::test]
async fn permission_check() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: vec!["user1".to_string(), "user2".to_string()],
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await.unwrap();

    assert!(dingtalk.check_permission("user1"));
    assert!(dingtalk.check_permission("user2"));
    assert!(!dingtalk.check_permission("user3"));

    // | 分割不再匹配（精确匹配，对齐 HKUDS/nanobot#1677）
    assert!(!dingtalk.check_permission("user1|extra"));
}

/// 测试权限检查功能 - 空白名单拒绝所有（deny-by-default，对齐 HKUDS/nanobot#1403）
#[tokio::test]
async fn permission_check_empty_denies() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await.unwrap();

    assert!(!dingtalk.check_permission("user1"));
    assert!(!dingtalk.check_permission("any_user"));
}

/// 测试通配符允许所有人
#[tokio::test]
async fn permission_check_wildcard_allows_all() {
    let config = DingTalkConfig {
        enabled: false,
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        allow_from: vec!["*".to_string()],
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let dingtalk = DingTalk::new(config, inbound_tx).await.unwrap();

    assert!(dingtalk.check_permission("user1"));
    assert!(dingtalk.check_permission("any_user"));
}
