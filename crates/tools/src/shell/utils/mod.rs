//! Shell 工具函数模块
//!
//! 提供路径检测、输出截断等辅助功能。

use nanobot_utils::strings::truncate;
use regex::Regex;

/// 截断输出
pub fn truncate_output(s: String, max_len: usize) -> String {
    match truncate(&s, max_len) {
        Some(truncated) => format!("{}...(truncated, {} bytes total)", truncated, s.len()),
        None => s,
    }
}

/// 检测路径遍历尝试
pub fn detect_path_traversal(cmd: &str) -> bool {
    cmd.contains("..\\") || cmd.contains("../")
}

/// 从命令中提取 Windows 风格绝对路径
fn extract_windows_absolute_paths(cmd: &str) -> Vec<String> {
    // 匹配 Windows 风格绝对路径，如 C:\path\to\file
    let re = Regex::new(r#"[A-Za-z]:\\[^\s"'|><;]+"#).unwrap();
    re.find_iter(cmd).map(|m| m.as_str().to_string()).collect()
}

/// 从命令中提取 POSIX 风格绝对路径
fn extract_posix_absolute_paths(cmd: &str) -> Vec<String> {
    // 匹配 POSIX 风格绝对路径，如 /path/to/file
    let re = Regex::new(r#"(?:^|[\s|>])(/[^\s"'>]+)"#).unwrap();
    re.captures_iter(cmd).filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())).collect()
}

/// 从命令中提取所有绝对路径
pub fn extract_absolute_paths(cmd: &str) -> Vec<String> {
    let mut paths = extract_windows_absolute_paths(cmd);
    paths.extend(extract_posix_absolute_paths(cmd));
    paths
}

/// 构建包含 PATH 扩展的环境变量并应用到命令
pub fn add_envs(cmd: &mut tokio::process::Command, append_path: &str) {
    cmd.envs(std::env::vars());

    if append_path.trim().is_empty() {
        return;
    }

    const KEY: &str = "PATH";
    const ENV_PATH_SEP: char = if cfg!(windows) { ';' } else { ':' };

    match std::env::var(KEY) {
        Ok(v) => cmd.env(KEY, format!("{v}{ENV_PATH_SEP}{append_path}")),
        Err(_) => cmd.env(KEY, append_path),
    };
}

#[cfg(test)]
mod tests;
