//! Tests for SessionManager.

use std::path::PathBuf;

use nanobot_session::{Message, Session, SessionManager};
use serde_json::json;
use tempfile::TempDir;

fn create_temp_workspace() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace = temp_dir.path().to_path_buf();
    (temp_dir, workspace)
}

#[test]
fn session_manager_new() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace.clone());

    assert_eq!(manager.workspace(), &workspace);
    assert!(manager.sessions_dir().ends_with("sessions"));
}

#[test]
fn get_or_create_new_session() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    let session = manager.get_or_create("test:123");
    assert_eq!(session.key, "test:123");
    assert!(session.messages.is_empty());
}

#[test]
fn get_or_create_cached_session() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create session and add a message
    {
        let session = manager.get_or_create("test:123");
        let mut session = session;
        session.add_message(Message::user("Hello"));
        manager.save(&session).expect("Failed to save session");
    }

    // Get session again from cache
    let session = manager.get_or_create("test:123");
    assert_eq!(session.messages.len(), 1);
}

#[test]
fn save_and_load_session() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create and save a session
    let mut session = Session::new("test:456");
    session.add_message(Message::user("Hello"));
    session.add_message(Message::assistant("Hi there"));
    manager.save(&session).expect("Failed to save session");

    // Invalidate cache to force reload from disk
    manager.invalidate("test:456");

    // Load session from disk
    let loaded = manager.get_or_create("test:456");
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[0].content(), "Hello");
    assert_eq!(loaded.messages[1].content(), "Hi there");
}

#[test]
fn invalidate_session() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create session
    let session = manager.get_or_create("test:789");

    // Modify and save
    let mut session = session;
    session.add_message(Message::user("Test"));
    manager.save(&session).expect("Failed to save session");

    // Invalidate cache
    manager.invalidate("test:789");

    // Get session again - should load from disk
    let loaded = manager.get_or_create("test:789");
    assert_eq!(loaded.messages.len(), 1);
}

#[test]
fn list_sessions() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create multiple sessions
    for i in 0..3 {
        let mut session = Session::new(format!("test:{i}"));
        session.add_message(Message::user(format!("Message {i}")));
        manager.save(&session).expect("Failed to save session");
    }

    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 3);

    // Check that sessions are sorted by updated_at descending
    for i in 0..sessions.len() - 1 {
        assert!(sessions[i].updated_at >= sessions[i + 1].updated_at);
    }
}

#[test]
fn session_info_fields() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create and save a session
    let session = Session::new("test:info");
    manager.save(&session).expect("Failed to save session");

    let sessions = manager.list_sessions();
    assert!(!sessions.is_empty());

    let info = sessions
        .iter()
        .find(|s| s.key == "test:info")
        .expect("Session not found");
    assert_eq!(info.key, "test:info");
    assert!(info.path.ends_with(".jsonl"));
}

#[test]
fn persist_metadata() {
    let (_temp, workspace) = create_temp_workspace();
    let manager = SessionManager::new(workspace);

    // Create session with metadata
    let mut session = Session::new("test:metadata");
    session.metadata.insert("custom".to_string(), json!("value"));
    session.last_consolidated = 5;
    manager.save(&session).expect("Failed to save session");

    // Invalidate cache and reload
    manager.invalidate("test:metadata");
    let loaded = manager.get_or_create("test:metadata");

    assert_eq!(loaded.metadata.get("custom").unwrap(), "value");
    assert_eq!(loaded.last_consolidated, 5);
}
