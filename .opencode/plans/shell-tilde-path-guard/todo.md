# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/tools/src/shell/utils/mod.rs` | 修改 | 新增 `extract_tilde_paths()`，`extract_absolute_paths()` 合并结果 |
| `crates/tools/src/shell/utils/tests.rs` | 修改 | 新增 tilde 路径提取测试 |
| `crates/tools/src/shell/mod.rs` | 修改 | `validate_paths_in_workspace()` 对 `~` 路径 expanduser 后检查 |
| `crates/tools/src/shell/tests.rs` | 修改 | 新增 tilde 路径拦截集成测试 |

## 任务列表

### 1. 新增 tilde 路径提取

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/tools/src/shell/utils/mod.rs`, `crates/tools/src/shell/utils/tests.rs`
- 验收标准: `extract_tilde_paths("cat ~/.nanobot/config.json")` 返回 `["~/.nanobot/config.json"]`；`extract_absolute_paths()` 包含 tilde 路径
- 风险/注意点: 正则需要区分独立的 `~` 和单词中间的 `~`（如 `file~backup` 不应匹配）
- 信心评估: 5
- 步骤:
  - [ ] 新增 `fn extract_tilde_paths(cmd: &str) -> Vec<String>`，正则匹配 `(?:^|[\s|>])(~[/\\][^\s"'>]*)` 和独立的 `~`
  - [ ] `extract_absolute_paths()` 中追加 `paths.extend(extract_tilde_paths(cmd))`
  - [ ] 在 `tests.rs` 中新增测试：`cat ~/.nanobot/config.json` → `["~/.nanobot/config.json"]`；`echo hello` → 空；`cat ~/a ~/b` → 两个路径；`file~backup` → 空（不匹配）
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

### 2. validate_paths_in_workspace 支持 tilde 展开

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/tools/src/shell/mod.rs`, `crates/tools/src/shell/tests.rs`
- 验收标准: `restrict_to_workspace = true` 时，`cat ~/.nanobot/config.json` 被拦截；工作空间内的 tilde 路径（如果 workspace 恰好在 home 下）允许通过
- 风险/注意点: 需要用 `dirs::home_dir()` 或 `std::env::var("HOME")` 展开 `~`；展开后路径变为绝对路径，现有的 `is_absolute()` + `starts_with(workspace)` 检查自然生效
- 信心评估: 5
- 步骤:
  - [ ] 在 `validate_paths_in_workspace()` 中，对 `extract_absolute_paths()` 返回的每个路径，如果以 `~` 开头，使用 `nanobot_config::expand_tilde()` 展开为绝对路径（config crate 已有此函数）
  - [ ] 展开后的路径走现有的 `is_absolute()` + `canonicalize()` + `starts_with(workspace)` 检查
  - [ ] 在 `crates/tools/src/shell/tests.rs` 中新增测试：构造 ExecTool 并验证 `~/.nanobot/config.json` 被 security_guard 拦截
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 验证无警告
