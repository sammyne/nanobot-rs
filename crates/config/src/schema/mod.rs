//! 配置管理模块
//!
//! 负责加载、保存和验证 nanobot 的配置文件。
//!
//! # 配置文件格式
//!
//! 配置文件位于 `~/.nanobot/config.json`，采用 JSON 格式。
//!
//! # 配置示例
//!
//! ```json
//! {
//!   "providers": {
//!     "custom": {
//!       "apiKey": "sk-your-api-key",
//!       "apiBase": "https://api.example.com/v1"
//!     }
//!   },
//!   "agents": {
//!     "defaults": {
//!       "workspace": "~/nanobot/workspace",
//!       "model": "anthropic/claude-opus-4-5",
//!       "maxTokens": 8192,
//!       "temperature": 0.1
//!     }
//!   },
//!   "channels": {
//!     "dingtalk": {
//!       "enabled": true,
//!       "clientId": "your-client-id",
//!       "clientSecret": "your-client-secret",
//!       "allowFrom": ["user1", "user2"]
//!     }
//!   },
//!   "gateway": {
//!     "host": "0.0.0.0",
//!     "port": 18790
//!   },
//!   "tools": {
//!     "mcpServers": {
//!       "filesystem": {
//!         "command": "npx",
//!         "args": ["@modelcontextprotocol/server-filesystem", "/path/to/allowed/directory"],
//!         "env": {
//!           "NODE_ENV": "production"
//!         }
//!       },
//!       "remote-mcp": {
//!         "url": "https://mcp-server.example.com/sse",
//!         "headers": {
//!           "Authorization": "Bearer your-token"
//!         },
//!         "toolTimeout": 30
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! # MCP 服务器配置
//!
//! MCP（Model Context Protocol）服务器配置位于 `tools.mcpServers` 字段中。
//!
//! ## Stdio 类型（本地进程）
//!
//! ```json
//! {
//!   "command": "npx",
//!   "args": ["@modelcontextprotocol/server-filesystem", "/path/to/dir"],
//!   "env": {
//!     "NODE_ENV": "production"
//!   }
//! }
//! ```
//!
//! - `command`: 要执行的命令（必需）
//! - `args`: 命令行参数数组（可选）
//! - `env`: 环境变量键值对（可选）
//!
//! ## HTTP 类型（远程服务）
//!
//! ```json
//! {
//!   "url": "https://mcp-server.example.com/sse",
//!   "headers": {
//!     "Authorization": "Bearer your-token"
//!   },
//!   "toolTimeout": 30
//! }
//! ```
//!
//! - `url`: 服务器 URL（必需）
//! - `headers`: HTTP 请求头键值对（可选）
//! - `toolTimeout`: 工具调用超时时间，单位秒（可选，默认 30）
//!
//! # 配置验证
//!
//! 配置加载时会自动验证：
//! - `apiKey` 长度必须大于等于 3
//! - `workspace` 不能为空
//! - `model` 不能为空
//! - `maxTokens` 必须大于 0
//! - 如果 `apiBase` 不为空，必须以 `http://` 或 `https://` 开头
//! - 如果启用的钉钉通道必须配置 `clientId` 和 `clientSecret`

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{CONFIG_PATH, NANOBOT_HOME_DIR};

mod agent;
mod channel;
mod gateway;
mod mcp;
mod provider;
mod tools;

pub use agent::{AgentDefaults, AgentsConfig};
pub use channel::{ChannelsConfig, DingTalkConfig, FeishuConfig};
pub use gateway::{GatewayConfig, HeartbeatConfig};
pub use mcp::McpServerConfig;
pub use provider::{ProviderConfig, ProvidersConfig};
pub use tools::ToolsConfig;

pub use crate::utils::expand_tilde;

/// 配置相关错误
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件不存在: {0}")]
    NotFound(String),

    #[error("配置文件格式错误: {0}")]
    Parse(String),

    #[error("配置验证失败: {0}")]
    Validation(String),

    #[error("IO 错误: {0}")]
    Io(#[from] io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(String),

    #[error("环境变量解析错误: {0}")]
    Environment(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        ConfigError::Json(e.to_string())
    }
}

/// 应用配置（兼容 HKUDS 版本）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub providers: ProvidersConfig,

    #[serde(default)]
    pub agents: AgentsConfig,

    /// 通道配置
    #[serde(default)]
    pub channels: ChannelsConfig,

    /// 网关配置
    #[serde(default)]
    pub gateway: GatewayConfig,

    /// 工具配置
    #[serde(default)]
    pub tools: ToolsConfig,
}

impl Config {
    /// 创建新配置
    pub fn new(provider: ProviderConfig) -> Self {
        Self {
            providers: ProvidersConfig { custom: Some(provider) },
            agents: AgentsConfig::default(),
            channels: ChannelsConfig::default(),
            gateway: GatewayConfig::default(),
            tools: ToolsConfig::default(),
        }
    }

    /// 获取 ProviderConfig（兼容简化版接口）
    pub fn provider(&self) -> ProviderConfig {
        if let Some(custom) = &self.providers.custom { custom.clone() } else { ProviderConfig::default() }
    }

    /// 从指定路径加载配置（内部实现）
    ///
    /// 统一的配置加载逻辑，供测试和生产环境共用。
    /// 从指定路径加载配置
    ///
    /// # 返回值
    ///
    /// - `Ok(Some(config))` - 配置文件存在且加载成功
    /// - `Ok(None)` - 配置文件不存在
    /// - `Err(e)` - 配置加载或验证失败
    fn load_from_path(path: &Path) -> Result<Option<Self>, ConfigError> {
        if !path.exists() {
            return Ok(None);
        }

        // 使用 config 库统一从文件和环境变量加载配置
        // 环境变量使用 convert_case 将 snake_case 转换为 camelCase，与 JSON 文件的 key 匹配
        let mut config: Config = config::Config::builder()
            .add_source(config::File::from(path).format(config::FileFormat::Json))
            .add_source(
                config::Environment::with_prefix("NANOBOT")
                    .prefix_separator("_")
                    .separator("__")
                    .convert_case(config::Case::Camel)
                    .ignore_empty(true),
            )
            .build()
            .map_err(|e| ConfigError::Parse(format!("配置加载失败: {e}")))?
            .try_deserialize()
            .map_err(|e| ConfigError::Parse(format!("配置反序列化失败: {e}")))?;

        // 处理路径中的 ~ 展开
        config.agents.defaults.workspace = expand_tilde(&config.agents.defaults.workspace);

        config.validate()?;

        Ok(Some(config))
    }

    /// 从文件加载配置
    ///
    /// 配置加载顺序：
    /// 1. 从 `~/.nanobot/config.json` 加载基础配置
    /// 2. 从环境变量覆盖配置项（环境变量优先级更高）
    ///
    /// # 环境变量命名规范
    ///
    /// - 前缀：`NANOBOT_`（单下划线）
    /// - 层级分隔符：双下划线 `__`
    /// - 字段命名：snake_case（如 `API_KEY` 而非 `APIKEY`）
    ///
    /// ## 示例
    ///
    /// | 配置路径 | 环境变量 |
    /// |---------|---------|
    /// | `providers.custom.apiKey` | `NANOBOT_PROVIDERS__CUSTOM__API_KEY` |
    /// | `agents.defaults.model` | `NANOBOT_AGENTS__DEFAULTS__MODEL` |
    /// | `gateway.port` | `NANOBOT_GATEWAY__PORT` |
    ///
    /// # 返回值
    ///
    /// - `Ok(Some(config))` - 配置文件存在且加载成功
    /// - `Ok(None)` - 配置文件不存在
    /// - `Err(e)` - 配置加载或验证失败
    pub fn load() -> Result<Option<Self>, ConfigError> {
        let path = CONFIG_PATH.clone();
        Self::load_from_path(&path)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = NANOBOT_HOME_DIR.clone();
        let config_path = CONFIG_PATH.clone();

        // 创建配置目录（如果不存在）
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // 序列化配置为 JSON
        let content = serde_json::to_string_pretty(self)?;

        // 写入文件
        let mut file = fs::File::create(&config_path)?;

        // 设置文件权限为 600（仅当前用户可读写）
        file.set_permissions(fs::Permissions::from_mode(0o600))?;

        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证 agents.defaults
        let defaults = &self.agents.defaults;

        // 验证 workspace
        if defaults.workspace.as_os_str().is_empty() {
            return Err(ConfigError::Validation("workspace 不能为空".to_string()));
        }

        // 验证 model
        if defaults.model.is_empty() {
            return Err(ConfigError::Validation("model 不能为空".to_string()));
        }

        // 验证 max_tokens
        if defaults.max_tokens == 0 {
            return Err(ConfigError::Validation("max_tokens 必须大于 0".to_string()));
        }

        // 验证 providers.custom
        if let Some(custom) = &self.providers.custom {
            // api_base 可以是 None（使用默认值）或有效 URL
            if let Some(api_base) = &custom.api_base
                && !api_base.is_empty()
                && !api_base.starts_with("http://")
                && !api_base.starts_with("https://")
            {
                return Err(ConfigError::Validation("api_base 必须以 http:// 或 https:// 开头".to_string()));
            }

            // api_key 可以是 None（某些 OAuth 提供者不需要）
            // 如果不是空字符串，验证长度
            if !custom.api_key.is_empty() && custom.api_key.len() < 3 {
                return Err(ConfigError::Validation("api_key 长度不能少于 3 个字符".to_string()));
            }
        }

        // 验证 channels 配置
        if let Some(dingtalk) = &self.channels.dingtalk {
            dingtalk.validate()?;
        }

        if let Some(feishu) = &self.channels.feishu {
            feishu.validate()?;
        }

        // 验证 gateway 配置
        self.gateway.validate()?;

        Ok(())
    }

    /// 脱敏的 API Key（用于日志显示）
    pub fn masked_api_key(&self) -> String {
        let key = self.providers.custom.as_ref().map(|c| c.api_key.as_str()).unwrap_or("");
        nanobot_utils::strings::redact(key)
    }
}

#[cfg(test)]
mod tests;
