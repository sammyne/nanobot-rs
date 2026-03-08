use tokio::sync::mpsc;

use super::*;

/// 测试通道管理器创建
#[tokio::test]
async fn channel_manager_creation() {
    let config = ChannelsConfig::default();

    let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMessage>(16);
    let (inbound_tx, inbound_rx) = mpsc::channel::<InboundMessage>(16);

    // 发送端不需要在这个测试中使用，但需要保留以避免通道关闭
    drop(outbound_tx);
    drop(inbound_rx);

    let manager = ChannelManager::new(config, outbound_rx, inbound_tx).await;
    assert!(manager.is_ok());

    let manager = manager.unwrap();
    assert_eq!(manager.get_status().await.len(), 0);
}
