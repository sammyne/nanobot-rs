//! 日志模块测试

use super::*;

#[test]
fn mask_sensitive_long() {
    let key = "sk-1234567890abcdefghijklmnop";
    let masked = mask_sensitive(key);
    assert_eq!(masked, "sk-1****mnop");
}

#[test]
fn mask_sensitive_short() {
    let key = "abc";
    let masked = mask_sensitive(key);
    assert_eq!(masked, "***");
}

#[test]
fn mask_sensitive_8_chars() {
    let key = "12345678";
    let masked = mask_sensitive(key);
    assert_eq!(masked, "********");
}

#[test]
fn mask_sensitive_9_chars() {
    let key = "123456789";
    let masked = mask_sensitive(key);
    assert_eq!(masked, "1234****6789");
}
