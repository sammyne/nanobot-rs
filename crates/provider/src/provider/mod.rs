//! LLM 提供者抽象层
//!
//! 提供统一的 LLM 提供者接口，支持 OpenAI 和兼容 OpenAI 的服务商。

use anyhow::Result;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};
use nanobot_config::{Config as NanobotConfig, ProviderConfig};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

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

/// OpenAI 提供者
pub struct OpenAIProvider {
    /// 客户端
    client: Client<OpenAIConfig>,

    /// 模型名称
    model: String,

    /// 请求超时（秒）
    timeout: u64,
}

impl OpenAIProvider {
    /// 创建新的 OpenAI 提供者
    pub fn new(config: &ProviderConfig, model: &str) -> Result<Self> {
        Self::new_with_timeout(config, model, 120)
    }

    /// 创建新的 OpenAI 提供者，指定超时时间
    pub fn new_with_timeout(config: &ProviderConfig, model: &str, timeout: u64) -> Result<Self> {
        let api_base = config
            .api_base
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        info!(
            "初始化 OpenAI 提供者: model={}, base_url={}",
            model, api_base
        );

        // 创建自定义配置
        let openai_config = OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(&config.api_key);

        // 创建客户端
        let client = Client::with_config(openai_config);

        Ok(Self {
            client,
            model: model.to_string(),
            timeout,
        })
    }

    /// 从应用配置创建
    pub fn from_config(config: &NanobotConfig) -> Result<Self> {
        let provider_config = config.provider();
        let model = &config.agents.defaults.model;
        Self::new(&provider_config, model)
    }

    /// 将消息转换为 OpenAI 请求消息
    fn convert_messages(&self, messages: &[Message]) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut result = Vec::new();

        for msg in messages {
            let chat_msg = match msg.role.as_str() {
                "system" => ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessageArgs::default()
                        .content(msg.content.as_str())
                        .build()?,
                ),
                "user" => ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(msg.content.as_str())
                        .build()?,
                ),
                "assistant" => {
                    // async-openai 的 assistant 消息需要用不同的方式处理
                    // 暂时使用 user 消息格式
                    ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(msg.content.as_str())
                            .build()?,
                    )
                }
                _ => {
                    warn!("未知的消息角色: {}, 跳过", msg.role);
                    continue;
                }
            };
            result.push(chat_msg);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIProvider {
    async fn chat(&self, messages: &[Message]) -> Result<String> {
        debug!("发送聊天请求, 消息数量: {}", messages.len());

        // 转换消息格式
        let chat_messages = self.convert_messages(messages)?;

        // 构建请求
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(chat_messages)
            .build()?;

        // 发送请求（带超时）
        let response = tokio::time::timeout(
            Duration::from_secs(self.timeout),
            self.client.chat().create(request),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(|e| ProviderError::Api(e.to_string()))?;

        // 提取回复内容
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| ProviderError::Api("响应中没有内容".to_string()))?;

        info!("收到 LLM 响应, 长度: {}", content.len());

        Ok(content)
    }
}

#[cfg(test)]
mod tests;
