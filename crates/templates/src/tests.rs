//! 测试模板文件的获取和验证

use super::*;

#[test]
fn get_user_template() {
    let content = user_template();
    assert!(content.contains("# User Profile"));
}

#[test]
fn get_agents_template() {
    let content = agents_template();
    assert!(content.contains("# Agent Instructions"));
}

#[test]
fn get_soul_template() {
    let content = soul_template();
    assert!(content.contains("# Soul"));
}

#[test]
fn get_tools_template() {
    let content = tools_template();
    assert!(content.contains("# Tool Usage Notes"));
}

#[test]
fn get_memory_template() {
    let content = memory_template();
    assert!(content.contains("# Long-term Memory"));
}

#[test]
fn get_nonexistent_template() {
    let result = get_template("NONEXISTENT.md");
    assert!(result.is_none());
}
