//! Shell 执行工具
//!
//! 提供安全的 Shell 命令执行功能。

use crate::core::{optional_param, require_param, u64_param, Tool, ToolError, ToolResult};
use async_trait::async_trait;
use schemars::schema::SchemaObject;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info};

/// Shell 执行结果
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShellResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Shell 工具
pub struct ShellTool {
    workspace: String,
    default_timeout_secs: u64,
}

/// 危险命令关键字
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "> /dev/sda",
    "mkfs",
    "dd if=/dev/zero",
    "format",
    ":(){ :|:& };:", // fork bomb
    "chmod -R 777 /",
    "chmod -R 000 /",
];

impl ShellTool {
    pub fn new(workspace: impl Into<String>) -> Self {
        Self {
            workspace: workspace.into(),
            default_timeout_secs: 30,
        }
    }

    /// 设置默认超时
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.default_timeout_secs = secs;
        self
    }

    /// 安全检查：检查是否包含危险命令
    fn check_safety(cmd: &str) -> Result<(), ToolError> {
        let cmd_lower = cmd.to_lowercase();
        for pattern in DANGEROUS_PATTERNS {
            if cmd_lower.contains(&pattern.to_lowercase()) {
                return Err(ToolError::PermissionDenied {
                    path: cmd.to_string(),
                    allowed: Some("危险命令被拒绝".to_string()),
                });
            }
        }
        Ok(())
    }

    /// 截断输出
    fn truncate_output(s: String, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{}...(truncated, {} bytes total)", &s[..max_len], s.len())
        } else {
            s
        }
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
                    "description": "超时时间（毫秒，可选，默认30秒）",
                    "default": 30000
                }
            },
            "required": ["command"]
        }))
        .unwrap_or_default()
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let command = require_param(&params, "command")?;
        let cwd = optional_param(&params, "cwd");
        let timeout_ms = u64_param(&params, "timeout_ms", self.default_timeout_secs * 1000);

        debug!("执行 Shell 命令: {}", command);

        // 安全检查
        Self::check_safety(&command)?;

        // 解析工作目录
        let working_dir = cwd.map(PathBuf::from).unwrap_or_else(|| {
            PathBuf::from(&self.workspace)
        });

        // 执行命令
        let child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::execution(format!("命令启动失败: {}", e)))?;

        let output = timeout(
            Duration::from_millis(timeout_ms),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| ToolError::Timeout {
            limit: timeout_ms / 1000,
            elapsed: Duration::from_millis(timeout_ms),
        })?
        .map_err(|e| ToolError::execution(format!("命令等待失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // 截断长输出
        const MAX_OUTPUT: usize = 10000;
        let stdout = Self::truncate_output(stdout, MAX_OUTPUT);
        let stderr = Self::truncate_output(stderr, MAX_OUTPUT);

        info!(
            "Shell 命令完成: {} (exit_code={})",
            command,
            output.status.code().unwrap_or(-1)
        );

        let result = ShellResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
        };

        Ok(serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "结果序列化失败".to_string()))
    }
}