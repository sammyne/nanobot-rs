//! 工作空间初始化模块测试

use std::fs;

use tempfile::TempDir;

use super::initializer::WorkspaceInitializer;

#[test]
fn initialize_workspace() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().to_path_buf();

    // Act
    let initializer = WorkspaceInitializer::new(workspace_path.clone());
    initializer.initialize().unwrap();

    // Assert
    assert!(workspace_path.exists());
    assert!(workspace_path.join("memory").exists());
    assert!(workspace_path.join("skills").exists());

    // 检查根级别文件
    assert!(workspace_path.join("USER.md").exists());
    assert!(workspace_path.join("AGENTS.md").exists());
    assert!(workspace_path.join("SOUL.md").exists());
    assert!(workspace_path.join("TOOLS.md").exists());

    // 检查 memory 文件
    assert!(workspace_path.join("memory/MEMORY.md").exists());
    assert!(workspace_path.join("memory/HISTORY.md").exists());
}

#[test]
fn dont_overwrite_existing_files() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().to_path_buf();
    let user_file = workspace_path.join("USER.md");
    fs::write(&user_file, "existing content").unwrap();

    // Act
    let initializer = WorkspaceInitializer::new(workspace_path.clone());
    initializer.initialize().unwrap();

    // Assert
    let content = fs::read_to_string(&user_file).unwrap();
    assert_eq!(content, "existing content");
}
