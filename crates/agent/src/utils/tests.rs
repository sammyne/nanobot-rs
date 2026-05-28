//! utils 模块测试

use super::*;

/// parse_system_message_target 测试用例结构
struct ParseTargetCase {
    name: &'static str,
    chat_id: &'static str,
    expect_channel: &'static str,
    expect_chat_id: &'static str,
    expect_session_key: &'static str,
}

/// parse_system_message_target 函数测试
#[test]
fn parse_system_message_target_various_cases() {
    let test_vector = [
        ParseTargetCase {
            name: "标准格式 - telegram:12345",
            chat_id: "telegram:12345",
            expect_channel: "telegram",
            expect_chat_id: "12345",
            expect_session_key: "telegram:12345",
        },
        ParseTargetCase {
            name: "标准格式 - slack:channel-general",
            chat_id: "slack:channel-general",
            expect_channel: "slack",
            expect_chat_id: "channel-general",
            expect_session_key: "slack:channel-general",
        },
        ParseTargetCase {
            name: "无分隔符 - 使用默认 cli",
            chat_id: "simple_id",
            expect_channel: "cli",
            expect_chat_id: "simple_id",
            expect_session_key: "cli:simple_id",
        },
        ParseTargetCase {
            name: "多个冒号 - 只分割第一个",
            chat_id: "wechat:group:123",
            expect_channel: "wechat",
            expect_chat_id: "group:123",
            expect_session_key: "wechat:group:123",
        },
        ParseTargetCase {
            name: "空字符串前缀",
            chat_id: ":test_id",
            expect_channel: "",
            expect_chat_id: "test_id",
            expect_session_key: ":test_id",
        },
    ];

    for case in test_vector {
        let (channel, chat_id, session_key) = parse_system_message_target(case.chat_id);

        assert_eq!(channel, case.expect_channel, "case[{}]: channel mismatch", case.name);
        assert_eq!(chat_id, case.expect_chat_id, "case[{}]: chat_id mismatch", case.name);
        assert_eq!(session_key, case.expect_session_key, "case[{}]: session_key mismatch", case.name);
    }
}

#[test]
fn persist_small_result_returns_unchanged() {
    let result = "short result";
    let out = maybe_persist_tool_result(result, 16000, "call1", "cli:direct", std::path::Path::new("/tmp"));
    assert_eq!(out, result);
}

#[test]
fn persist_large_result_writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path();
    let big = "x".repeat(20000);

    let out = maybe_persist_tool_result(&big, 16000, "call1", "test:chat", workspace);

    // 返回值包含预览和文件路径
    assert!(out.contains("xxxx"), "should contain preview");
    assert!(out.contains("20000 chars total"), "should mention total size");
    assert!(out.contains("call1.txt"), "should mention file path");

    // 文件应存在且内容完整
    let file = workspace.join(".nanobot/tool-results/test:chat/call1.txt");
    assert!(file.exists());
    assert_eq!(std::fs::read_to_string(&file).unwrap().len(), 20000);
}

#[test]
fn persist_exactly_at_threshold_returns_unchanged() {
    let result = "y".repeat(16000);
    let out = maybe_persist_tool_result(&result, 16000, "call2", "s", std::path::Path::new("/tmp"));
    assert_eq!(out, result);
}
