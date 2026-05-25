# 需求

## 目标与背景

HKUDS/nanobot 有一个内置的 memory 技能（`nanobot/skills/memory/SKILL.md`，`always: true`），始终加载到系统提示中，教 LLM 如何使用记忆系统：文件结构、搜索历史的方法、哪些文件不可手动编辑。Python 版的 core identity（`identity.md`）中不包含 `## Memory` 段落，记忆说明完全由 skill 承载。

nanobot-rs 当前没有这个技能。记忆相关的提示写在 `ContextBuilder::build_core_identity()` 中：

```
## Workspace
- Long-term memory: {workspace}/memory/MEMORY.md
- History log: {workspace}/memory/HISTORY.md (grep-searchable). Each entry starts with [YYYY-MM-DD HH:MM].

## Memory
- Remember important facts: write to {workspace}/memory/MEMORY.md
- Recall past events: grep {workspace}/memory/HISTORY.md
```

问题：
1. `## Memory` 段落与 Python 版架构不一致（Python 版由 skill 承载，core identity 不含此段落）
2. 缺少 HISTORY.md 的具体行格式说明（`[YYYY-MM-DD HH:MM] ROLE: content`）
3. 缺少 grep 搜索的具体示例（按关键词、按日期、按日期范围、计数、上下文行）
4. 缺少不可编辑文件的警告（SOUL.md、USER.md、MEMORY.md 由 consolidation 管理）

## 方案比较（强制）

### 方案 1: 内置技能 + 精简 core_identity（推荐，与 Python 版对齐）

- 思路: 新增 `crates/skills/builtin/memory/SKILL.md`（`always: true`），同时删除 `build_core_identity()` 中的 `## Memory` 段落，`## Workspace` 部分的文件路径列表补充简短注释
- 优点:
  - 与 HKUDS/nanobot 架构完全对齐
  - 记忆说明集中在 skill 中，无冗余
  - 用户可通过工作空间 `skills/memory/SKILL.md` 覆盖定制
- 缺点:
  - 如果技能加载失败，系统提示中不再有记忆使用说明（仅剩 Workspace 部分的文件路径）
- 工作量估算: S

### 方案 2: 仅新增技能，保留 core_identity 不变（最小可行版）

- 思路: 新增 `crates/skills/builtin/memory/SKILL.md`（`always: true`），不修改 `build_core_identity()`
- 优点:
  - 改动最小，只新增一个文件
  - 技能加载失败时仍有基本记忆信息
- 缺点:
  - 系统提示中有两处记忆信息，存在冗余
  - 与 Python 版架构不一致
- 工作量估算: S

### 推荐

方案 1。与 Python 版架构对齐，消除冗余。技能加载失败是极端边界情况（内置技能编译时嵌入，不依赖外部资源），不值得为此保留冗余信息。

## 功能需求列表

### 核心功能

1. 新增 `crates/skills/builtin/memory/SKILL.md`，frontmatter 设置 `always: true`，内容包括：
   - 文件结构说明（SOUL.md、USER.md、MEMORY.md、HISTORY.md 各自用途）
   - HISTORY.md 搜索方法（含具体 exec grep 示例）
   - 不可编辑文件警告
2. 修改 `build_core_identity()`：
   - 删除 `## Memory` 段落
   - `## Workspace` 中的文件路径补充注释（如 `automatically managed by consolidation — do not edit directly`）

### 扩展功能

- 无

## 非功能需求

- **兼容性**：frontmatter 格式需与现有 `SkillMeta` 解析兼容
- **可维护性**：`## Workspace` 中的文件路径注释与 SKILL.md 内容不矛盾

## 边界与不做事项

- 不实现内置 grep 工具（LLM 通过 exec 工具调用系统 grep）
- 不修改 `build_system_prompt()` 的组装逻辑

## 假设与约束

- **技术假设**：`include_dir!` 会自动包含新增的 `builtin/memory/` 目录，无需额外配置
- **环境约束**：目标系统有 `grep` 命令可用

## 待确认事项

无
