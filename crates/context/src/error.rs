//! Context module error types.

use thiserror::Error;

/// Context builder errors.
#[derive(Error, Debug)]
pub enum ContextError {
    /// File I/O error
    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid path error
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// MIME type detection error
    #[error("Failed to detect media type: {0}")]
    MediaType(String),

    /// Memory operation error
    #[error("Memory operation failed: {0}")]
    Memory(#[from] nanobot_memory::MemoryError),
}
