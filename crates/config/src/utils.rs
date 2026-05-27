//! 工具函数模块
//!
//! 提供通用的工具函数。

use std::path::{Path, PathBuf};

/// 将路径中的 ~ 替换为用户主目录
pub fn expand_tilde(path: &Path) -> PathBuf {
    nanobot_utils::paths::expand_tilde(path)
}
