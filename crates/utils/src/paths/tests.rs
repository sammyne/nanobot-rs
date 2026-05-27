use std::path::Path;

use super::*;

#[test]
fn expand_tilde_with_subpath() {
    let result = expand_tilde(Path::new("~/documents/file.txt"));
    assert!(result.is_absolute());
    assert!(result.to_str().unwrap().ends_with("documents/file.txt"));
}

#[test]
fn expand_tilde_bare() {
    let result = expand_tilde(Path::new("~"));
    assert!(result.is_absolute());
    // Should be the home directory itself
    assert!(!result.to_str().unwrap().contains('~'));
}

#[test]
fn expand_tilde_no_tilde() {
    let result = expand_tilde(Path::new("/etc/passwd"));
    assert_eq!(result, Path::new("/etc/passwd"));
}

#[test]
fn expand_tilde_relative_path() {
    let result = expand_tilde(Path::new("relative/path"));
    assert_eq!(result, Path::new("relative/path"));
}
