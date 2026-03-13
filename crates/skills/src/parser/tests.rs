use super::*;
use crate::models::SkillMetadata;

#[test]
fn parse_frontmatter_basic() {
    let content = "---\ndescription: Test skill\nalways: true\n---\n# Content";
    let result = parse_frontmatter(content);
    assert_eq!(result, Some("description: Test skill\nalways: true".to_string()));
}

#[test]
fn parse_frontmatter_no_frontmatter() {
    let content = "# Just markdown\nNo frontmatter here";
    let result = parse_frontmatter(content);
    assert!(result.is_none());
}

#[test]
fn strip_frontmatter_basic() {
    let content = "---\ndescription: Test\n---\n# Heading\n\nContent here";
    let result = strip_frontmatter(content);
    assert_eq!(result, "# Heading\n\nContent here");
}

#[test]
fn parse_skill_meta_basic() {
    let yaml = "description: My skill\nalways: true";
    let meta = parse_skill_meta(yaml);
    assert_eq!(meta.description, "My skill");
    assert!(meta.always);
}

#[test]
fn parse_skill_meta_empty() {
    let meta = parse_skill_meta("");
    assert!(meta.description.is_empty());
    assert!(!meta.always);
}

#[test]
fn parse_skill_meta_with_metadata_object() {
    let yaml = r#"
description: Test skill
metadata:
  nanobot:
    emoji: "🦞"
    always: true
"#;
    let meta = parse_skill_meta(yaml);
    assert_eq!(meta.description, "Test skill");
    assert!(meta.metadata.is_some());

    let metadata = meta.metadata.unwrap();
    if let SkillMetadata::Nanobot(nanobot) = metadata {
        assert_eq!(nanobot.emoji, Some("🦞".to_string()));
        assert!(nanobot.always);
    } else {
        panic!("Expected Nanobot variant");
    }
}

#[test]
fn parse_skill_meta_with_openclaw_metadata() {
    let yaml = r#"
description: OpenClaw skill
metadata:
  openclaw:
    always: true
"#;
    let meta = parse_skill_meta(yaml);
    let metadata = meta.metadata.unwrap();
    if let SkillMetadata::OpenClaw(openclaw) = metadata {
        assert!(openclaw.always);
    } else {
        panic!("Expected OpenClaw variant");
    }
}

#[test]
fn serde_enum_direct() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestMeta {
        #[serde(default)]
        emoji: Option<String>,
        #[serde(default)]
        always: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        Nanobot(TestMeta),
        OpenClaw(TestMeta),
    }

    // 需要通过字段来使用 singleton_map
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestContainer {
        #[serde(default, with = "serde_yaml::with::singleton_map")]
        metadata: Option<TestEnum>,
    }

    let yaml = r#"metadata:
  nanobot:
    emoji: "🦞"
    always: true"#;

    let result: Result<TestContainer, _> = serde_yaml::from_str(yaml);
    println!("Result: {result:?}");
    if let Err(ref e) = result {
        println!("Error: {e}");
    }

    assert!(result.is_ok());
    let container = result.unwrap();
    assert!(matches!(container.metadata, Some(TestEnum::Nanobot(_))));
}

#[test]
fn serde_enum_json() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestMeta {
        #[serde(default)]
        emoji: Option<String>,
        #[serde(default)]
        always: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        Nanobot(TestMeta),
        OpenClaw(TestMeta),
    }

    // JSON format test
    let json = r#"{"nanobot": {"emoji": "🦞", "always": true}}"#;

    let result: Result<TestEnum, _> = serde_json::from_str(json);
    println!("JSON Result: {result:?}");
    if let Err(ref e) = result {
        println!("JSON Error: {e}");
    }

    // Serialization test
    let meta = TestEnum::Nanobot(TestMeta { emoji: Some("🦞".to_string()), always: true });
    let serialized = serde_json::to_string(&meta).unwrap();
    println!("JSON Serialized: {serialized}");

    assert!(result.is_ok());
}

#[test]
fn serde_yaml_with_json_format() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct TestMeta {
        #[serde(default)]
        emoji: Option<String>,
        #[serde(default)]
        always: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum TestEnum {
        Nanobot(TestMeta),
        OpenClaw(TestMeta),
    }

    // 需要通过字段来使用 singleton_map
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestContainer {
        #[serde(default, with = "serde_yaml::with::singleton_map")]
        metadata: Option<TestEnum>,
    }

    // Use JSON-like YAML format
    let yaml = r#"{"metadata": {"nanobot": {"emoji": "🦞", "always": true}}}"#;

    let result: Result<TestContainer, _> = serde_yaml::from_str(yaml);
    println!("YAML (JSON-like) Result: {result:?}");
    if let Err(ref e) = result {
        println!("Error: {e}");
    }

    assert!(result.is_ok());
    let container = result.unwrap();
    assert!(matches!(container.metadata, Some(TestEnum::Nanobot(_))));
}

#[test]
fn serde_untagged_enum() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct NanobotMeta {
        #[serde(default)]
        emoji: Option<String>,
        #[serde(default)]
        always: bool,
    }

    type OpenClawMeta = NanobotMeta;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged, rename_all = "lowercase")]
    enum TestEnum {
        Nanobot(NanobotMeta),
        OpenClaw(OpenClawMeta),
    }

    // This won't work because untagged can't distinguish between same types
    // But let's test the format
    let yaml = r#"emoji: "🦞"
always: true"#;

    let result: Result<TestEnum, _> = serde_yaml::from_str(yaml);
    println!("Untagged Result: {result:?}");

    // Check serialization
    let meta = TestEnum::Nanobot(NanobotMeta { emoji: Some("🦞".to_string()), always: true });
    let serialized = serde_yaml::to_string(&meta).unwrap();
    println!("Untagged Serialized:\n{serialized}");
}
