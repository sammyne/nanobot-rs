//! Context builder module for constructing LLM context.
//!
//! This module provides `ContextBuilder` for assembling system prompts
//! and message lists for LLM interactions.

mod builder;
mod error;

pub use builder::ContextBuilder;
pub use error::ContextError;
