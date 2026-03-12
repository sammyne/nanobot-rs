//! Gateway 命令单元测试

use nanobot_session::SessionInfo;

use super::*;

/// 测试 GatewayCmd 默认参数
#[test]
fn gateway_cmd_default_args() {
    let cmd = GatewayCmd { port: Some(18790) };

    assert_eq!(cmd.port, Some(18790));
}

/// 测试 GatewayCmd 自定义参数
#[test]
fn gateway_cmd_custom_args() {
    let cmd = GatewayCmd { port: Some(8080) };

    assert_eq!(cmd.port, Some(8080));
}

/// 测试 GatewayCmd Debug trait
#[test]
fn gateway_cmd_debug_trait() {
    let cmd = GatewayCmd { port: Some(18790) };

    let debug_str = format!("{cmd:?}");
    assert!(debug_str.contains("18790"));
}

/// 测试 pick_heartbeat_target - 选择最近更新的启用渠道
#[test]
fn pick_heartbeat_target_selects_recent_enabled_channel() {
    let enabled_channels = vec!["telegram".to_string(), "slack".to_string()];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "telegram:123456".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "slack:abc123".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    assert_eq!(channel, "telegram");
    assert_eq!(chat_id, "123456");
}

/// 测试 pick_heartbeat_target - 跳过内部渠道
#[test]
fn pick_heartbeat_target_skips_internal_channels() {
    let enabled_channels = vec!["telegram".to_string()];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "cli:direct".to_string(),
            created_at: now - chrono::Duration::hours(1),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "system:internal".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now - chrono::Duration::minutes(5),
            path: "/path/to/session2".to_string(),
        },
        SessionInfo {
            key: "telegram:123456".to_string(),
            created_at: now - chrono::Duration::hours(3),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session3".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    assert_eq!(channel, "telegram");
    assert_eq!(chat_id, "123456");
}

/// 测试 pick_heartbeat_target - 跳过未启用的渠道
#[test]
fn pick_heartbeat_target_skips_disabled_channels() {
    let enabled_channels = vec!["telegram".to_string()];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "slack:abc123".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "telegram:123456".to_string(),
            created_at: now - chrono::Duration::hours(3),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    // 应该选择 telegram，虽然它不是最新的，但是它是唯一启用的渠道
    assert_eq!(channel, "telegram");
    assert_eq!(chat_id, "123456");
}

/// 测试 pick_heartbeat_target - 空会话列表返回默认值
#[test]
fn pick_heartbeat_target_returns_default_for_empty_sessions() {
    let enabled_channels = vec![];
    let sessions = vec![];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    assert_eq!(channel, "cli");
    assert_eq!(chat_id, "direct");
}

/// 测试 pick_heartbeat_target - 只有内部渠道时返回默认值
#[test]
fn pick_heartbeat_target_returns_default_when_only_internal_channels() {
    let enabled_channels = vec![];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "cli:direct".to_string(),
            created_at: now - chrono::Duration::hours(1),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "system:internal".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now,
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    assert_eq!(channel, "cli");
    assert_eq!(chat_id, "direct");
}

/// 测试 pick_heartbeat_target - 无启用的外部渠道时返回默认值
#[test]
fn pick_heartbeat_target_returns_default_when_no_enabled_external_channels() {
    let enabled_channels = vec![]; // 没有启用的渠道
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "telegram:123456".to_string(),
            created_at: now - chrono::Duration::hours(1),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "slack:abc123".to_string(),
            created_at: now - chrono::Duration::hours(2),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    assert_eq!(channel, "cli");
    assert_eq!(chat_id, "direct");
}

/// 测试 pick_heartbeat_target - 处理空 chat_id 的会话
#[test]
fn pick_heartbeat_target_handles_empty_chat_id() {
    let enabled_channels = vec!["telegram".to_string(), "slack".to_string()];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "telegram:".to_string(), // 空 chat_id
            created_at: now - chrono::Duration::hours(2),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "slack:abc123".to_string(),
            created_at: now - chrono::Duration::hours(3),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    // 应该跳过空 chat_id 的会话，选择下一个有效的
    assert_eq!(channel, "slack");
    assert_eq!(chat_id, "abc123");
}

/// 测试 pick_heartbeat_target - 处理无效的 session_key 格式
#[test]
fn pick_heartbeat_target_handles_invalid_session_key_format() {
    let enabled_channels = vec!["telegram".to_string()];
    let now = chrono::Utc::now();

    let sessions = vec![
        SessionInfo {
            key: "invalid_format".to_string(), // 不包含冒号
            created_at: now - chrono::Duration::hours(2),
            updated_at: now,
            path: "/path/to/session1".to_string(),
        },
        SessionInfo {
            key: "telegram:123456".to_string(),
            created_at: now - chrono::Duration::hours(3),
            updated_at: now - chrono::Duration::hours(1),
            path: "/path/to/session2".to_string(),
        },
    ];

    let (channel, chat_id) = GatewayCmd::pick_heartbeat_target(&enabled_channels, &sessions);

    // 应该跳过无效格式的会话，选择下一个有效的
    assert_eq!(channel, "telegram");
    assert_eq!(chat_id, "123456");
}
