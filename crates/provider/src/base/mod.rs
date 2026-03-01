//! Provider 基础 trait 和类型定义

use anyhow::Result;
use thiserror::Error;

/// 提供者相关错误
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("LLM API 调用失败: {0}")]
    Api(String),

    #[error("请求超时")]
    Timeout,

    #[error("配置错误: {0}")]
    Config(String),
}

/// 聊天消息
#[derive(Debug, Clone)]
pub struct Message {
    /// 角色（user/assistant/system）
    pub role: String,

    /// 消息内容
    pub content: String,
}

impl Message {
    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// LLM 提供者 trait
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, messages: &[Message]) -> Result<String>;
}
