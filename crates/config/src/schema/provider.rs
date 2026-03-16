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

    /// 自定义请求头（例如 AiHubMix 的 APP-Code）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

/// Providers 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersConfig {
    #[serde(default)]
    pub custom: Option<ProviderConfig>,
}
