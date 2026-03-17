//! 飞书通道测试
//!
//! 包含飞书通道的单元测试。

use feishu_sdk::event::Event;
use tokio::sync::mpsc;

use super::*;

/// 测试飞书通道创建
#[tokio::test]
async fn feishu_channel_creation() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await;
    assert!(channel.is_ok());
}

/// 测试飞书通道创建时验证配置
#[tokio::test]
async fn feishu_channel_validation_empty_app_id() {
    let config = FeishuConfig {
        enabled: true,
        app_id: "".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await;
    assert!(channel.is_err());
}

/// 测试飞书通道创建时验证配置
#[tokio::test]
async fn feishu_channel_validation_empty_app_secret() {
    let config = FeishuConfig {
        enabled: true,
        app_id: "test_app_id".to_string(),
        app_secret: "".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await;
    assert!(channel.is_err());
}

/// 测试权限检查功能
#[tokio::test]
async fn permission_check_with_whitelist() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: vec!["user1".to_string(), "user2".to_string()],
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    assert!(channel.check_permission("user1"));
    assert!(channel.check_permission("user2"));
    assert!(!channel.check_permission("user3"));

    // 测试带分隔符的发送者 ID
    assert!(channel.check_permission("user1|extra"));
    assert!(!channel.check_permission("user3|extra"));
}

/// 测试权限检查功能 - 空白名单允许所有
#[tokio::test]
async fn permission_check_empty_whitelist() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    // 空白名单应允许所有用户
    assert!(channel.check_permission("user1"));
    assert!(channel.check_permission("user2"));
    assert!(channel.check_permission("any_user"));
}

/// 测试通道名称
#[tokio::test]
async fn channel_name() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();
    assert_eq!(channel.name(), "feishu");
}

/// 测试通道运行状态
#[tokio::test]
async fn channel_running_state() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    // 初始状态应为未运行
    assert!(!channel.is_running());
}

/// 测试消息上下文保存
#[tokio::test]
async fn message_context_save_and_retrieve() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    // 创建一个测试事件
    let test_event = Event {
        type_: Some("im.message.receive_v1".to_string()),
        event_type: None,
        event: Some(serde_json::json!({
            "message": {
                "chat_id": "test_chat_id",
                "content": "{\"text\":\"test\"}",
                "message_type": "text"
            },
            "sender": {
                "sender_id": {
                    "open_id": "user1",
                    "union_id": None::<String>,
                    "user_id": None::<String>,
                },
                "sender_type": "user"
            }
        })),
        challenge: None,
        header: None,
        schema: None,
        token: None,
    };

    // 保存到上下文
    channel.message_context.write().await.insert("test_chat_id".to_string(), test_event.clone());

    // 从上下文读取
    let retrieved = channel.message_context.read().await.get("test_chat_id").cloned();
    assert!(retrieved.is_some());
}

/// 测试克隆功能
#[tokio::test]
async fn channel_clone() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    // 克隆通道
    let cloned_channel = channel.clone();

    // 验证克隆后的通道具有相同的名称
    assert_eq!(channel.name(), cloned_channel.name());
}

/// 测试消息上下文中找不到消息的情况
#[tokio::test]
async fn message_context_not_found() {
    let config = FeishuConfig {
        enabled: false,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: Vec::new(),
    };

    let (inbound_tx, _inbound_rx) = mpsc::channel::<crate::messages::InboundMessage>(16);

    let channel = Feishu::new(config, inbound_tx).await.unwrap();

    // 尝试从不存在的上下文中获取消息
    let context = channel.message_context.read().await;
    let retrieved = context.get("non_existent_chat_id");

    assert!(retrieved.is_none());
}

/// 测试飞书配置序列化和反序列化
#[tokio::test]
async fn feishu_config_serialization() {
    let config = FeishuConfig {
        enabled: true,
        app_id: "test_app_id".to_string(),
        app_secret: "test_app_secret".to_string(),
        allow_from: vec!["user1".to_string(), "user2".to_string()],
    };

    // 序列化
    let serialized = serde_json::to_string(&config).unwrap();

    // 反序列化
    let deserialized: FeishuConfig = serde_json::from_str(&serialized).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.app_id, deserialized.app_id);
    assert_eq!(config.app_secret, deserialized.app_secret);
    assert_eq!(config.allow_from, deserialized.allow_from);
}
