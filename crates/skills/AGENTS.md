# skills crate

技能发现、加载和管理（工作空间目录 + 内置技能）。

## 架构

```
┌──────────────┐
│ SkillsLoader │
└──────┬───────┘
       │
       ├── scan ──► ┌────────────────────────────────┐
       │            │ skills/          (工作空间,高优先级) │
       │            │   ├── skill-a/SKILL.md         │
       │            │   └── skill-b/SKILL.md         │
       │            └────────────────────────────────┘
       │
       ├── scan ──► ┌────────────────────────────────┐
       │            │ builtin-skills/  (编译时嵌入)    │
       │            │   └── tavily-search/SKILL.md   │
       │            └────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────┐
│ list_skills()                    │
│  1. WalkDir(depth=1) 扫描       │
│  2. seen_names 去重              │
│  3. load_skill_file() 解析 YAML │
│  4. check_requirements()        │
│     ├── bins: which 子进程       │
│     └── env: env::var           │
└──────────────────────────────────┘
```

## 关键类型

- **`SkillsLoader`** -- 从 `skills/` 和 `builtin-skills/` 目录加载技能
  - `new(workspace)` -- 创建加载器，确保内置技能已解压
  - `list_skills(filter_unavailable) -> Vec<Skill>` -- 列出所有技能（工作空间优先于内置）
  - `load_skill(name) -> Option<String>` -- 加载指定技能的 SKILL.md 内容
  - `load_skills_for_context(skill_names) -> String` -- 格式化技能内容供 LLM 上下文使用（去除 frontmatter）
  - `build_skills_summary() -> String` -- 所有技能的 XML 摘要供系统提示使用
  - `get_always_skills() -> Vec<String>` -- 返回 always=true 的技能名称列表
  - `get_skill_metadata(name) -> Option<Skill>` -- 获取技能元数据
- **`Skill`** -- `name`, `path`, `source`, `meta`；`description()`, `is_always()`, `effective_requires()`
- **`SkillMeta`** -- `description`, `always`, `requires`, `metadata`
- **`SkillMetadata`** (enum) -- `Nanobot(NanobotMeta)` | `OpenClaw(OpenClawMeta)`
- **`SkillSource`** (enum) -- `Workspace` | `Builtin`
- **`Requires`** -- `bins: Vec<String>`, `env: Vec<String>`
- **`InstallInfo`** -- `id`, `kind`, `formula`, `package`, `bins`, `label`

## 内部依赖

无（叶子 crate）
