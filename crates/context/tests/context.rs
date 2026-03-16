//! Integration tests for context module.
//!
//! These tests verify the public API of the context module.

use std::fs;
use std::path::PathBuf;

use nanobot_context::ContextBuilder;
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
    assert!(identity.contains("Tool Call Guidelines"), "Should include tool guidelines");
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
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let messages = builder.build_messages(&[], "Hello, assistant!", None, None, None).unwrap();
    let user_content = &messages[1].content();

    assert!(user_content.starts_with("Hello, assistant!"), "Should preserve original content");
    assert!(user_content.contains("[Runtime Context]"), "Should add runtime context block");
    assert!(user_content.contains("Current Time:"), "Should include current time");
    assert!(!user_content.contains("Channel:"), "Should not include channel when not provided");
}

#[test]
fn runtime_context_injects_channel_info() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let messages = builder.build_messages(&[], "Hello", None, Some("telegram"), Some("chat-123")).unwrap();
    let user_content = &messages[1].content();

    assert!(user_content.contains("Channel: telegram"), "Should include channel");
    assert!(user_content.contains("Chat ID: chat-123"), "Should include chat ID");
}

#[test]
fn image_encoding_returns_none_for_missing_file() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let path = PathBuf::from("/nonexistent/image.png");
    let messages = builder.build_messages(&[], "Hello", Some(&[path]), None, None).unwrap();
    let user_content = &messages[1].content();

    // Missing file should not add image info
    assert!(!user_content.contains("[Image attached:"), "Should not include image info for missing file");
    assert!(user_content.contains("Hello"), "Should include original message");
}

#[test]
fn image_encoding_returns_none_for_non_image() {
    let workspace = create_test_workspace(&[("test.txt", "Not an image")]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let path = workspace.path().join("test.txt");
    let messages = builder.build_messages(&[], "Hello", Some(&[path]), None, None).unwrap();
    let user_content = &messages[1].content();

    // Non-image file should not add image info
    assert!(!user_content.contains("[Image attached:"), "Should not include image info for non-image file");
    assert!(user_content.contains("Hello"), "Should include original message");
}

#[test]
fn message_building_creates_system_and_user() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let messages = builder.build_messages(&[], "Hello", None, None, None).unwrap();

    assert_eq!(messages.len(), 2, "Should have system and user messages");
    assert_eq!(messages[0].role(), "system", "First message should be system");
    assert_eq!(messages[1].role(), "user", "Second message should be user");
    assert!(messages[1].content().contains("Hello"), "User message should contain original content");
    assert!(messages[1].content().contains("[Runtime Context]"), "User message should have runtime context");
}

#[test]
fn message_building_includes_history() {
    let workspace = create_test_workspace(&[]);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let history = vec![Message::user("Previous question"), Message::assistant("Previous answer")];

    let messages = builder.build_messages(&history, "New question", None, None, None).unwrap();

    assert_eq!(messages.len(), 4, "Should have system, history (2), and user messages");
    assert_eq!(messages[0].role(), "system", "First should be system");
    assert_eq!(messages[1].role(), "user", "Second should be user from history");
    assert_eq!(messages[2].role(), "assistant", "Third should be assistant from history");
    assert_eq!(messages[3].role(), "user", "Fourth should be current user");
}

#[test]
fn tool_message_created_correctly() {
    let message = Message::tool("call-123", "Tool output");

    assert_eq!(message.role(), "tool", "Should be tool message");
    assert_eq!(message.content(), "Tool output", "Should have tool result");
    assert_eq!(message.tool_call_id(), Some("call-123"), "Should have tool call ID");
}

#[test]
fn assistant_message_created_correctly() {
    let message = Message::assistant("Response");

    assert_eq!(message.role(), "assistant", "Should be assistant message");
    assert_eq!(message.content(), "Response", "Should have content");
}

// ========== Bootstrap File Tests ==========

#[test]
fn bootstrap_files_loads_all_files() {
    let files = vec![
        ("AGENTS.md", "This is the AGENTS.md content"),
        ("SOUL.md", "This is the SOUL.md content"),
        ("USER.md", "This is the USER.md content"),
        ("TOOLS.md", "This is the TOOLS.md content"),
        ("IDENTITY.md", "This is the IDENTITY.md content"),
    ];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    assert!(bootstrap.contains("## AGENTS.md"), "Should include AGENTS.md section");
    assert!(bootstrap.contains("This is the AGENTS.md content"), "Should include AGENTS.md content");
    assert!(bootstrap.contains("## SOUL.md"), "Should include SOUL.md section");
    assert!(bootstrap.contains("This is the SOUL.md content"), "Should include SOUL.md content");
    assert!(bootstrap.contains("## USER.md"), "Should include USER.md section");
    assert!(bootstrap.contains("## TOOLS.md"), "Should include TOOLS.md section");
    assert!(bootstrap.contains("## IDENTITY.md"), "Should include IDENTITY.md section");
}

#[test]
fn bootstrap_files_handles_missing_files() {
    // Only create AGENTS.md, leave others missing
    let files = vec![("AGENTS.md", "AGENTS content")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    // Should include only AGENTS.md
    assert!(bootstrap.contains("## AGENTS.md"), "Should include AGENTS.md section");
    assert!(!bootstrap.contains("## SOUL.md"), "Should not include missing SOUL.md");
    assert!(!bootstrap.contains("## USER.md"), "Should not include missing USER.md");
}

#[test]
fn bootstrap_files_skips_empty_files() {
    let files = vec![
        ("AGENTS.md", "Non-empty content"),
        ("SOUL.md", "   \n\n   "), // Only whitespace
        ("USER.md", ""),           // Empty
    ];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    assert!(bootstrap.contains("## AGENTS.md"), "Should include non-empty AGENTS.md");
    assert!(!bootstrap.contains("## SOUL.md"), "Should skip whitespace-only SOUL.md");
    assert!(!bootstrap.contains("## USER.md"), "Should skip empty USER.md");
}

#[test]
fn bootstrap_files_returns_empty_when_no_valid_files() {
    let files = vec![
        ("SOUL.md", "   \n\n   "), // Only whitespace
        ("USER.md", ""),           // Empty
    ];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    assert!(bootstrap.is_empty(), "Should return empty string when no valid files");
}

#[test]
fn bootstrap_files_handles_io_errors() {
    let files = vec![("AGENTS.md", "AGENTS content")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    // Should successfully load AGENTS.md even if other files don't exist
    assert!(bootstrap.contains("## AGENTS.md"), "Should load existing file");
}

#[test]
fn bootstrap_files_maintains_order() {
    let files = vec![("AGENTS.md", "1"), ("SOUL.md", "2"), ("USER.md", "3"), ("TOOLS.md", "4"), ("IDENTITY.md", "5")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    // Check that sections appear in the correct order
    let agents_pos = bootstrap.find("## AGENTS.md").unwrap();
    let soul_pos = bootstrap.find("## SOUL.md").unwrap();
    let user_pos = bootstrap.find("## USER.md").unwrap();
    let tools_pos = bootstrap.find("## TOOLS.md").unwrap();
    let identity_pos = bootstrap.find("## IDENTITY.md").unwrap();

    assert!(
        agents_pos < soul_pos && soul_pos < user_pos && user_pos < tools_pos && tools_pos < identity_pos,
        "Sections should be in the correct order"
    );
}

#[test]
fn system_prompt_includes_bootstrap_files() {
    let files = vec![("AGENTS.md", "Bootstrap content")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let prompt = builder.build_system_prompt().unwrap();

    assert!(prompt.contains("# nanobot"), "Should include core identity");
    assert!(prompt.contains("## AGENTS.md"), "Should include bootstrap section");
    assert!(prompt.contains("Bootstrap content"), "Should include bootstrap content");
}

#[test]
fn system_prompt_uses_separator_with_bootstrap_files() {
    let files = vec![("AGENTS.md", "Bootstrap content")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let prompt = builder.build_system_prompt().unwrap();

    let core_identity_end = prompt.find("# nanobot").unwrap();
    let separator = prompt.find("---").unwrap();
    let bootstrap_section = prompt.find("## AGENTS.md").unwrap();

    assert!(
        core_identity_end < separator && separator < bootstrap_section,
        "Separator should be between core identity and bootstrap content"
    );
}

#[test]
fn system_prompt_skips_separator_without_bootstrap_files() {
    // No bootstrap files, create only memory
    let workspace = create_test_workspace(&[]);
    let memory_dir = workspace.path().join("memory");
    fs::create_dir_all(&memory_dir).expect("Failed to create memory dir");
    fs::write(memory_dir.join("MEMORY.md"), "Memory content").expect("Failed to write memory");

    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    assert!(prompt.contains("# nanobot"), "Should include core identity");
    assert!(prompt.contains("# Memory"), "Should include memory section");

    // Verify there are no bootstrap sections
    assert!(!prompt.contains("## AGENTS.md"), "Should not have bootstrap files");
    assert!(!prompt.contains("## SOUL.md"), "Should not have bootstrap files");
    assert!(!prompt.contains("## USER.md"), "Should not have bootstrap files");
}

#[test]
fn system_prompt_assembly_order() {
    let files = vec![("AGENTS.md", "Bootstrap section")];
    let workspace = create_test_workspace(&files);

    // Create memory
    let memory_dir = workspace.path().join("memory");
    fs::create_dir_all(&memory_dir).expect("Failed to create memory dir");
    fs::write(memory_dir.join("MEMORY.md"), "Memory section").expect("Failed to write memory");

    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    // Verify all sections exist
    assert!(prompt.contains("# nanobot"), "Should have core identity");
    assert!(prompt.contains("## AGENTS.md"), "Should have bootstrap section");
    assert!(prompt.contains("# Memory\n\n"), "Should have memory section header");

    // Find positions of key sections in the full prompt
    let core_pos = prompt.find("# nanobot").expect("Should find core identity");
    let bootstrap_pos = prompt.find("## AGENTS.md").expect("Should find bootstrap section");
    let memory_pos = prompt.find("# Memory\n\n").expect("Should find memory section header");

    // Verify order: core identity -> bootstrap -> memory
    assert!(
        core_pos < bootstrap_pos && bootstrap_pos < memory_pos,
        "Order should be: core identity -> bootstrap -> memory"
    );
}

// ========== End-to-End Integration Tests ==========

#[test]
fn end_to_end_full_bootstrap_integration() {
    // Create a realistic set of bootstrap files
    let files = vec![
        ("AGENTS.md", "# Agent Configuration\n\nYou are configured as a helpful assistant."),
        ("SOUL.md", "# Core Principles\n\nBe helpful, honest, and respectful."),
        ("USER.md", "# User Preferences\n\nUser prefers concise answers."),
        ("TOOLS.md", "# Available Tools\n\n- read_file: Read file contents\n- write_file: Write file contents"),
        ("IDENTITY.md", "# Identity\n\nName: nanobot\nVersion: 1.0.0"),
    ];
    let workspace = create_test_workspace(&files);

    // Create memory
    let memory_dir = workspace.path().join("memory");
    fs::create_dir_all(&memory_dir).expect("Failed to create memory dir");
    fs::write(memory_dir.join("MEMORY.md"), "# Memory\n\nImportant information stored here.")
        .expect("Failed to write memory");

    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    // Verify all bootstrap sections are present
    assert!(prompt.contains("## AGENTS.md"), "Should include AGENTS.md section");
    assert!(prompt.contains("## SOUL.md"), "Should include SOUL.md section");
    assert!(prompt.contains("## USER.md"), "Should include USER.md section");
    assert!(prompt.contains("## TOOLS.md"), "Should include TOOLS.md section");
    assert!(prompt.contains("## IDENTITY.md"), "Should include IDENTITY.md section");

    // Verify bootstrap content is included
    assert!(prompt.contains("You are configured as a helpful assistant"), "Should include AGENTS.md content");
    assert!(prompt.contains("Be helpful, honest, and respectful"), "Should include SOUL.md content");
    assert!(prompt.contains("User prefers concise answers"), "Should include USER.md content");
    assert!(prompt.contains("- read_file: Read file contents"), "Should include TOOLS.md content");
    assert!(prompt.contains("Name: nanobot"), "Should include IDENTITY.md content");

    // Verify order
    let parts: Vec<&str> = prompt.split("\n\n---\n\n").collect();
    assert!(parts.len() >= 3, "Should have at least 3 parts (core, bootstrap, memory)");

    // First part should be core identity
    assert!(parts[0].contains("# nanobot"), "First part should be core identity");

    // Bootstrap parts should be in the middle
    let bootstrap_start = prompt.find("## AGENTS.md").unwrap();
    let bootstrap_end = prompt.find("# Memory\n\n").unwrap();
    let bootstrap_content = &prompt[bootstrap_start..bootstrap_end];
    assert!(bootstrap_content.contains("## AGENTS.md"), "Should contain bootstrap files");
    assert!(bootstrap_content.contains("## SOUL.md"), "Should contain bootstrap files");
    assert!(bootstrap_content.contains("## USER.md"), "Should contain bootstrap files");
    assert!(bootstrap_content.contains("## TOOLS.md"), "Should contain bootstrap files");
    assert!(bootstrap_content.contains("## IDENTITY.md"), "Should contain bootstrap files");

    // Last parts should include memory
    assert!(prompt.contains("# Memory\n\n"), "Should include memory section");
}

#[test]
fn end_to_end_empty_bootstrap_scenario() {
    // Create workspace without bootstrap files
    let workspace = create_test_workspace(&[]);

    // Create memory only
    let memory_dir = workspace.path().join("memory");
    fs::create_dir_all(&memory_dir).expect("Failed to create memory dir");
    fs::write(memory_dir.join("MEMORY.md"), "Memory content").expect("Failed to write memory");

    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();
    let prompt = builder.build_system_prompt().unwrap();

    // Should still build successfully without bootstrap files
    assert!(prompt.contains("# nanobot"), "Should include core identity");
    assert!(!prompt.contains("## AGENTS.md"), "Should not include bootstrap files");
    assert!(!prompt.contains("## SOUL.md"), "Should not include bootstrap files");
    assert!(prompt.contains("# Memory\n\n"), "Should include memory section");
}

#[test]
fn end_to_output_format_consistency() {
    // Verify that the output format matches expectations
    let files = vec![("AGENTS.md", "Test content")];
    let workspace = create_test_workspace(&files);
    let builder = ContextBuilder::new(workspace.path().to_path_buf()).unwrap();

    let bootstrap = builder.load_bootstrap_files();

    // Check format: should have header, double newline, then content
    assert_eq!(bootstrap, "## AGENTS.md\n\nTest content", "Bootstrap format should match Python version");
}
