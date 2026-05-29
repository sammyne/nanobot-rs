//! Memory module error types.

use thiserror::Error;

/// Memory operation errors.
#[derive(Error, Debug)]
pub enum MemoryError {
    /// File I/O error
    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    /// LLM API call failed
    #[error("LLM API call failed: {0}")]
    LlmApi(String),
}
