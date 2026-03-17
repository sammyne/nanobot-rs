//! ToolRegistry 集成测试
//!
//! 测试工具注册表的基本功能，包括注册、注销和执行工具。

use nanobot_tools::ToolRegistry;
use serde_json::json;
use tempfile::TempDir;

/// 准备临时工作环境
fn setup() -> TempDir {
    TempDir::new().unwrap()
}

/// 创建测试用的 ToolContext
fn test_context() -> nanobot_tools::ToolContext {
    nanobot_tools::ToolContext::new("test-channel".to_string(), "12345".to_string())
}

#[derive(Debug)]
struct RegistryTestCase {
    expected_count: usize,
}

/// 表驱动测试：基本操作测试
#[tokio::test]
async fn registry_basic_operations() {
    let test_cases = vec![RegistryTestCase { expected_count: 5 }];

    for case in test_cases {
        let temp_dir = setup();
        let workspace = temp_dir.path().to_str().unwrap();
        let mut registry = ToolRegistry::new(workspace, None::<&str>);

        assert!(registry.contains("read_file"));
        assert!(registry.contains("write_file"));
        assert!(registry.contains("edit_file"));
        assert!(registry.contains("list_dir"));
        assert!(registry.contains("shell"));

        let definitions = registry.get_definitions();
        assert_eq!(definitions.len(), case.expected_count);

        assert!(registry.unregister("shell"));
        assert!(!registry.contains("shell"));
        assert_eq!(registry.tool_names().len(), 4);

        assert!(!registry.unregister("nonexistent"));
    }
}

#[derive(Debug)]
struct DefaultToolTestCase {
    tool_name: &'static str,
}

/// 表驱动测试：默认工具注册
#[tokio::test]
async fn registry_register_default_tools() {
    let test_cases = vec![
        DefaultToolTestCase { tool_name: "read_file" },
        DefaultToolTestCase { tool_name: "write_file" },
        DefaultToolTestCase { tool_name: "edit_file" },
        DefaultToolTestCase { tool_name: "list_dir" },
        DefaultToolTestCase { tool_name: "shell" },
    ];

    let temp_dir = setup();
    let workspace = temp_dir.path().to_str().unwrap();
    let registry = ToolRegistry::new(workspace, None::<&str>);

    let names = registry.tool_names();
    for case in test_cases {
        assert!(names.contains(&case.tool_name.to_string()));
    }
}

#[derive(Debug)]
struct ExecuteErrorTestCase {
    tool_name: &'static str,
    input: serde_json::Value,
    expected_error_contains: &'static str,
}

/// 表驱动测试：执行不存在的工具
#[tokio::test]
async fn registry_execute_nonexistent_tool() {
    let test_cases =
        vec![ExecuteErrorTestCase { tool_name: "nonexistent", input: json!({}), expected_error_contains: "不存在" }];

    for case in test_cases {
        let temp_dir = setup();
        let workspace = temp_dir.path().to_str().unwrap();
        let registry = ToolRegistry::new(workspace, None::<&str>);
        let ctx = test_context();

        let result: Result<String, _> = registry.execute(&ctx, case.tool_name, case.input).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains(case.expected_error_contains),
            "错误消息应包含 '{}', 实际: {}",
            case.expected_error_contains,
            err_msg
        );
        assert!(err_msg.contains("可用工具"));
    }
}
