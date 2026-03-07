use super::*;

/// 测试钉钉配置验证功能
#[test]
#[allow(clippy::field_reassign_with_default)]
fn dingtalk_config_validation() {
    let mut config = DingTalkConfig::default();
    config.enabled = true;
    assert!(config.validate().is_err());

    config.client_id = "test_client_id".to_string();
    assert!(config.validate().is_err());

    config.client_secret = "test_client_secret".to_string();
    assert!(config.validate().is_ok());
}
