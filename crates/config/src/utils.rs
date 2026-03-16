//! 工具函数模块
//!
//! 提供通用的工具函数。

use std::path::{Path, PathBuf};

use super::HOME;

/// 将路径中的 ~ 替换为用户主目录
pub fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(first) = path.iter().next()
        && first == "~"
    {
        let mut new_path = HOME.clone();
        for component in path.iter().skip(1) {
            new_path.push(component);
        }
        return new_path;
    }
    path.to_path_buf()
}
