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
