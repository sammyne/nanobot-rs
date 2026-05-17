# 需求

## 目标与背景

Anthropic Messages API 的 `input_schema` 要求顶层必须是 `type: "object"` 的 JSON Schema，不支持顶层 `oneOf`/`allOf`/`anyOf`。

当前项目中有两个工具（`cron`、`heartbeat`）使用 Rust enum + `schemars::schema_for!` 生成 schema，产出的 JSON Schema 顶层为 `oneOf`，不满足 Anthropic 要求。现有的 `sanitize_input_schema` 函数通过暴力移除 `oneOf` 并补 `type: "object"` 来"修复"，但结果是一个空壳 schema（`{"type":"object"}`），模型完全丧失参数结构信息。

本次修复目标：
1. 移除 `sanitize_input_schema`，让不合规 schema 直接暴露为 Anthropic API 错误，配合日志便于排查。
2. 从源头修复 `cron` 和 `heartbeat` 两个工具的 schema，使其生成 Anthropic 兼容的扁平 object schema。

## 方案比较

### 方案 1: mirror struct 生成 schema + 互操作测试

- 思路: 为每个 enum 定义一个同构的 mirror struct（如 `CronArgsSchema`），struct 的字段是 enum 所有变体字段的并集（非公共字段用 `Option`），tag 字段为普通字段。struct 上 `#[derive(JsonSchema, Serialize, Deserialize)]`，schemars 自动生成扁平 object schema。`schema_for!` 改为指向 mirror struct。编写互操作测试验证 enum 和 struct 的序列化/反序列化可互换，防止两者漂移。
- 优点: schema 由 schemars 自动生成，无手写 JSON；互操作测试自动捕获 enum/struct 字段不一致；enum 保留类型安全和穷举检查；上层调用链只改 `schema_for!` 的类型参数。
- 缺点: 每个 enum 多一个 mirror struct；新增变体时需同步更新 struct（但测试会报错提醒）。

### 方案 2: 手动实现 `JsonSchema` trait

- 思路: 移除 enum 上的 `#[derive(JsonSchema)]`，手动实现 `JsonSchema` trait，在 `json_schema()` 中用 `json_schema!` 宏返回扁平 object schema。
- 优点: 无额外类型，`schema_for!` 调用不变。
- 缺点: schema 内容手写，与 enum 字段无编译期关联，新增变体时容易遗漏更新且无自动检测手段。

### 方案 3: 将 enum 改为 struct + 手动 dispatch

- 思路: 将 `CronArgs` 从 enum 改为 struct，用 `#[derive(JsonSchema)]` 自动生成合规 schema。
- 优点: schema 和类型定义统一。
- 缺点: 丧失 enum 的类型安全和穷举检查；需要大量重写 execute 逻辑。

### 推荐

方案 1。schema 自动生成，互操作测试兜底，enum 类型安全不受影响。

## Mirror struct 结构定义

### CronArgsSchema（对应 `CronArgs`）

```rust
/// Mirror struct for CronArgs schema generation.
///
/// Fields are the union of all CronArgs variants.
/// Must stay in sync — interop tests will catch drift.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CronArgsSchema {
    /// add: create a scheduled job (requires message, schedule)
    /// list: show all jobs (no extra params)
    /// remove: delete a job (requires job_id)
    #[schemars(extend("enum" = ["add", "list", "remove"]))]
    action: String,

    /// Reminder message to display. Required when action="add".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    message: String,

    /// Schedule definition. Required when action="add".
    ///
    /// Use kind="cron" for time-based scheduling (e.g. daily at 8 AM).
    /// Use kind="every" only for interval-based scheduling.
    /// Use kind="at" for one-time execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule: Option<CronScheduleArgs>,

    /// Job ID to remove. Required when action="remove".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    job_id: String,
}
```

### ActionSchema（对应 heartbeat `Action`）

```rust
/// Mirror struct for Action schema generation.
///
/// Must stay in sync — interop tests will catch drift.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ActionSchema {
    /// skip: no active tasks, do nothing
    /// run: execute the described tasks (requires tasks)
    #[schemars(extend("enum" = ["skip", "run"]))]
    action: String,

    /// Natural language summary of active tasks to execute.
    /// Required when action="run".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    tasks: String,
}
```

### 设计要点

- 只为顶层 tool schema 的 enum 建 mirror struct（`CronArgs`、`Action`），嵌套 enum（`CronScheduleArgs`）保留 `#[derive(JsonSchema)]` 不动。
- tag 字段用 `String` 类型 + `#[schemars(extend("enum" = [...]))]` 添加合法值约束。
- 字段描述使用标准 Rust doc comment（`///`），schemars 自动转为 schema 的 `description`。
- 变体特有的 `String` 字段用 `#[serde(default, skip_serializing_if = "String::is_empty")]`，非 `String` 字段用 `Option<T>` + `skip_serializing_if`，均不在 `required` 中，doc comment 注明归属哪个动作。
- mirror struct 仅用于 schema 生成和互操作测试，不参与业务逻辑。

## 功能需求列表

### 核心功能

1. **移除 `sanitize_input_schema`**：删除 `crates/provider/src/anthropic/mod.rs` 中的 `sanitize_input_schema` 函数及其调用，`bind_tools` 直接将 `td.parameters` 作为 `input_schema` 传递。
2. **保留 schema 调试日志**：`bind_tools` 中保留 `debug!` 日志，输出每个工具的 `input_schema` JSON，便于排查 Anthropic API 返回的 schema 校验错误。
3. **修复 `cron` 工具 schema**：定义 mirror struct `CronArgsSchema`，字段为 `CronArgs` 所有变体字段的并集，`schedule` 字段直接使用 `Option<CronScheduleArgs>`（嵌套 enum 的 `oneOf` 不在顶层，Anthropic 接受）。移除 `CronArgs` 上的 `#[derive(JsonSchema)]`，保留 `CronScheduleArgs` 上的 `#[derive(JsonSchema)]` 和 `#[schemars(inline)]` 不变。`CRON_PARAMETERS` 改为 `schema_for!(CronArgsSchema)`。编写互操作测试覆盖每个变体的 enum↔struct 序列化/反序列化。
4. **修复 `heartbeat` 工具 schema**：将 `Action` enum 从 externally tagged 改为 internally tagged（添加 `#[serde(tag = "action")]`），定义 mirror struct `ActionSchema`，`HEARTBEAT_TOOL` 改为 `schema_for!(ActionSchema).to_value()`。同步更新 `decide()` 中的重试错误提示信息以匹配新的 JSON 格式。编写互操作测试。

### 扩展功能

无。

## 非功能需求

- **兼容性**：修改后的 schema 必须同时兼容 OpenAI 和 Anthropic 两个 provider（扁平 object schema 对两者都合规）。
- **可维护性**：mirror struct 与 enum 定义紧邻，互操作测试覆盖所有变体，新增变体时若未同步更新 struct 则测试失败。
- **测试要求**：
  - 移除因 `sanitize_input_schema` 存在而编写的测试（`bind_tools_adds_missing_type_field`、`bind_tools_strips_top_level_combinators`、`cron_args_schema_has_no_ref`）。
  - 新增 cron 和 heartbeat 的 enum↔struct 互操作测试，覆盖每个变体。
  - 现有的 cron 功能测试（add/list/remove）和 anthropic provider 测试保持通过。

## 边界与不做事项

- 不修改 MCP 工具的 schema 处理（来自外部服务器，不可控）。
- 不修改其他工具（read_file、write_file、edit_file、list_dir、shell、spawn、save_memory）的 schema，它们已经是合规的扁平 object schema。
- 不修改 `Tool` trait 的 `fn parameters(&self) -> Schema` 签名。

## 假设与约束

- **技术假设**：serde 的 internally tagged enum（`#[serde(tag = "action")]`）与扁平 struct 的 JSON 格式一致，两者可互相反序列化（互操作测试验证）。
- **技术假设**：serde 的 internally tagged enum 对 unit variant（如 `Action::Skip`）生成 `{"action":"skip"}`，可被正确反序列化。

## 待确认事项

无
