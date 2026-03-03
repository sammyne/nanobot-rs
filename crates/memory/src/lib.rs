//! Memory management module for persistent agent memory.
//!
//! This module implements a two-layer memory system:
//! - **Long-term memory (MEMORY.md)**: Persistent facts and knowledge
//! - **History log (HISTORY.md)**: Grep-searchable conversation summaries
//!
//! Memory consolidation is triggered when conversation history exceeds
//! a configured threshold, using LLM to extract and compress key information.

mod error;
mod store;

pub use error::MemoryError;
pub use store::MemoryStore;
