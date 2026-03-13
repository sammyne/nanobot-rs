//! 消息类型定义
//!
//! 定义通道框架中使用的数据结构，包括入站消息和出站消息。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 入站消息
///
/// 表示从聊天平台接收到的消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    /// 通道名称
    pub channel: String,

    /// 发送者标识
    pub sender_id: String,

    /// 聊天标识
    pub chat_id: String,

    /// 消息文本内容
    pub content: String,

    /// 媒体文件路径列表
    #[serde(default)]
    pub media: Vec<String>,

    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
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
            media: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 添加媒体文件
    pub fn add_media(mut self, media: impl Into<String>) -> Self {
        self.media.push(media.into());
        self
    }

    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// 获取会话标识
    ///
    /// 返回格式为 "channel:chat_id" 的字符串，用于唯一标识一个会话。
    pub fn session_key(&self) -> String {
        format!("{}:{}", self.channel, self.chat_id)
    }
}

/// 出站消息
///
/// 表示要发送到聊天平台的消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    /// 目标通道
    pub channel: String,

    /// 目标聊天
    pub chat_id: String,

    /// 消息文本内容
    pub content: String,

    /// 媒体文件路径
    #[serde(default)]
    pub media: Vec<String>,

    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OutboundMessage {
    /// 创建新的出站消息
    pub fn new(channel: impl Into<String>, chat_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            chat_id: chat_id.into(),
            content: content.into(),
            media: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 创建进度消息
    ///
    /// 进度消息用于通知用户当前处理进度，可选择是否标记为工具提示。
    pub fn progress(content: impl Into<String>, is_tool_hint: bool) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("_progress".to_string(), serde_json::Value::Bool(true));
        if is_tool_hint {
            metadata.insert("_tool_hint".to_string(), serde_json::Value::Bool(true));
        }
        Self { channel: String::new(), chat_id: String::new(), content: content.into(), media: Vec::new(), metadata }
    }

    /// 检查是否为进度消息
    pub fn is_progress(&self) -> bool {
        self.metadata.get("_progress").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    /// 检查是否为工具提示
    pub fn is_tool_hint(&self) -> bool {
        self.metadata.get("_tool_hint").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    /// 添加媒体文件
    pub fn add_media(mut self, media: impl Into<String>) -> Self {
        self.media.push(media.into());
        self
    }

    /// 添加元数据
    pub fn add_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}
