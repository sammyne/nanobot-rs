//! Skills management for nanobot agent.
//!
//! This crate provides functionality to discover, load, and manage skills
//! from workspace and built-in directories.

pub mod builtin;
pub mod dependency;
pub mod loader;
pub mod models;
pub mod parser;
pub mod version;

pub use loader::SkillsLoader;
pub use models::{InstallInfo, NanobotMeta, OpenClawMeta, Requires, Skill, SkillMeta, SkillMetadata, SkillSource};
