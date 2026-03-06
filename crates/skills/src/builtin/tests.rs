//! Tests for builtin skills management.

use std::fs;

use tempfile::TempDir;

use super::*;

#[test]
fn extracts_builtin_skills_from_embedded_resources() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Initialize from embedded resources
    initialize_builtin_skills(&builtin_dir).unwrap();

    // Verify that builtin skills were extracted
    assert!(builtin_dir.exists());

    // Verify tavily-search skill exists (from embedded builtin dir)
    assert!(builtin_dir.join("tavily-search/SKILL.md").exists());
}

#[test]
fn removes_existing_builtin_skills_directory() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Create builtin-skills directory
    fs::create_dir_all(builtin_dir.join("skill1")).unwrap();
    fs::write(builtin_dir.join("skill1/SKILL.md"), "test").unwrap();

    // Remove
    remove_builtin_skills(&builtin_dir).unwrap();

    // Verify
    assert!(!builtin_dir.exists());
}

#[test]
fn handles_nonexistent_directory_gracefully() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Should not error when directory doesn't exist
    remove_builtin_skills(&builtin_dir).unwrap();
}

#[test]
fn initializes_builtin_skills_on_first_run() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Ensure (first time)
    ensure_builtin_skills(&builtin_dir).unwrap();

    // Verify
    assert!(builtin_dir.join("tavily-search/SKILL.md").exists());
    assert!(builtin_dir.join("VERSION").exists());

    let version = fs::read_to_string(builtin_dir.join("VERSION")).unwrap();
    assert_eq!(version, crate::version::crate_version());
}

#[test]
fn updates_skills_when_version_mismatches() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Create old version
    fs::create_dir_all(builtin_dir.join("skill1")).unwrap();
    fs::write(builtin_dir.join("skill1/SKILL.md"), "old skill").unwrap();
    fs::write(builtin_dir.join("VERSION"), "0.0.1").unwrap();

    // Ensure (should update)
    ensure_builtin_skills(&builtin_dir).unwrap();

    // Verify updated - old skill should be replaced with actual builtin skills
    assert!(builtin_dir.join("tavily-search/SKILL.md").exists());

    let version = fs::read_to_string(builtin_dir.join("VERSION")).unwrap();
    assert_eq!(version, crate::version::crate_version());
}

#[test]
fn preserves_user_modifications_when_version_matches() {
    let temp_workspace = TempDir::new().unwrap();
    let builtin_dir = temp_workspace.path().join("builtin-skills");

    // Initialize first time
    ensure_builtin_skills(&builtin_dir).unwrap();

    // Modify user content
    let skill_path = builtin_dir.join("tavily-search/SKILL.md");
    fs::write(&skill_path, "user modified").unwrap();

    // Ensure again (should not update)
    ensure_builtin_skills(&builtin_dir).unwrap();

    // Verify user modification preserved
    let content = fs::read_to_string(&skill_path).unwrap();
    assert_eq!(content, "user modified");
}
