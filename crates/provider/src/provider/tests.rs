//! LLM 提供者模块测试

use super::*;

#[test]
fn message_user() {
    let msg = Message::user("Hello");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello");
}

#[test]
fn message_assistant() {
    let msg = Message::assistant("Hi there!");
    assert_eq!(msg.role, "assistant");
    assert_eq!(msg.content, "Hi there!");
}

#[test]
fn message_system() {
    let msg = Message::system("You are a helpful assistant.");
    assert_eq!(msg.role, "system");
    assert_eq!(msg.content, "You are a helpful assistant.");
}

#[test]
fn provider_creation() {
    let config = ProviderConfig {
        api_base: Some("https://api.openai.com/v1".to_string()),
        api_key: "test-key".to_string(),
        extra_headers: None,
    };

    let provider = OpenAIProvider::new(&config, "gpt-4o-mini");
    assert!(provider.is_ok());
}
