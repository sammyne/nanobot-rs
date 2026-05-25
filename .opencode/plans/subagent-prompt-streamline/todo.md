# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/subagent/Cargo.toml` | 修改 | 添加 `nanobot-skills` 依赖 |
| `crates/subagent/src/manager.rs` | 修改 | 精简 `build_subagent_prompt()`，集成 skills 摘要 |

## 任务列表

### ✅ 1. 添加 nanobot-skills 依赖

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/subagent/Cargo.toml`
- 验收标准: `cargo check -p nanobot-subagent` 通过
- 风险/注意点: 使用 `.workspace = true` 引用
- 信心评估: 5
- 步骤:
  - [ ] 在 `[dependencies]` 中添加 `nanobot-skills.workspace = true`

### ✅ 2. 精简 build_subagent_prompt 并集成 skills

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/subagent/src/manager.rs`
- 验收标准: 系统提示不含 "What You Can Do/Cannot Do"，不含 task 描述，包含 skills 摘要（如有）
- 风险/注意点: `SkillsLoader::build_skills_summary()` 返回 `Result<String>`，失败时 warn 并跳过
- 信心评估: 5
- 步骤:
  - [ ] 修改 `build_subagent_prompt` 签名：移除 `task_description` 参数，改为 `fn build_subagent_prompt(&self) -> String`
  - [ ] 更新 `run_subagent` 中的调用：`self.build_subagent_prompt()` 不再传 task
  - [ ] 重写提示文本：
    - 保留 `## Role`（移除 task 描述，改为通用 "You are a subagent spawned by the main agent to complete a specific task."）
    - 保留 `## Current Time`（chrono 逻辑不变）
    - 规则压缩为一句话，不再用 `## Rules` 标题
    - 移除 `## What You Can Do` 和 `## What You Cannot Do`
    - 保留 `## Workspace`
  - [ ] 在提示末尾条件追加 skills 摘要：`SkillsLoader::new(self.workspace.clone()).build_skills_summary()`，非空时追加 `## Skills\n\nRead SKILL.md with read_file to use a skill.\n\n{summary}`

### ✅ 3. 全量验证

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: 全部
- 验收标准: `cargo +nightly fmt`、`cargo clippy -- -D warnings -D clippy::uninlined_format_args`、`cargo test` 全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`

## 实现建议

- `SkillsLoader::new(workspace)` 接受 `PathBuf`，`self.workspace` 已是 `PathBuf`，直接 clone 传入
- `build_skills_summary()` 返回 `Result<String>`，用 `unwrap_or_default()` 或 `match` + warn 处理错误
- 提示文本用 `let mut parts = vec![...]` + `parts.push(...)` + `parts.join("\n\n")` 组装，与 Python 版结构对齐
