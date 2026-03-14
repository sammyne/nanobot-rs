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

use std::io::{
    Write, {self},
};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use tracing::{debug, info};

pub mod gateway;
pub mod mcp;

pub use gateway::GatewayConfig;

/// 用户主目录路径，获取失败时直接 panic
pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| env::home_dir().expect("无法获取用户主目录"));

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

/// 配置文件名称
pub const CONFIG_FILE_NAME: &str = "config.json";

/// 配置目录名称
pub const CONFIG_DIR_NAME: &str = ".nanobot";

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
        if self.enabled {
            if self.client_id.is_empty() {
                return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_id".to_string()));
            }
            if self.client_secret.is_empty() {
                return Err(ConfigError::Validation("启用的钉钉通道必须配置 client_secret".to_string()));
            }
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
    pub dingtalk: Option<DingTalkConfig>,
}

/// 工具配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsConfig {
    /// MCP 服务器配置
    #[serde(default)]
    pub mcp_servers: std::collections::HashMap<String, mcp::McpServerConfig>,
}

/// 应用配置（兼容 HKUDS 版本）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub providers: ProvidersSection,

    #[serde(default)]
    pub agents: AgentsSection,

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

/// Providers 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersSection {
    #[serde(default)]
    pub custom: Option<ProviderConfig>,
}

/// Agents 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentsSection {
    #[serde(default)]
    pub defaults: AgentDefaults,
}

/// Agent 默认配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefaults {
    /// 工作目录路径
    #[serde(default = "default_workspace", deserialize_with = "deserialize_path_with_tilde")]
    pub workspace: PathBuf,

    /// 模型名称
    #[serde(default = "default_model")]
    pub model: String,

    /// 最大 token 数
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// 温度参数
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// 最大工具迭代次数
    #[serde(default = "default_max_tool_iterations")]
    pub max_tool_iterations: usize,

    /// 记忆窗口大小
    #[serde(default = "default_memory_window")]
    pub memory_window: usize,
}

fn default_workspace() -> PathBuf {
    HOME.join(CONFIG_DIR_NAME).join("workspace")
}

fn default_model() -> String {
    "anthropic/claude-opus-4-5".to_string()
}

fn default_max_tokens() -> usize {
    8192
}

fn default_temperature() -> f64 {
    0.1
}

fn default_max_tool_iterations() -> usize {
    40
}

fn default_memory_window() -> usize {
    100
}

/// 反序列化路径，将 ~ 替换为用户主目录
fn deserialize_path_with_tilde<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path: PathBuf = Deserialize::deserialize(deserializer)?;
    Ok(expand_tilde(&path))
}

/// 将路径中的 ~ 替换为用户主目录
fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(first) = path.iter().next()
        && first == "~"
    {
        let mut new_path = HOME.clone();
        for component in path.iter().skip(1) {
            new_path.push(component);
        }
        return new_path;
    }
    path.to_path_buf()
}

impl Default for AgentDefaults {
    fn default() -> Self {
        Self {
            workspace: default_workspace(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            max_tool_iterations: default_max_tool_iterations(),
            memory_window: default_memory_window(),
        }
    }
}

impl Config {
    /// 创建新配置
    pub fn new(provider: ProviderConfig) -> Self {
        Self {
            providers: ProvidersSection { custom: Some(provider) },
            agents: AgentsSection::default(),
            channels: ChannelsConfig::default(),
            gateway: GatewayConfig::default(),
            tools: ToolsConfig::default(),
        }
    }

    /// 获取 ProviderConfig（兼容简化版接口）
    pub fn provider(&self) -> ProviderConfig {
        if let Some(custom) = &self.providers.custom { custom.clone() } else { ProviderConfig::default() }
    }

    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(HOME.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
    }

    /// 获取配置目录路径
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        Ok(HOME.join(CONFIG_DIR_NAME))
    }

    /// 从指定路径加载配置（内部实现）
    ///
    /// 统一的配置加载逻辑，供测试和生产环境共用。
    fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound("配置文件不存在，请运行 'nanobot onboard' 进行配置".to_string()));
        }

        debug!("从 {:?} 加载配置", path);

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
        info!("配置加载成功");

        Ok(config)
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
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;
        Self::load_from_path(&path)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_path()?;

        // 创建配置目录（如果不存在）
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
            debug!("创建配置目录: {:?}", config_dir);
        }

        // 序列化配置为 JSON
        let content = serde_json::to_string_pretty(self)?;

        // 写入文件
        let mut file = fs::File::create(&config_path)?;

        // 设置文件权限为 600（仅当前用户可读写）
        file.set_permissions(fs::Permissions::from_mode(0o600))?;

        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        info!("配置保存到 {:?}", config_path);

        Ok(())
    }

    /// 检查配置文件是否存在
    pub fn exists() -> bool {
        if let Ok(path) = Self::config_path() { path.exists() } else { false }
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

        // 验证 gateway 配置
        self.gateway.validate()?;

        debug!("配置验证通过");
        Ok(())
    }

    /// 脱敏的 API Key（用于日志显示）
    pub fn masked_api_key(&self) -> String {
        let key = self.providers.custom.as_ref().map(|c| c.api_key.as_str()).unwrap_or("");

        if key.len() <= 8 {
            return "*".repeat(key.len());
        }

        let start = &key[..4];
        let end = &key[key.len() - 4..];
        format!("{start}****{end}")
    }
}

#[cfg(test)]
mod tests;
