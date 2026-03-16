//! Agent 配置模块
//!
//! 定义 Agent 的默认配置。

use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use crate::DEFAULT_WORKSPACE_PATH;
use crate::utils::expand_tilde;

/// Agents 配置段
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentsConfig {
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
    DEFAULT_WORKSPACE_PATH.clone()
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
