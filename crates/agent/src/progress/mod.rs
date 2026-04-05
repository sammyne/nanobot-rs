//! 进度追踪模块
//!
//! 提供进度通知的核心抽象和默认实现。

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::error;

use crate::OutboundMessage;

/// 进度追踪器
///
/// 定义进度通知的核心接口，用于追踪 Agent 的处理进度。
///
/// # 参数说明
///
/// * `content` - 进度内容（思考内容或工具提示）
/// * `is_tool_hint` - 是否为工具提示
///     - `true`: 工具调用提示（如 `web_search("query")`）
///     - `false`: 思考内容（清理后的文本）
///
/// # 返回值
///
/// 成功返回 `Ok(())`，失败返回错误
///
/// # 示例
///
/// ## 使用闭包
///
/// ```rust
/// use nanobot_agent::ProgressTracker;
/// use std::sync::Arc;
///
/// let tracker: Arc<dyn ProgressTracker> = Arc::new(|content: String, is_tool_hint: bool| {
///     println!("[Progress] {} (tool_hint={})", content, is_tool_hint);
/// });
///
/// // 可以在异步上下文中调用
/// // tracker.track("思考中...".to_string(), false).await?;
/// ```
///
/// ## 使用 ChannelProgressTracker
///
/// ```rust
/// use nanobot_agent::{ProgressTracker, ChannelProgressTracker};
/// use tokio::sync::mpsc;
/// use std::sync::Arc;
///
/// # #[tokio::main]
/// # async fn main() {
/// let (tx, mut rx) = mpsc::channel(10);
/// let tracker = Arc::new(ChannelProgressTracker::new(
///     tx,
///     "cli".to_string(),
///     "direct".to_string(),
/// ));
///
/// // 在后台接收进度消息
/// tokio::spawn(async move {
///     while let Some(msg) = rx.recv().await {
///         if msg.is_progress() {
///             println!("[进度] {}", msg.content);
///         }
///     }
/// });
/// # }
/// ```
#[async_trait]
pub trait ProgressTracker: Send + Sync {
    /// 追踪进度
    ///
    /// # 参数
    ///
    /// * `content` - 进度内容
    /// * `is_tool_hint` - 是否为工具提示（true 表示工具调用提示，false 表示思考内容）
    ///
    /// # 返回值
    ///
    /// 成功返回 `Ok(())`，失败返回错误
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()>;
}

/// 通过消息通道发送进度的默认实现
///
/// 适用于交互式模式，通过 `mpsc::Sender<OutboundMessage>` 发送进度消息。
/// 消息格式与 Python 版本保持一致：
/// - `metadata["_progress"]`: 标记为进度消息
/// - `metadata["_tool_hint"]`: 标记是否为工具提示
///
/// # 示例
///
/// ```rust
/// use nanobot_agent::ChannelProgressTracker;
/// use nanobot_agent::ProgressTracker;
/// use tokio::sync::mpsc;
/// use std::sync::Arc;
///
/// # #[tokio::main]
/// # async fn main() {
/// let (tx, mut rx) = mpsc::channel(10);
/// let tracker = ChannelProgressTracker::new(
///     tx,
///     "cli".to_string(),
///     "direct".to_string(),
/// );
///
/// // 发送进度消息
/// tracker.track("正在思考...".to_string(), false).await.unwrap();
///
/// // 接收并验证
/// if let Some(msg) = rx.recv().await {
///     assert!(msg.is_progress());
///     assert!(!msg.is_tool_hint());
/// }
/// # }
/// ```
pub struct ChannelProgressTracker {
    /// 出站消息发送端
    tx: mpsc::Sender<OutboundMessage>,
    /// 通道名称
    channel: String,
    /// 聊天标识
    chat_id: String,
}

impl ChannelProgressTracker {
    /// 创建新的 ChannelProgressTracker
    ///
    /// # 参数
    ///
    /// * `tx` - 出站消息发送端
    /// * `channel` - 通道名称
    /// * `chat_id` - 聊天标识
    pub fn new(tx: mpsc::Sender<OutboundMessage>, channel: String, chat_id: String) -> Self {
        Self { tx, channel, chat_id }
    }
}

#[async_trait]
impl ProgressTracker for ChannelProgressTracker {
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()> {
        // 构造进度消息
        let msg = OutboundMessage::progress(&self.channel, &self.chat_id, content, is_tool_hint);

        // 发送消息
        self.tx.send(msg).await.map_err(|e| {
            error!("发送进度消息失败: {}", e);
            anyhow::anyhow!("发送进度消息失败: {e}")
        })?;

        Ok(())
    }
}

#[async_trait]
impl<F> ProgressTracker for F
where
    F: Fn(String, bool) + Send + Sync,
{
    async fn track(&self, content: String, is_tool_hint: bool) -> Result<()> {
        self(content, is_tool_hint);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
