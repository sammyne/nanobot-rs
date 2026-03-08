//! Gateway 命令单元测试

use super::*;

/// 测试 GatewayCmd 默认参数
#[test]
fn gateway_cmd_default_args() {
    let cmd = GatewayCmd { port: 18790 };

    assert_eq!(cmd.port, 18790);
}

/// 测试 GatewayCmd 自定义参数
#[test]
fn gateway_cmd_custom_args() {
    let cmd = GatewayCmd { port: 8080 };

    assert_eq!(cmd.port, 8080);
}

/// 测试 GatewayCmd Debug trait
#[test]
fn gateway_cmd_debug_trait() {
    let cmd = GatewayCmd { port: 18790 };

    let debug_str = format!("{:?}", cmd);
    assert!(debug_str.contains("18790"));
}
