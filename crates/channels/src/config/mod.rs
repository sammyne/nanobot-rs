//! 配置管理模块
//!
//! 定义通道配置结构。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{ChannelError, ChannelResult};

/// 钉钉通道配置
///
/// 钉钉通道的配置字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub fn validate(&self) -> ChannelResult<()> {
        if self.enabled {
            if self.client_id.is_empty() {
                return Err(ChannelError::ConfigError(
                    "启用的钉钉通道必须配置 client_id".to_string(),
                ));
            }
            if self.client_secret.is_empty() {
                return Err(ChannelError::ConfigError(
                    "启用的钉钉通道必须配置 client_secret".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// 所有通道的配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelsConfig {
    /// 钉钉通道配置
    #[serde(default)]
    pub dingtalk: Option<DingTalkConfig>,

    /// 其他通道的配置（使用动态类型）
    #[serde(default)]
    pub others: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests;
