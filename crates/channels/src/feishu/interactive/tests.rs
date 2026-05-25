use serde_json::json;

use super::*;

#[test]
fn title_string() {
    let content = json!({"title": "Hello World"});
    assert_eq!(extract_interactive_content(&content), "title: Hello World");
}

#[test]
fn title_dict_with_content() {
    let content = json!({"title": {"content": "Card Title"}});
    assert_eq!(extract_interactive_content(&content), "title: Card Title");
}

#[test]
fn title_dict_with_text() {
    let content = json!({"title": {"text": "Card Title"}});
    assert_eq!(extract_interactive_content(&content), "title: Card Title");
}

#[test]
fn elements_double_nested_markdown() {
    let content = json!({
        "elements": [
            [
                {"tag": "markdown", "content": "**bold text**"},
                {"tag": "plain_text", "content": "plain"}
            ],
            [
                {"tag": "markdown", "content": "second row"}
            ]
        ]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("**bold text**"));
    assert!(result.contains("plain"));
    assert!(result.contains("second row"));
}

#[test]
fn elements_flat_array_compat() {
    // 某些卡片 elements 直接是元素列表（一维数组），应兼容
    let content = json!({
        "elements": [
            {"tag": "markdown", "content": "flat element"}
        ]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("flat element"));
}

#[test]
fn header_title() {
    let content = json!({
        "header": {
            "title": {"content": "Header Title"}
        }
    });
    assert_eq!(extract_interactive_content(&content), "title: Header Title");
}

#[test]
fn card_recursive() {
    let content = json!({
        "card": {
            "header": {
                "title": {"text": "Inner Card Title"}
            },
            "elements": [
                [{"tag": "markdown", "content": "inner content"}]
            ]
        }
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("inner content"));
    assert!(result.contains("title: Inner Card Title"));
}

#[test]
fn div_with_text_dict() {
    let content = json!({
        "elements": [[{"tag": "div", "text": {"content": "div text"}}]]
    });
    assert!(extract_interactive_content(&content).contains("div text"));
}

#[test]
fn div_with_text_string() {
    let content = json!({
        "elements": [[{"tag": "div", "text": "plain div"}]]
    });
    assert!(extract_interactive_content(&content).contains("plain div"));
}

#[test]
fn div_with_fields() {
    let content = json!({
        "elements": [[{
            "tag": "div",
            "fields": [
                {"text": {"content": "field1"}},
                {"text": {"content": "field2"}}
            ]
        }]]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("field1"));
    assert!(result.contains("field2"));
}

#[test]
fn button_with_text_and_url() {
    let content = json!({
        "elements": [[{
            "tag": "button",
            "text": {"content": "Click me"},
            "url": "https://example.com"
        }]]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("Click me"));
    assert!(result.contains("link: https://example.com"));
}

#[test]
fn button_with_multi_url() {
    let content = json!({
        "elements": [[{
            "tag": "button",
            "text": {"content": "Open"},
            "multi_url": {"url": "https://example.com/multi"}
        }]]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("link: https://example.com/multi"));
}

#[test]
fn link_element() {
    let content = json!({
        "elements": [[{
            "tag": "a",
            "href": "https://example.com",
            "text": "Example"
        }]]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("link: https://example.com"));
    assert!(result.contains("Example"));
}

#[test]
fn img_with_alt() {
    let content = json!({
        "elements": [[{"tag": "img", "alt": {"content": "photo"}}]]
    });
    assert!(extract_interactive_content(&content).contains("photo"));
}

#[test]
fn img_without_alt() {
    let content = json!({
        "elements": [[{"tag": "img"}]]
    });
    assert!(extract_interactive_content(&content).contains("[image]"));
}

#[test]
fn note_recursive() {
    let content = json!({
        "elements": [[{
            "tag": "note",
            "elements": [
                {"tag": "plain_text", "content": "note text"}
            ]
        }]]
    });
    assert!(extract_interactive_content(&content).contains("note text"));
}

#[test]
fn column_set_recursive() {
    let content = json!({
        "elements": [[{
            "tag": "column_set",
            "columns": [
                {"elements": [{"tag": "markdown", "content": "col1"}]},
                {"elements": [{"tag": "markdown", "content": "col2"}]}
            ]
        }]]
    });
    let result = extract_interactive_content(&content);
    assert!(result.contains("col1"));
    assert!(result.contains("col2"));
}

#[test]
fn empty_content() {
    assert_eq!(extract_interactive_content(&json!({})), "");
}

#[test]
fn empty_elements() {
    assert_eq!(extract_interactive_content(&json!({"elements": []})), "");
}

#[test]
fn null_content() {
    assert_eq!(extract_interactive_content(&json!(null)), "");
}

#[test]
fn elements_not_array_does_not_panic() {
    // 非预期类型应打印警告但不 panic
    let result = extract_interactive_content(&json!({"elements": "not an array"}));
    assert_eq!(result, "");
}

#[test]
fn element_not_object_does_not_panic() {
    let result = extract_interactive_content(&json!({"elements": [["not an object"]]}));
    assert_eq!(result, "");
}
