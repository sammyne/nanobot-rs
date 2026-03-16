//! 工具函数模块

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

#[cfg(test)]
mod tests;
