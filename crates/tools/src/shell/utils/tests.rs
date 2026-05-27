use super::*;

#[test]
fn path_traversal_detection() {
    assert!(detect_path_traversal("cat ../etc/passwd"));
    assert!(detect_path_traversal("cat ..\\windows\\system32"));
    assert!(!detect_path_traversal("cat /etc/passwd"));
    assert!(!detect_path_traversal("ls -la"));
}

#[test]
fn extract_windows_absolute_paths_from_command() {
    let paths = extract_windows_absolute_paths("copy C:\\Windows\\System32\\file.txt D:\\backup\\");
    assert_eq!(paths, vec!["C:\\Windows\\System32\\file.txt", "D:\\backup\\"]);
}

#[test]
fn extract_posix_absolute_paths_from_command() {
    let paths = extract_posix_absolute_paths("cat /etc/passwd > /tmp/output.txt");
    assert_eq!(paths, vec!["/etc/passwd", "/tmp/output.txt"]);
}

#[test]
fn extract_tilde_paths_basic() {
    let paths = extract_tilde_paths("cat ~/.nanobot/config.json");
    assert_eq!(paths, vec!["~/.nanobot/config.json"]);
}

#[test]
fn extract_tilde_paths_multiple() {
    let paths = extract_tilde_paths("cp ~/a ~/b");
    assert_eq!(paths, vec!["~/a", "~/b"]);
}

#[test]
fn extract_tilde_paths_no_match() {
    assert!(extract_tilde_paths("echo hello").is_empty());
    assert!(extract_tilde_paths("file~backup").is_empty());
}

#[test]
fn extract_absolute_paths_includes_tilde() {
    let paths = extract_absolute_paths("cat ~/.nanobot/config.json /etc/passwd");
    assert!(paths.contains(&"~/.nanobot/config.json".to_string()));
    assert!(paths.contains(&"/etc/passwd".to_string()));
}
