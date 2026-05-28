# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/tools.rs` | 修改 | `ExecToolConfig` 新增 `disabled` 字段 |
| `crates/config/src/schema/tests.rs` | 修改 | 新增 `disabled` 字段测试 |
| `crates/tools/src/registry.rs` | 修改 | 根据 `disabled` 跳过 ExecTool 注册 |
| `crates/tools/tests/registry.rs` | 修改 | 新增 `disabled=true` 时不注册 shell 工具的测试 |

## 任务列表

### 1. ✅ ExecToolConfig 新增 disabled 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/tools.rs`、`crates/config/src/schema/tests.rs`
- 验收标准: `cargo test -p nanobot-config` 通过；`disabled` 默认 `false`，camelCase 序列化为 `"disabled"`
- 风险/注意点: `ExecToolConfig` 已有 `#[serde(default)]` 在 struct 级别，新增字段的默认值 `false` 与 `bool` 零值一致，无需改 `Default` impl
- 信心评估: 5
- 步骤:
  - [ ] 在 `ExecToolConfig` 中添加 `pub disabled: bool` 字段
  - [ ] 在 `Default` impl 中添加 `disabled: false`
  - [ ] 在 `tests.rs` 中添加测试：默认值为 false；`{"disabled": true}` 反序列化正确；旧配置（不含 disabled）向后兼容
  - [ ] 运行 `cargo test -p nanobot-config` 验证通过

### 2. ✅ ToolRegistry 根据 disabled 跳过 ExecTool 注册

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/tools/src/registry.rs`、`crates/tools/tests/registry.rs`
- 验收标准: `cargo test -p nanobot-tools` 通过；`disabled=true` 时 ToolRegistry 不包含 `shell` 工具
- 风险/注意点: 只影响 ExecTool 注册，文件系统工具不受影响
- 信心评估: 5
- 步骤:
  - [ ] 在 `ToolRegistry::new()` 中用 `if !exec_config.disabled { ... }` 包裹 ExecTool 的创建和注册逻辑
  - [ ] 在 `crates/tools/tests/registry.rs` 中添加测试：`disabled=true` 时 registry 不包含 `shell`，但仍包含文件系统工具
  - [ ] 运行 `cargo test -p nanobot-tools` 验证通过

## 实现建议

- `ExecToolConfig` 已使用 `#[serde(rename_all = "camelCase", default)]`，新增 `disabled: bool` 字段零改动即可获得正确的默认值和序列化行为
- `subagent/src/manager.rs` 中使用 `ExecToolConfig::default()` 创建子代理的工具注册表，`disabled` 默认 false，行为不变
