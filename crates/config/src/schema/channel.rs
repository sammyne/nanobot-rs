//! Channel 配置模块
//!
//! 定义各种通信通道的配置。

use serde::{Deserialize, Serialize};

use super::ConfigError;

/// 钉钉通道配置
///
/// 钉钉通道的配置字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DingTalkConfig {
    /// 是否启用此通道
    #[serde(default)]
    pub enabled: bool,

    /// Client ID (AppKey)
    #[serde(default)]
    pub client_id: String,

    /// Client Secret (AppSecret)
    #[serde(default)]
    pub client_secret: String,

    /// 允许的用户列表（为空则允许所有用户）
    #[serde(default)]
    pub allow_from: Vec<String>,
}

impl DingTalkConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.client_id.is_empty() {
            return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_id".to_string()));
        }
        if self.client_secret.is_empty() {
            return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_secret".to_string()));
        }

        Ok(())
    }
}

/// 飞书通道配置
///
/// 飞书通道的配置字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeishuConfig {
    /// 是否启用此通道
    #[serde(default)]
    pub enabled: bool,

    /// App ID
    #[serde(default)]
    pub app_id: String,

    /// App Secret
    #[serde(default)]
    pub app_secret: String,

    /// 允许的用户列表（为空则允许所有用户）
    #[serde(default)]
    pub allow_from: Vec<String>,
}

impl FeishuConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.app_id.is_empty() {
            return Err(ConfigError::Validation("启用的飞书通道必须配置 app_id".to_string()));
        }
        if self.app_secret.is_empty() {
            return Err(ConfigError::Validation("启用的飞书通道必须配置 app_secret".to_string()));
        }

        Ok(())
    }
}

/// 所有通道的配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelsConfig {
    /// 钉钉通道配置
    #[serde(default)]
    pub dingtalk: DingTalkConfig,

    /// 飞书通道配置
    #[serde(default)]
    pub feishu: FeishuConfig,
}
