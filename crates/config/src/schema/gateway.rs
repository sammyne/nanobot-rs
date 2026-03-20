//! Gateway 配置模块
//!
//! 定义网关服务的监听配置。

use serde::{Deserialize, Serialize};

use super::ConfigError;

/// Heartbeat 服务配置
///
/// 独立定义以避免循环依赖
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatConfig {
    /// 是否启用心跳服务
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// 心跳检查间隔（秒）
    #[serde(default = "default_interval")]
    pub interval_s: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_interval() -> u64 {
    1800
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self { enabled: default_enabled(), interval_s: default_interval() }
    }
}

impl std::fmt::Display for HeartbeatConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HeartbeatConfig {{ enabled: {}, interval_s: {} }}", self.enabled, self.interval_s)
    }
}

impl HeartbeatConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.interval_s == 0 {
            return Err("interval_s must be greater than 0".to_string());
        }
        Ok(())
    }
}

/// 网关配置
///
/// 网关服务的监听参数配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,

    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,

    /// 心跳配置
    #[serde(default)]
    pub heartbeat: HeartbeatConfig,

    /// 健康检查服务端口（可选）
    #[serde(default)]
    pub health_check_port: Option<u16>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    18790
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            heartbeat: HeartbeatConfig::default(),
            health_check_port: None,
        }
    }
}

impl GatewayConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证 port > 0
        if self.port == 0 {
            return Err(ConfigError::Validation("gateway.port 必须大于 0".to_string()));
        }

        // 验证 host 非空
        if self.host.is_empty() {
            return Err(ConfigError::Validation("gateway.host 不能为空".to_string()));
        }

        // 验证 heartbeat 配置
        self.heartbeat.validate().map_err(|e| ConfigError::Validation(format!("gateway.heartbeat 配置错误: {e}")))?;

        // 验证 health_check_port 配置
        if let Some(port) = self.health_check_port
            && port == 0
        {
            return Err(ConfigError::Validation("gateway.health_check_port 必须大于 0".to_string()));
        }

        Ok(())
    }
}
