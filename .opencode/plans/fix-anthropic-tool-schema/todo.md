# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/anthropic/mod.rs` | 修改 | 移除 `sanitize_input_schema` 函数及调用 |
| `crates/provider/src/anthropic/tests.rs` | 修改 | 移除 sanitize 相关测试 |
| `crates/cron/src/tool/mod.rs` | 修改 | 定义 `CronArgsSchema`，移除 `CronArgs` 的 `JsonSchema` derive，改 `schema_for!` 目标 |
| `crates/cron/src/tool/tests.rs` | 修改 | 移除 `$ref` 测试，新增 enum↔struct 互操作测试 |
| `crates/heartbeat/src/service.rs` | 修改 | 改 `Action` 为 internally tagged，定义 `ActionSchema`，改 `schema_for!` 目标，更新重试提示 |
| `crates/heartbeat/src/service/tests.rs` | 新增 | 新增 enum↔struct 互操作测试 |

## 任务列表

### ✅ 1. 移除 `sanitize_input_schema`

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: `bind_tools` 直接将 `td.parameters` 赋给 `input_schema`，无任何 schema 修改逻辑；`debug!` 日志保留，输出每个工具的完整 `input_schema` JSON。
- 风险/注意点: 移除后，如果有其他工具的 schema 不合规（如 MCP 工具），Anthropic API 会直接报错。这是预期行为，日志可帮助定位。
- 步骤:
  - [ ] 在 `bind_tools` 方法中，将 `sanitize_input_schema(td.parameters)` 改为直接使用 `td.parameters`
  - [ ] 删除 `sanitize_input_schema` 函数（第 319-336 行）
  - [ ] 确认 `debug!` 日志保留，输出工具名和 `input_schema` JSON

### ✅ 2. 移除 anthropic provider 中 sanitize 相关测试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/tests.rs`
- 验收标准: 移除 `bind_tools_adds_missing_type_field` 和 `bind_tools_strips_top_level_combinators` 两个测试；`bind_tools_converts_to_anthropic_format` 测试保持通过。
- 风险/注意点: 无
- 步骤:
  - [ ] 删除 `bind_tools_adds_missing_type_field` 测试（第 133-152 行）
  - [ ] 删除 `bind_tools_strips_top_level_combinators` 测试（第 154-175 行）
  - [ ] 运行 `cargo test -p nanobot-provider` 确认剩余测试通过

### ✅ 3. 定义 `CronArgsSchema` mirror struct

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/cron/src/tool/mod.rs`
- 验收标准: `CronArgsSchema` 定义在 `CronArgs` 旁边；`CRON_PARAMETERS` 使用 `schema_for!(CronArgsSchema)`；`CronArgs` 不再 derive `JsonSchema`；`CronScheduleArgs` 保留 `#[derive(JsonSchema)]` 和 `#[schemars(inline)]` 不变。
- 风险/注意点: `CronArgsSchema` 的字段名、serde rename 规则必须与 `CronArgs` 的 serde 序列化输出一致，否则互操作测试会失败。
- 步骤:
  - [ ] 在 `CronArgs` 定义下方添加 `CronArgsSchema` struct，字段包括 `action: String`（带 `#[schemars(extend("enum" = ["add", "list", "remove"]))]`）、`message: String`（带 `#[serde(default, skip_serializing_if = "String::is_empty")]`）、`schedule: Option<CronScheduleArgs>`（带 `#[serde(skip_serializing_if = "Option::is_none")]`）、`job_id: String`（带 `#[serde(default, skip_serializing_if = "String::is_empty")]`），每个字段添加 doc comment 描述归属动作和是否必填
  - [ ] 从 `CronArgs` 的 derive 列表中移除 `JsonSchema`
  - [ ] 将 `CRON_PARAMETERS` 从 `schema_for!(CronArgs)` 改为 `schema_for!(CronArgsSchema)`
  - [ ] 确认 `CronScheduleArgs` 的 `#[derive(JsonSchema)]` 和 `#[schemars(inline)]` 保持不变

### ✅ 4. 更新 cron 工具测试

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/cron/src/tool/tests.rs`
- 验收标准: 移除 `cron_args_schema_has_no_ref` 测试；新增互操作测试覆盖 `CronArgs` 的 Add、List、Remove 三个变体，验证 enum→JSON→struct 和 struct→JSON→enum 双向反序列化成功。
- 风险/注意点: Add 变体包含嵌套的 `CronScheduleArgs`，互操作测试需覆盖至少一种 schedule 类型（如 `Every`）。
- 步骤:
  - [ ] 删除 `cron_args_schema_has_no_ref` 测试
  - [ ] 新增 `cron_args_add_interop` 测试：构造 `CronArgs::Add` 实例 → `serde_json::to_value` → `serde_json::from_value::<CronArgsSchema>` 成功；反向构造 `CronArgsSchema` 实例 → `serde_json::to_value` → `serde_json::from_value::<CronArgs>` 成功
  - [ ] 新增 `cron_args_list_interop` 测试：同上，覆盖 `CronArgs::List` 变体
  - [ ] 新增 `cron_args_remove_interop` 测试：同上，覆盖 `CronArgs::Remove` 变体
  - [ ] 运行 `cargo test -p nanobot-cron` 确认所有测试通过

### ✅ 5. 修复 heartbeat `Action` enum 并定义 `ActionSchema`

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/heartbeat/src/service.rs`
- 验收标准: `Action` 改为 internally tagged（`#[serde(tag = "action")]`）；`ActionSchema` 定义在 `Action` 旁边；`HEARTBEAT_TOOL` 使用 `schema_for!(ActionSchema).to_value()`；`Action` 不再 derive `JsonSchema`；`decide()` 中的重试错误提示信息匹配新的 JSON 格式。
- 风险/注意点: `Action` 从 externally tagged 改为 internally tagged 会改变 JSON 格式（`"skip"` → `{"action":"skip"}`），必须同步更新重试提示信息。
- 步骤:
  - [ ] 给 `Action` enum 添加 `#[serde(tag = "action")]`，将 `#[serde(rename_all = "camelCase")]` 改为 `#[serde(tag = "action", rename_all = "camelCase")]`
  - [ ] 从 `Action` 的 derive 列表中移除 `JsonSchema`，移除 `use schemars::JsonSchema` 导入
  - [ ] 在 `Action` 定义下方添加 `ActionSchema` struct，derive `JsonSchema, Serialize, Deserialize`，字段包括 `action: String`（带 `#[schemars(extend("enum" = ["skip", "run"]))]`）和 `tasks: String`（带 `#[serde(default, skip_serializing_if = "String::is_empty")]`），添加 `#[serde(rename_all = "camelCase")]`，每个字段添加 doc comment
  - [ ] 将 `HEARTBEAT_TOOL` 中的 `schemars::schema_for!(Action).to_value()` 改为 `schemars::schema_for!(ActionSchema).to_value()`
  - [ ] 更新 `decide()` 方法中的重试错误提示（约第 240-244 行），将 `"skip"` 改为 `{"action":"skip"}`，将 `{"run":{"tasks":"..."}}` 改为 `{"action":"run","tasks":"..."}`

### ✅ 6. 新增 heartbeat 互操作测试

- 优先级: P0
- 依赖项: 5
- 涉及文件: `crates/heartbeat/src/service.rs`（在文件末尾添加 `#[cfg(test)] mod tests;`）和 `crates/heartbeat/src/service/tests.rs`（新增）
- 验收标准: 互操作测试覆盖 `Action::Skip` 和 `Action::Run` 两个变体，验证 enum↔struct 双向反序列化成功。
- 风险/注意点: `Action` 和 `ActionSchema` 是 `pub(crate)` 或 private，测试需在同模块内。当前 `service.rs` 没有测试模块，需要新建。如果 `service.rs` 是单文件模块（非目录模块），测试直接写在文件末尾的 `#[cfg(test)] mod tests { ... }` 中。
- 步骤:
  - [ ] 在 `service.rs` 末尾添加 `#[cfg(test)]` 测试模块
  - [ ] 新增 `action_skip_interop` 测试：`Action::Skip` → JSON → `ActionSchema` 成功；反向 `ActionSchema { action: "skip", tasks: "" }` → JSON → `Action` 成功
  - [ ] 新增 `action_run_interop` 测试：`Action::Run { tasks: "..." }` → JSON → `ActionSchema` 成功；反向 `ActionSchema { action: "run", tasks: "..." }` → JSON → `Action` 成功
  - [ ] 运行 `cargo test -p nanobot-heartbeat` 确认所有测试通过

### ✅ 7. 全量验证

- 优先级: P1
- 依赖项: 2, 4, 6
- 涉及文件: 无（验证步骤）
- 验收标准: 全部编译通过，全部测试通过，clippy 无警告。
- 风险/注意点: 无
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 运行 `cargo doc --no-deps`

## 实现建议

- `CronArgsSchema` 和 `ActionSchema` 放在对应 enum 定义的正下方，用注释标明两者需同步维护。
- 互操作测试使用 `serde_json::to_value` + `serde_json::from_value` 做 enum↔struct 双向验证，不依赖字符串比较。
- heartbeat 的 `Action` 改为 internally tagged 后，`Action::Skip` 的 JSON 从 `"skip"` 变为 `{"action":"skip"}`，这是 object 类型，对 Anthropic 更友好。
