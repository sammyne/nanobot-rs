# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/skills/builtin/memory/SKILL.md` | 新增 | 内置 memory 技能，`always: true`，提供记忆系统使用说明 |
| `crates/context/src/builder/mod.rs` | 修改 | 删除 `## Memory` 段落，更新 `## Workspace` 文件路径注释 |
| `crates/skills/src/builtin/tests.rs` | 修改 | 新增 memory 技能提取验证 |

## 任务列表

### ✅ 1. 新增 memory 内置技能文件

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/skills/builtin/memory/SKILL.md`
- 验收标准: 文件存在，frontmatter 可被 `SkillMeta` 正确解析，`always: true` 生效
- 风险/注意点: frontmatter 格式必须与 `crates/skills/src/parser/mod.rs` 的 YAML 解析兼容
- 信心评估: 5（有 `tavily-search/SKILL.md` 作为参考实现）
- 步骤:
  - [ ] 创建 `crates/skills/builtin/memory/SKILL.md`，frontmatter 包含 `name: memory`、`description`、`always: true`
  - [ ] 编写 `# Memory` 正文：`## Structure` 列出 SOUL.md、USER.md、MEMORY.md、HISTORY.md 各自用途和管理方式
  - [ ] 编写 `## Search Past Events`：说明 HISTORY.md 行格式 `[YYYY-MM-DD HH:MM] ROLE: content`，给出 exec grep 示例（按关键词、按日期、按日期范围、计数、上下文行）
  - [ ] 编写 `## Important`：警告不可手动编辑 SOUL.md、USER.md、MEMORY.md
  - [ ] 运行 `cargo test -p nanobot-skills` 验证 frontmatter 解析和技能加载

### ✅ 2. 精简 build_core_identity() 中的记忆段落

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/context/src/builder/mod.rs`
- 验收标准: `build_core_identity()` 输出不含 `## Memory` 段落；`## Workspace` 中文件路径带简短注释
- 风险/注意点: 确认无测试断言 `## Memory` 内容（已确认：现有测试不测试 `build_core_identity()` 输出）
- 信心评估: 5
- 步骤:
  - [ ] 删除 `build_core_identity()` 中 `## Memory` 段落（当前第 88-90 行的 3 行内容）
  - [ ] 更新 `## Workspace` 中 MEMORY.md 路径注释：`(automatically managed by consolidation — do not edit directly)`
  - [ ] 更新 `## Workspace` 中 HISTORY.md 路径注释：`(append-only log, use exec with grep to search). Each entry starts with [YYYY-MM-DD HH:MM].`
  - [ ] 运行 `cargo test -p nanobot-context` 验证无测试失败

### ✅ 3. 补充 builtin tests 中的 memory 技能验证

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/skills/src/builtin/tests.rs`
- 验收标准: `extracts_builtin_skills_from_embedded_resources` 测试同时验证 `memory/SKILL.md` 存在
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `extracts_builtin_skills_from_embedded_resources` 测试中新增 `assert!(builtin_dir.join("memory/SKILL.md").exists())`
  - [ ] 运行 `cargo test -p nanobot-skills` 验证通过

## 实现建议

- SKILL.md 的 frontmatter 参考 `crates/skills/builtin/tavily-search/SKILL.md` 的格式，使用顶层 `always: true`（不需要嵌套在 `metadata.nanobot` 下，`SkillMeta` 支持顶层 `always` 字段）
- `## Workspace` 中的注释风格参考 Python 版 `identity.md`：简短括号注释，不展开说明
