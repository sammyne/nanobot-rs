//! 配置管理 crate
//!
//! 提供统一的配置加载、验证和管理功能。

use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

mod schema;
mod utils;

/// 用户主目录路径，获取失败时直接 panic
pub static HOME: LazyLock<PathBuf> = LazyLock::new(|| env::home_dir().expect("无法获取用户主目录"));

/// 配置目录路径
pub static NANOBOT_HOME_DIR: LazyLock<PathBuf> = LazyLock::new(|| HOME.join(".nanobot"));

/// 配置文件路径（新建配置的默认路径）
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| NANOBOT_HOME_DIR.join("config.yaml"));

/// 默认工作目录路径
pub static DEFAULT_WORKSPACE_PATH: LazyLock<PathBuf> = LazyLock::new(|| NANOBOT_HOME_DIR.join("workspace"));

/// 配置文件查找候选列表（按优先级排序）
const CONFIG_CANDIDATES: &[&str] = &["config.json", "config.yaml", "config.yml"];

/// 按优先级查找已存在的配置文件路径
///
/// 按 `config.json` > `config.yaml` > `config.yml` 的顺序在 `~/.nanobot/` 下查找，
/// 返回第一个存在的文件路径。如果都不存在则返回 `None`。
pub fn resolve_config_path() -> Option<PathBuf> {
    for filename in CONFIG_CANDIDATES {
        let path = NANOBOT_HOME_DIR.join(filename);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

// 公开导出
pub use schema::{
    AgentDefaults, ChannelsConfig, Config, ConfigError, DingTalkConfig, EmailConfig, ExecToolConfig, FeishuConfig,
    GatewayConfig, HeartbeatConfig, ImapConfig, McpServerConfig, ProviderConfig, ProvidersConfig, ReasoningEffort,
    SmtpConfig, ToolsConfig,
};
