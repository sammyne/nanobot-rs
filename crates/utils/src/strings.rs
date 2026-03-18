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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert!(truncate("", 5).is_none());
    }

    #[test]
    fn test_short_string() {
        assert!(truncate("hello", 10).is_none());
        assert!(truncate("test", 4).is_none());
    }

    #[test]
    fn test_long_string() {
        assert_eq!(truncate("hello world", 5), Some("hello"));
        assert_eq!(truncate("rust programming", 4), Some("rust"));
    }

    #[test]
    fn test_mixed_byte_characters() {
        // Chinese characters (3 bytes each)
        assert_eq!(truncate("你好世界", 2), Some("你好"));
        assert_eq!(truncate("你好世界", 3), Some("你好世"));
        assert!(truncate("你好世界", 4).is_none());

        // Mixed ASCII and Chinese
        assert!(truncate("hello你好", 7).is_none());
        assert_eq!(truncate("hello你好", 5), Some("hello"));
        assert_eq!(truncate("hello你好", 6), Some("hello你"));
    }

    #[test]
    fn test_boundary_cases() {
        // Truncate at exact character boundary
        assert_eq!(truncate("abcde", 3), Some("abc"));

        // Truncate at multi-byte character boundary
        assert_eq!(truncate("中文字符", 2), Some("中文"));

        // Zero max_chars (edge case)
        assert_eq!(truncate("hello", 0), Some(""));
        assert!(truncate("", 0).is_none());
    }

    #[test]
    fn test_exact_length() {
        assert!(truncate("hello", 5).is_none());
        assert!(truncate("你好", 2).is_none());
    }

    #[test]
    fn test_redact_empty_string() {
        assert_eq!(redact(""), "");
    }

    #[test]
    fn test_redact_short_strings() {
        assert_eq!(redact("a"), "*");
        assert_eq!(redact("abc"), "***");
        assert_eq!(redact("12345678"), "********");
    }

    #[test]
    fn test_redact_boundary_case() {
        // Exactly 8 characters - should all be masked
        assert_eq!(redact("12345678"), "********");
        // 9 characters - should show first 4 and last 4
        assert_eq!(redact("123456789"), "1234****6789");
    }

    #[test]
    fn test_redact_long_strings() {
        assert_eq!(redact("123456789"), "1234****6789");
        assert_eq!(redact("sk-1234567890abcdefghijklmnop"), "sk-1****mnop");
    }

    #[test]
    fn test_redact_multibyte_characters() {
        // 4 Chinese characters - all masked (4 chars <= 8)
        assert_eq!(redact("你好世界"), "****");
        // Mixed Chinese and ASCII with 7 chars - all masked
        assert_eq!(redact("你好世界123"), "*******");
        // Chinese at start with 10 chars - keep first 4 and last 4
        assert_eq!(redact("密钥12345678"), "密钥12****5678");
        // Chinese at end with 10 chars - keep first 4 and last 4
        assert_eq!(redact("12345678密钥"), "1234****78密钥");
        // All Chinese with more than 8 chars
        assert_eq!(redact("这是一个很长的测试字符串"), "这是一个****试字符串");
    }

    #[test]
    fn test_redact_api_key_like_strings() {
        assert_eq!(redact("sk-1234567890abcdef"), "sk-1****cdef");
        assert_eq!(redact("sk-test"), "*******"); // 7 chars, all masked
    }
}
