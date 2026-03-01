//! MessageBus 消息总线实现
//!
//! 提供异步消息队列，用于在 AgentLoop 和 CLI 之间路由消息。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 入站消息（用户发送到Agent）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    /// 渠道标识（如 "cli", "telegram" 等）
    pub channel: String,
    /// 发送者ID
    pub sender_id: String,
    /// 聊天会话ID
    pub chat_id: String,
    /// 消息内容
    pub content: String,
    /// 额外元数据
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl InboundMessage {
    /// 创建新的入站消息
    pub fn new(
        channel: impl Into<String>,
        sender_id: impl Into<String>,
        chat_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            channel: channel.into(),
            sender_id: sender_id.into(),
            chat_id: chat_id.into(),
            content: content.into(),
            metadata: None,
        }
    }

    /// 解析session_id为(channel, chat_id)
    pub fn from_session_id(
        session_id: &str,
        content: impl Into<String>,
    ) -> Option<(String, String, String)> {
        let parts: Vec<&str> = session_id.split(':').collect();
        if parts.len() >= 2 {
            let channel = parts[0].to_string();
            let chat_id = parts[1..].join(":"); // 处理chat_id中包含冒号的情况
            let _content = content.into();
            Some((channel, "user".to_string(), chat_id))
        } else {
            None
        }
    }
}

/// 出站消息（Agent响应到用户）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    /// 渠道标识
    pub channel: String,
    /// 聊天会话ID
    pub chat_id: String,
    /// 消息内容
    pub content: String,
    /// 额外元数据（可包含_progress、_tool_hint等）
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OutboundMessage {
    /// 创建新的出站消息
    pub fn new(
        channel: impl Into<String>,
        chat_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            channel: channel.into(),
            chat_id: chat_id.into(),
            content: content.into(),
            metadata: HashMap::new(),
        }
    }

    /// 创建进度消息
    pub fn progress(content: impl Into<String>, is_tool_hint: bool) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("_progress".to_string(), serde_json::Value::Bool(true));
        if is_tool_hint {
            metadata.insert("_tool_hint".to_string(), serde_json::Value::Bool(true));
        }
        Self {
            channel: String::new(),
            chat_id: String::new(),
            content: content.into(),
            metadata,
        }
    }

    /// 检查是否为进度消息
    pub fn is_progress(&self) -> bool {
        self.metadata
            .get("_progress")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// 检查是否为工具提示
    pub fn is_tool_hint(&self) -> bool {
        self.metadata
            .get("_tool_hint")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_basic() {
        // 测试入站消息
        let (inbound_tx, mut inbound_rx) = tokio::sync::mpsc::channel(1);
        let inbound = InboundMessage::new("cli", "user", "chat1", "Hello");
        inbound_tx.send(inbound.clone()).await.unwrap();

        let received = inbound_rx.recv().await.unwrap();
        assert_eq!(received.channel, "cli");
        assert_eq!(received.content, "Hello");

        // 测试出站消息
        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel(1);
        let outbound = OutboundMessage::new("cli", "chat1", "World");
        outbound_tx.send(outbound).await.unwrap();

        let received = outbound_rx.recv().await.unwrap();
        assert_eq!(received.content, "World");
    }

    #[tokio::test]
    async fn test_progress_message() {
        let progress = OutboundMessage::progress("thinking...", false);
        assert!(progress.is_progress());
        assert!(!progress.is_tool_hint());

        let tool_hint = OutboundMessage::progress("using tool...", true);
        assert!(tool_hint.is_progress());
        assert!(tool_hint.is_tool_hint());
    }

    #[test]
    fn test_parse_session_id() {
        let result = InboundMessage::from_session_id("cli:direct", "hello");
        assert!(result.is_some());
        let (channel, sender_id, chat_id) = result.unwrap();
        assert_eq!(channel, "cli");
        assert_eq!(sender_id, "user");
        assert_eq!(chat_id, "direct");

        // 测试包含多个冒号的情况
        let result = InboundMessage::from_session_id("telegram:123:456", "hello");
        assert!(result.is_some());
        let (channel, _sender_id, chat_id) = result.unwrap();
        assert_eq!(channel, "telegram");
        assert_eq!(chat_id, "123:456");
    }
}
