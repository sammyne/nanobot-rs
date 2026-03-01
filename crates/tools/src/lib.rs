//! Nanobot Tools - AI Agent 工具实现
//!
//! 提供文件系统工具（read_file, write_file, edit_file, list_dir）和 Shell 执行工具。

pub mod core;
pub mod fs;
pub mod registry;
pub mod shell;

// 重新导出核心类型
pub use core::{Tool, ToolDefinition, ToolError, ToolResult};
pub use fs::{EditFileTool, ListDirTool, ReadFileTool, WriteFileTool};
pub use registry::ToolRegistry;
pub use shell::ShellTool;

#[cfg(test)]
mod tests;
