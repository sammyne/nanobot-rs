//! YAML frontmatter parser for skill files.

use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::models::SkillMeta;

/// Parses YAML frontmatter from a skill markdown file.
///
/// The frontmatter is expected to be enclosed in `---` delimiters at the
/// beginning of the file.
pub fn parse_frontmatter(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let remaining = &content[3..];
    remaining.find("\n---").map(|end_pos| remaining[..end_pos].trim().to_string())
}

/// Extracts the content after the frontmatter.
pub fn strip_frontmatter(content: &str) -> String {
    if !content.starts_with("---") {
        return content.to_string();
    }

    // Find the closing --- and return content after it
    let remaining = &content[3..];
    if let Some(end_pos) = remaining.find("\n---") {
        remaining[end_pos + 5..].trim().to_string()
    } else {
        content.to_string()
    }
}

/// Parses skill metadata from YAML frontmatter content.
pub fn parse_skill_meta(yaml_content: &str) -> SkillMeta {
    if yaml_content.is_empty() {
        return SkillMeta::default();
    }

    serde_yaml::from_str(yaml_content).unwrap_or_default()
}

/// Loads and parses a skill file, returning the metadata and content.
pub fn load_skill_file(path: &Path) -> Result<(SkillMeta, String)> {
    let content = fs::read_to_string(path)?;

    let meta =
        if let Some(yaml) = parse_frontmatter(&content) { parse_skill_meta(&yaml) } else { SkillMeta::default() };

    Ok((meta, content))
}

#[cfg(test)]
mod tests;
