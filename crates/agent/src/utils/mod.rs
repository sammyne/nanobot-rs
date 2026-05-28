//! 工具函数模块

use std::path::Path;

use tracing::warn;

/// 解析系统消息的目标路由信息
///
/// 解析 chat_id 字符串，提取 channel、target_chat_id 和 session_key。
///
/// # Arguments
/// * `chat_id` - 原始 chat_id 字符串
///
/// # Returns
/// 返回元组 (channel, target_chat_id, session_key)
///
/// # Examples
/// - "telegram:12345" -> ("telegram", "12345", "telegram:12345")
/// - "simple_id" -> ("cli", "simple_id", "cli:simple_id")
pub fn parse_system_message_target(chat_id: &str) -> (&str, &str, String) {
    if let Some((channel, target_chat_id)) = chat_id.split_once(':') {
        let session_key = format!("{channel}:{target_chat_id}");
        (channel, target_chat_id, session_key)
    } else {
        // 无分隔符，使用默认值 "cli" 作为 channel
        let session_key = format!("cli:{chat_id}");
        ("cli", chat_id, session_key)
    }
}

/// 大工具结果持久化
///
/// 当工具结果超过 `max_chars` 时，将完整结果写入磁盘文件，
/// 返回包含预览和文件路径的引用字符串。
/// 未超过阈值时原样返回。
///
/// fail-open：写入失败时返回原始结果（截断到 max_chars）。
pub fn maybe_persist_tool_result(
    result: &str,
    max_chars: usize,
    tool_call_id: &str,
    session_key: &str,
    workspace: &Path,
) -> String {
    if result.len() <= max_chars {
        return result.to_string();
    }

    let dir = workspace.join(".nanobot").join("tool-results").join(session_key);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("创建工具结果目录失败: {e}，使用截断结果");
        return truncate_result(result, max_chars);
    }

    let file_path = dir.join(format!("{tool_call_id}.txt"));
    let tmp_path = dir.join(format!("{tool_call_id}.txt.tmp"));

    // 原子写入
    if let Err(e) = std::fs::write(&tmp_path, result) {
        warn!("写入工具结果临时文件失败: {e}，使用截断结果");
        return truncate_result(result, max_chars);
    }
    if let Err(e) = std::fs::rename(&tmp_path, &file_path) {
        warn!("重命名工具结果文件失败: {e}，使用截断结果");
        let _ = std::fs::remove_file(&tmp_path);
        return truncate_result(result, max_chars);
    }

    // 构造引用字符串：1200 字符预览 + 文件路径
    let preview_len = 1200.min(result.len());
    let preview = &result[..preview_len];
    format!(
        "{preview}\n\n[... truncated {total} chars total — full output saved to {path}]",
        total = result.len(),
        path = file_path.display()
    )
}

fn truncate_result(result: &str, max_chars: usize) -> String {
    let half = max_chars / 2;
    let head = &result[..half];
    let tail = &result[result.len() - half..];
    format!("{head}\n\n[... {total} chars truncated ...]\n\n{tail}", total = result.len() - max_chars)
}

#[cfg(test)]
mod tests;
