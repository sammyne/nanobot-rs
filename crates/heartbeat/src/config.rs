//! Configuration module for heartbeat service

use std::fmt;

use serde::{Deserialize, Serialize};

/// Heartbeat service configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeartbeatConfig {
    /// Whether heartbeat is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Heartbeat interval in seconds (default: 1800 = 30 minutes)
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            interval_seconds: default_interval_seconds(),
        }
    }
}

impl HeartbeatConfig {
    /// Create a new heartbeat config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns `HeartbeatError::InvalidConfig` if the configuration is invalid
    pub fn validate(&self) -> Result<(), crate::error::HeartbeatError> {
        if self.interval_seconds == 0 {
            return Err(crate::error::HeartbeatError::InvalidConfig(
                "interval_seconds must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Create a new heartbeat config with custom values
    pub fn with_values(enabled: bool, interval_seconds: u64) -> Self {
        Self {
            enabled,
            interval_seconds,
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_interval_seconds() -> u64 {
    1800 // 30 minutes
}

impl fmt::Display for HeartbeatConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HeartbeatConfig {{ enabled: {}, interval_seconds: {} }}",
            self.enabled, self.interval_seconds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HeartbeatConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_seconds, 1800);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = HeartbeatConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_interval() {
        let config = HeartbeatConfig::with_values(true, 0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_with_values() {
        let config = HeartbeatConfig::with_values(false, 3600);
        assert!(!config.enabled);
        assert_eq!(config.interval_seconds, 3600);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = HeartbeatConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: HeartbeatConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
