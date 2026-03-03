//! Integration tests for MemoryStore

use nanobot_memory::MemoryStore;

#[test]
fn new_creates_memory_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace.clone()).expect("Failed to create MemoryStore");

    // Verify memory directory exists
    assert!(workspace.join("memory").exists());
    assert!(store.read_long_term().expect("Failed to read").is_empty());
}

#[test]
fn read_long_term_returns_empty_when_file_not_exists() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace).expect("Failed to create MemoryStore");

    let content = store.read_long_term().expect("Failed to read");
    assert!(content.is_empty());
}

#[test]
fn write_and_read_long_term() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace).expect("Failed to create MemoryStore");

    let test_content = "# Test Memory\n\nThis is a test memory entry.";
    store.write_long_term(test_content).expect("Failed to write");

    let content = store.read_long_term().expect("Failed to read");
    assert_eq!(content, test_content);
}

#[test]
fn append_history() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace).expect("Failed to create MemoryStore");

    let entry1 = "[2026-03-03 10:00] User asked about Rust.";
    let entry2 = "[2026-03-03 10:05] Assistant explained ownership.";

    store.append_history(entry1).expect("Failed to append entry1");
    store.append_history(entry2).expect("Failed to append entry2");

    // Verify history file contains both entries
    let history_path = temp_dir.path().join("memory").join("HISTORY.md");
    let content = std::fs::read_to_string(history_path).expect("Failed to read history");

    assert!(content.contains(entry1));
    assert!(content.contains(entry2));
}

#[test]
fn get_memory_context_returns_empty_when_no_memory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace).expect("Failed to create MemoryStore");

    let context = store.get_memory_context().expect("Failed to get context");
    assert!(context.is_empty());
}

#[test]
fn get_memory_context_formats_memory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();

    let store = MemoryStore::new(workspace).expect("Failed to create MemoryStore");

    let test_content = "# User Preferences\n- Likes Rust\n- Uses VS Code";
    store.write_long_term(test_content).expect("Failed to write");

    let context = store.get_memory_context().expect("Failed to get context");
    assert!(context.starts_with("## Long-term Memory\n"));
    assert!(context.contains(test_content));
}
