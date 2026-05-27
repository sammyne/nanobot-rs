# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | 新增 `ToolChoice` 枚举，`Options` 新增 `tool_choice` 字段 |
| `crates/provider/src/openai/mod.rs` | 修改 | `chat()` 中传递 `tool_choice` 到请求 |
| `crates/provider/src/anthropic/mod.rs` | 修改 | `chat()` 中传递 `tool_choice` 到请求 |
| `crates/memory/src/store.rs` | 修改 | 整合调用设置 `tool_choice: Some(ToolChoice::Required)` |

## 任务列表

### 1. Options 新增 tool_choice 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`
- 验收标准: `ToolChoice` 枚举定义完成；`Options` 包含 `tool_choice: Option<ToolChoice>`；`Options::default()` 中为 `None`
- 信心评估: 5
- 步骤:
  - [ ] 在 `base/mod.rs` 中新增 `ToolChoice` 枚举：`Auto`、`Required`、`Named(String)`
  - [ ] `Options` 新增 `pub tool_choice: Option<ToolChoice>` 字段
  - [ ] `Default` impl 中设为 `tool_choice: None`
  - [ ] 运行 `cargo check -p nanobot-provider`

### 2. OpenAI provider 传递 tool_choice

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/openai/mod.rs`
- 验收标准: 当 `options.tool_choice` 为 `Some` 时，请求中包含对应的 `tool_choice` 字段
- 信心评估: 4
- 步骤:
  - [ ] 在 `chat()` 方法中，`builder.build()` 前，根据 `options.tool_choice` 调用 `builder.tool_choice()` 设置值（`Auto` → `ChatCompletionToolChoiceOption::Auto`，`Required` → `ChatCompletionToolChoiceOption::Required`，`Named(name)` → `ChatCompletionToolChoiceOption::Named` 对应结构）
  - [ ] `None` 时不设置（保持 API 默认）
  - [ ] 运行 `cargo check -p nanobot-provider`

### 3. Anthropic provider 传递 tool_choice

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: 当 `options.tool_choice` 为 `Some` 时，请求 JSON 中包含 `tool_choice` 字段
- 信心评估: 4
- 步骤:
  - [ ] 在 `AnthropicRequest` 结构中新增 `#[serde(skip_serializing_if = "Option::is_none")] tool_choice: Option<serde_json::Value>` 字段
  - [ ] 在 `chat()` 构建 `AnthropicRequest` 时，根据 `options.tool_choice` 转换为 Anthropic 格式（`Auto` → `{"type":"auto"}`，`Required` → `{"type":"any"}`，`Named(name)` → `{"type":"tool","name":"..."}`）
  - [ ] `None` 时设为 `None`（不序列化）
  - [ ] 运行 `cargo check -p nanobot-provider`

### 4. 整合调用设置 tool_choice

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/memory/src/store.rs`
- 验收标准: `consolidate_internal` 中 LLM 调用使用 `tool_choice: Some(ToolChoice::Required)`
- 信心评估: 5
- 步骤:
  - [ ] 在 `consolidate_internal` 中，构造 Options 时（当前使用 `self.options`），克隆后设置 `tool_choice: Some(ToolChoice::Required)`
  - [ ] 运行 `cargo test -p nanobot-memory` 验证通过
  - [ ] 运行 `cargo test` 全量验证
