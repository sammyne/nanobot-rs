//! 核心通道抽象
//!
//! 定义了所有通道实现必须遵循的接口。

use async_trait::async_trait;

use crate::error::ChannelResult;
use crate::messages::OutboundMessage;

/// 通道 trait
///
/// 所有通道实现都必须实现此 trait，以提供统一的接口。
#[async_trait]
pub trait Channel: Send + Sync {
    /// 启动通道
    ///
    /// 初始化通道资源并开始接收消息。
    /// 此方法应该实现通道的初始化逻辑，如建立 WebSocket 连接等。
    ///
    /// # 错误
    ///
    /// 如果通道启动失败，返回 `ChannelError::StartFailed`。
    async fn start(&self) -> ChannelResult<()>;

    /// 停止通道
    ///
    /// 清理通道资源并停止接收消息。
    /// 此方法应该取消所有后台任务并关闭连接。
    ///
    /// # 错误
    ///
    /// 如果通道停止失败，返回 `ChannelError::StopFailed`。
    async fn stop(&self) -> ChannelResult<()>;

    /// 发送消息
    ///
    /// 通过通道发送出站消息到目标聊天。
    ///
    /// # 参数
    ///
    /// * `msg` - 要发送的消息
    ///
    /// # 错误
    ///
    /// 如果消息发送失败，返回 `ChannelError::SendFailed`。
    async fn send(&self, msg: OutboundMessage) -> ChannelResult<()>;

    /// 检查通道是否正在运行
    ///
    /// 返回通道的当前运行状态。
    fn is_running(&self) -> bool;

    /// 获取通道名称
    ///
    /// 返回通道的唯一标识符。
    fn name(&self) -> &str;
}
