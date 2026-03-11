//! Error types for heartbeat service

use thiserror::Error;

/// Heartbeat service errors
#[derive(Error, Debug)]
pub enum HeartbeatError {
    #[error("heartbeat service is already running")]
    AlreadyRunning,

    #[error("heartbeat service is not running")]
    NotRunning,

    #[error("heartbeat service is disabled")]
    Disabled,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("failed to read heartbeat file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("LLM provider error: {0}")]
    ProviderError(#[source] anyhow::Error),

    #[error("failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("execute callback error: {0}")]
    ExecuteError(#[source] anyhow::Error),

    #[error("notify callback error: {0}")]
    NotifyError(#[source] anyhow::Error),
}
