//! Tests for Session data model.

use nanobot_session::{Message, Session};

#[test]
fn session_new() {
    let session = Session::new("test:123");
    assert_eq!(session.key, "test:123");
    assert!(session.messages.is_empty());
    assert!(session.metadata.is_empty());
    assert_eq!(session.last_consolidated, 0);
}

#[test]
fn session_add_message() {
    let mut session = Session::new("test:123");
    let msg = Message::user("Hello");
    session.add_message(msg);
    assert_eq!(session.messages.len(), 1);
    assert!(matches!(session.messages[0], Message::User { .. }));
    assert_eq!(session.messages[0].content(), "Hello");
}

#[test]
fn session_get_history_max_messages() {
    let mut session = Session::new("test:123");

    // Add 10 messages
    for i in 0..10 {
        let msg = Message::user(format!("Message {i}"));
        session.add_message(msg);
    }

    // Get history with max_messages = 5
    let mut history = Vec::new();
    session.get_history(5, 0, &mut history);
    assert_eq!(history.len(), 5);

    // Should get the last 5 messages
    assert_eq!(history[0].content(), "Message 5");
    assert_eq!(history[4].content(), "Message 9");
}

#[test]
fn session_get_history_user_alignment() {
    let mut session = Session::new("test:123");

    // Add messages starting with assistant (non-user)
    let msg1 = Message::assistant("Hi there");
    let msg2 = Message::tool("123", "result");
    let msg3 = Message::user("User message");
    let msg4 = Message::assistant("Response");

    session.add_message(msg1);
    session.add_message(msg2);
    session.add_message(msg3);
    session.add_message(msg4);

    let mut history = Vec::new();
    session.get_history(10, 0, &mut history);

    // Should drop leading non-user messages
    assert_eq!(history.len(), 2);
    assert!(matches!(history[0], Message::User { .. }));
    assert!(matches!(history[1], Message::Assistant { .. }));
}

#[test]
fn session_clear() {
    let mut session = Session::new("test:123");

    // Add some messages
    session.add_message(Message::user("Hello"));
    session.last_consolidated = 5;

    session.clear();

    assert!(session.messages.is_empty());
    assert_eq!(session.last_consolidated, 0);
}

#[test]
fn session_get_history_with_consolidation() {
    let mut session = Session::new("test:123");

    // Add 10 messages
    for i in 0..10 {
        let msg = Message::user(format!("Message {i}"));
        session.add_message(msg);
    }

    // Set last_consolidated to 5
    session.last_consolidated = 5;

    // Get history should only return messages after index 5
    let mut history = Vec::new();
    session.get_history(10, 0, &mut history);
    assert_eq!(history.len(), 5);
    assert_eq!(history[0].content(), "Message 5");
}

#[test]
fn save_turn_strips_runtime_context() {
    let mut session = Session::new("test:123");

    let msg_with_context =
        Message::user("帮我查一下今天的日程\n\n[Runtime Context]\nCurrent Time: 2026-05-24 10:00 (Saturday) (+08:00)");

    session.save_turn(&[msg_with_context], 0);

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content(), "帮我查一下今天的日程");
}

#[test]
fn save_turn_preserves_message_without_runtime_context() {
    let mut session = Session::new("test:123");

    let msg = Message::user("Hello, world!");

    session.save_turn(&[msg], 0);

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content(), "Hello, world!");
}

#[test]
fn save_turn_strips_runtime_context_with_channel_info() {
    let mut session = Session::new("test:123");

    let content = "查看群消息\n\n[Runtime Context]\nCurrent Time: 2026-05-24 10:00 (Saturday) (+08:00)\nChannel: feishu\nChat ID: oc_xxx";
    let msg = Message::user(content);

    session.save_turn(&[msg], 0);

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].content(), "查看群消息");
}

#[test]
fn session_get_history_token_budget() {
    let mut session = Session::new("test:token");

    // Add user messages with known sizes
    // Each message: "msgN" (4 bytes) + "x"*40 (40 bytes) = 44 bytes → 11 content tokens + 4 overhead = 15 tokens
    for i in 0..5 {
        session.add_message(Message::user(format!("msg{i}{}", "x".repeat(40))));
    }

    // With max_tokens=0, all 5 messages returned (no token limit)
    let mut buf = Vec::new();
    let count = session.get_history(100, 0, &mut buf);
    assert_eq!(count, 5);

    // Budget of 30 should fit exactly 2 messages (2 * 15 = 30)
    let mut buf = Vec::new();
    let count = session.get_history(100, 30, &mut buf);
    assert_eq!(count, 2);

    // Budget of 15 should fit exactly 1 message
    let mut buf = Vec::new();
    let count = session.get_history(100, 15, &mut buf);
    assert_eq!(count, 1);

    // Budget of 14 should fit 0 messages (each message = 15 tokens)
    let mut buf = Vec::new();
    let count = session.get_history(100, 14, &mut buf);
    assert_eq!(count, 0);
}
