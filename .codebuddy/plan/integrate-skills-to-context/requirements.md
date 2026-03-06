# 需求文档：将 Skills 组件集成到 Rust 版上下文构建模块

## 引言

本需求文档描述了参照 Python 版本 `nanobot` 的 skills 集成方式，将 Rust 版本的 `SkillsLoader` 组件集成到 Rust 版本上下文构建模块 (`crates/context`) 的功能需求。

**参考实现：** Python 版本的 `ContextBuilder`（位于 `_nanobot/nanobot/agent/context.py`）已经具备完整的 skills 集成能力，包括：
- 在系统提示中加载标记为 `always=true` 的技能的完整内容
- 为 agent 提供所有可用技能的摘要列表（按需加载机制）
- 依赖检查和可用性标识

**当前状态：** Rust 版本的 `SkillsLoader` 已在 `crates/skills` 中实现完成，但尚未集成到 `crates/context` 的 `ContextBuilder` 中。

**目标：** 参照 Python 版本的实现模式，在 Rust 版本中实现相同的集成效果。

## 需求

### 需求 1：ContextBuilder 集成 SkillsLoader 实例

**用户故事：** 作为开发者，我希望 `ContextBuilder` 持有 `SkillsLoader` 实例，以便能够访问和操作技能数据。

#### 验收标准

1. WHEN `ContextBuilder::new()` 被调用 THEN 系统 SHALL 创建一个 `SkillsLoader` 实例并存储在结构体中。
2. IF `SkillsLoader` 初始化失败 THEN 系统 SHALL 记录警告日志但 SHALL NOT 阻止 `ContextBuilder` 的创建（优雅降级）。
3. WHEN `ContextBuilder` 结构体定义完成 THEN 系统 SHALL 包含 `skills: SkillsLoader` 字段。
4. WHEN 需要直接访问技能加载器 THEN 系统 SHALL 提供 `skills()` 方法返回 `SkillsLoader` 的不可变引用。

---

### 需求 2：系统提示中集成 Active Skills 内容

**用户故事：** 作为 AI 助手，我希望系统提示中自动包含标记为 `always=true` 的技能完整内容，以便我能够直接使用这些核心能力而无需手动加载。

#### 验收标准

1. WHEN `build_system_prompt()` 方法被调用 THEN 系统 SHALL 调用 `skills.get_always_skills()` 获取所有需要常驻加载的技能名称列表。
2. IF `always` 技能列表不为空 THEN 系统 SHALL 调用 `skills.load_skills_for_context()` 加载这些技能的完整内容（去除 frontmatter）。
3. WHEN active skills 内容生成完成 THEN 系统 SHALL 在系统提示中添加 `# Active Skills` 章节，内容为格式化后的技能内容。
4. IF active skills 内容为空 THEN 系统 SHALL 跳过该章节的添加。
5. WHEN 拼接系统提示各部分 THEN 系统 SHALL 使用 `\n\n---\n\n` 作为分隔符连接各部分内容。

---

### 需求 3：系统提示中集成 Skills 摘要信息

**用户故事：** 作为 AI 助手，我希望系统提示中显示所有可用技能的摘要信息，以便我了解有哪些技能可以按需加载使用。

#### 验收标准

1. WHEN `build_system_prompt()` 方法被调用 THEN 系统 SHALL 调用 `skills.build_skills_summary()` 获取所有技能的 XML 格式摘要。
2. IF 技能摘要不为空 THEN 系统 SHALL 在系统提示中添加 `# Skills` 章节，包含指导说明和 XML 格式的技能列表。
3. WHEN 构建 Skills 章节 THEN 系统 SHALL 包含以下指导说明：
   - "The following skills extend your capabilities. To use a skill, read its SKILL.md file using the read_file tool."
   - "Skills with available=\"false\" need dependencies installed first - you can try installing them with apt/brew."
4. IF 技能摘要为空 THEN 系统 SHALL 跳过该章节的添加。
5. WHEN Skills 章节添加完成 THEN 该章节 SHALL 位于 Active Skills 章节之后。

---

### 需求 4：系统提示组装顺序

**用户故事：** 作为开发者，我希望系统提示的各部分按照正确的顺序组装，以便 AI 助手能够获得结构化的上下文信息。

#### 验收标准

1. WHEN `build_system_prompt()` 方法被调用 THEN 系统 SHALL 按照以下顺序组装系统提示：
   - Memory Context（记忆上下文，如有）
   - Active Skills（常驻技能，如有）
   - Skills Summary（技能摘要，如有）
2. IF 任意章节内容为空或不可用 THEN 系统 SHALL 跳过该章节并继续处理后续章节。
3. WHEN 所有章节处理完成 THEN 系统 SHALL 使用 `\n\n---\n\n` 分隔符连接所有非空章节。

---

### 需求 5：依赖关系管理

**用户故事：** 作为开发者，我希望 `context` crate 正确声明对 `skills` crate 的依赖，以便编译和运行时能够正确链接。

#### 验收标准

1. WHEN `crates/context/Cargo.toml` 被更新 THEN 系统 SHALL 添加 `nanobot-skills` 作为依赖项。
2. WHEN `builder.rs` 被更新 THEN 系统 SHALL 添加 `use nanobot_skills::SkillsLoader;` 导入语句。
3. IF 需要使用 skills 模块的数据类型 THEN 系统 SHALL 导出必要的公共类型（如 `Skill`、`SkillSource` 等）。

---

### 需求 6：错误处理与日志

**用户故事：** 作为运维人员，我希望系统能够正确处理和记录 skills 相关的错误，以便问题排查和监控。

#### 验收标准

1. WHEN `SkillsLoader` 初始化过程中的内置技能同步失败 THEN 系统 SHALL 记录警告级别日志，包含具体错误信息。
2. WHEN `get_always_skills()` 或 `build_skills_summary()` 调用失败 THEN 系统 SHALL 记录错误日志，但 SHALL NOT 导致系统提示构建失败。
3. IF skills 功能部分失败 THEN 系统 SHALL 继续构建系统提示的其他部分，实现优雅降级。
4. WHEN skills 相关操作成功完成 THEN 系统 SHALL 记录调试级别日志，包含加载的技能数量等信息。

---

### 需求 7：保持现有功能兼容性

**用户故事：** 作为开发者，我希望集成 skills 功能后，现有的 memory、bootstrap 等功能继续正常工作，以便系统保持稳定。

#### 验收标准

1. WHEN `ContextBuilder::new()` 被调用 THEN 系统 SHALL 继续正确初始化 `MemoryStore`。
2. WHEN `build_system_prompt()` 被调用 THEN 系统 SHALL 继续正确加载和包含 memory context。
3. WHEN Bootstrap 文件存在 THEN 系统 SHALL 继续正确加载并包含在系统提示中。
4. WHEN `build_messages()` 和其他现有方法被调用 THEN 系统 SHALL 保持与原有行为一致的输出。
5. WHEN 现有测试运行 THEN 所有测试 SHALL 继续通过。
