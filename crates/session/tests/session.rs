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
    session.get_history(5, &mut history);
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
    session.get_history(10, &mut history);

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
    session.get_history(10, &mut history);
    assert_eq!(history.len(), 5);
    assert_eq!(history[0].content(), "Message 5");
}
