# 需求

## 目标与背景

子代理的系统提示（`build_subagent_prompt()`）存在以下问题：

1. **冗余章节**："What You Can Do" / "What You Cannot Do" 与 tool 定义重复，LLM 已从 tool schema 知道自己能做什么
2. **无 skills 感知**：子代理无法发现工作空间中的 skills（SKILL.md），而主代理通过 `SkillsLoader::build_skills_summary()` 已具备此能力
3. **task 描述双重注入**：`task_description` 同时出现在系统提示的 Role 段和第一条 user 消息中，浪费 token
4. **规则冗长**：4 条规则可压缩为 2 句话

对齐 Python 版 PR #1347，精简子代理提示并集成 skills 发现。

## 方案比较（强制）

### 方案 1: 仅精简提示文本（最小可行版）

- 思路: 移除 "What You Can Do/Cannot Do"，压缩规则，移除系统提示中的 task 描述。不加 skills
- 优点: 零新依赖，改动 < 10 行
- 缺点: 子代理仍无法发现 skills
- 工作量估算: S

### 方案 2: 精简提示 + 集成 SkillsLoader（理想架构）

- 思路: 在方案 1 基础上，添加 `nanobot-skills` 依赖，调用 `SkillsLoader::build_skills_summary()` 将 skills 摘要注入系统提示
- 优点: 子代理能动态发现 skills，与 Python 版行为对齐
- 缺点: 新增一个 crate 依赖
- 工作量估算: S

### 推荐

方案 2。`nanobot-skills` 是叶子 crate（无传递依赖），引入成本极低。子代理能发现 skills 对实际使用有明确价值（如子代理需要读取 SKILL.md 来完成任务）。

## 功能需求列表

### 核心功能

1. **移除 "What You Can Do" / "What You Cannot Do" 章节**
2. **移除系统提示中的 task 描述**：`build_subagent_prompt()` 不再接受 `task_description` 参数，task 仅通过第一条 user 消息传递
3. **压缩规则**：4 条规则压缩为 2 句话："Stay focused on the assigned task. Your final response will be reported back to the main agent."
4. **集成 skills 摘要**：调用 `SkillsLoader::build_skills_summary()`，非空时追加 `## Skills` 段

### 扩展功能

- 无

## 非功能需求

- **兼容性**：时间格式化保持现有 chrono 逻辑不变（不引入 `nanobot-context` 依赖）
- **可维护性**：提示文本保持在单个 `format!` 宏中，skills 段条件追加
- **测试要求**：无需新增测试（纯文本变更，现有集成测试已覆盖 subagent 流程）

## 边界与不做事项

- 不引入 `nanobot-context` 依赖（时间格式化仅 2 行 chrono，不值得）
- 不修改 `run_subagent()` 的执行逻辑
- 不修改 `announce_result()` 的通知格式

## 假设与约束

- **技术假设**：`SkillsLoader::new(workspace)` 会自动初始化内置 skills（已在主代理启动时完成），子代理复用同一 workspace 目录
- **资源约束**：`nanobot-skills` 已在 workspace dependencies 中声明

## 待确认事项

- 无
