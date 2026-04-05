use super::*;

struct ToolCallPreviewCase {
    name: &'static str,
    tool_name: &'static str,
    arguments: serde_json::Value,
    expected: &'static str,
}

#[test]
fn tool_call_preview() {
    let test_vector = [
        ToolCallPreviewCase {
            name: "字符串参数",
            tool_name: "web_search",
            arguments: serde_json::json!({ "query": "Rust programming" }),
            expected: "web_search(query=\"Rust programming\")",
        },
        ToolCallPreviewCase {
            name: "数组格式参数 - 非对象",
            tool_name: "tool",
            arguments: serde_json::json!([1, 2, 3]),
            expected: "tool",
        },
        ToolCallPreviewCase {
            name: "字符串格式参数 - 非对象",
            tool_name: "tool",
            arguments: serde_json::json!("just a string"),
            expected: "tool",
        },
        ToolCallPreviewCase {
            name: "空对象", tool_name: "empty", arguments: serde_json::json!({}), expected: "empty"
        },
        ToolCallPreviewCase {
            name: "无效 JSON",
            tool_name: "broken",
            arguments: serde_json::json!("__INVALID_JSON__"),
            expected: "broken",
        },
        ToolCallPreviewCase {
            name: "布尔值参数",
            tool_name: "toggle",
            arguments: serde_json::json!({ "enabled": true }),
            expected: "toggle(enabled=true)",
        },
        ToolCallPreviewCase {
            name: "嵌套对象作为第一个参数",
            tool_name: "complex",
            arguments: serde_json::json!({ "config": { "nested": "value" } }),
            expected: "complex",
        },
        ToolCallPreviewCase {
            name: "null 值参数",
            tool_name: "nullable",
            arguments: serde_json::json!({ "value": null }),
            expected: "nullable(value=null)",
        },
        ToolCallPreviewCase {
            name: "空字符串参数",
            tool_name: "empty_str",
            arguments: serde_json::json!({ "value": "" }),
            expected: "empty_str(value=\"\")",
        },
        ToolCallPreviewCase {
            name: "特殊字符 - 路径",
            tool_name: "special",
            arguments: serde_json::json!({ "path": "/usr/local/bin/node" }),
            expected: "special(path=\"/usr/local/bin/node\")",
        },
        ToolCallPreviewCase {
            name: "整数参数",
            tool_name: "calc",
            arguments: serde_json::json!({ "value": 42 }),
            expected: "calc(value=42)",
        },
        ToolCallPreviewCase {
            name: "负数参数",
            tool_name: "calc",
            arguments: serde_json::json!({ "num": -123 }),
            expected: "calc(num=-123)",
        },
        ToolCallPreviewCase {
            name: "浮点数参数",
            tool_name: "calc",
            arguments: serde_json::json!({ "pi": 3.14159 }),
            expected: "calc(pi=3.14159)",
        },
        ToolCallPreviewCase {
            name: "布尔值 true 参数",
            tool_name: "toggle",
            arguments: serde_json::json!({ "flag": true }),
            expected: "toggle(flag=true)",
        },
        ToolCallPreviewCase {
            name: "布尔值 false 参数",
            tool_name: "toggle",
            arguments: serde_json::json!({ "flag": false }),
            expected: "toggle(flag=false)",
        },
        ToolCallPreviewCase {
            name: "null 参数",
            tool_name: "nullable",
            arguments: serde_json::json!({ "data": null }),
            expected: "nullable(data=null)",
        },
        ToolCallPreviewCase {
            name: "数组值参数 - 跳过",
            tool_name: "array",
            arguments: serde_json::json!({ "items": [1, 2, 3] }),
            expected: "array",
        },
        ToolCallPreviewCase {
            name: "对象值参数 - 跳过",
            tool_name: "obj",
            arguments: serde_json::json!({ "config": { "key": "value" } }),
            expected: "obj",
        },
        ToolCallPreviewCase {
            name: "零值参数",
            tool_name: "calc",
            arguments: serde_json::json!({ "value": 0 }),
            expected: "calc(value=0)",
        },
        ToolCallPreviewCase {
            name: "空数组参数",
            tool_name: "empty_arr",
            arguments: serde_json::json!({ "items": [] }),
            expected: "empty_arr",
        },
        ToolCallPreviewCase {
            name: "深层嵌套对象参数",
            tool_name: "nested",
            arguments: serde_json::json!({ "deep": { "level1": { "level2": { "level3": "value" } } } }),
            expected: "nested",
        },
        ToolCallPreviewCase {
            name: "Unicode 字符串",
            tool_name: "unicode",
            arguments: serde_json::json!({ "text": "你好世界测试" }),
            expected: "unicode(text=\"你好世界测试\")",
        },
        ToolCallPreviewCase {
            name: "Emoji 字符串",
            tool_name: "emoji",
            arguments: serde_json::json!({ "text": "Hello 🎉 World" }),
            expected: "emoji(text=\"Hello 🎉 World\")",
        },
        ToolCallPreviewCase {
            name: "负浮点数参数",
            tool_name: "neg",
            arguments: serde_json::json!({ "temp": -273.15 }),
            expected: "neg(temp=-273.15)",
        },
    ];

    for case in test_vector {
        let tool_call = if case.arguments == serde_json::json!("__INVALID_JSON__") {
            ToolCall {
                id: "test_id".to_string(),
                name: case.tool_name.to_string(),
                arguments: "not valid json".to_string(),
            }
        } else {
            ToolCall::new("test_id", case.tool_name, case.arguments)
        };
        let result = tool_call.preview();
        assert_eq!(result, case.expected, "case[{}]: mismatch", case.name);
    }
}

struct ToolCallPreviewDynamicCase {
    name: &'static str,
    tool_name: &'static str,
    arguments: serde_json::Value,
    check: fn(&str, &serde_json::Value) -> bool,
}

#[test]
fn tool_call_preview_dynamic() {
    let test_vector = [
        ToolCallPreviewDynamicCase {
            name: "长字符串截断到 40 字符",
            tool_name: "search",
            arguments: serde_json::json!({ "q": "x".repeat(50) }),
            check: |result, _args| {
                let expected_arg = "x".repeat(40);
                result == &format!("search(q=\"{expected_arg}…\")")
            },
        },
        ToolCallPreviewDynamicCase {
            name: "恰好 40 字符不截断",
            tool_name: "exact",
            arguments: serde_json::json!({ "text": "x".repeat(40) }),
            check: |result, _args| {
                let exact_40 = "x".repeat(40);
                result == &format!("exact(text=\"{exact_40}\")")
            },
        },
        ToolCallPreviewDynamicCase {
            name: "39 字符不截断",
            tool_name: "chars39",
            arguments: serde_json::json!({ "text": "x".repeat(39) }),
            check: |result, _args| {
                let str_39 = "x".repeat(39);
                result == &format!("chars39(text=\"{str_39}\")")
            },
        },
        ToolCallPreviewDynamicCase {
            name: "41 字符截断",
            tool_name: "chars41",
            arguments: serde_json::json!({ "text": "x".repeat(41) }),
            check: |result, _args| {
                let str_41 = "x".repeat(41);
                result.starts_with("chars41(text=\"") && result.ends_with("\")") && !result.contains(&str_41)
            },
        },
        ToolCallPreviewDynamicCase {
            name: "长 Unicode 字符串",
            tool_name: "long_unicode",
            arguments: serde_json::json!({ "text": "你好".repeat(25) }),
            check: |result, _| result.starts_with("long_unicode(text=\"") && result.ends_with("\")"),
        },
        ToolCallPreviewDynamicCase {
            name: "科学计数法数字",
            tool_name: "science",
            arguments: serde_json::json!({ "exp": 1.5e10 }),
            check: |result, _| result.starts_with("science(exp="),
        },
        ToolCallPreviewDynamicCase {
            name: "大整数",
            tool_name: "big",
            arguments: serde_json::json!({ "value": 9999999999999999i64 }),
            check: |result, _| result.starts_with("big(value="),
        },
    ];

    for case in test_vector {
        let tool_call = ToolCall::new("test_id", case.tool_name, case.arguments.clone());
        let result = tool_call.preview();
        assert!((case.check)(&result, &case.arguments), "case[{}]: check failed, result={}", case.name, result);
    }
}
