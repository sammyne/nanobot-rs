//! Tests for version management.

use std::path::Path;

use tempfile::TempDir;

use super::*;

#[test]
fn returns_semver_format_version() {
    let version = crate_version();
    // Version should be in semver format
    assert!(version.split('.').count() >= 2);
}

#[test]
fn write_and_read_version_file() {
    let temp_dir = TempDir::new().unwrap();
    let version_path = temp_dir.path().join("VERSION");

    write_version_file(&version_path, "1.2.3").unwrap();
    let read_version = read_version_file(&version_path).unwrap();

    assert_eq!(read_version, "1.2.3");
}

#[test]
fn checks_version_correctly() {
    let current = crate_version();
    assert!(version_matches(current));
    assert!(!version_matches("0.0.0"));
}

#[test]
fn read_version_file_nonexistent() {
    let result = read_version_file(Path::new("/nonexistent/VERSION"));
    assert!(result.is_err());
}

#[test]
fn write_version_file_creates_parent_dirs() {
    let temp_dir = TempDir::new().unwrap();
    let version_path = temp_dir.path().join("subdir/nested/VERSION");

    write_version_file(&version_path, "2.0.0").unwrap();
    let read_version = read_version_file(&version_path).unwrap();

    assert_eq!(read_version, "2.0.0");
}
