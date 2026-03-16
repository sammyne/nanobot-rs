//! 配置模块测试

use std::path::PathBuf;

use super::*;
use crate::HOME;

#[test]
fn default_config() {
    let config = Config::default();
    let provider = config.provider();
    assert_eq!(provider.api_base, None);
    assert_eq!(provider.api_key, "");
    assert!(provider.extra_headers.is_none());
}

#[test]
fn validate_empty_workspace() {
    let mut config = Config::default();
    config.agents.defaults.workspace = PathBuf::new();
    assert!(config.validate().is_err());
}

#[test]
fn validate_empty_model() {
    let mut config = Config::default();
    config.agents.defaults.model = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn validate_zero_max_tokens() {
    let mut config = Config::default();
    config.agents.defaults.max_tokens = 0;
    assert!(config.validate().is_err());
}

#[test]
fn validate_invalid_api_base() {
    let mut config = Config::default();
    config.providers.custom = Some(ProviderConfig {
        api_key: "test-key".to_string(),
        api_base: Some("invalid-url".to_string()),
        extra_headers: None,
    });
    assert!(config.validate().is_err());
}

#[test]
fn validate_short_api_key() {
    let mut config = Config::default();
    config.providers.custom = Some(ProviderConfig {
        api_key: "ab".to_string(),
        api_base: Some("https://api.example.com".to_string()),
        extra_headers: None,
    });
    assert!(config.validate().is_err());
}

#[test]
fn validate_success() {
    let config = Config::new(ProviderConfig {
        api_base: Some("https://api.openai.com/v1".to_string()),
        api_key: "test-api-key-12345".to_string(),
        extra_headers: None,
    });
    assert!(config.validate().is_ok());
}

#[test]
fn validate_without_custom_provider() {
    let config = Config::default();
    // agents.defaults 有默认值，应该验证通过
    assert!(config.validate().is_ok());
}

#[test]
fn masked_api_key() {
    let config = Config::new(ProviderConfig {
        api_base: Some("https://api.openai.com/v1".to_string()),
        api_key: "sk-1234567890abcdefghijklmnop".to_string(),
        extra_headers: None,
    });
    let masked = config.masked_api_key();
    assert!(masked.starts_with("sk-1"));
    assert!(masked.ends_with("mnop"));
    assert!(masked.contains("****"));
}

#[test]
fn masked_api_key_short() {
    let mut config = Config::default();
    config.providers.custom = Some(ProviderConfig {
        api_key: "abc".to_string(),
        api_base: Some("https://api.example.com".to_string()),
        extra_headers: None,
    });
    let masked = config.masked_api_key();
    assert_eq!(masked, "***");
}

#[test]
fn load_hkuds_config() {
    let hkuds_json = r#"{
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

    // 验证可以反序列化为 HKUDS 格式
    let config: Config = serde_json::from_str(hkuds_json).unwrap();
    assert_eq!(config.providers.custom.as_ref().unwrap().api_key, "ms-9b01b6f2-1336-4f0d-ac2b-7922f1d66119");
    assert_eq!(config.agents.defaults.model, "MiniMax/MiniMax-M2.5");

    // 验证 provider() 方法
    let provider = config.provider();
    assert_eq!(provider.api_base, Some("https://api-inference.modelscope.cn/v1".to_string()));
    assert_eq!(provider.api_key, "ms-9b01b6f2-1336-4f0d-ac2b-7922f1d66119");
}

#[test]
fn config_from_provider() {
    let provider_config = ProviderConfig {
        api_base: Some("https://api.example.com/v1".to_string()),
        api_key: "sk-test-12345".to_string(),
        extra_headers: None,
    };

    let config = Config::new(provider_config.clone());
    assert!(config.validate().is_ok());

    let retrieved = config.provider();
    assert_eq!(retrieved.api_base, provider_config.api_base);
    assert_eq!(retrieved.api_key, provider_config.api_key);
}

#[test]
fn load_partial_config_fill_defaults() {
    // 测试加载部分配置时自动填充默认值
    let partial_json = r#"{
            "providers": {
                "custom": {
                    "apiKey": "sk-test-key"
                }
            }
        }"#;

    let config: Config = serde_json::from_str(partial_json).unwrap();

    // api_base 应该自动填充为 None
    assert_eq!(config.providers.custom.as_ref().unwrap().api_base, None);

    // agents.defaults 应该使用默认值
    assert_eq!(config.agents.defaults.model, "anthropic/claude-opus-4-5");
    assert_eq!(config.agents.defaults.max_tokens, 8192);
    assert_eq!(config.agents.defaults.temperature, 0.1);

    // 验证配置是有效的
    assert!(config.validate().is_ok());
}

#[test]
fn load_config_with_all_fields_present() {
    // 测试加载完整配置时使用提供的值而非默认值
    let full_json = r#"{
            "providers": {
                "custom": {
                    "apiKey": "sk-custom-key",
                    "apiBase": "https://custom.api.com/v1",
                    "extraHeaders": {
                        "X-Custom-Header": "value"
                    }
                }
            },
            "agents": {
                "defaults": {
                    "workspace": "/tmp/workspace",
                    "model": "custom-model",
                    "maxTokens": 4096,
                    "temperature": 0.5
                }
            }
        }"#;

    let config: Config = serde_json::from_str(full_json).unwrap();

    // 验证使用的是提供的值而非默认值
    assert_eq!(config.providers.custom.as_ref().unwrap().api_key, "sk-custom-key");
    assert_eq!(config.providers.custom.as_ref().unwrap().api_base, Some("https://custom.api.com/v1".to_string()));
    assert_eq!(
        config.providers.custom.as_ref().unwrap().extra_headers,
        Some({
            let mut headers = std::collections::HashMap::new();
            headers.insert("X-Custom-Header".to_string(), "value".to_string());
            headers
        })
    );
    assert_eq!(config.agents.defaults.workspace, PathBuf::from("/tmp/workspace"));
    assert_eq!(config.agents.defaults.model, "custom-model");
    assert_eq!(config.agents.defaults.max_tokens, 4096);
    assert_eq!(config.agents.defaults.temperature, 0.5);

    // 验证配置是有效的
    assert!(config.validate().is_ok());
}

#[test]
fn load_empty_config_fill_all_defaults() {
    // 测试加载空配置时自动填充所有默认值
    let empty_json = r#"{}"#;

    let config: Config = serde_json::from_str(empty_json).unwrap();

    // providers.custom 应该为 None（因为它是 Option 类型且有 default）
    assert!(config.providers.custom.is_none());

    // agents.defaults 应该自动填充默认值（~ 会被替换为 HOME）
    assert_eq!(config.agents.defaults.workspace, HOME.join(".nanobot/workspace"));
    assert_eq!(config.agents.defaults.model, "anthropic/claude-opus-4-5");
    assert_eq!(config.agents.defaults.max_tokens, 8192);
    assert_eq!(config.agents.defaults.temperature, 0.1);
    assert_eq!(config.agents.defaults.max_tool_iterations, 40);
    assert_eq!(config.agents.defaults.memory_window, 100);

    // 验证配置是有效的
    assert!(config.validate().is_ok());
}

#[test]
fn provider_config_with_extra_headers() {
    let mut headers = std::collections::HashMap::new();
    headers.insert("APP-Code".to_string(), "test-code".to_string());
    headers.insert("X-API-Version".to_string(), "v1".to_string());

    let provider_config = ProviderConfig {
        api_base: Some("https://api.example.com/v1".to_string()),
        api_key: "sk-test-key".to_string(),
        extra_headers: Some(headers.clone()),
    };

    let config = Config::new(provider_config);
    let retrieved = config.provider();

    assert_eq!(retrieved.extra_headers, Some(headers));
}

#[test]
fn agent_defaults_full_config() {
    let json = r#"{
            "agents": {
                "defaults": {
                    "workspace": "/custom/workspace",
                    "model": "gpt-4",
                    "maxTokens": 16000,
                    "temperature": 0.8,
                    "maxToolIterations": 50,
                    "memoryWindow": 200
                }
            }
        }"#;

    let config: Config = serde_json::from_str(json).unwrap();

    assert_eq!(config.agents.defaults.workspace, PathBuf::from("/custom/workspace"));
    assert_eq!(config.agents.defaults.model, "gpt-4");
    assert_eq!(config.agents.defaults.max_tokens, 16000);
    assert_eq!(config.agents.defaults.temperature, 0.8);
    assert_eq!(config.agents.defaults.max_tool_iterations, 50);
    assert_eq!(config.agents.defaults.memory_window, 200);
}

#[test]
fn extra_headers_skip_serializing_if_none() {
    // 测试 extra_headers 为 None 时不序列化该字段
    let provider_config = ProviderConfig {
        api_base: Some("https://api.example.com/v1".to_string()),
        api_key: "sk-test-key".to_string(),
        extra_headers: None,
    };

    let config = Config::new(provider_config);
    let json = serde_json::to_string_pretty(&config).unwrap();

    // 验证 JSON 中不包含 extraHeaders 字段
    assert!(!json.contains("extraHeaders"));

    // 反序列化回来，验证配置仍然有效
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.provider().extra_headers, None);
}

#[test]
fn extra_headers_serialize_when_some() {
    // 测试 extra_headers 有值时正常序列化该字段
    let mut headers = std::collections::HashMap::new();
    headers.insert("APP-Code".to_string(), "test-code".to_string());

    let provider_config = ProviderConfig {
        api_base: Some("https://api.example.com/v1".to_string()),
        api_key: "sk-test-key".to_string(),
        extra_headers: Some(headers),
    };

    let config = Config::new(provider_config);
    let json = serde_json::to_string_pretty(&config).unwrap();

    // 验证 JSON 中包含 extraHeaders 字段
    assert!(json.contains("extraHeaders"));
    assert!(json.contains("APP-Code"));
    assert!(json.contains("test-code"));
}

#[test]
fn gateway_config_default_values() {
    // 测试 GatewayConfig 的默认值
    let gateway = GatewayConfig::default();
    assert_eq!(gateway.host, "0.0.0.0");
    assert_eq!(gateway.port, 18790);
}

#[test]
fn gateway_config_validate_success() {
    // 测试有效的 gateway 配置
    let gateway = GatewayConfig::default();
    assert!(gateway.validate().is_ok());
}

#[test]
fn gateway_config_validate_zero_port() {
    // 测试 port 为 0 时验证失败
    let gateway = GatewayConfig { port: 0, ..Default::default() };
    let result = gateway.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::Validation(msg) if msg == "gateway.port 必须大于 0"));
}

#[test]
fn gateway_config_validate_empty_host() {
    // 测试 host 为空时验证失败
    let gateway = GatewayConfig { host: String::new(), ..Default::default() };
    let result = gateway.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::Validation(msg) if msg == "gateway.host 不能为空"));
}

#[test]
fn config_with_gateway_section() {
    // 测试配置文件中包含 gateway 节
    let json = r#"{
            "gateway": {
                "host": "127.0.0.1",
                "port": 8080
            }
        }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.gateway.host, "127.0.0.1");
    assert_eq!(config.gateway.port, 8080);
    assert!(config.validate().is_ok());
}

#[test]
fn config_with_partial_gateway_section() {
    // 测试配置文件中只包含部分 gateway 字段时自动填充默认值
    let json = r#"{
            "gateway": {
                "port": 9090
            }
        }"#;

    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.gateway.host, "0.0.0.0"); // 使用默认值
    assert_eq!(config.gateway.port, 9090); // 使用配置值
    assert!(config.validate().is_ok());
}

#[test]
fn config_gateway_serialization() {
    // 测试 gateway 配置的序列化
    let config = Config::default();
    let json = serde_json::to_string_pretty(&config).unwrap();

    // 验证序列化后包含 gateway 字段
    assert!(json.contains("gateway"));
    assert!(json.contains("host"));
    assert!(json.contains("port"));
}

// ==================== 环境变量覆盖测试 ====================

#[test]
fn env_override_string_field() {
    // 测试环境变量覆盖字符串字段

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // 创建基础配置文件
    let config_content = r#"{
        "providers": {
            "custom": {
                "apiKey": "original-key"
            }
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置环境变量覆盖
    // 前缀 NANOBOT_（由 with_prefix("NANOBOT") + prefix_separator("_") 组成）
    // 层级分隔符 __（separator("__")）
    // 使用 snake_case（API_KEY），config 库会将其转换为 camelCase（apiKey）
    temp_env::with_var("NANOBOT_PROVIDERS__CUSTOM__API_KEY", Some("env-override-key"), || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证环境变量覆盖了配置文件中的值
        assert_eq!(result.providers.custom.as_ref().unwrap().api_key, "env-override-key");
    });
}

#[test]
fn env_override_number_field() {
    // 测试环境变量覆盖数字字段

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "gateway": {
            "port": 18790
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置环境变量覆盖
    // config 库会将 PORT 转换为 port（全小写）
    temp_env::with_var("NANOBOT_GATEWAY__PORT", Some("8080"), || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证环境变量覆盖了配置文件中的值
        assert_eq!(result.gateway.port, 8080);
    });
}

#[test]
fn env_override_boolean_field() {
    // 测试环境变量覆盖布尔字段

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "channels": {
            "dingtalk": {
                "enabled": false,
                "clientId": "test-client-id",
                "clientSecret": "test-client-secret"
            }
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置环境变量覆盖
    // 使用 snake_case（ENABLED），config 库会保持为 enabled
    temp_env::with_var("NANOBOT_CHANNELS__DINGTALK__ENABLED", Some("true"), || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证环境变量覆盖了配置文件中的值
        assert!(result.channels.dingtalk.as_ref().unwrap().enabled);
    });
}

#[test]
fn env_override_nested_field() {
    // 测试环境变量覆盖嵌套字段

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "agents": {
            "defaults": {
                "model": "original-model"
            }
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置环境变量覆盖
    // config 库会将 MODEL 转换为 model（全小写）
    temp_env::with_var("NANOBOT_AGENTS__DEFAULTS__MODEL", Some("gpt-4"), || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证环境变量覆盖了配置文件中的值
        assert_eq!(result.agents.defaults.model, "gpt-4");
    });
}

#[test]
fn env_no_override_when_not_set() {
    // 测试环境变量未设置时使用配置文件中的值

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "providers": {
            "custom": {
                "apiKey": "file-key"
            }
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 不设置环境变量
    temp_env::with_var_unset("NANOBOT_PROVIDERS__CUSTOM__API_KEY", || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证使用配置文件中的值
        assert_eq!(result.providers.custom.as_ref().unwrap().api_key, "file-key");
    });
}

#[test]
fn env_override_multiple_fields() {
    // 测试同时覆盖多个字段

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "providers": {
            "custom": {
                "apiKey": "original-key",
                "apiBase": "https://original.api.com"
            }
        },
        "gateway": {
            "port": 18790
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置多个环境变量
    // 使用 snake_case（API_KEY），config 库会转换为 camelCase（apiKey）
    temp_env::with_vars(
        [("NANOBOT_PROVIDERS__CUSTOM__API_KEY", Some("new-key")), ("NANOBOT_GATEWAY__PORT", Some("9000"))],
        || {
            let result = Config::load_from_path(&config_path).unwrap().unwrap();

            // 验证所有环境变量都生效
            assert_eq!(result.providers.custom.as_ref().unwrap().api_key, "new-key");
            assert_eq!(
                result.providers.custom.as_ref().unwrap().api_base,
                Some("https://original.api.com".to_string())
            ); // 未被覆盖
            assert_eq!(result.gateway.port, 9000);
        },
    );
}

#[test]
fn env_empty_value_ignored() {
    // 测试空环境变量值被忽略，使用配置文件中的值

    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "providers": {
            "custom": {
                "apiKey": "file-key"
            }
        }
    }"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 设置空环境变量
    temp_env::with_var("NANOBOT_PROVIDERS__CUSTOM__API_KEY", Some(""), || {
        let result = Config::load_from_path(&config_path).unwrap().unwrap();

        // 验证空值被忽略，使用配置文件中的值
        assert_eq!(result.providers.custom.as_ref().unwrap().api_key, "file-key");
    });
}
