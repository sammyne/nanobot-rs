//! Skills loader implementation.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::dependency::{check_requirements, get_missing_requirements};
use crate::models::{Skill, SkillSource};
use crate::parser::{load_skill_file, strip_frontmatter};

/// Loader for agent skills.
///
/// Skills are markdown files (SKILL.md) that teach the agent how to use
/// specific tools or perform certain tasks.
pub struct SkillsLoader {
    /// Workspace skills directory.
    workspace: PathBuf,
    /// Built-in skills directory.
    builtin: Option<PathBuf>,
}

impl SkillsLoader {
    /// Creates a new SkillsLoader.
    ///
    /// # Arguments
    /// * `workspace` - The workspace root directory
    /// * `builtin_skills_dir` - Optional built-in skills directory
    pub fn new(workspace: PathBuf, builtin_skills_dir: Option<PathBuf>) -> Self {
        let workspace_skills = workspace.join("skills");
        Self {
            workspace: workspace_skills,
            builtin: builtin_skills_dir,
        }
    }

    /// Lists all available skills.
    ///
    /// # Arguments
    /// * `filter_unavailable` - If true, filter out skills with unmet requirements
    pub fn list_skills(&self, filter_unavailable: bool) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();
        let mut seen_names = HashSet::new();

        // Workspace skills (highest priority)
        if self.workspace.exists() {
            self.scan_skills_dir(&self.workspace, SkillSource::Workspace, &mut skills, &mut seen_names)?;
        }

        // Built-in skills
        if let Some(ref builtin_dir) = self.builtin
            && builtin_dir.exists()
        {
            self.scan_skills_dir(builtin_dir, SkillSource::Builtin, &mut skills, &mut seen_names)?;
        }

        // Filter by requirements if requested
        if filter_unavailable {
            skills.retain(|s| self.is_skill_available(s));
        }

        Ok(skills)
    }

    /// Loads a skill by name.
    ///
    /// # Arguments
    /// * `name` - Skill name (directory name)
    ///
    /// # Returns
    /// The skill content or None if not found
    pub fn load_skill(&self, name: &str) -> Option<String> {
        // Check workspace first
        let workspace_skill = self.workspace.join(name).join("SKILL.md");
        if workspace_skill.exists() {
            return fs::read_to_string(&workspace_skill).ok();
        }

        // Check built-in
        if let Some(ref builtin_dir) = self.builtin {
            let builtin_skill = builtin_dir.join(name).join("SKILL.md");
            if builtin_skill.exists() {
                return fs::read_to_string(&builtin_skill).ok();
            }
        }

        None
    }

    /// Loads specific skills for inclusion in agent context.
    ///
    /// # Arguments
    /// * `skill_names` - List of skill names to load
    ///
    /// # Returns
    /// Formatted skills content with frontmatter stripped
    pub fn load_skills_for_context(&self, skill_names: &[String]) -> String {
        let parts: Vec<String> = skill_names
            .iter()
            .filter_map(|name| {
                self.load_skill(name).map(|content| {
                    let stripped = strip_frontmatter(&content);
                    format!("### Skill: {}\n\n{}", name, stripped)
                })
            })
            .collect();

        parts.join("\n\n---\n\n")
    }

    /// Builds a summary of all skills in XML format.
    pub fn build_skills_summary(&self) -> Result<String> {
        let all_skills = self.list_skills(false)?;
        if all_skills.is_empty() {
            return Ok(String::new());
        }

        let mut lines = vec!["<skills>".to_string()];

        for skill in &all_skills {
            let available = self.is_skill_available(skill);
            let name = escape_xml(&skill.name);
            let desc = escape_xml(skill.description());
            let path = skill.path.display().to_string();

            lines.push(format!("  <skill available=\"{}\">", available));
            lines.push(format!("    <name>{}</name>", name));
            lines.push(format!("    <description>{}</description>", desc));
            lines.push(format!("    <location>{}</location>", path));

            // Show missing requirements for unavailable skills
            if !available {
                let missing = self.get_missing_requirements_for_skill(skill);
                if !missing.is_empty() {
                    lines.push(format!("    <requires>{}</requires>", escape_xml(&missing)));
                }
            }

            lines.push("  </skill>".to_string());
        }

        lines.push("</skills>".to_string());
        Ok(lines.join("\n"))
    }

    /// Gets skills marked as always=true that meet requirements.
    pub fn get_always_skills(&self) -> Result<Vec<String>> {
        let skills = self.list_skills(true)?;
        Ok(skills.into_iter().filter(|s| s.is_always()).map(|s| s.name).collect())
    }

    /// Gets metadata from a skill's frontmatter.
    pub fn get_skill_metadata(&self, name: &str) -> Option<Skill> {
        let path = self.find_skill_path(name)?;
        let source = if path.starts_with(&self.workspace) {
            SkillSource::Workspace
        } else {
            SkillSource::Builtin
        };

        match load_skill_file(&path) {
            Ok((meta, _content)) => {
                let mut skill = Skill::new(name.to_string(), path, source);
                skill.meta = meta;
                Some(skill)
            }
            Err(_) => Some(Skill::new(name.to_string(), path, source)),
        }
    }

    // Private helper methods

    fn scan_skills_dir(
        &self,
        dir: &Path,
        source: SkillSource,
        skills: &mut Vec<Skill>,
        seen_names: &mut HashSet<String>,
    ) -> Result<()> {
        for entry in WalkDir::new(dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if !entry.file_type().is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if seen_names.contains(&name) {
                continue;
            }

            let skill_file = entry.path().join("SKILL.md");
            if !skill_file.exists() {
                continue;
            }

            seen_names.insert(name.clone());

            let skill = match load_skill_file(&skill_file) {
                Ok((meta, _content)) => {
                    let mut s = Skill::new(name, skill_file, source);
                    s.meta = meta;
                    s
                }
                Err(_) => Skill::new(name, skill_file, source),
            };

            skills.push(skill);
        }

        Ok(())
    }

    fn find_skill_path(&self, name: &str) -> Option<PathBuf> {
        // Check workspace first
        let workspace_skill = self.workspace.join(name).join("SKILL.md");
        if workspace_skill.exists() {
            return Some(workspace_skill);
        }

        // Check built-in
        if let Some(ref builtin_dir) = self.builtin {
            let builtin_skill = builtin_dir.join(name).join("SKILL.md");
            if builtin_skill.exists() {
                return Some(builtin_skill);
            }
        }

        None
    }

    fn is_skill_available(&self, skill: &Skill) -> bool {
        check_requirements(&skill.meta.requires)
    }

    fn get_missing_requirements_for_skill(&self, skill: &Skill) -> String {
        let missing = get_missing_requirements(&skill.meta.requires);
        missing.join(", ")
    }
}

/// Escapes special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests;
