//! Gateway 配置模块
//!
//! 定义网关服务的监听配置。

use serde::{Deserialize, Serialize};

use super::ConfigError;

/// 网关配置
///
/// 网关服务的监听参数配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,

    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
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

        Ok(())
    }
}
