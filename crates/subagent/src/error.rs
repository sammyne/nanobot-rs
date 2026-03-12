//! Subagent 错误类型定义

use thiserror::Error;

/// Subagent 错误类型
#[derive(Error, Debug)]
pub enum SubagentError {
    /// LLM 提供商错误
    #[error("LLM 提供商错误: {0}")]
    Provider(#[from] anyhow::Error),

    /// 工具执行错误
    #[error("工具执行错误: {0}")]
    Tool(String),

    /// 任务执行超时
    #[error("任务执行超时: 已达到最大迭代次数 {0}")]
    Timeout(usize),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 无效参数
    #[error("无效参数: {0}")]
    InvalidParam(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

/// Subagent 结果类型别名
pub type SubagentResult<T> = Result<T, SubagentError>;
