use std::sync::Arc;

use super::*;
use crate::store::MemoryStore;

#[test]
fn dream_cursor_read_write() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().to_path_buf();
    let memory = Arc::new(MemoryStore::new(workspace.clone()).unwrap());
    let config = DreamConfig::default();
    let dream = Dream::new(memory, workspace, config).unwrap();

    // Default cursor is 0 when file doesn't exist.
    assert_eq!(dream.read_dream_cursor(), 0);

    // Write and read back.
    dream.write_dream_cursor(42).unwrap();
    assert_eq!(dream.read_dream_cursor(), 42);

    // Overwrite with a different value.
    dream.write_dream_cursor(100).unwrap();
    assert_eq!(dream.read_dream_cursor(), 100);
}

#[test]
fn phase1_prompt_format() {
    let entries = vec![
        HistoryEntry {
            cursor: 1,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            content: "User decided to use PostgreSQL".to_string(),
        },
        HistoryEntry {
            cursor: 2,
            timestamp: "2026-01-01T01:00:00Z".to_string(),
            content: "Deployed to staging".to_string(),
        },
    ];

    let prompt = build_phase1_prompt(&entries, "existing memory", "soul style", "user prefs");

    // History entries are included.
    assert!(prompt.contains("User decided to use PostgreSQL"), "prompt should contain entry 1");
    assert!(prompt.contains("Deployed to staging"), "prompt should contain entry 2");
    assert!(prompt.contains("cursor=1"), "prompt should contain cursor 1");
    assert!(prompt.contains("cursor=2"), "prompt should contain cursor 2");

    // Memory file contents are included.
    assert!(prompt.contains("existing memory"), "prompt should contain MEMORY.md content");
    assert!(prompt.contains("soul style"), "prompt should contain SOUL.md content");
    assert!(prompt.contains("user prefs"), "prompt should contain USER.md content");

    // Section headers are present.
    assert!(prompt.contains("## New History Entries"));
    assert!(prompt.contains("## Current MEMORY.md"));
    assert!(prompt.contains("## Current SOUL.md"));
    assert!(prompt.contains("## Current USER.md"));

    // Empty contents show "(empty)".
    let prompt_empty = build_phase1_prompt(&entries, "", "", "");
    assert_eq!(prompt_empty.matches("(empty)").count(), 3);
}

#[test]
fn phase2_parse_instructions() {
    // parse_phase1_response: extract [FILE] fact lines from mixed text.
    let response = "\
Here are the facts I identified:

[MEMORY.md] Project migrated from SQLite to PostgreSQL
[USER.md] User prefers vim keybindings
Some commentary line
[SOUL.md] Avoid using emoji in responses

That's all.";

    let instructions = parse_phase1_response(response);
    assert_eq!(instructions.len(), 3);
    assert_eq!(instructions[0], "[MEMORY.md] Project migrated from SQLite to PostgreSQL");
    assert_eq!(instructions[1], "[USER.md] User prefers vim keybindings");
    assert_eq!(instructions[2], "[SOUL.md] Avoid using emoji in responses");

    // parse_instruction: split into (file, fact).
    let (file, fact) = parse_instruction("[MEMORY.md] Project uses PostgreSQL").unwrap();
    assert_eq!(file, "MEMORY.md");
    assert_eq!(fact, "Project uses PostgreSQL");

    let (file, fact) = parse_instruction("  [USER.md]   prefers dark mode  ").unwrap();
    assert_eq!(file, "USER.md");
    assert_eq!(fact, "prefers dark mode");

    // Invalid formats return None.
    assert!(parse_instruction("no brackets here").is_none());
    assert!(parse_instruction("[MEMORY.md]").is_none());
    assert!(parse_instruction("[MEMORY.md]   ").is_none());
    assert!(parse_instruction("").is_none());
}
