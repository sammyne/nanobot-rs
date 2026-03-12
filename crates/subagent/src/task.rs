//! 子代理任务相关类型定义

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 子代理任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 唯一任务标识符
    pub id: String,
    /// 任务描述
    pub description: String,
    /// 任务标签（可读名称）
    pub label: String,
    /// 来源通道
    pub channel: String,
    /// 聊天标识
    pub chat_id: String,
}

impl Task {
    /// 创建新的任务
    pub fn new(
        description: impl Into<String>,
        label: impl Into<String>,
        channel: impl Into<String>,
        chat_id: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            label: label.into(),
            channel: channel.into(),
            chat_id: chat_id.into(),
        }
    }

    /// 从描述生成标签（取前 30 个字符）
    pub fn label_from_description(description: &str) -> String {
        description.chars().take(30).collect::<String>().trim().to_string()
    }
}
