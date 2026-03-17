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

/// 配置文件路径
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| NANOBOT_HOME_DIR.join("config.json"));

/// 默认工作目录路径
pub static DEFAULT_WORKSPACE_PATH: LazyLock<PathBuf> = LazyLock::new(|| NANOBOT_HOME_DIR.join("workspace"));

// 公开导出
pub use schema::{
    AgentDefaults, ChannelsConfig, Config, ConfigError, DingTalkConfig, FeishuConfig, GatewayConfig, HeartbeatConfig,
    McpServerConfig, ProviderConfig, ToolsConfig,
};
