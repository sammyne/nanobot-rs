//! MCP 配置测试

use std::collections::HashMap;

use nanobot_config::McpServerConfig;

/// 测试 Stdio 配置创建
#[test]
fn mcp_stdio_config() {
    let transport =
        McpServerConfig::stdio("/usr/bin/test-mcp").with_args(vec!["--port".to_string(), "8080".to_string()]);

    if let McpServerConfig::Stdio { command, args, .. } = &transport {
        assert_eq!(command, "/usr/bin/test-mcp");
        assert_eq!(args, &vec!["--port".to_string(), "8080".to_string()]);
    } else {
        panic!("Expected Stdio transport");
    }
}

/// 测试 HTTP 配置创建
#[test]
fn mcp_http_config() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());

    let transport = McpServerConfig::http("http://localhost:3000/mcp").with_headers(headers.clone()).with_timeout(45);

    if let McpServerConfig::Http { url, headers: h, tool_timeout } = &transport {
        assert_eq!(url, "http://localhost:3000/mcp");
        assert_eq!(h, &headers);
        assert_eq!(*tool_timeout, 45);
    } else {
        panic!("Expected Http transport");
    }
}

/// 测试配置序列化/反序列化
#[test]
fn mcp_config_serialization() {
    let transport = McpServerConfig::stdio("/bin/test");
    let yaml = serde_yaml::to_string(&transport).unwrap();
    println!("YAML:\n{yaml}");

    let deserialized: McpServerConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(transport, deserialized);
}

/// 测试工具名称格式
#[test]
fn mcp_tool_name_format() {
    // 测试工具名称格式化逻辑
    let server_name = "test-server";
    let original_name = "read_file";
    let wrapped_name = format!("mcp_{server_name}_{original_name}");
    assert_eq!(wrapped_name, "mcp_test-server_read_file");
}
