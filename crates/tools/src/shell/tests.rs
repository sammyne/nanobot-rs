use std::path::PathBuf;

use super::*;

#[test]
fn default_deny_patterns() {
    let options = ShellToolOptions::default();
    assert!(!options.deny_patterns.is_empty());
}

#[test]
fn check_deny_patterns() {
    let tool = ShellTool::new(ShellToolOptions::default());

    // 应该被拒绝的命令
    assert!(tool.check_deny_patterns("rm -rf /").is_err());
    assert!(tool.check_deny_patterns("rm -fr /home").is_err());
    assert!(tool.check_deny_patterns("dd if=/dev/zero").is_err());
    assert!(tool.check_deny_patterns("mkfs.ext4 /dev/sda1").is_err());
    assert!(tool.check_deny_patterns("shutdown now").is_err());

    // 应该通过的命令
    assert!(tool.check_deny_patterns("ls -la").is_ok());
    assert!(tool.check_deny_patterns("echo hello").is_ok());
}

#[test]
fn check_allow_patterns() {
    let options =
        ShellToolOptions { allow_patterns: vec![r"\bls\b".to_string(), r"\becho\b".to_string()], ..Default::default() };
    let tool = ShellTool::new(options);

    // 在白名单中的命令
    assert!(tool.check_allow_patterns("ls -la").is_ok());
    assert!(tool.check_allow_patterns("echo hello").is_ok());

    // 不在白名单中的命令
    assert!(tool.check_allow_patterns("cat file.txt").is_err());
}

#[test]
fn security_guard() {
    let tool = ShellTool::new(ShellToolOptions::default());
    let cwd = PathBuf::from(".");

    // 危险命令应被拒绝
    assert!(tool.security_guard("rm -rf /", &cwd).is_err());

    // 安全命令应通过
    assert!(tool.security_guard("ls -la", &cwd).is_ok());
}

#[test]
fn security_guard_with_restrict_to_workspace() {
    let options =
        ShellToolOptions { restrict_to_workspace: true, workspace: Some(PathBuf::from("/tmp")), ..Default::default() };
    let tool = ShellTool::new(options);
    let cwd = PathBuf::from("/tmp");

    // 路径遍历应被拒绝
    assert!(tool.security_guard("cat ../etc/passwd", &cwd).is_err());
}
