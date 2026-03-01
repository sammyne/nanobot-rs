//! 配置模块测试

use super::*;

#[test]
fn default_config() {
    let config = Config::default();
    assert_eq!(config.provider.base_url, DEFAULT_BASE_URL);
    assert_eq!(config.provider.model, "gpt-4o-mini");
    assert!(config.provider.api_key.is_empty());
}

#[test]
fn validate_empty_base_url() {
    let mut config = Config::default();
    config.provider.base_url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn validate_invalid_base_url() {
    let mut config = Config::default();
    config.provider.base_url = "invalid-url".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn validate_empty_api_key() {
    let config = Config::default();
    assert!(config.validate().is_err());
}

#[test]
fn validate_empty_model() {
    let mut config = Config::default();
    config.provider.api_key = "test-key".to_string();
    config.provider.model = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn validate_success() {
    let mut config = Config::default();
    config.provider.api_key = "test-api-key-12345".to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn masked_api_key() {
    let mut config = Config::default();
    config.provider.api_key = "sk-1234567890abcdefghijklmnop".to_string();
    let masked = config.masked_api_key();
    assert!(masked.starts_with("sk-1"));
    assert!(masked.ends_with("mnop"));
    assert!(masked.contains("****"));
}

#[test]
fn masked_api_key_short() {
    let mut config = Config::default();
    config.provider.api_key = "abc".to_string();
    let masked = config.masked_api_key();
    assert_eq!(masked, "***");
}

#[test]
fn load_legacy_config() {
    use super::LegacyConfig;

    let legacy_json = r#"{
        "providers": {
            "custom": {
                "apiKey": "ms-9b01b6f2-1336-4f0d-ac2b-7922f1d66119",
                "apiBase": "https://api-inference.modelscope.cn/v1"
            }
        },
        "agents": {
            "defaults": {
                "model": "MiniMax/MiniMax-M2.5"
            }
        }
    }"#;

    // 验证可以反序列化为旧版格式
    let legacy: LegacyConfig = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(legacy.providers.custom.as_ref().unwrap().api_key, "ms-9b01b6f2-1336-4f0d-ac2b-7922f1d66119");
    assert_eq!(legacy.agents.defaults.model, "MiniMax/MiniMax-M2.5");

    // 模拟从旧版配置转换
    let provider_config = legacy.providers.custom
        .map(|custom| ProviderConfig {
            base_url: custom.api_base,
            api_key: custom.api_key,
            model: legacy.agents.defaults.model,
        })
        .unwrap();

    assert_eq!(provider_config.base_url, "https://api-inference.modelscope.cn/v1");
    assert_eq!(provider_config.api_key, "ms-9b01b6f2-1336-4f0d-ac2b-7922f1d66119");
    assert_eq!(provider_config.model, "MiniMax/MiniMax-M2.5");
}

