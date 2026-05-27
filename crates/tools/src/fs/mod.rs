//! 文件系统工具
//!
//! 提供 read_file, write_file, edit_file, list_dir 等文件操作。

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info};

use crate::core::{Tool, ToolContext, ToolError, ToolResult};

/// 读取文件的最大字符数限制（128K 字符）
const MAX_CHARS: usize = 128_000;

/// 读取文件的最大字节数限制（512KB，覆盖 UTF-8 最坏情况 1 char = 4 bytes）
const MAX_BYTES: usize = MAX_CHARS * 4;

/// 解析并验证路径
///
/// 将路径解析为绝对路径，并检查是否在允许目录内
fn resolve_path(path: &str, workspace: &Path, allowed_dir: Option<&Path>) -> Result<PathBuf, ToolError> {
    let path_obj = Path::new(path);

    // 解析为绝对路径
    let absolute = if path_obj.is_absolute() { path_obj.to_path_buf() } else { workspace.join(path_obj) };

    // 规范化路径
    let canonical = absolute.canonicalize().unwrap_or_else(|_| absolute.clone());

    // 检查允许目录限制
    if let Some(allowed) = allowed_dir {
        let allowed_path = Path::new(allowed);
        if !canonical.starts_with(allowed_path) {
            return Err(ToolError::PermissionDenied {
                path: path.to_string(),
                allowed: Some(allowed.to_string_lossy().to_string()),
            });
        }
    }

    Ok(canonical)
}

// ==================== ReadFileTool ====================

/// ReadFile 参数结构
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    /// 文件或目录路径，支持相对路径（基于workspace）或绝对路径
    pub path: String,
    /// 起始行号（1-indexed，默认 1）
    #[serde(default = "default_offset")]
    pub offset: usize,
    /// 最大读取行数（默认 2000）
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_offset() -> usize {
    1
}

fn default_limit() -> usize {
    2000
}

/// Lazy-initialized global schema for ReadFileArgs
static READ_FILE_PARAMETERS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schemars::schema_for!(ReadFileArgs));

/// 读取文件工具
pub struct ReadFileTool {
    workspace: PathBuf,
    allowed_dir: Option<PathBuf>,
}

impl ReadFileTool {
    /// 创建新的读取文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<PathBuf>, allowed_dir: Option<impl Into<PathBuf>>) -> Self {
        Self { workspace: workspace.into(), allowed_dir: allowed_dir.map(|v| v.into()) }
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

    fn parameters(&self) -> Schema {
        READ_FILE_PARAMETERS_SCHEMA.clone()
    }

    async fn execute(&self, _ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let args: ReadFileArgs = serde_json::from_value(params)?;
        let path = resolve_path(&args.path, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("读取文件: {:?} (offset={}, limit={})", path, args.offset, args.limit);

        // 检查是否存在且为文件
        let metadata = fs::metadata(&path).await.map_err(ToolError::io)?;

        if !metadata.is_file() {
            return Err(ToolError::path(format!("路径不是文件: {}", args.path)));
        }

        // 检查文件大小，防止读取过大文件导致 OOM
        let file_size = metadata.len();
        if file_size > MAX_BYTES as u64 {
            return Err(ToolError::execution(format!(
                "文件过大 ({file_size} bytes)，超过 {MAX_BYTES} bytes 限制。请使用 exec 工具的 head/tail/grep 命令处理大文件。"
            )));
        }

        // 读取内容
        let content = fs::read_to_string(&path).await.map_err(ToolError::io)?;

        // 按行分页，输出带行号
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let start = (args.offset.max(1) - 1).min(total); // 0-indexed
        let end = (start + args.limit).min(total);

        if start >= total {
            return Ok(format!("(File has {total} lines. Offset {} is beyond the end.)", args.offset));
        }

        let numbered: Vec<String> =
            lines[start..end].iter().enumerate().map(|(i, line)| format!("{}: {line}", start + i + 1)).collect();

        let mut result = numbered.join("\n");

        // 附加分页提示
        if end < total {
            result.push_str(&format!(
                "\n\n(Showing lines {}-{} of {total}. Use offset={} to continue.)",
                start + 1,
                end,
                end + 1
            ));
        }

        info!("成功读取文件: {} (lines {}-{} of {})", args.path, start + 1, end, total);
        Ok(result)
    }
}

// ==================== WriteFileTool ====================

/// WriteFile 参数结构
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteFileArgs {
    /// 目标文件路径
    pub path: String,
    /// 要写入的文件内容
    pub content: String,
}

/// Lazy-initialized global schema for WriteFileArgs
static WRITE_FILE_PARAMETERS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schemars::schema_for!(WriteFileArgs));

/// 写入文件工具
pub struct WriteFileTool {
    workspace: PathBuf,
    allowed_dir: Option<PathBuf>,
}

impl WriteFileTool {
    /// 创建新的写入文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<PathBuf>, allowed_dir: Option<impl Into<PathBuf>>) -> Self {
        Self { workspace: workspace.into(), allowed_dir: allowed_dir.map(|v| v.into()) }
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

    fn parameters(&self) -> Schema {
        WRITE_FILE_PARAMETERS_SCHEMA.clone()
    }

    async fn execute(&self, _ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let args: WriteFileArgs = serde_json::from_value(params)?;
        let path = resolve_path(&args.path, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("写入文件: {:?}", path);

        // 创建父目录
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(ToolError::io)?;
        }

        // 写入文件
        fs::write(&path, &args.content).await.map_err(ToolError::io)?;

        info!("成功写入文件: {} ({} bytes)", args.path, args.content.len());
        Ok(format!("文件写入成功: {} ({} bytes)", args.path, args.content.len()))
    }
}

// ==================== EditFileTool ====================

/// EditFile 参数结构
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditFileArgs {
    /// 目标文件路径
    pub path: String,
    /// 要替换的原始文本（需完全匹配）
    pub old_text: String,
    /// 替换后的新文本
    pub new_text: String,
    /// 是否替换所有匹配（默认 false，多匹配时报错）
    #[serde(default)]
    pub replace_all: bool,
}

/// Lazy-initialized global schema for EditFileArgs
static EDIT_FILE_PARAMETERS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schemars::schema_for!(EditFileArgs));

/// 编辑文件工具
pub struct EditFileTool {
    workspace: PathBuf,
    allowed_dir: Option<PathBuf>,
}

impl EditFileTool {
    /// 创建新的编辑文件工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<PathBuf>, allowed_dir: Option<impl Into<PathBuf>>) -> Self {
        Self { workspace: workspace.into(), allowed_dir: allowed_dir.map(|v| v.into()) }
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

    fn parameters(&self) -> Schema {
        EDIT_FILE_PARAMETERS_SCHEMA.clone()
    }

    async fn execute(&self, _ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let args: EditFileArgs = serde_json::from_value(params)?;
        let path = resolve_path(&args.path, &self.workspace, self.allowed_dir.as_deref())?;

        debug!("编辑文件: {:?} (replace_all={})", path, args.replace_all);

        // 读取文件内容
        let content = fs::read_to_string(&path).await.map_err(ToolError::io)?;

        // 查找匹配位置
        let matches = Self::find_matches(&content, &args.old_text);
        match matches.len() {
            0 => Err(ToolError::execution("未找到匹配的文本。请确保 old_text 与文件内容完全匹配。".to_string())),
            1 => {
                let new_content = content.replacen(&args.old_text, &args.new_text, 1);
                fs::write(&path, new_content).await.map_err(ToolError::io)?;

                info!("成功编辑文件: {}", args.path);
                Ok(format!("文件编辑成功: {}", args.path))
            }
            n if args.replace_all => {
                let new_content = content.replace(&args.old_text, &args.new_text);
                fs::write(&path, new_content).await.map_err(ToolError::io)?;

                info!("成功编辑文件: {} (替换 {} 处)", args.path, n);
                Ok(format!("文件编辑成功: {}（已替换 {n} 处匹配）", args.path))
            }
            n => Err(ToolError::execution(format!(
                "找到 {n} 处匹配，无法确定唯一位置。请提供更多上下文，或使用 replace_all=true 替换所有匹配。"
            ))),
        }
    }
}

// ==================== ListDirTool ====================

/// ListDir 参数结构
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListDirArgs {
    /// 目录路径
    pub path: String,
    /// 是否递归列出子目录
    #[serde(default)]
    pub recursive: bool,
    /// 最大条目数（默认 200）
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_max_entries() -> usize {
    200
}

/// Lazy-initialized global schema for ListDirArgs
static LIST_DIR_PARAMETERS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schemars::schema_for!(ListDirArgs));

/// 列出目录工具
pub struct ListDirTool {
    workspace: PathBuf,
    allowed_dir: Option<PathBuf>,
}

impl ListDirTool {
    /// 创建新的列出目录工具
    ///
    /// # Arguments
    /// * `workspace` - 工作目录
    /// * `allowed_dir` - 允许访问的目录限制（可选）
    pub fn new(workspace: impl Into<PathBuf>, allowed_dir: Option<impl Into<PathBuf>>) -> Self {
        Self { workspace: workspace.into(), allowed_dir: allowed_dir.map(|v| v.into()) }
    }

    /// 格式化目录条目
    fn format_entry(path: &Path, metadata: &std::fs::Metadata) -> String {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let size = if metadata.is_file() { format!(" ({} bytes)", metadata.len()) } else { String::new() };
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

    fn parameters(&self) -> Schema {
        LIST_DIR_PARAMETERS_SCHEMA.clone()
    }

    async fn execute(&self, _ctx: &ToolContext, params: serde_json::Value) -> ToolResult {
        let args: ListDirArgs = serde_json::from_value(params)?;
        let path = resolve_path(&args.path, &self.workspace, self.allowed_dir.as_deref())?;

        // 检查是否为目录
        let metadata = fs::metadata(&path).await.map_err(ToolError::io)?;

        if !metadata.is_dir() {
            return Err(ToolError::path(format!("路径不是目录: {}", args.path)));
        }

        let mut results = vec![format!("目录: {}", args.path)];

        if args.recursive {
            let mut entries: Vec<_> = walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()).collect();
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
            let mut entries = fs::read_dir(&path).await.map_err(ToolError::io)?;

            while let Some(entry) = entries.next_entry().await.map_err(ToolError::io)? {
                if let Ok(meta) = entry.metadata().await {
                    let formatted = Self::format_entry(&entry.path(), &meta);
                    results.push(format!("  {formatted}"));
                }
            }
        }

        let total = results.len() - 1;
        if total > args.max_entries {
            results.truncate(args.max_entries + 1);
            results.push(format!("... 共 {total} 个条目，已显示前 {} 个", args.max_entries));
        }

        info!("成功列出目录: {} ({} 项)", args.path, total.min(args.max_entries));
        Ok(results.join("\n"))
    }
}

#[cfg(test)]
mod tests;
