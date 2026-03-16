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
