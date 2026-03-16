//! 错误处理模块
//!
//! 定义通道框架中使用的错误类型。

use thiserror::Error;

/// 通道错误类型
///
/// 涵盖通道框架中可能出现的各种错误情况。
#[derive(Error, Debug)]
pub enum ChannelError {
    /// 通道启动失败
    #[error("通道启动失败: {0}")]
    StartFailed(String),

    /// 通道停止失败
    #[error("通道停止失败: {0}")]
    StopFailed(String),

    /// 消息发送失败
    #[error("消息发送失败: {0}")]
    SendFailed(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// API 错误
    #[error("API 错误: {0}")]
    Api(String),

    /// 认证错误
    #[error("认证错误: {0}")]
    Auth(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    Network(String),

    /// 权限错误
    #[error("权限错误: {0}")]
    Permission(String),

    /// 消息解析错误
    #[error("消息解析错误: {0}")]
    Parse(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML 序列化/反序列化错误
    #[error("YAML 错误: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// HTTP 请求错误
    #[error("HTTP 错误: {0}")]
    Http(#[from] reqwest::Error),
}

/// 从 ConfigError 转换为 ChannelError
impl From<nanobot_config::ConfigError> for ChannelError {
    fn from(e: nanobot_config::ConfigError) -> Self {
        ChannelError::Config(e.to_string())
    }
}

/// 通道操作结果类型
pub type ChannelResult<T> = Result<T, ChannelError>;
