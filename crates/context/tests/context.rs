//! Integration tests for context builder.

use std::fs;
use std::path::PathBuf;

use nanobot_context::{ContextBuilder, ContextError};
use nanobot_provider::Message;
use tempfile::TempDir;

// Test helper: create a temp workspace with optional files
fn create_test_workspace(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    for (filename, content) in files {
        fs::write(dir.path().join(filename), content).expect("Failed to write file");
    }
    dir
}

#[test]
fn context_builder_initializes_with_valid_workspace() {
    let workspace = create_test_workspace(&[]);
    let result = ContextBuilder::new(workspace.path().to_path_buf());
    assert!(result.is_ok(), "Should create ContextBuilder with valid workspace");
}

#[test]
fn context_builder_rejects_nonexistent_workspace() {
    let result = ContextBuilder::new(PathBuf::from("/nonexistent/path"));
    assert!(result.is_err(), "Should reject non-existent workspace");

    match result {
        Err(ContextError::InvalidPath(msg)) => {
            assert!(msg.contains("nonexistent"), "Error message should mention path");
        }
        _ => panic!("Expected InvalidPath error"),
    }
}

#[test]
fn core_identity_contains_required_info() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let identity = builder.build_core_identity();

    assert!(identity.contains("# nanobot"), "Should include nanobot header");
    assert!(identity.contains("## Runtime"), "Should include Runtime section");
    assert!(identity.contains("## Workspace"), "Should include Workspace section");
    assert!(identity.contains("MEMORY.md"), "Should mention memory file");
    assert!(identity.contains("HISTORY.md"), "Should mention history file");
    assert!(
        identity.contains("Tool Call Guidelines"),
        "Should include tool guidelines"
    );
}

#[test]
fn system_prompt_contains_core_identity() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    assert!(prompt.contains("# nanobot"), "Should include core identity");
}

#[test]
fn system_prompt_uses_separator_with_memory() {
    // Create workspace with memory content
    let workspace = create_test_workspace(&[]);

    // Create memory directory and file with content
    let memory_dir = workspace.path().join("memory");
    fs::create_dir_all(&memory_dir).expect("Failed to create memory dir");
    fs::write(memory_dir.join("MEMORY.md"), "# Test Memory\nSome memory content").expect("Failed to write memory");

    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    assert!(prompt.contains("# nanobot"), "Should include core identity");
    assert!(prompt.contains("---"), "Should use separator when memory exists");
    assert!(prompt.contains("# Memory"), "Should include memory section");
}

#[test]
fn runtime_context_injects_time() {
    let content = "Hello, assistant!";
    let result = ContextBuilder::inject_runtime_context(content, None, None);

    assert!(
        result.starts_with("Hello, assistant!"),
        "Should preserve original content"
    );
    assert!(result.contains("[Runtime Context]"), "Should add runtime context block");
    assert!(result.contains("Current Time:"), "Should include current time");
    assert!(
        !result.contains("Channel:"),
        "Should not include channel when not provided"
    );
}

#[test]
fn runtime_context_injects_channel_info() {
    let content = "Hello";
    let result = ContextBuilder::inject_runtime_context(content, Some("telegram"), Some("chat-123"));

    assert!(result.contains("Channel: telegram"), "Should include channel");
    assert!(result.contains("Chat ID: chat-123"), "Should include chat ID");
}

#[test]
fn image_encoding_returns_none_for_missing_file() {
    let path = PathBuf::from("/nonexistent/image.png");
    let result = ContextBuilder::encode_image_to_base64(&path).unwrap();
    assert!(result.is_none(), "Should return None for non-existent file");
}

#[test]
fn image_encoding_returns_none_for_non_image() {
    let workspace = create_test_workspace(&[("test.txt", "Not an image")]);
    let path = workspace.path().join("test.txt");
    let result = ContextBuilder::encode_image_to_base64(&path).unwrap();
    assert!(result.is_none(), "Should return None for non-image file");
}

#[test]
fn image_encoding_encodes_valid_image() {
    let workspace = create_test_workspace(&[]);

    // Create a minimal PNG file (1x1 transparent pixel)
    let png_data = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00,
        0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D,
        0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let image_path = workspace.path().join("test.png");
    fs::write(&image_path, png_data).expect("Failed to write test image");

    let result = ContextBuilder::encode_image_to_base64(&image_path).unwrap();
    assert!(result.is_some(), "Should encode valid image");

    let data_url = result.unwrap();
    assert!(
        data_url.starts_with("data:image/png;base64,"),
        "Should have correct MIME type"
    );
}

#[test]
fn message_building_creates_system_and_user() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let messages = builder.build_messages(&[], "Hello", None, None, None).unwrap();

    assert_eq!(messages.len(), 2, "Should have system and user messages");
    assert_eq!(messages[0].role(), "system", "First message should be system");
    assert_eq!(messages[1].role(), "user", "Second message should be user");
    assert!(
        messages[1].content().contains("Hello"),
        "User message should contain original content"
    );
    assert!(
        messages[1].content().contains("[Runtime Context]"),
        "User message should have runtime context"
    );
}

#[test]
fn message_building_includes_history() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let history = vec![
        Message::user("Previous question"),
        Message::assistant("Previous answer"),
    ];

    let messages = builder
        .build_messages(&history, "New question", None, None, None)
        .unwrap();

    assert_eq!(messages.len(), 4, "Should have system, history (2), and user messages");
    assert_eq!(messages[0].role(), "system", "First should be system");
    assert_eq!(messages[1].role(), "user", "Second should be user from history");
    assert_eq!(
        messages[2].role(),
        "assistant",
        "Third should be assistant from history"
    );
    assert_eq!(messages[3].role(), "user", "Fourth should be current user");
}

#[test]
fn tool_result_appends_correctly() {
    let mut messages = vec![Message::user("Question")];

    ContextBuilder::append_tool_result(&mut messages, "call-123", "Tool output");

    assert_eq!(messages.len(), 2, "Should have 2 messages");
    assert_eq!(messages[1].role(), "tool", "Should be tool message");
    assert_eq!(messages[1].content(), "Tool output", "Should have tool result");
    assert_eq!(messages[1].tool_call_id(), Some("call-123"), "Should have tool call ID");
}

#[test]
fn assistant_message_appends_correctly() {
    let mut messages = vec![Message::user("Question")];

    ContextBuilder::append_assistant_message(&mut messages, "Response", vec![]);

    assert_eq!(messages.len(), 2, "Should have 2 messages");
    assert_eq!(messages[1].role(), "assistant", "Should be assistant message");
    assert_eq!(messages[1].content(), "Response", "Should have content");
}
