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

#[test]
fn estimate_tokens_empty() {
    assert_eq!(estimate_tokens(""), 0);
}

#[test]
fn estimate_tokens_ascii() {
    assert_eq!(estimate_tokens("hello"), 1); // 5 bytes / 4 = 1
    assert_eq!(estimate_tokens("a]"), 0); // 2 bytes / 4 = 0
    assert_eq!(estimate_tokens("abcd"), 1); // 4 bytes / 4 = 1
    assert_eq!(estimate_tokens(&"x".repeat(100)), 25); // 100 bytes / 4 = 25
}

#[test]
fn estimate_tokens_multibyte() {
    // "你好" = 6 bytes UTF-8 → 6/4 = 1
    assert_eq!(estimate_tokens("你好"), 1);
    // "你好世界" = 12 bytes UTF-8 → 12/4 = 3
    assert_eq!(estimate_tokens("你好世界"), 3);
}
