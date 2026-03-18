//! Integration tests for the strings module

use nanobot_utils::strings::redact;

#[test]
fn redact_long() {
    let key = "sk-1234567890abcdefghijklmnop";
    let masked = redact(key);
    assert_eq!(masked, "sk-1****mnop");
}

#[test]
fn redact_short() {
    let key = "abc";
    let masked = redact(key);
    assert_eq!(masked, "***");
}

#[test]
fn redact_8_chars() {
    let key = "12345678";
    let masked = redact(key);
    assert_eq!(masked, "********");
}

#[test]
fn redact_9_chars() {
    let key = "123456789";
    let masked = redact(key);
    assert_eq!(masked, "1234****6789");
}
