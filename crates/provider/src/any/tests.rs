use nanobot_config::{Config, ProviderConfig, ProvidersConfig};

use super::*;

#[test]
fn from_config_custom_creates_openai() {
    let config = Config::new(ProvidersConfig::Custom(ProviderConfig {
        api_key: "sk-test".to_string(),
        api_base: Some("https://api.openai.com/v1".to_string()),
        extra_headers: None,
    }));

    let provider = AnyProvider::from_config(&config).unwrap();
    assert!(matches!(provider, AnyProvider::OpenAI(_)));
}

#[test]
fn from_config_anthropic_creates_anthropic() {
    let config = Config::new(ProvidersConfig::Anthropic(ProviderConfig {
        api_key: "sk-ant-test".to_string(),
        api_base: None,
        extra_headers: None,
    }));

    let provider = AnyProvider::from_config(&config).unwrap();
    assert!(matches!(provider, AnyProvider::Anthropic(_)));
}
