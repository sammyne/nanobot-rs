//! Nanobot Tools - AI Agent 工具实现
//!
//! 提供文件系统工具（read_file, write_file, edit_file, list_dir）、Shell 执行工具和 Cron 调度工具。

pub mod core;
pub mod cron;
pub mod cron_types;
pub mod fs;
pub mod registry;
pub mod shell;

// 重新导出核心类型
pub use core::{Tool, ToolDefinition, ToolError, ToolResult};

pub use cron::{CronService, CronTool};
pub use cron_types::{CronJob, CronPayload, CronSchedule, CronStore};
pub use fs::{EditFileTool, ListDirTool, ReadFileTool, WriteFileTool};
pub use registry::ToolRegistry;
pub use shell::ShellTool;

#[cfg(test)]
mod tests;
