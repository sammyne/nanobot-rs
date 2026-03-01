//! Agent 命令测试

use super::*;

#[test]
fn agent_args_default() {
    let args = AgentArgs { system: None };
    assert!(args.system.is_none());
}

#[test]
fn agent_args_with_system() {
    let args = AgentArgs {
        system: Some("You are a helpful assistant.".to_string()),
    };
    assert_eq!(
        args.system,
        Some("You are a helpful assistant.".to_string())
    );
}
