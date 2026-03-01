//! Nanobot CLI 库
//!
//! 提供 nanobot 命令行工具的核心功能。

pub mod commands;
pub mod logging;

pub use commands::{AgentCmd, OnboardCmd};
pub use logging::init as init_logging;
