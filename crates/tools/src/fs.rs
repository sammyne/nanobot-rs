//! 文件系统工具
//!
//! 提供 read_file, write_file, edit_file, list_dir 等文件操作。

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::schema::SchemaObject;
use tokio::fs;
use tracing::{debug, info};

use crate::core::{Tool, ToolError, ToolResult, bool_param, require_param};

/// 解析并验证路径
///
/// 将路径解析为绝对路径，并检查是否在允许目录内
fn resolve_path(path: &str, workspace: &str, allowed_dir: Option<&str>) -> Result<PathBuf, ToolError> {
    let path_obj = Path::new(path);

    // 解析为绝对路径
    let absolute = if path_obj.is_absolute() {
        path_obj.to_path_buf()
    } else {
        Path::new(workspace).join(path_obj)
    };

    // 规范化路径
    let canonical = absolute.canonicalize().unwrap_or_else(|_| absolute.clone());

    // 检查允许目录限制
    if let Some(allowed) = allowed_dir {
        let allowed_path = Path::new(allowed);
        if !canonical.starts_with(allowed_path) {
            return Err(ToolError::PermissionDenied {
                path: path.to_string(),
                allowed: Some(allowed.to_string()),
            });
        }
    }

    Ok(canonical)
}

/// 生成基本的路径参数 Schema
fn path_param_schema() -> SchemaObject {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "文件或目录路径，支持相对路径（基于workspace）或绝对路径"
            }
        },
        "required": ["path"]
    }))
    .unwrap_or_default()
}

// ==================== ReadFileTool ====================

/// 读取文件工具
pub struct ReadFileTool {
    workspace: String,
    allowed_dir: Option<String>,
}

impl ReadFileTool {
    /// 创建新的读取文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<String>, allowed_dir: Option<impl Into<String>>) -> Self {
        Self {
            workspace: workspace.into(),
            allowed_dir: allowed_dir.map(|v| v.into()),
        }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取指定文件的内容。支持相对路径（基于 workspace）或绝对路径。"
    }

    fn parameters(&self) -> SchemaObject {
        path_param_schema()
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let path_str = require_param(&params, "path")?;
        let path = resolve_path(&path_str, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("读取文件: {:?}", path);

        // 检查是否存在且为文件
        let metadata = fs::metadata(&path).await.map_err(ToolError::io)?;

        if !metadata.is_file() {
            return Err(ToolError::path(format!("路径不是文件: {path_str}")));
        }

        // 读取内容
        let content = fs::read_to_string(&path).await.map_err(ToolError::io)?;

        info!("成功读取文件: {} ({} bytes)", path_str, content.len());
        Ok(content)
    }
}

// ==================== WriteFileTool ====================

/// 写入文件工具
pub struct WriteFileTool {
    workspace: String,
    allowed_dir: Option<String>,
}

impl WriteFileTool {
    /// 创建新的写入文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<String>, allowed_dir: Option<impl Into<String>>) -> Self {
        Self {
            workspace: workspace.into(),
            allowed_dir: allowed_dir.map(|v| v.into()),
        }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "将内容写入指定文件。如果父目录不存在会自动创建。支持相对路径或绝对路径。"
    }

    fn parameters(&self) -> SchemaObject {
        serde_json::from_value(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标文件路径"
                },
                "content": {
                    "type": "string",
                    "description": "要写入的文件内容"
                }
            },
            "required": ["path", "content"]
        }))
        .unwrap_or_default()
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let path_str = require_param(&params, "path")?;
        let content = require_param(&params, "content")?;

        let path = resolve_path(&path_str, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("写入文件: {:?}", path);

        // 创建父目录
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(ToolError::io)?;
        }

        // 写入文件
        fs::write(&path, &content).await.map_err(ToolError::io)?;

        info!("成功写入文件: {} ({} bytes)", path_str, content.len());
        Ok(format!("文件写入成功: {} ({} bytes)", path_str, content.len()))
    }
}

// ==================== EditFileTool ====================

/// 编辑文件工具
pub struct EditFileTool {
    workspace: String,
    allowed_dir: Option<String>,
}

impl EditFileTool {
    /// 创建新的编辑文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<String>, allowed_dir: Option<impl Into<String>>) -> Self {
        Self {
            workspace: workspace.into(),
            allowed_dir: allowed_dir.map(|v| v.into()),
        }
    }

    /// 查找文本匹配位置，统计匹配次数
    fn find_matches(content: &str, old_text: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;

        while let Some(pos) = content[start..].find(old_text) {
            positions.push(start + pos);
            start += pos + old_text.len();
        }

        positions
    }
}

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "编辑文件内容，将 old_text 替换为 new_text。需要完全匹配（建议包含3行上下文确保唯一性）。"
    }

    fn parameters(&self) -> SchemaObject {
        serde_json::from_value(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目标文件路径"
                },
                "old_text": {
                    "type": "string",
                    "description": "要替换的原始文本（需完全匹配）"
                },
                "new_text": {
                    "type": "string",
                    "description": "替换后的新文本"
                }
            },
            "required": ["path", "old_text", "new_text"]
        }))
        .unwrap_or_default()
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let path_str = require_param(&params, "path")?;
        let old_text = require_param(&params, "old_text")?;
        let new_text = require_param(&params, "new_text")?;

        let path = resolve_path(&path_str, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("编辑文件: {:?}", path);

        // 读取文件内容
        let content = fs::read_to_string(&path).await.map_err(ToolError::io)?;

        // 查找匹配位置
        let _matches = Self::find_matches(&content, &old_text);

        let matches = Self::find_matches(&content, &old_text);
        match matches.len() {
            0 => {
                // 无匹配，返回错误和上下文
                Err(ToolError::execution(
                    "未找到匹配的文本。请确保 old_text 与文件内容完全匹配。".to_string(),
                ))
            }
            1 => {
                // 唯一匹配，执行替换
                let new_content = content.replacen(&old_text, &new_text, 1);
                fs::write(&path, new_content).await.map_err(ToolError::io)?;

                info!("成功编辑文件: {}", path_str);
                Ok(format!("文件编辑成功: {path_str}"))
            }
            n => {
                // 多次匹配，警告用户
                Err(ToolError::execution(format!(
                    "找到 {n} 处匹配，无法确定唯一位置。请提供更多上下文。"
                )))
            }
        }
    }
}

// ==================== ListDirTool ====================

/// 列出目录工具
pub struct ListDirTool {
    workspace: String,
    allowed_dir: Option<String>,
}

impl ListDirTool {
    /// 创建新的列出目录工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<String>, allowed_dir: Option<impl Into<String>>) -> Self {
        Self {
            workspace: workspace.into(),
            allowed_dir: allowed_dir.map(|v| v.into()),
        }
    }

    /// 格式化目录条目
    fn format_entry(path: &Path, metadata: &std::fs::Metadata) -> String {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let size = if metadata.is_file() {
            format!(" ({} bytes)", metadata.len())
        } else {
            String::new()
        };
        let kind = if metadata.is_dir() { "[DIR]" } else { "[FILE]" };
        format!("{kind} {name}{size}")
    }
}

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &str {
        "list_dir"
    }

    fn description(&self) -> &str {
        "列出指定目录的内容。可选项 recursive 支持递归列出。"
    }

    fn parameters(&self) -> SchemaObject {
        serde_json::from_value(serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "目录路径"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "是否递归列出子目录",
                    "default": false
                }
            },
            "required": ["path"]
        }))
        .unwrap_or_default()
    }

    async fn execute(&self, params: serde_json::Value) -> ToolResult {
        let path_str = require_param(&params, "path")?;
        let recursive = bool_param(&params, "recursive");

        let path = resolve_path(&path_str, &self.workspace, self.allowed_dir.as_deref())?;

        // 检查是否为目录
        let metadata = fs::metadata(&path).await.map_err(ToolError::io)?;

        if !metadata.is_dir() {
            return Err(ToolError::path(format!("路径不是目录: {path_str}")));
        }

        let mut results = vec![format!("目录: {}", path_str)];

        if recursive {
            // 递归遍历
            let mut entries: Vec<_> = walkdir::WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by(|a, b| a.path().cmp(b.path()));

            for entry in entries {
                let depth = entry.depth();
                let indent = "  ".repeat(depth);
                if let Ok(meta) = entry.metadata() {
                    let formatted = Self::format_entry(entry.path(), &meta);
                    results.push(format!("{indent}{formatted}"));
                }
            }
        } else {
            // 非递归列出
            let mut entries = fs::read_dir(&path).await.map_err(ToolError::io)?;

            while let Some(entry) = entries.next_entry().await.map_err(ToolError::io)? {
                if let Ok(meta) = entry.metadata().await {
                    let formatted = Self::format_entry(&entry.path(), &meta);
                    results.push(format!("  {formatted}"));
                }
            }
        }

        info!("成功列出目录: {} ({} 项)", path_str, results.len() - 1);
        Ok(results.join("\n"))
    }
}
