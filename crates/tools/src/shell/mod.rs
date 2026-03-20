//! Shell 执行工具
//!
//! 提供安全的 Shell 命令执行功能。

mod utils;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::LazyLock;
use std::time::Duration;

use async_trait::async_trait;
use regex::Regex;
use schemars::schema::SchemaObject;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info};
use utils::{detect_path_traversal, extract_absolute_paths, truncate_output};

use crate::core::{Tool, ToolContext, ToolError, ToolResult, optional_param, require_param, u64_param};

/// Shell 执行结果
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShellResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Shell 工具配置选项
#[derive(Debug, Clone)]
pub struct ShellToolOptions {
    /// 正则表达式拒绝模式列表
    pub deny_patterns: Vec<String>,
    /// 正则表达式允许模式列表（白名单）
    pub allow_patterns: Vec<String>,
    /// 是否限制在工作空间内
    pub restrict_to_workspace: bool,
    /// 要追加到 PATH 环境变量的路径
    pub path_append: String,
    /// 命令执行超时时间（秒）
    pub timeout: u64,
    /// 工作空间路径（可选，默认为当前目录）
    pub workspace: Option<PathBuf>,
}

impl Default for ShellToolOptions {
    fn default() -> Self {
        Self {
            deny_patterns: vec![
                r"\brm\s+-[rf]{1,2}\b".to_string(),            // rm -r, rm -rf, rm -fr
                r"\bdel\s+/[fq]\b".to_string(),                // del /f, del /q
                r"\brmdir\s+/s\b".to_string(),                 // rmdir /s
                r"(?:^|[;&|]\s*)format\b".to_string(),         // format (as standalone command only)
                r"\b(mkfs|diskpart)\b".to_string(),            // disk operations
                r"\bdd\s+if=".to_string(),                     // dd
                r">\s*/dev/sd".to_string(),                    // write to disk
                r"\b(shutdown|reboot|poweroff)\b".to_string(), // system power
                r":\(\)\s*\{.*\};\s*:".to_string(),            // fork bomb
            ],
            allow_patterns: vec![],
            restrict_to_workspace: false,
            path_append: String::new(),
            timeout: 60,
            workspace: None,
        }
    }
}

/// Shell 工具
pub struct ShellTool {
    options: ShellToolOptions,
    deny_patterns: Vec<Regex>,
    allow_patterns: Option<Vec<Regex>>,
}

/// ShellTool 的参数 Schema
static SHELL_PARAMETERS_SCHEMA: LazyLock<SchemaObject> = LazyLock::new(|| {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "要执行的 shell 命令"
            },
            "cwd": {
                "type": "string",
                "description": "工作目录（可选，默认为 workspace）"
            },
            "timeout_ms": {
                "type": "integer",
                "description": "超时时间（毫秒，可选，默认60秒）",
                "default": 60000
            }
        },
        "required": ["command"]
    }))
    .expect("JSON schema is valid for shell")
});

impl ShellTool {
    /// 从配置选项创建 ShellTool 实例
    pub fn new(options: ShellToolOptions) -> Self {
        let deny_patterns: Vec<Regex> = options.deny_patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();

        let allow_patterns: Option<Vec<Regex>> = if options.allow_patterns.is_empty() {
            None
        } else {
            Some(options.allow_patterns.iter().filter_map(|p| Regex::new(p).ok()).collect())
        };

        Self { options, deny_patterns, allow_patterns }
    }

    /// 设置默认超时
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.options.timeout = secs;
        self
    }

    /// 检查命令是否匹配拒绝模式
    fn check_deny_patterns(&self, cmd: &str) -> Result<(), ToolError> {
        let cmd_lower = cmd.to_lowercase();
        for pattern in &self.deny_patterns {
            if pattern.is_match(&cmd_lower) {
                return Err(ToolError::PermissionDenied {
                    path: cmd.to_string(),
                    allowed: Some("危险命令被拒绝 (dangerous pattern detected)".to_string()),
                });
            }
        }
        Ok(())
    }

    /// 检查命令是否匹配允许模式
    fn check_allow_patterns(&self, cmd: &str) -> Result<(), ToolError> {
        if let Some(ref patterns) = self.allow_patterns {
            let cmd_lower = cmd.to_lowercase();
            if !patterns.iter().any(|p| p.is_match(&cmd_lower)) {
                return Err(ToolError::PermissionDenied {
                    path: cmd.to_string(),
                    allowed: Some("命令被拒绝 (not in allowlist)".to_string()),
                });
            }
        }
        Ok(())
    }

    /// 验证命令中的路径是否在工作空间内
    fn validate_paths_in_workspace(&self, cmd: &str, cwd: &Path) -> Result<(), ToolError> {
        for raw in extract_absolute_paths(cmd) {
            let path = PathBuf::from(raw.trim());
            if path.is_absolute() {
                // 尝试规范化路径
                let canonical_path = if path.exists() { path.canonicalize().unwrap_or(path) } else { path };
                let canonical_cwd = if cwd.exists() {
                    cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf())
                } else {
                    cwd.to_path_buf()
                };

                // 检查路径是否在工作空间内
                if !canonical_path.starts_with(&canonical_cwd) && canonical_path != canonical_cwd {
                    return Err(ToolError::PermissionDenied {
                        path: cmd.to_string(),
                        allowed: Some("路径超出工作空间范围 (path outside working dir)".to_string()),
                    });
                }
            }
        }
        Ok(())
    }

    /// 统一的安全守卫方法
    fn security_guard(&self, cmd: &str, cwd: &Path) -> Result<(), ToolError> {
        // 1. 拒绝模式检查
        self.check_deny_patterns(cmd)?;

        // 2. 允许模式检查
        self.check_allow_patterns(cmd)?;

        // 3. 工作空间限制检查
        if self.options.restrict_to_workspace {
            if detect_path_traversal(cmd) {
                return Err(ToolError::PermissionDenied {
                    path: cmd.to_string(),
                    allowed: Some("检测到路径遍历尝试 (path traversal detected)".to_string()),
                });
            }
            self.validate_paths_in_workspace(cmd, cwd)?;
        }

        Ok(())
    }

    /// 构建包含 PATH 扩展的环境变量
    fn build_env_with_path(&self) -> Option<std::collections::HashMap<String, String>> {
        if self.options.path_append.is_empty() {
            return None;
        }

        let mut env_map = std::collections::HashMap::new();
        for (key, value) in std::env::vars() {
            env_map.insert(key, value);
        }

        let path_sep = if cfg!(windows) { ";" } else { ":" };
        let current_path = env_map.get("PATH").cloned().unwrap_or_default();
        let new_path = format!("{}{}{}", current_path, path_sep, self.options.path_append);
        env_map.insert("PATH".to_string(), new_path);

        Some(env_map)
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "执行 Shell 命令。支持设置工作目录和超时。危险命令会被拦截。"
    }

    fn parameters(&self) -> SchemaObject {
        SHELL_PARAMETERS_SCHEMA.clone()
    }

    async fn execute(&self, _ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let command = require_param(&params, "command")?;
        let cwd = optional_param(&params, "cwd");
        let timeout_ms = u64_param(&params, "timeout_ms", self.options.timeout * 1000);

        debug!("执行 Shell 命令: {}", command);

        // 解析工作目录
        let working_dir =
            cwd.map(PathBuf::from).or_else(|| self.options.workspace.clone()).unwrap_or_else(|| PathBuf::from("."));

        // 安全守卫检查
        self.security_guard(&command, &working_dir)?;

        // 构建命令
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&command).current_dir(&working_dir).stdout(Stdio::piped()).stderr(Stdio::piped());

        // 应用 PATH 环境变量扩展
        if let Some(env) = self.build_env_with_path() {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        // 执行命令
        let child = cmd.spawn().map_err(|e| ToolError::execution(format!("命令启动失败: {e}")))?;

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| ToolError::Timeout { limit: timeout_ms / 1000, elapsed: Duration::from_millis(timeout_ms) })?
            .map_err(|e| ToolError::execution(format!("命令等待失败: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // 截断长输出
        const MAX_OUTPUT: usize = 10000;
        let stdout = truncate_output(stdout, MAX_OUTPUT);
        let stderr = truncate_output(stderr, MAX_OUTPUT);

        info!("Shell 命令完成: {} (exit_code={})", command, output.status.code().unwrap_or(-1));

        let result = ShellResult { exit_code: output.status.code().unwrap_or(-1), stdout, stderr };

        Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| "结果序列化失败".to_string()))
    }
}

#[cfg(test)]
mod tests;
