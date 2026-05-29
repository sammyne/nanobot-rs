//! Memory management module for persistent agent memory.
//!
//! This module implements a two-layer memory system:
//! - **Long-term memory (MEMORY.md)**: Persistent facts and knowledge
//! - **History log (history.jsonl)**: Structured conversation summaries with cursor tracking
//!
//! Memory consolidation is triggered when conversation history exceeds
//! a configured threshold, using LLM to extract and compress key information.

mod error;
pub mod gitstore;
pub mod history;
mod store;

pub use error::MemoryError;
pub use gitstore::{CommitInfo, GitStore};
pub use history::{History, HistoryEntry};
pub use store::{
    MAX_CONSOLIDATION_ROUNDS, MAX_FAILURES_BEFORE_RAW_ARCHIVE, MemoryStore, consolidate_memory, should_consolidate,
};
