# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/templates/src/lib.rs` | 修改 | 新增 `all_templates()` 枚举函数 |
| `crates/templates/src/tests.rs` | 修改 | 新增 `all_templates()` 测试 |
| `crates/nanobot/src/utils/mod.rs` | 修改 | 新增 `sync_workspace_templates()` 函数 |
| `crates/nanobot/src/utils/tests.rs` | 新增 | `sync_workspace_templates()` 单元测试 |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | 启动时调用 `sync_workspace_templates` |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | 启动时调用 `sync_workspace_templates` |
| `crates/nanobot/src/commands/onboard/workspace/initializer.rs` | 修改 | 模板创建部分改用 `sync_workspace_templates` |
| `crates/nanobot/src/commands/onboard/workspace/tests.rs` | 修改 | 适配重构后的初始化逻辑 |

## 任务列表

### 1. ✅ nanobot-templates crate 新增 `all_templates()` 枚举函数

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/templates/src/lib.rs`, `crates/templates/src/tests.rs`
- 验收标准: `all_templates()` 返回包含 6 个条目的 `HashMap<&str, &str>`（相对路径 → 内容），包含 `memory/MEMORY.md`
- 风险/注意点: `Dir::files()` 仅返回当前层级文件，需递归遍历 `dirs()` 才能获取 `memory/MEMORY.md`
- 信心评估: 4（`include_dir` API 明确，只需确认递归遍历方式）
- 步骤:
  - [ ] 在 `lib.rs` 中新增 `all_templates()` 公共函数，递归遍历 `TEMPLATES_DIR`，返回 `HashMap<&'static str, &'static str>`（路径 → 内容，路径使用 `/` 分隔符）。实现方式：定义递归辅助函数 `collect_files(dir: &Dir, map: &mut HashMap<...>)`，对 `dir.files()` 收集当前层级文件，再对 `dir.dirs()` 递归调用自身。对每个 `File` 取 `path().to_str()` 和 `contents_utf8()`，过滤掉 `None`
  - [ ] 在 `tests.rs` 中新增测试 `all_templates_returns_all_embedded_files`：断言返回的 HashMap 包含 `AGENTS.md`、`SOUL.md`、`USER.md`、`TOOLS.md`、`HEARTBEAT.md`、`memory/MEMORY.md` 共 6 个 key，且每个 value 非空
  - [ ] 运行 `cargo test -p nanobot-templates` 验证通过

### 2. ✅ utils 模块新增 `sync_workspace_templates()` 函数

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/nanobot/src/utils/mod.rs`, `crates/nanobot/src/utils/tests.rs`
- 验收标准: 函数在空目录中创建所有模板文件并返回文件名列表；在已有文件的目录中跳过已有文件、仅创建缺失文件；自动创建子目录（如 `memory/`）
- 风险/注意点: 无
- 信心评估: 5（逻辑与现有 `create_file_if_not_exists` 完全一致，只是提取为独立函数）
- 步骤:
  - [ ] 在 `utils/mod.rs` 中新增 `pub fn sync_workspace_templates(workspace: &Path) -> Result<Vec<String>>`。遍历 `nanobot_templates::all_templates()` 的 HashMap，对每个 `(relative_path, content)`：用 `workspace.join(relative_path)` 构造目标路径；如果父目录不存在则 `fs::create_dir_all`；如果文件不存在则 `fs::write` 并将 `relative_path` 加入结果列表；如果文件已存在则跳过。返回新创建的文件路径列表
  - [ ] 在 `utils/mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 新建 `utils/tests.rs`，编写以下测试：
    - `sync_creates_all_templates_in_empty_dir`：在 tempdir 中调用，断言返回列表包含 6 个条目，所有文件均存在且内容非空
    - `sync_skips_existing_files`：预先写入自定义内容的 `USER.md`，调用后断言 `USER.md` 内容未变，返回列表不包含 `USER.md`
    - `sync_creates_subdirectories`：在空 tempdir 中调用，断言 `memory/MEMORY.md` 存在
  - [ ] 运行 `cargo test -p nanobot` 验证通过

### 3. ✅ agent 和 gateway 命令启动时调用模板同步

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`, `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: `nanobot agent` 和 `nanobot gateway` 启动时自动补全缺失模板文件；已有文件不被覆盖；有新文件时打印 info 日志
- 风险/注意点: 同步失败不应阻断启动流程，用 `warn!` 记录错误后继续
- 信心评估: 5（插入一行函数调用）
- 步骤:
  - [ ] 在 `AgentCmd::run()` 中，加载配置（`Config::load()`）成功后、初始化 provider 前，调用 `crate::utils::sync_workspace_templates(&config.agents.defaults.workspace)`。如果返回非空列表，用 `info!` 打印同步了哪些文件。如果返回 `Err`，用 `warn!` 记录错误但不 return Err（不阻断启动）
  - [ ] 在 `GatewayCmd::run()` 中，`load_config()` 成功后、`init_provider()` 前，同样调用 `sync_workspace_templates` 并处理结果，逻辑与 agent 相同
  - [ ] 手动验证：删除工作空间中某个模板文件，运行 `cargo run -- agent -m "hi"`，确认文件被自动创建且日志中有同步提示

### 4. ✅ 重构 `WorkspaceInitializer` 复用 `sync_workspace_templates`

- 优先级: P1
- 依赖项: 2
- 涉及文件: `crates/nanobot/src/commands/onboard/workspace/initializer.rs`, `crates/nanobot/src/commands/onboard/workspace/tests.rs`
- 验收标准: `nanobot onboard` 行为不变（创建所有模板文件 + `memory/HISTORY.md` + `skills/` 目录）；`WorkspaceInitializer` 不再维护独立的模板文件列表
- 风险/注意点: `sync_workspace_templates` 不负责 `memory/HISTORY.md`（空文件）和 `skills/` 目录，这两项仍由 `WorkspaceInitializer` 处理
- 信心评估: 5（直接替换函数调用）
- 步骤:
  - [ ] 在 `initializer.rs` 中删除 `create_root_templates()` 方法和 `create_file_if_not_exists()` 方法
  - [ ] 在 `initialize()` 中，将步骤 2（`create_root_templates`）和步骤 3 中 `memory/MEMORY.md` 的创建替换为 `crate::utils::sync_workspace_templates(&self.workspace_path)?`。对返回的新建文件列表，逐个打印 `✓ Created file: {name}`
  - [ ] 保留 `create_memory_dir()` 中 `memory/` 目录创建和 `memory/HISTORY.md`（空文件）创建逻辑，但移除 `memory/MEMORY.md` 的创建（已由 sync 处理）。`HISTORY.md` 的创建需要内联一个简单的 exists-check + write，因为 `create_file_if_not_exists` 已被删除
  - [ ] 保留 `create_skills_dir()` 不变
  - [ ] 更新 `tests.rs`：现有两个测试（`initialize_workspace` 和 `dont_overwrite_existing_files`）的断言不变，确认行为一致
  - [ ] 运行 `cargo test -p nanobot` 验证通过

## 实现建议

- `all_templates()` 返回 `HashMap<&'static str, &'static str>`，在类型层面表达"路径→内容映射"的语义，且强制路径唯一
- `sync_workspace_templates` 中路径拼接使用 `Path::join`，它会自动处理 `/` 分隔符的跨平台问题
- 错误处理参考 `init_cron_service` 的模式：用 `.with_context()` 添加语义化上下文
- agent/gateway 中对 sync 失败的容错处理参考 `OnboardCmd::run()` 中 `initializer.initialize()` 的模式（打印警告，不中断流程）
