//! 消息类型测试
//!
//! 按照 AGENTS.md 规范编写的表驱动测试

use super::*;

/// InboundMessage::new 构造测试
#[test]
fn inbound_message_construct() {
    let msg = InboundMessage::new("cli", "user123", "chat456", "Hello, world!");

    assert_eq!(msg.channel, "cli");
    assert_eq!(msg.sender_id, "user123");
    assert_eq!(msg.chat_id, "chat456");
    assert_eq!(msg.content, "Hello, world!");
    assert!(msg.metadata.is_empty());
}

/// 出站消息测试用例结构
struct OutboundCase {
    name: &'static str,
    channel: &'static str,
    chat_id: &'static str,
    content: &'static str,
    is_tool_hint: bool,
    #[allow(dead_code)]
    expect_is_progress: bool,
    expect_is_tool_hint: bool,
}

/// OutboundMessage::progress 构造测试
#[test]
fn outbound_message_progress_construct() {
    let test_vector = [
        OutboundCase {
            name: "普通进度消息",
            channel: "cli",
            chat_id: "chat1",
            content: "thinking...",
            is_tool_hint: false,
            expect_is_progress: true,
            expect_is_tool_hint: false,
        },
        OutboundCase {
            name: "工具提示消息",
            channel: "telegram",
            chat_id: "chat2",
            content: "using tool...",
            is_tool_hint: true,
            expect_is_progress: true,
            expect_is_tool_hint: true,
        },
    ];

    for case in test_vector {
        let msg = OutboundMessage::progress(case.channel, case.chat_id, case.content, case.is_tool_hint);

        assert!(msg.is_progress(), "case[{}]: expected is_progress() to be true", case.name);
        assert_eq!(msg.is_tool_hint(), case.expect_is_tool_hint, "case[{}]: is_tool_hint() mismatch", case.name);
        assert_eq!(msg.content, case.content, "case[{}]: content mismatch", case.name);
        assert_eq!(msg.channel, case.channel, "case[{}]: channel mismatch", case.name);
        assert_eq!(msg.chat_id, case.chat_id, "case[{}]: chat_id mismatch", case.name);
    }
}

/// OutboundMessage::new 构造测试
#[test]
fn outbound_message_construct() {
    let msg = OutboundMessage::new("telegram", "chat789", "Response content");

    assert_eq!(msg.channel, "telegram");
    assert_eq!(msg.chat_id, "chat789");
    assert_eq!(msg.content, "Response content");
    assert!(msg.metadata.is_empty());
}

/// OutboundMessage 进度检测测试
#[test]
fn outbound_message_progress_detection() {
    // 进度消息
    let progress_msg = OutboundMessage::progress("cli", "chat1", "thinking...", false);
    assert!(progress_msg.is_progress());

    // 普通消息
    let mut normal_msg = OutboundMessage::new("cli", "chat1", "normal");
    assert!(!normal_msg.is_progress());

    // 手动添加进度标记
    normal_msg.metadata.insert("_progress".to_string(), serde_json::json!(true));
    assert!(normal_msg.is_progress());
}

/// OutboundMessage 工具提示检测测试
#[test]
fn outbound_message_tool_hint_detection() {
    // 工具提示消息
    let tool_hint = OutboundMessage::progress("cli", "chat1", "using tool...", true);
    assert!(tool_hint.is_tool_hint());

    // 普通进度消息
    let normal_progress = OutboundMessage::progress("cli", "chat1", "thinking...", false);
    assert!(!normal_progress.is_tool_hint());

    // 普通消息
    let normal_msg = OutboundMessage::new("cli", "chat1", "normal");
    assert!(!normal_msg.is_tool_hint());
}
