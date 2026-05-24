//! Tests for context builder private functions.

use std::fs;
use std::path::PathBuf;

use nanobot_provider::UserContent;
use tempfile::TempDir;

use super::{build_user_content, encode_image_to_base64, inject_runtime_context};

// Test helper: create a temp workspace with optional files
fn create_test_workspace(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    for (filename, content) in files {
        fs::write(dir.path().join(filename), content).expect("Failed to write file");
    }
    dir
}

// ========== encode_image_to_base64 Tests ==========

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

    let result = encode_image_to_base64(&image_path).unwrap();
    assert!(result.is_some(), "Should encode valid image");

    let (media_type, data) = result.unwrap();
    assert_eq!(media_type, "image/png", "Should have correct MIME type");
    assert!(!data.is_empty(), "Should have non-empty base64 data");
}

#[test]
fn image_encoding_returns_none_for_missing_file() {
    let path = PathBuf::from("/nonexistent/image.png");
    let result = encode_image_to_base64(&path).unwrap();
    assert!(result.is_none(), "Should return None for missing file");
}

#[test]
fn image_encoding_returns_none_for_non_image() {
    let workspace = create_test_workspace(&[("test.txt", "Not an image")]);
    let path = workspace.path().join("test.txt");

    let result = encode_image_to_base64(&path).unwrap();
    assert!(result.is_none(), "Should return None for non-image file");
}

#[test]
fn image_encoding_returns_none_for_directory() {
    let workspace = create_test_workspace(&[]);
    let dir_path = workspace.path().to_path_buf();

    let result = encode_image_to_base64(&dir_path).unwrap();
    assert!(result.is_none(), "Should return None for directory");
}

// ========== build_user_content Tests ==========

#[test]
fn build_user_content_returns_text_when_no_media() {
    let result = build_user_content("Hello, world!", None).unwrap();
    assert!(matches!(result, UserContent::Text(ref s) if s == "Hello, world!"));
}

#[test]
fn build_user_content_returns_text_when_empty_media() {
    let media: Vec<PathBuf> = vec![];
    let result = build_user_content("Hello", Some(&media)).unwrap();
    assert!(matches!(result, UserContent::Text(ref s) if s == "Hello"));
}

#[test]
fn build_user_content_returns_parts_for_valid_image() {
    let workspace = create_test_workspace(&[]);

    // Create a minimal PNG file
    let png_data = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
        0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
        0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let image_path = workspace.path().join("test.png");
    fs::write(&image_path, png_data).expect("Failed to write test image");

    let media = vec![image_path];
    let result = build_user_content("Hello", Some(&media)).unwrap();

    assert!(matches!(result, UserContent::Parts(_)), "Should return Parts for image");
    let text = result.text();
    assert!(text.contains("Hello"), "Should include original text");
}

#[test]
fn build_user_content_skips_invalid_images() {
    let workspace = create_test_workspace(&[("not_image.txt", "Not an image")]);
    let path = workspace.path().join("not_image.txt");

    let media = vec![path];
    let result = build_user_content("Hello", Some(&media)).unwrap();

    assert!(matches!(result, UserContent::Text(ref s) if s == "Hello"));
}

// ========== inject_runtime_context Tests ==========

#[test]
fn inject_runtime_context_adds_time_info() {
    let result = inject_runtime_context(UserContent::Text("Hello".to_string()), None, None);
    let text = result.text();

    assert!(text.starts_with("Hello"), "Should preserve original content");
    assert!(text.contains("[Runtime Context]"), "Should add runtime context block");
    assert!(text.contains("Current Time:"), "Should include current time");
    assert!(!text.contains("Channel:"), "Should not include channel when not provided");
    assert!(!text.contains("Chat ID:"), "Should not include chat ID when not provided");
}

#[test]
fn inject_runtime_context_adds_channel_info() {
    let result = inject_runtime_context(UserContent::Text("Hello".to_string()), Some("telegram"), None);
    let text = result.text();

    assert!(text.contains("Channel: telegram"), "Should include channel");
    assert!(!text.contains("Chat ID:"), "Should not include chat ID when not provided");
}

#[test]
fn inject_runtime_context_adds_chat_id_info() {
    let result = inject_runtime_context(UserContent::Text("Hello".to_string()), None, Some("chat-123"));
    let text = result.text();

    assert!(!text.contains("Channel:"), "Should not include channel when not provided");
    assert!(text.contains("Chat ID: chat-123"), "Should include chat ID");
}

#[test]
fn inject_runtime_context_adds_all_info() {
    let result = inject_runtime_context(UserContent::Text("Hello".to_string()), Some("telegram"), Some("chat-123"));
    let text = result.text();

    assert!(text.contains("Channel: telegram"), "Should include channel");
    assert!(text.contains("Chat ID: chat-123"), "Should include chat ID");
    assert!(text.contains("Current Time:"), "Should include current time");
}

#[test]
fn inject_runtime_context_preserves_multiline_content() {
    let multiline = "Line 1\nLine 2\nLine 3";
    let result = inject_runtime_context(UserContent::Text(multiline.to_string()), None, None);
    let text = result.text();

    assert!(text.contains("Line 1\nLine 2\nLine 3"), "Should preserve multiline content");
    assert!(text.contains("[Runtime Context]"), "Should add runtime context");
}
