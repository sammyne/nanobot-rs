//! Provider 配置模块
//!
//! 定义 LLM 提供者的配置。

use serde::{Deserialize, Serialize};

/// LLM 提供者配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// API Key
    #[serde(default)]
    pub api_key: String,

    /// API Base URL
    #[serde(default)]
    pub api_base: Option<String>,

    /// 自定义请求头（例如 AiHubMix 的 APP-Code、Anthropic 的 anthropic-beta）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

/// Providers 配置段
///
/// 使用 serde 外部标签格式，JSON 中通过键名区分 provider 类型：
/// - `{"custom": {"apiKey": "..."}}`：OpenAI 兼容端点
/// - `{"anthropic": {"apiKey": "..."}}`：Anthropic Messages API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProvidersConfig {
    /// OpenAI 兼容端点（默认）
    Custom(ProviderConfig),
    /// Anthropic Messages API
    Anthropic(ProviderConfig),
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self::Custom(ProviderConfig::default())
    }
}

impl ProvidersConfig {
    /// 获取内部 `ProviderConfig` 的引用
    pub fn provider_config(&self) -> &ProviderConfig {
        match self {
            Self::Custom(config) | Self::Anthropic(config) => config,
        }
    }
}
