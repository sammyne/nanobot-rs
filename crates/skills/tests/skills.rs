//! Integration tests for nanobot-skills crate.

use std::fs;

use nanobot_skills::{SkillMeta, SkillSource, SkillsLoader};
use tempfile::TempDir;

/// Creates a test skill in the workspace skills directory.
fn create_workspace_skill(workspace: &std::path::Path, name: &str, content: &str) {
    let skill_dir = workspace.join("skills").join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

/// Creates a test skill in the builtin skills directory.
fn create_builtin_skill(builtin: &std::path::Path, name: &str, content: &str) {
    let skill_dir = builtin.join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();
}

#[test]
fn workspace_skill_priority_over_builtin() {
    let workspace = TempDir::new().unwrap();
    let builtin = TempDir::new().unwrap();

    // Create same skill in both workspace and builtin
    let workspace_content = "---\ndescription: Workspace version\n---\n# Workspace Content";
    let builtin_content = "---\ndescription: Builtin version\n---\n# Builtin Content";

    create_workspace_skill(workspace.path(), "test-skill", workspace_content);
    create_builtin_skill(builtin.path(), "test-skill", builtin_content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), Some(builtin.path().to_path_buf()));

    let skills = loader.list_skills(false).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].source, SkillSource::Workspace);
    assert_eq!(skills[0].meta.description, "Workspace version");
}

#[test]
fn fallback_to_builtin_when_workspace_missing() {
    let workspace = TempDir::new().unwrap();
    let builtin = TempDir::new().unwrap();

    // Create skill only in builtin
    let builtin_content = "---\ndescription: Builtin only\n---\n# Content";
    create_builtin_skill(builtin.path(), "builtin-only", builtin_content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), Some(builtin.path().to_path_buf()));

    let skills = loader.list_skills(false).unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].source, SkillSource::Builtin);
    assert_eq!(skills[0].name, "builtin-only");
}

#[test]
fn filter_unavailable_skills() {
    let workspace = TempDir::new().unwrap();

    // Create available skill
    create_workspace_skill(
        workspace.path(),
        "available",
        "---\ndescription: Available\n---\n# Content",
    );

    // Create unavailable skill with missing CLI requirement
    let unavailable_content = r#"---
description: Unavailable
requires:
  bins:
    - definitely_not_a_real_command_xyz
---
# Content"#;
    create_workspace_skill(workspace.path(), "unavailable", unavailable_content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);

    // Without filter
    let all_skills = loader.list_skills(false).unwrap();
    assert_eq!(all_skills.len(), 2);

    // With filter
    let available_skills = loader.list_skills(true).unwrap();
    assert_eq!(available_skills.len(), 1);
    assert_eq!(available_skills[0].name, "available");
}

#[test]
fn skills_summary_shows_missing_requirements() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: Skill with requirements
requires:
  bins:
    - nonexistent_cli_tool
  env:
    - NONEXISTENT_ENV_VAR
---
# Content"#;
    create_workspace_skill(workspace.path(), "requires-skill", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let summary = loader.build_skills_summary().unwrap();

    assert!(summary.contains("available=\"false\""));
    assert!(summary.contains("CLI: nonexistent_cli_tool"));
    assert!(summary.contains("ENV: NONEXISTENT_ENV_VAR"));
}

#[test]
fn always_skill_detection() {
    let workspace = TempDir::new().unwrap();

    // Create always skill (in frontmatter)
    create_workspace_skill(
        workspace.path(),
        "always-skill",
        "---\ndescription: Always\nalways: true\n---\n# Content",
    );

    // Create always skill (in metadata.nanobot)
    let nanobot_always = r#"---
description: Nanobot Always
metadata: {"nanobot": {"always": true}}
---
# Content"#;
    create_workspace_skill(workspace.path(), "nanobot-always", nanobot_always);

    // Create normal skill
    create_workspace_skill(
        workspace.path(),
        "normal-skill",
        "---\ndescription: Normal\n---\n# Content",
    );

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let always_skills = loader.get_always_skills().unwrap();

    assert_eq!(always_skills.len(), 2);
    assert!(always_skills.contains(&"always-skill".to_string()));
    assert!(always_skills.contains(&"nanobot-always".to_string()));
    assert!(!always_skills.contains(&"normal-skill".to_string()));
}

#[test]
fn load_skills_for_context_strips_frontmatter() {
    let workspace = TempDir::new().unwrap();

    create_workspace_skill(
        workspace.path(),
        "test",
        "---\ndescription: Test\n---\n# Test Content\n\nThis is the body.",
    );

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let context = loader.load_skills_for_context(&["test".to_string()]);

    assert!(!context.contains("---"));
    assert!(!context.contains("description:"));
    assert!(context.contains("### Skill: test"));
    assert!(context.contains("# Test Content"));
    assert!(context.contains("This is the body."));
}

#[test]
fn multiple_skills_in_context() {
    let workspace = TempDir::new().unwrap();

    create_workspace_skill(workspace.path(), "skill-a", "---\n---\n# Skill A");
    create_workspace_skill(workspace.path(), "skill-b", "---\n---\n# Skill B");

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let context = loader.load_skills_for_context(&["skill-a".to_string(), "skill-b".to_string()]);

    assert!(context.contains("### Skill: skill-a"));
    assert!(context.contains("# Skill A"));
    assert!(context.contains("---"));
    assert!(context.contains("### Skill: skill-b"));
    assert!(context.contains("# Skill B"));
}

#[test]
fn empty_directory_returns_empty_list() {
    let workspace = TempDir::new().unwrap();
    // Don't create any skills

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert!(skills.is_empty());
}

#[test]
fn invalid_yaml_returns_default_metadata() {
    let workspace = TempDir::new().unwrap();

    // Create skill with invalid YAML
    create_workspace_skill(
        workspace.path(),
        "invalid",
        "---\ninvalid yaml content :::\n---\n# Content",
    );

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "invalid");
    // Should have default metadata, not crash
    assert!(skills[0].meta.description.is_empty());
}

#[test]
fn skill_without_frontmatter() {
    let workspace = TempDir::new().unwrap();

    // Create skill without frontmatter
    create_workspace_skill(
        workspace.path(),
        "no-frontmatter",
        "# Just Markdown\n\nNo frontmatter here.",
    );

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "no-frontmatter");
    assert!(skills[0].meta.description.is_empty());
}

#[test]
fn directory_without_skill_file_ignored() {
    let workspace = TempDir::new().unwrap();

    // Create directory without SKILL.md
    let skill_dir = workspace.path().join("skills").join("empty-dir");
    fs::create_dir_all(&skill_dir).unwrap();
    // No SKILL.md file

    // Create valid skill
    create_workspace_skill(workspace.path(), "valid", "---\n---\n# Valid");

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "valid");
}

#[test]
fn openclaw_metadata_key_supported() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: OpenClaw Skill
metadata: {"openclaw": {"always": true, "custom": "value"}}
---
# Content"#;
    create_workspace_skill(workspace.path(), "openclaw-skill", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let always_skills = loader.get_always_skills().unwrap();

    assert_eq!(always_skills.len(), 1);
    assert!(always_skills.contains(&"openclaw-skill".to_string()));
}

#[test]
fn description_fallback_to_name() {
    let workspace = TempDir::new().unwrap();

    // Create skill without description
    create_workspace_skill(workspace.path(), "no-desc-skill", "---\n---\n# Content");

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let summary = loader.build_skills_summary().unwrap();

    // Description should fall back to name
    assert!(summary.contains("<description>no-desc-skill</description>"));
}

#[test]
fn skill_emoji_from_metadata() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: ClawHub Skill
metadata:
  nanobot:
    emoji: "🦞"
---
# Content"#;
    create_workspace_skill(workspace.path(), "clawhub", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert_eq!(skills[0].emoji(), Some("🦞"));
}

#[test]
fn skill_install_methods() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: GitHub Skill
metadata:
  nanobot:
    install:
      - id: brew
        kind: brew
        formula: gh
        bins:
          - gh
        label: Install GitHub CLI (brew)
      - id: apt
        kind: apt
        package: gh
        bins:
          - gh
        label: Install GitHub CLI (apt)
---
# Content"#;
    create_workspace_skill(workspace.path(), "github", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    let install_methods = skills[0].install_methods();
    assert_eq!(install_methods.len(), 2);
    assert_eq!(install_methods[0].id, "brew");
    assert_eq!(install_methods[0].formula, Some("gh".to_string()));
    assert_eq!(install_methods[1].id, "apt");
    assert_eq!(install_methods[1].package, Some("gh".to_string()));
}

#[test]
fn effective_requires_from_metadata() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: Skill with metadata requires
requires:
  bins:
    - default_cli
metadata:
  nanobot:
    requires:
      bins:
        - specific_cli
---
# Content"#;
    create_workspace_skill(workspace.path(), "test-requires", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    // Platform-specific requires should override top-level
    let effective = skills[0].effective_requires();
    assert_eq!(effective.bins, vec!["specific_cli"]);
}

#[test]
fn top_level_requires_when_no_metadata_requires() {
    let workspace = TempDir::new().unwrap();

    let content = r#"---
description: Skill with top-level requires only
requires:
  bins:
    - default_cli
metadata:
  nanobot:
    emoji: "🔧"
---
# Content"#;
    create_workspace_skill(workspace.path(), "test-top-requires", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    // Should use top-level requires when metadata has no requires
    let effective = skills[0].effective_requires();
    assert_eq!(effective.bins, vec!["default_cli"]);
}

#[test]
fn parse_json_like_metadata_field() {
    let content = r#"
name: github
description: "Interact with GitHub using the `gh` CLI."
metadata: {"nanobot":{"emoji":"🐙","requires":{"bins":["gh"]},"install":[{"id":"brew","kind":"brew","formula":"gh","bins":["gh"],"label":"Install GitHub CLI (brew)"},{"id":"apt","kind":"apt","package":"gh","bins":["gh"],"label":"Install GitHub CLI (apt)"}]}}
"#;

    let md: SkillMeta = serde_yaml::from_str(content).expect("failed");
    println!("Parsed metadata: {:?}", md);
}

/// Test parsing actual github SKILL.md format with flow mapping metadata.
/// This is the real-world format used in the Python version.
#[test]
fn github_skill_real_format() {
    let workspace = TempDir::new().unwrap();

    // Exact format from _nanobot/nanobot/skills/github/SKILL.md
    let content = r#"---
name: github
description: "Interact with GitHub using the `gh` CLI."
metadata: {"nanobot":{"emoji":"🐙","requires":{"bins":["gh"]},"install":[{"id":"brew","kind":"brew","formula":"gh","bins":["gh"],"label":"Install GitHub CLI (brew)"},{"id":"apt","kind":"apt","package":"gh","bins":["gh"],"label":"Install GitHub CLI (apt)"}]}}
---
# GitHub Skill

Use the `gh` CLI to interact with GitHub.
"#;
    create_workspace_skill(workspace.path(), "github", content);

    let loader = SkillsLoader::new(workspace.path().to_path_buf(), None);
    let skills = loader.list_skills(false).unwrap();

    assert_eq!(skills.len(), 1);
    let skill = &skills[0];

    // Verify description
    assert_eq!(skill.meta.description, "Interact with GitHub using the `gh` CLI.");

    // Verify emoji
    assert_eq!(skill.emoji(), Some("🐙"));

    // Verify requires from metadata
    let effective = skill.effective_requires();
    assert_eq!(effective.bins, vec!["gh"]);
    assert!(effective.env.is_empty());

    // Verify install methods
    let install = skill.install_methods();
    assert_eq!(install.len(), 2);
    assert_eq!(install[0].id, "brew");
    assert_eq!(install[0].kind, "brew");
    assert_eq!(install[0].formula, Some("gh".to_string()));
    assert_eq!(install[0].bins, vec!["gh"]);
    assert_eq!(install[0].label, "Install GitHub CLI (brew)");

    assert_eq!(install[1].id, "apt");
    assert_eq!(install[1].kind, "apt");
    assert_eq!(install[1].package, Some("gh".to_string()));
    assert_eq!(install[1].label, "Install GitHub CLI (apt)");
}
