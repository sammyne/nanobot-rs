//! 统一 Provider 枚举包装器
//!
//! 根据 `ProvidersConfig` 枚举变体自动选择具体的 Provider 实现。

use anyhow::Result;
use nanobot_config::{Config as NanobotConfig, ProvidersConfig};
use nanobot_tools::ToolDefinition;

use crate::anthropic::AnthropicLike;
use crate::auto_retry::AutoRetryProvider;
use crate::openai::OpenAILike;
use crate::{Message, MeteredMessage, Options, Provider};

/// 统一 Provider 枚举，包装所有支持的 Provider 实现
#[derive(Clone)]
pub enum AnyProvider {
    /// OpenAI 兼容端点（async-openai 内部已有重试）
    OpenAI(OpenAILike),
    /// Anthropic Messages API（通过 AutoRetryProvider 添加重试能力）
    Anthropic(AutoRetryProvider<AnthropicLike>),
}

impl AnyProvider {
    /// 根据应用配置创建对应的 Provider
    ///
    /// 通过 `config.providers` 枚举变体决定使用哪个 Provider：
    /// - `ProvidersConfig::Custom` → `OpenAILike`
    /// - `ProvidersConfig::Anthropic` → `AutoRetryProvider<AnthropicLike>`
    pub fn from_config(config: &NanobotConfig) -> Result<Self> {
        let model = &config.agents.defaults.model;
        match &config.providers {
            ProvidersConfig::Custom(pc) => {
                let provider = OpenAILike::new(pc, model)?;
                Ok(Self::OpenAI(provider))
            }
            ProvidersConfig::Anthropic(pc) => {
                let provider = AnthropicLike::new(pc, model)?;
                Ok(Self::Anthropic(AutoRetryProvider::new(provider)))
            }
        }
    }
}

#[async_trait::async_trait]
impl Provider for AnyProvider {
    async fn chat(&self, messages: &[Message], options: &Options) -> Result<MeteredMessage> {
        match self {
            Self::OpenAI(p) => p.chat(messages, options).await,
            Self::Anthropic(p) => p.chat(messages, options).await,
        }
    }

    fn bind_tools(&mut self, tools: Vec<ToolDefinition>) {
        match self {
            Self::OpenAI(p) => p.bind_tools(tools),
            Self::Anthropic(p) => p.bind_tools(tools),
        }
    }
}

#[cfg(test)]
mod tests;
