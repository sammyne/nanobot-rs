# nanobot-skills

Skills management for nanobot agent. This crate provides functionality to discover, load, and manage skills from workspace and built-in directories.

## Features

- **Builtin Skills**: Automatically initialized skills that ship with the crate
- **Custom Skills**: User-defined skills in the workspace
- **Priority Management**: Workspace skills override builtin skills with the same name
- **Version Management**: Automatic synchronization of builtin skills with crate version

## Usage

### Basic Usage

```rust
use nanobot_skills::SkillsLoader;
use std::path::PathBuf;

// Create a skills loader for your workspace
let workspace = PathBuf::from("/path/to/workspace");
let loader = SkillsLoader::new(workspace);

// List all available skills
let skills = loader.list_skills(false)?;
for skill in &skills {
    println!("- {} (from {:?})", skill.name, skill.source);
}

// Load a specific skill
if let Some(content) = loader.load_skill("github") {
    println!("Skill content:\n{}", contents);
}
```

## Builtin Skills

### Technical Implementation

Builtin skills are **embedded into the binary at compile time** using the `include_dir` crate. This approach solves the problem that `CARGO_MANIFEST_DIR` is only available during compilation, not at runtime when the binary is deployed.

```rust
// At compile time, the builtin/ directory is embedded
static BUILTIN_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/builtin");

// At runtime, files are extracted from the embedded resources
pub fn initialize_builtin_skills(workspace: &Path) -> Result<()> {
    let builtin_dir = workspace.join("builtin-skills");
    extract_dir(&BUILTIN_DIR, &builtin_dir)?;
    // ...
}
```

This ensures builtin skills are always available regardless of where the binary is installed.

### Automatic Initialization

When `SkillsLoader::new()` is called for the first time, it automatically:

1. Extracts embedded builtin skills to `workspace/builtin-skills/`
2. Creates a `VERSION` file to track the crate version
3. Makes builtin skills available for use

### Version Management

The builtin skills are synchronized with the crate version:

- On first use: Builtin skills are copied to `workspace/builtin-skills/`
- On subsequent use: Version is checked
  - If version matches: Builtin skills are left unchanged (user modifications preserved)
  - If version differs: Builtin skills directory is recreated with new version

This ensures:
- Users can view and understand builtin skills
- Users can customize builtin skills for their needs
- Updates to the crate automatically update builtin skills

### Builtin Skills Directory Structure

```
workspace/
├── skills/              # User-defined skills (higher priority)
│   └── my-skill/
│       └── SKILL.md
└── builtin-skills/      # Builtin skills (auto-initialized)
    ├── VERSION          # Tracks crate version
    └── tavily-search/   # Example builtin skill
        ├── SKILL.md
        └── scripts/
            └── search.sh
```

## Skill Priority

When skills have the same name:

1. **Workspace skills** (`workspace/skills/`) have the highest priority
2. **Builtin skills** (`workspace/builtin-skills/`) serve as fallback

This allows users to:
- View builtin skills to understand their implementation
- Create custom versions with the same name to override builtin behavior
- Keep custom versions in `workspace/skills/` for persistence

## Skill Format

Skills are markdown files with YAML frontmatter:

```markdown
---
description: Interact with GitHub using the gh CLI
requires:
  bins:
    - gh
  env:
    - GITHUB_TOKEN
metadata:
  nanobot:
    emoji: "🐙"
    always: true
---

# GitHub Skill

Use the `gh` CLI to interact with GitHub...
```

### Frontmatter Fields

- `description`: Skill description
- `requires`: Dependencies (CLI tools, environment variables)
  - `bins`: Required CLI tools
  - `env`: Required environment variables
- `metadata`: Additional metadata
  - `nanobot.emoji`: Emoji for display
  - `nanobot.always`: Always include in context
  - `nanobot.requires`: Platform-specific requirements
  - `nanobot.install`: Installation methods

## API Reference

### `SkillsLoader`

```rust
impl SkillsLoader {
    /// Create a new skills loader
    pub fn new(workspace: PathBuf) -> Self;
    
    /// List all available skills
    pub fn list_skills(&self, filter_unavailable: bool) -> Result<Vec<Skill>>;
    
    /// Load a skill by name
    pub fn load_skill(&self, name: &str) -> Option<String>;
    
    /// Load specific skills for context
    pub fn load_skills_for_context(&self, skill_names: &[String]) -> String;
    
    /// Build XML summary of all skills
    pub fn build_skills_summary(&self) -> Result<String>;
    
    /// Get skills marked as always=true
    pub fn get_always_skills(&self) -> Result<Vec<String>>;
}
```

## Testing

```bash
# Run all tests
cargo test -p nanobot-skills

# Run unit tests only
cargo test -p nanobot-skills --lib

# Run integration tests only
cargo test -p nanobot-skills --test skills
```

## License

MIT
