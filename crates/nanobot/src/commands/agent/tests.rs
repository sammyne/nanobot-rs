// Agent 命令测试

use super::*;

#[test]
fn agent_cmd_default() {
    let cmd = AgentCmd { message: None, session: "cli:direct".to_string() };
    assert!(cmd.message.is_none());
    assert_eq!(cmd.session, "cli:direct");
}

#[test]
fn agent_cmd_with_message() {
    let cmd = AgentCmd { message: Some("Hello".to_string()), session: "telegram:12345".to_string() };
    assert_eq!(cmd.message, Some("Hello".to_string()));
    assert_eq!(cmd.session, "telegram:12345");
}

#[test]
fn is_exit_command_various() {
    assert!(is_exit_command("exit"));
    assert!(is_exit_command("quit"));
    assert!(is_exit_command("/exit"));
    assert!(is_exit_command("/quit"));
    assert!(is_exit_command(":q"));
    assert!(is_exit_command("EXIT"));
    assert!(is_exit_command("Quit"));
    assert!(!is_exit_command("hello"));
    assert!(!is_exit_command(""));
}
