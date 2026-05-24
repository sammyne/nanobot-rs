//! sync_workspace_templates 单元测试

use std::fs;

use tempfile::TempDir;

use super::*;

#[test]
fn sync_creates_all_templates_in_empty_dir() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Act
    let created = sync_workspace_templates(temp_dir.path()).unwrap();

    // Assert
    assert_eq!(created.len(), 6);
    for path in &created {
        let full_path = temp_dir.path().join(path);
        assert!(full_path.exists(), "file should exist: {path}");
        let content = fs::read_to_string(&full_path).unwrap();
        assert!(!content.is_empty(), "file should not be empty: {path}");
    }
}

#[test]
fn sync_skips_existing_files() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let user_file = temp_dir.path().join("USER.md");
    fs::write(&user_file, "my custom content").unwrap();

    // Act
    let created = sync_workspace_templates(temp_dir.path()).unwrap();

    // Assert
    assert!(!created.contains(&"USER.md"), "should not recreate existing file");
    let content = fs::read_to_string(&user_file).unwrap();
    assert_eq!(content, "my custom content");
}

#[test]
fn sync_creates_subdirectories() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();

    // Act
    sync_workspace_templates(temp_dir.path()).unwrap();

    // Assert
    let memory_file = temp_dir.path().join("memory/MEMORY.md");
    assert!(memory_file.exists(), "memory/MEMORY.md should be created");
    let content = fs::read_to_string(&memory_file).unwrap();
    assert!(!content.is_empty());
}
