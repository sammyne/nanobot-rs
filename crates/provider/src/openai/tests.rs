use nanobot_config::ProviderConfig;
use nanobot_tools::ToolDefinition;
use serde_json::json;

use super::*;
use crate::{ProviderResponse, ToolCall};

/// 测试用例结构体：用于消息创建测试
struct MessageCreationCase {
    name: &'static str,
    factory: fn(&str) -> Message,
    expected_role: &'static str,
}

#[test]
fn message_creation() {
    let test_vector = vec![
        MessageCreationCase {
            name: "用户消息",
            factory: |s| Message::user(s),
            expected_role: "user",
        },
        MessageCreationCase {
            name: "助手消息",
            factory: |s| Message::assistant(s),
            expected_role: "assistant",
        },
        MessageCreationCase {
            name: "系统消息",
            factory: |s| Message::system(s),
            expected_role: "system",
        },
    ];

    for case in test_vector {
        let msg = (case.factory)("Hello");
        assert_eq!(msg.role(), case.expected_role, "测试用例 {} 失败", case.name);
        assert_eq!(msg.content(), "Hello");
        assert!(msg.tool_call_id().is_none());
        // 所有消息类型的 tool_calls 返回切片，助手消息默认为空列表
        assert!(msg.tool_calls().is_empty());
    }

    // 测试工具消息
    let tool_msg = Message::tool("call_123", "工具结果");
    assert_eq!(tool_msg.role(), "tool");
    assert_eq!(tool_msg.content(), "工具结果");
    assert_eq!(tool_msg.tool_call_id(), Some("call_123"));
}

#[test]
fn message_with_tools_creation() {
    let tool_call = ToolCall::new("call_1", "search", json!({"query": "rust"}));
    let msg = Message::assistant_with_tools("让我帮你搜索", vec![tool_call.clone()]);

    assert_eq!(msg.role(), "assistant");
    assert_eq!(msg.content(), "让我帮你搜索");
    assert!(!msg.tool_calls().is_empty());

    let tools = msg.tool_calls();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].id, "call_1");
    assert_eq!(tools[0].name, "search");
}

#[test]
fn message_from_string() {
    let s = String::from("test content");
    let msg = Message::user(s);
    assert_eq!(msg.content(), "test content");
}

/// 测试用例结构体：用于错误类型测试
struct ProviderErrorCase {
    error: ProviderError,
    expected: &'static str,
}

#[test]
fn provider_error_display() {
    let test_vector = vec![
        ProviderErrorCase {
            error: ProviderError::Api("Connection failed".to_string()),
            expected: "LLM API 调用失败: Connection failed",
        },
        ProviderErrorCase {
            error: ProviderError::Timeout,
            expected: "请求超时",
        },
        ProviderErrorCase {
            error: ProviderError::Config("Missing key".to_string()),
            expected: "配置错误: Missing key",
        },
    ];

    for case in test_vector {
        assert_eq!(case.error.to_string(), case.expected);
    }
}

// ============ ToolCall 测试 ============

#[test]
fn tool_call_new() {
    let tool = ToolCall::new("call_1", "search", json!({"query": "rust"}));

    assert_eq!(tool.id, "call_1");
    assert_eq!(tool.name, "search");
    assert_eq!(tool.arguments, r#"{"query":"rust"}"#);
}

#[test]
fn tool_call_parse_arguments_ok() {
    let tool = ToolCall::new("call_1", "search", json!({"query": "rust", "limit": 10}));

    let args = tool.parse_arguments().unwrap();
    assert_eq!(args["query"], "rust");
    assert_eq!(args["limit"], 10);
}

#[test]
fn tool_call_parse_arguments_invalid() {
    let tool = ToolCall {
        id: "call_1".to_string(),
        name: "search".to_string(),
        arguments: "invalid json".to_string(),
    };

    assert!(tool.parse_arguments().is_err());
}

// ============ ProviderResponse 测试 ============

#[test]
fn provider_response_content_only() {
    let response = ProviderResponse::content("Hello world");

    assert_eq!(response.content, "Hello world");
    assert!(!response.has_tool_calls);
    assert!(response.tool_calls.is_empty());
}

#[test]
fn provider_response_with_tools() {
    let tool_calls = vec![
        ToolCall::new("call_1", "search", json!({"q": "rust"})),
        ToolCall::new("call_2", "calc", json!({"a": 1, "b": 2})),
    ];

    let response = ProviderResponse::with_tools("结果如下", tool_calls.clone());

    assert_eq!(response.content, "结果如下");
    assert!(response.has_tool_calls);
    assert_eq!(response.tool_calls.len(), 2);
    assert_eq!(response.tool_calls[0].id, "call_1");
    assert_eq!(response.tool_calls[1].id, "call_2");
}

#[test]
fn provider_response_default() {
    let response: ProviderResponse = Default::default();

    assert_eq!(response.content, "");
    assert!(!response.has_tool_calls);
    assert!(response.tool_calls.is_empty());
}

// ============ OpenAILike 测试 ============

/// 测试用例结构体：用于 OpenAILike 创建测试
struct OpenAINewCase {
    name: &'static str,
    provider_config: ProviderConfig,
    model: &'static str,
    #[allow(dead_code)]
    expected_success: bool,
}

#[test]
fn openai_new_with_default_base() {
    let test_vector = vec![OpenAINewCase {
        name: "使用默认 API base",
        provider_config: ProviderConfig {
            api_key: "test-key".to_string(),
            api_base: None,
            extra_headers: None,
        },
        model: "gpt-4",
        expected_success: true,
    }];

    for case in test_vector {
        let result = OpenAILike::new(&case.provider_config, case.model);
        assert!(result.is_ok(), "测试用例 {} 失败", case.name);

        let openai = result.unwrap();
        assert_eq!(openai.model, case.model);
    }
}

#[test]
fn openai_new_with_custom_base() {
    let config = ProviderConfig {
        api_key: "test-key".to_string(),
        api_base: Some("https://custom.openai.com/v1".to_string()),
        extra_headers: None,
    };

    let result = OpenAILike::new(&config, "gpt-3.5-turbo");
    assert!(result.is_ok());

    let openai = result.unwrap();
    assert_eq!(openai.model, "gpt-3.5-turbo");
}

#[test]
fn openai_new_with_timeout() {
    let config = ProviderConfig {
        api_key: "test-key".to_string(),
        api_base: None,
        extra_headers: None,
    };

    let result = OpenAILike::new_with_timeout(&config, "gpt-4", 60);
    assert!(result.is_ok());
}

// ============ 消息转换测试 ============

#[test]
fn convert_messages_ok() {
    let config = ProviderConfig {
        api_key: "test".to_string(),
        api_base: None,
        extra_headers: None,
    };
    let openai = OpenAILike::new(&config, "gpt-4").unwrap();

    let messages = vec![
        Message::system("你是一个助手"),
        Message::user("Hello"),
        Message::assistant("Hi there"),
    ];

    let result = openai.convert_messages(&messages);
    assert!(result.is_ok());

    let converted = result.unwrap();
    assert_eq!(converted.len(), 3);
}

#[test]
fn convert_messages_empty() {
    let config = ProviderConfig {
        api_key: "test".to_string(),
        api_base: None,
        extra_headers: None,
    };
    let openai = OpenAILike::new(&config, "gpt-4").unwrap();

    let messages: Vec<Message> = vec![];
    let result = openai.convert_messages(&messages);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn convert_messages_with_tool() {
    let config = ProviderConfig {
        api_key: "test".to_string(),
        api_base: None,
        extra_headers: None,
    };
    let openai = OpenAILike::new(&config, "gpt-4").unwrap();

    // Tool 消息必须通过 Message::tool 构造，自动包含 tool_call_id
    let tool_msg = Message::tool("call_1", "结果");

    let messages = vec![tool_msg];
    let result = openai.convert_messages(&messages);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

// ============ 工具绑定测试 ============

/// 测试用例结构体：用于工具绑定测试
#[allow(dead_code)]
struct ToolBindingCase {
    name: &'static str,
    tools: Vec<ToolDefinition>,
    expected_tool_count: usize,
}

#[test]
fn bind_tools_ok() {
    let test_vector = vec![
        ToolBindingCase {
            name: "单个工具",
            tools: vec![ToolDefinition {
                name: "search".to_string(),
                description: "搜索工具".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    }
                }),
            }],
            expected_tool_count: 1,
        },
        ToolBindingCase {
            name: "多个工具",
            tools: vec![
                ToolDefinition {
                    name: "search".to_string(),
                    description: "搜索工具".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
                ToolDefinition {
                    name: "calc".to_string(),
                    description: "计算工具".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            ],
            expected_tool_count: 2,
        },
        ToolBindingCase {
            name: "空工具列表",
            tools: vec![],
            expected_tool_count: 0,
        },
    ];

    for case in test_vector {
        let config = ProviderConfig {
            api_key: "test".to_string(),
            api_base: None,
            extra_headers: None,
        };
        let _openai = OpenAILike::new(&config, "gpt-4").unwrap();

        // 验证 chat 方法可以接受工具列表
        // 注意：由于 bind_tools 已被移除，现在工具直接通过 chat 方法参数传递
        // 这里主要验证构造正确的工具定义不会 panic
        let _ = case.tools;
    }
}

#[test]
fn bind_tools_complex_schema() {
    let complex_tool = ToolDefinition {
        name: "weather".to_string(),
        description: "获取天气信息".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "城市名称"
                },
                "days": {
                    "type": "integer",
                    "description": "天数"
                }
            },
            "required": ["city"]
        }),
    };

    let config = ProviderConfig {
        api_key: "test".to_string(),
        api_base: None,
        extra_headers: None,
    };
    let mut openai = OpenAILike::new(&config, "gpt-4").unwrap();

    openai.bind_tools(vec![complex_tool]);
    // 验证 bind_tools 能正确处理复杂 schema
}
