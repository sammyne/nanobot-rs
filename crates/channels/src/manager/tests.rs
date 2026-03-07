use std::collections::HashMap;

use super::*;

/// 测试通道管理器创建
#[tokio::test]
async fn channel_manager_creation() {
    let config = ChannelsConfig {
        dingtalk: None,
        others: HashMap::new(),
    };

    let manager = ChannelManager::new(config).await;
    assert!(manager.is_ok());

    let manager = manager.unwrap();
    assert_eq!(manager.get_status().await.len(), 0);
}
