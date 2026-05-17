use nanobot_config::ProviderConfig;
use nanobot_tools::ToolDefinition;
use serde_json::json;

use super::*;

// ============ convert_messages 测试 ============

#[test]
fn convert_messages_extracts_system() {
    let messages = [Message::system("You are helpful."), Message::user("Hello")];

    let (system, msgs) = convert_messages(&messages);

    assert_eq!(system, Some("You are helpful.".to_string()));
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, "user");
}

#[test]
fn convert_messages_multiple_system_joined() {
    let messages = [Message::system("You are helpful."), Message::system("Be concise."), Message::user("Hello")];

    let (system, msgs) = convert_messages(&messages);

    assert_eq!(system, Some("You are helpful.\nBe concise.".to_string()));
    assert_eq!(msgs.len(), 1);
}

#[test]
fn convert_messages_user_and_assistant() {
    let messages = [Message::user("Hello"), Message::assistant("Hi there")];

    let (system, msgs) = convert_messages(&messages);

    assert!(system.is_none());
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, "user");
    assert_eq!(msgs[1].role, "assistant");

    // 验证 content blocks
    assert!(matches!(&msgs[0].content[0], ContentBlock::Text { text } if text == "Hello"));
    assert!(matches!(&msgs[1].content[0], ContentBlock::Text { text } if text == "Hi there"));
}

#[test]
fn convert_messages_assistant_with_tool_calls() {
    let tool_call = ToolCall::new("toolu_01", "get_weather", json!({"location": "Paris"}));
    let messages = [Message::assistant_with_tools("Let me check.", vec![tool_call])];

    let (_, msgs) = convert_messages(&messages);

    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, "assistant");
    assert_eq!(msgs[0].content.len(), 2);

    assert!(matches!(&msgs[0].content[0], ContentBlock::Text { text } if text == "Let me check."));
    assert!(
        matches!(&msgs[0].content[1], ContentBlock::ToolUse { id, name, input } if id == "toolu_01" && name == "get_weather" && input["location"] == "Paris")
    );
}

#[test]
fn convert_messages_consecutive_tools_merged() {
    let messages = [
        Message::user("What's the weather?"),
        Message::assistant_with_tools(
            "",
            vec![
                ToolCall::new("t1", "get_weather", json!({"city": "Paris"})),
                ToolCall::new("t2", "get_weather", json!({"city": "London"})),
            ],
        ),
        Message::tool("t1", "22 degrees"),
        Message::tool("t2", "18 degrees"),
    ];

    let (_, msgs) = convert_messages(&messages);

    // user, assistant, user(tool_results)
    assert_eq!(msgs.len(), 3);

    // 最后一个消息应该是 user 角色，包含两个 tool_result blocks
    let tool_msg = &msgs[2];
    assert_eq!(tool_msg.role, "user");
    assert_eq!(tool_msg.content.len(), 2);
    assert!(
        matches!(&tool_msg.content[0], ContentBlock::ToolResult { tool_use_id, content } if tool_use_id == "t1" && content == "22 degrees")
    );
    assert!(
        matches!(&tool_msg.content[1], ContentBlock::ToolResult { tool_use_id, content } if tool_use_id == "t2" && content == "18 degrees")
    );
}

#[test]
fn convert_messages_empty() {
    let messages: Vec<Message> = vec![];
    let (system, msgs) = convert_messages(&messages);

    assert!(system.is_none());
    assert!(msgs.is_empty());
}

// ============ bind_tools 测试 ============

#[test]
fn bind_tools_converts_to_anthropic_format() {
    let config = ProviderConfig { api_key: "test".to_string(), api_base: None, extra_headers: None };
    let mut provider = AnthropicLike::new(&config, "claude-sonnet-4-20250514").unwrap();

    let tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get weather for a city".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        }),
    }];

    provider.bind_tools(tools);

    assert_eq!(provider.tools.len(), 1);
    assert_eq!(provider.tools[0].name, "get_weather");
    assert_eq!(provider.tools[0].description, "Get weather for a city");
    // 验证 parameters → input_schema 映射
    assert_eq!(provider.tools[0].input_schema["type"], "object");
    assert!(provider.tools[0].input_schema["properties"]["location"].is_object());
}

#[test]
fn bind_tools_adds_missing_type_field() {
    let config = ProviderConfig { api_key: "test".to_string(), api_base: None, extra_headers: None };
    let mut provider = AnthropicLike::new(&config, "claude-sonnet-4-20250514").unwrap();

    let tools = vec![ToolDefinition {
        name: "mcp_tool".to_string(),
        description: "An MCP tool".to_string(),
        parameters: json!({
            "properties": {
                "input": {"type": "string"}
            }
        }),
    }];

    provider.bind_tools(tools);

    assert_eq!(provider.tools[0].input_schema["type"], "object");
    assert!(provider.tools[0].input_schema["properties"]["input"].is_object());
}

#[test]
fn bind_tools_strips_top_level_combinators() {
    let config = ProviderConfig { api_key: "test".to_string(), api_base: None, extra_headers: None };
    let mut provider = AnthropicLike::new(&config, "claude-sonnet-4-20250514").unwrap();

    let tools = vec![ToolDefinition {
        name: "complex_tool".to_string(),
        description: "A tool with oneOf".to_string(),
        parameters: json!({
            "oneOf": [
                {"type": "object", "properties": {"a": {"type": "string"}}},
                {"type": "object", "properties": {"b": {"type": "number"}}}
            ]
        }),
    }];

    provider.bind_tools(tools);

    // oneOf 应被移除，type 应被补充
    assert!(provider.tools[0].input_schema.get("oneOf").is_none());
    assert_eq!(provider.tools[0].input_schema["type"], "object");
}

// ============ AnthropicResponse 反序列化测试 ============

#[test]
fn response_deserialize_text_only() {
    let json = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": "Hello!"}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 5}
    }"#;

    let resp: AnthropicResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.content.len(), 1);
    assert!(matches!(&resp.content[0], ContentBlock::Text { text } if text == "Hello!"));
    assert_eq!(resp.stop_reason, Some("end_turn".to_string()));
}

#[test]
fn response_deserialize_with_tool_use() {
    let json = r#"{
        "id": "msg_456",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": "Let me check."},
            {"type": "tool_use", "id": "toolu_01", "name": "get_weather", "input": {"location": "Paris"}}
        ],
        "stop_reason": "tool_use",
        "usage": {"input_tokens": 50, "output_tokens": 30}
    }"#;

    let resp: AnthropicResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.content.len(), 2);
    assert!(matches!(&resp.content[0], ContentBlock::Text { text } if text == "Let me check."));
    assert!(
        matches!(&resp.content[1], ContentBlock::ToolUse { id, name, input } if id == "toolu_01" && name == "get_weather" && input["location"] == "Paris")
    );
}

#[test]
fn response_deserialize_with_thinking_block() {
    let json = r#"{
        "id": "msg_789",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "thinking", "thinking": "Let me think...", "signature": "abc123"},
            {"type": "text", "text": "Here is my answer."}
        ],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 100, "output_tokens": 50}
    }"#;

    let resp: AnthropicResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.content.len(), 2);
    assert!(matches!(&resp.content[0], ContentBlock::Thinking { thinking, .. } if thinking == "Let me think..."));
    assert!(matches!(&resp.content[1], ContentBlock::Text { text } if text == "Here is my answer."));
}

// ============ AnthropicLike 构造测试 ============

#[test]
fn new_with_default_api_base() {
    let config = ProviderConfig { api_key: "sk-ant-test".to_string(), api_base: None, extra_headers: None };
    let provider = AnthropicLike::new(&config, "claude-sonnet-4-20250514").unwrap();

    assert_eq!(provider.api_base, "https://api.anthropic.com/v1");
    assert_eq!(provider.model, "claude-sonnet-4-20250514");
    assert_eq!(provider.timeout, 120);
}

#[test]
fn new_with_custom_api_base() {
    let config = ProviderConfig {
        api_key: "sk-ant-test".to_string(),
        api_base: Some("https://custom.anthropic.proxy.com".to_string()),
        extra_headers: None,
    };
    let provider = AnthropicLike::new(&config, "claude-sonnet-4-20250514").unwrap();

    assert_eq!(provider.api_base, "https://custom.anthropic.proxy.com");
}

#[test]
fn new_with_timeout() {
    let config = ProviderConfig { api_key: "sk-ant-test".to_string(), api_base: None, extra_headers: None };
    let provider = AnthropicLike::new_with_timeout(&config, "claude-sonnet-4-20250514", 60).unwrap();

    assert_eq!(provider.timeout, 60);
}
