use std::fs;

use tempfile::TempDir;

use super::*;

fn create_test_skill(workspace: &Path, name: &str, content: &str) -> PathBuf {
    let skill_dir = workspace.join("skills").join(name);
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_file = skill_dir.join("SKILL.md");
    fs::write(&skill_file, content).unwrap();
    skill_file
}

#[test]
fn list_skills_basic() {
    let temp = TempDir::new().unwrap();
    let content = "---\ndescription: Test skill\n---\n# Content";
    create_test_skill(temp.path(), "test-skill", content);

    let loader = SkillsLoader::new(temp.path().to_path_buf());
    let skills = loader.list_skills(false).unwrap();

    // Should include both the test skill and builtin skills
    assert!(!skills.is_empty());
    let test_skill = skills.iter().find(|s| s.name == "test-skill");
    assert!(test_skill.is_some());
}

#[test]
fn load_skill_existing() {
    let temp = TempDir::new().unwrap();
    let content = "---\ndescription: Test\n---\n# Test Content";
    create_test_skill(temp.path(), "my-skill", content);

    let loader = SkillsLoader::new(temp.path().to_path_buf());
    let loaded = loader.load_skill("my-skill").unwrap();

    assert!(loaded.contains("# Test Content"));
}

#[test]
fn load_skill_nonexistent() {
    let temp = TempDir::new().unwrap();
    let loader = SkillsLoader::new(temp.path().to_path_buf());

    assert!(loader.load_skill("nonexistent").is_none());
}

#[test]
fn build_skills_summary_empty() {
    let temp = TempDir::new().unwrap();
    // Create a workspace with no custom skills
    let loader = SkillsLoader::new(temp.path().to_path_buf());

    let summary = loader.build_skills_summary().unwrap();
    // May contain builtin skills, so we only check that it's valid XML
    if !summary.is_empty() {
        assert!(summary.contains("<skills>"));
        assert!(summary.contains("</skills>"));
    }
}

#[test]
fn build_skills_summary_with_skills() {
    let temp = TempDir::new().unwrap();
    let content = "---\ndescription: A test skill\n---\n# Content";
    create_test_skill(temp.path(), "test", content);

    let loader = SkillsLoader::new(temp.path().to_path_buf());
    let summary = loader.build_skills_summary().unwrap();

    assert!(summary.contains("<skills>"));
    assert!(summary.contains("<name>test</name>"));
    assert!(summary.contains("<description>A test skill</description>"));
}

#[test]
fn escape_xml_special_chars() {
    assert_eq!(escape_xml("a & b"), "a &amp; b");
    assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
}
