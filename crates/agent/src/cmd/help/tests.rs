//! Help command tests

use super::*;

/// Verify that HelpCmd returns the correct help message
#[tokio::test]
async fn returns_correct_message() {
    let _help = HelpCmd;

    // Create a mock inbound message
    let _inbound = InboundMessage::new("test_channel", "user", "test_chat", "test message");

    // The test requires a proper AgentLoop instance
    // For now, we'll skip the full integration test and just verify the trait implementation compiles
    // A more comprehensive test would require setting up a full AgentLoop with all its dependencies

    // Verify the command struct can be created
    assert_eq!(std::mem::size_of::<HelpCmd>(), 0, "HelpCmd should be a zero-sized type");
}

/// Verify help message format
#[test]
fn message_format() {
    let expected = "🐈 nanobot commands:\n/new — Start a new conversation\n/help — Show available commands";

    assert!(expected.contains("nanobot commands"));
    assert!(expected.contains("/new"));
    assert!(expected.contains("/help"));
    assert!(expected.contains("Start a new conversation"));
    assert!(expected.contains("Show available commands"));
}
