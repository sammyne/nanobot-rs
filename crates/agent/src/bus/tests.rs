//! Tests for the `bus` module.
//!
//! This module contains unit tests for `InboundMessage` and `OutboundMessage`,
//! including serialization, construction, and metadata handling.

use super::*;

#[tokio::test]
async fn message_basic() {
    // Test inbound message
    let (inbound_tx, mut inbound_rx) = tokio::sync::mpsc::channel(1);
    let inbound = InboundMessage::new("cli", "user", "chat1", "Hello");
    inbound_tx.send(inbound.clone()).await.unwrap();

    let received = inbound_rx.recv().await.unwrap();
    assert_eq!(received.channel, "cli");
    assert_eq!(received.content, "Hello");

    // Test outbound message
    let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel(1);
    let outbound = OutboundMessage::new("cli", "chat1", "World");
    outbound_tx.send(outbound).await.unwrap();

    let received = outbound_rx.recv().await.unwrap();
    assert_eq!(received.content, "World");
}

#[tokio::test]
async fn progress_message() {
    let progress = OutboundMessage::progress("thinking...", false);
    assert!(progress.is_progress());
    assert!(!progress.is_tool_hint());

    let tool_hint = OutboundMessage::progress("using tool...", true);
    assert!(tool_hint.is_progress());
    assert!(tool_hint.is_tool_hint());
}

#[test]
fn parse_session_id() {
    let result = InboundMessage::from_session_id("cli:direct", "hello");
    assert!(result.is_some());
    let (channel, sender_id, chat_id) = result.unwrap();
    assert_eq!(channel, "cli");
    assert_eq!(sender_id, "user");
    assert_eq!(chat_id, "direct");

    // Test with multiple colons
    let result = InboundMessage::from_session_id("telegram:123:456", "hello");
    assert!(result.is_some());
    let (channel, _sender_id, chat_id) = result.unwrap();
    assert_eq!(channel, "telegram");
    assert_eq!(chat_id, "123:456");
}
