// This module provides safe string manipulation functions

/// Safely truncates a string to the specified maximum character count.
///
/// This function ensures that truncation always occurs at valid UTF-8 character boundaries,
/// preventing panics that can occur when using byte-based indexing on strings containing
/// multi-byte characters (like Chinese characters).
///
/// # Arguments
///
/// * `s` - A reference to the string to truncate
/// * `max_chars` - The maximum number of characters to return
///
/// # Returns
///
/// Returns:
/// - `Some(truncated_str)` if the string is longer than `max_chars` and was truncated
/// - `None` if the string is already within the limit and doesn't need truncation
///
/// # Safety
///
/// This function is safe and will never panic, as it uses `char_indices()` to find valid
/// UTF-8 character boundaries rather than using byte-based indexing.
///
/// # Examples
///
/// ```
/// use nanobot_utils::strings::truncate;
///
/// // String within limit returns None
/// assert!(truncate("hello", 10).is_none());
/// assert!(truncate("", 5).is_none());
///
/// // String exceeding limit returns Some with truncated string
/// assert_eq!(truncate("hello world", 5), Some("hello"));
///
/// // Multi-byte characters are handled safely
/// assert_eq!(truncate("你好世界", 2), Some("你好"));
/// assert!(truncate("你好世界", 4).is_none());
/// ```
pub fn truncate(s: &str, max_chars: usize) -> Option<&str> {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => Some(&s[..i]),
        None => None,
    }
}

/// Redacts a string by masking parts of it.
///
/// # Behavior
/// - If length <= 8: mask all characters
/// - If length > 8: keep first 4 and last 4 characters, mask the middle
///
/// This function uses character-based counting and iteration, making it safe for
/// strings containing multi-byte UTF-8 characters (e.g., Chinese characters).
///
/// # Examples
///
/// ```
/// use nanobot_utils::strings::redact;
///
/// // Short strings are fully masked
/// assert_eq!(redact("abc"), "***");
/// assert_eq!(redact("12345678"), "********");
///
/// // Longer strings keep first 4 and last 4 characters
/// assert_eq!(redact("123456789"), "1234****6789");
///
/// // Multi-byte characters are handled safely
/// assert_eq!(redact("你好世界"), "****");  // 4 chars, all masked
/// assert_eq!(redact("你好世界123"), "*******");  // 7 chars, all masked
/// assert_eq!(redact("密钥12345678"), "密钥12****5678");
/// ```
pub fn redact(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 8 {
        "*".repeat(chars.len())
    } else {
        let start: String = chars.iter().take(4).collect();
        let end: String = chars.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
        format!("{start}****{end}")
    }
}

/// 估算文本的 token 数量
///
/// 基于字节长度的粗略估算：1 token 约等于 4 字节。
/// 对英文和代码足够准确，中文场景可后续替换为精确 tokenizer。
///
/// # Examples
///
/// ```
/// use nanobot_utils::strings::estimate_tokens;
///
/// assert_eq!(estimate_tokens(""), 0);
/// assert_eq!(estimate_tokens("hello"), 1);  // 5 bytes / 4 = 1
/// ```
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

#[cfg(test)]
mod tests;
