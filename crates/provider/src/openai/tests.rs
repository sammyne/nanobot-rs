use super::*;

#[test]
fn message_creation() {
    let user_msg = Message::user("Hello");
    assert_eq!(user_msg.role, "user");
    assert_eq!(user_msg.content, "Hello");

    let assistant_msg = Message::assistant("Hi there");
    assert_eq!(assistant_msg.role, "assistant");
    assert_eq!(assistant_msg.content, "Hi there");

    let system_msg = Message::system("System prompt");
    assert_eq!(system_msg.role, "system");
    assert_eq!(system_msg.content, "System prompt");
}

#[test]
fn message_from_string() {
    let s = String::from("test content");
    let msg = Message::user(s);
    assert_eq!(msg.content, "test content");
}

#[test]
fn provider_error_display() {
    let err = ProviderError::Api("Connection failed".to_string());
    assert_eq!(err.to_string(), "LLM API 调用失败: Connection failed");

    let err = ProviderError::Timeout;
    assert_eq!(err.to_string(), "请求超时");

    let err = ProviderError::Config("Missing key".to_string());
    assert_eq!(err.to_string(), "配置错误: Missing key");
}
