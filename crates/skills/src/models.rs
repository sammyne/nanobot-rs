//! Core data structures for skills.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Requirements for a skill to be available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requires {
    /// Required CLI tools that must be available in PATH.
    #[serde(default)]
    pub bins: Vec<String>,

    /// Required environment variables that must be set.
    #[serde(default)]
    pub env: Vec<String>,
}

/// Installation method for a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    /// Unique identifier for this install method.
    pub id: String,

    /// Kind of package manager (e.g., "brew", "apt", "npm").
    pub kind: String,

    /// Package formula name (for brew).
    #[serde(default)]
    pub formula: Option<String>,

    /// Package name (for apt).
    #[serde(default)]
    pub package: Option<String>,

    /// Binary files provided after installation.
    #[serde(default)]
    pub bins: Vec<String>,

    /// Human-readable label for the install option.
    pub label: String,
}

/// Nanobot-specific metadata from the `metadata.nanobot` field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NanobotMeta {
    /// Emoji icon for the skill.
    #[serde(default)]
    pub emoji: Option<String>,

    /// Whether the skill should always be loaded into context.
    #[serde(default)]
    pub always: bool,

    /// Dependencies required by this skill (overrides top-level requires).
    #[serde(default)]
    pub requires: Option<Requires>,

    /// Available installation methods.
    #[serde(default)]
    pub install: Vec<InstallInfo>,
}

/// OpenClaw-specific metadata from the `metadata.openclaw` field.
/// Shares the same structure as NanobotMeta.
pub type OpenClawMeta = NanobotMeta;

/// Skill metadata parsed from YAML frontmatter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMeta {
    /// Human-readable description of the skill.
    #[serde(default)]
    pub description: String,

    /// Whether the skill should always be loaded into context.
    #[serde(default)]
    pub always: bool,

    /// Dependencies required for this skill to function.
    #[serde(default)]
    pub requires: Requires,

    /// Platform-specific metadata (nanobot or openclaw).
    #[serde(default)]
    #[serde(with = "serde_yaml::with::singleton_map")]
    pub metadata: Option<SkillMetadata>,
}

/// Platform-specific metadata enumeration.
///
/// Supports two platforms: nanobot and openclaw.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillMetadata {
    /// Nanobot-specific metadata.
    Nanobot(NanobotMeta),
    /// OpenClaw-specific metadata.
    OpenClaw(OpenClawMeta),
}

/// Source location of a skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillSource {
    /// Skill from workspace skills/ directory (higher priority).
    Workspace,
    /// Built-in skill from the application.
    Builtin,
}

/// A loaded skill with its metadata and availability status.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (directory name).
    pub name: String,

    /// Path to the SKILL.md file.
    pub path: PathBuf,

    /// Source of the skill (workspace or builtin).
    pub source: SkillSource,

    /// Parsed metadata from frontmatter.
    pub meta: SkillMeta,
}

impl Skill {
    /// Creates a new skill with the given name and path.
    pub fn new(name: String, path: PathBuf, source: SkillSource) -> Self {
        Self {
            name,
            path,
            source,
            meta: SkillMeta::default(),
        }
    }

    /// Returns the skill description, falling back to the name if not set.
    pub fn description(&self) -> &str {
        if self.meta.description.is_empty() {
            &self.name
        } else {
            &self.meta.description
        }
    }

    /// Checks if this skill should always be loaded into context.
    pub fn is_always(&self) -> bool {
        // Check top-level always flag
        if self.meta.always {
            return true;
        }

        // Check platform-specific always flag
        self.meta
            .metadata
            .as_ref()
            .map(|m| match m {
                SkillMetadata::Nanobot(meta) => meta.always,
                SkillMetadata::OpenClaw(meta) => meta.always,
            })
            .unwrap_or(false)
    }

    /// Gets the effective requires (platform-specific overrides top-level).
    pub fn effective_requires(&self) -> &Requires {
        self.meta
            .metadata
            .as_ref()
            .and_then(|m| match m {
                SkillMetadata::Nanobot(meta) => meta.requires.as_ref(),
                SkillMetadata::OpenClaw(meta) => meta.requires.as_ref(),
            })
            .unwrap_or(&self.meta.requires)
    }

    /// Returns the emoji for this skill if defined.
    pub fn emoji(&self) -> Option<&str> {
        self.meta.metadata.as_ref().and_then(|m| match m {
            SkillMetadata::Nanobot(meta) => meta.emoji.as_deref(),
            SkillMetadata::OpenClaw(meta) => meta.emoji.as_deref(),
        })
    }

    /// Returns available installation methods.
    pub fn install_methods(&self) -> &[InstallInfo] {
        self.meta
            .metadata
            .as_ref()
            .map(|m| match m {
                SkillMetadata::Nanobot(meta) => meta.install.as_slice(),
                SkillMetadata::OpenClaw(meta) => meta.install.as_slice(),
            })
            .unwrap_or_default()
    }
}
