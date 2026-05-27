//! 路径处理工具函数

use std::path::{Path, PathBuf};

/// 将路径中的 `~` 替换为用户主目录
///
/// # Examples
///
/// ```no_run
/// use nanobot_utils::paths::expand_tilde;
/// use std::path::Path;
///
/// let expanded = expand_tilde(Path::new("~/documents"));
/// assert!(expanded.is_absolute());
/// ```
pub fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(first) = path.iter().next()
        && first == "~"
    {
        #[allow(deprecated)]
        let home = std::env::home_dir().expect("无法获取用户主目录");
        let mut new_path = home;
        for component in path.iter().skip(1) {
            new_path.push(component);
        }
        return new_path;
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests;
