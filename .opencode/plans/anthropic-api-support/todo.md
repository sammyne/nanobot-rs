# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `Cargo.toml` | 修改 | 工作空间添加 `reqwest` 依赖 |
| `crates/provider/Cargo.toml` | 修改 | 添加 `reqwest` 依赖 |
| `crates/config/src/schema/provider.rs` | 修改 | `ProvidersConfig` 从 struct 改为枚举 |
| `crates/config/src/schema/mod.rs` | 修改 | `Config::provider()`、`Config::new()`、`Config::validate()`、`Config::masked_api_key()` 适配枚举 |
| `crates/config/src/schema/tests.rs` | 修改 | 适配枚举的测试更新，新增 Anthropic 配置解析测试 |
| `crates/provider/src/anthropic/mod.rs` | 新增 | `AnthropicLike` 结构体及 `Provider` trait 实现 |
| `crates/provider/src/anthropic/tests.rs` | 新增 | Anthropic 消息转换和工具调用的单元测试 |
| `crates/provider/src/any/mod.rs` | 新增 | `AnyProvider` 枚举包装器及 `Provider` trait 实现 |
| `crates/provider/src/any/tests.rs` | 新增 | `AnyProvider` 分发和工厂方法的单元测试 |
| `crates/provider/src/lib.rs` | 修改 | 新增 `anthropic` 和 `any` 模块，导出 `AnthropicLike` 和 `AnyProvider` |
| `crates/nanobot/src/commands/agent/mod.rs` | 修改 | `OpenAILike` 替换为 `AnyProvider` |
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | `OpenAILike` 替换为 `AnyProvider` |

## 任务列表

### 1. ✅ 工作空间添加 reqwest 依赖

- 优先级: P0
- 依赖项: 无
- 涉及文件: `Cargo.toml`, `crates/provider/Cargo.toml`
- 验收标准: `cargo check -p nanobot-provider` 通过
- 风险/注意点: `reqwest` 已在 `crates/channels/Cargo.toml` 中以 crate 级别使用（`version = "0.12", features = ["json"]`），提升为工作空间依赖后需确保 channels crate 也改为引用工作空间版本
- 步骤:
  - [ ] 在 `Cargo.toml` 的 `[workspace.dependencies]` 中添加 `reqwest`，使用表语法声明 `version = "0.12"` 和 `features = ["json"]`
  - [ ] 在 `crates/provider/Cargo.toml` 的 `[dependencies]` 中添加 `reqwest.workspace = true`
  - [ ] 将 `crates/channels/Cargo.toml` 中的 `reqwest` 行改为 `reqwest.workspace = true`
  - [ ] 运行 `cargo check` 验证编译通过

### 2. ✅ ProvidersConfig 改为枚举

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/provider.rs`, `crates/config/src/schema/mod.rs`, `crates/config/src/schema/tests.rs`
- 验收标准: `{"providers":{"custom":{...}}}` 和 `{"providers":{"anthropic":{...}}}` 两种 JSON 格式均能正确反序列化；不含 `providers` 字段的旧配置默认为 `Custom` 变体；`Config::provider()` 从任意变体中提取 `ProviderConfig`
- 风险/注意点: serde 默认的外部标签枚举格式与当前 JSON 结构 `{"custom":{...}}` 天然兼容；枚举需要手动实现 `Default`（默认为 `Custom(ProviderConfig::default())`）；`Config::new()` 签名从接受 `ProviderConfig` 改为接受 `ProvidersConfig`；现有测试中所有 `providers.custom` 的访问方式需要适配
- 步骤:
  - [ ] 将 `ProvidersConfig` 从 struct 改为枚举，定义两个变体：`Custom(ProviderConfig)` 和 `Anthropic(ProviderConfig)`，保留 `#[serde(rename_all = "camelCase")]`，派生 `Debug, Clone, Serialize, Deserialize`
  - [ ] 为 `ProvidersConfig` 手动实现 `Default`：返回 `ProvidersConfig::Custom(ProviderConfig::default())`
  - [ ] 为 `ProvidersConfig` 添加 `provider_config(&self) -> &ProviderConfig` 方法：match 两个变体，返回内部 `ProviderConfig` 的引用
  - [ ] 修改 `Config::provider(&self) -> ProviderConfig`：调用 `self.providers.provider_config().clone()`
  - [ ] 修改 `Config::new()`：签名改为接受 `ProvidersConfig`，直接赋值给 `providers` 字段
  - [ ] 修改 `Config::validate()`：将 `providers.custom` 的验证逻辑改为对 `providers.provider_config()` 统一验证（`api_base` URL 格式、`api_key` 最小长度）
  - [ ] 修改 `Config::masked_api_key()`：从 `self.providers.provider_config().api_key` 获取 key
  - [ ] 更新模块文档注释中的 JSON 示例，展示 `providers.custom` 和 `providers.anthropic` 两种配置格式
  - [ ] 更新现有测试：`Config::new()` 调用改为传入 `ProvidersConfig::Custom(ProviderConfig {...})`
  - [ ] 更新现有测试：`config.providers.custom.as_ref().unwrap()` 改为 match 枚举或使用 `config.providers.provider_config()`
  - [ ] 编写测试：反序列化 `{"providers":{"anthropic":{"apiKey":"sk-ant-xxx"}}}` 得到 `ProvidersConfig::Anthropic` 变体
  - [ ] 编写测试：反序列化 `{"providers":{"custom":{"apiKey":"sk-xxx"}}}` 得到 `ProvidersConfig::Custom` 变体
  - [ ] 编写测试：反序列化 `{}` 得到默认的 `ProvidersConfig::Custom(ProviderConfig::default())`
  - [ ] 编写测试：`provider_config()` 从两种变体中均能正确提取 `ProviderConfig`
  - [ ] 运行 `cargo test -p nanobot-config` 验证全部通过

### 3. ✅ 实现 Anthropic Messages API 客户端

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`, `crates/provider/src/anthropic/tests.rs`
- 验收标准: `AnthropicLike` 实现 `Provider` trait；消息格式转换正确（覆盖 system 提取、tool_use/tool_result 转换）；工具绑定正确转换为 Anthropic 格式
- 风险/注意点: Anthropic 的 `Message::Tool` 需要合并为 `user` 角色的 `tool_result` content block；相邻的多个 `Message::Tool` 需合并到同一个 `user` 消息中；Anthropic 的 `max_tokens` 是必填字段；tool call 参数在 Anthropic 响应中是已解析的 JSON 对象（非字符串）
- 步骤:
  - [ ] 创建 `crates/provider/src/anthropic/mod.rs` 文件
  - [ ] 定义 Anthropic API 请求体的 serde 结构体：`AnthropicRequest`（含 `model`, `max_tokens`, `system`, `messages`, `temperature`, `tools` 字段）
  - [ ] 定义 Anthropic 消息格式的 serde 结构体：`AnthropicMessage`（含 `role`, `content` 字段），`ContentBlock` 枚举（`Text { text }`, `ToolUse { id, name, input }`, `ToolResult { tool_use_id, content }`）
  - [ ] 定义 Anthropic 工具定义的 serde 结构体：`AnthropicTool`（含 `name`, `description`, `input_schema` 字段）
  - [ ] 定义 Anthropic API 响应体的 serde 结构体：`AnthropicResponse`（含 `id`, `content`, `stop_reason`, `usage` 字段）
  - [ ] 定义 `AnthropicLike` 结构体（含 `client: reqwest::Client`, `api_key: String`, `api_base: String`, `model: String`, `timeout: u64`, `tools: Arc<Vec<AnthropicTool>>`, `extra_headers: Option<HashMap<String, String>>`），派生 `Clone`
  - [ ] 实现 `AnthropicLike::new(config: &ProviderConfig, model: &str) -> Result<Self>`：从 `ProviderConfig` 构建，默认 `api_base` 为 `"https://api.anthropic.com"`，默认 `timeout` 为 120 秒
  - [ ] 实现 `AnthropicLike::new_with_timeout(config: &ProviderConfig, model: &str, timeout: u64) -> Result<Self>`
  - [ ] 实现消息转换函数 `convert_messages(messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>)`：遍历 `messages`，提取 `Message::System` 的 content 拼接为 `system` 参数；将 `Message::User` 转为 `AnthropicMessage { role: "user", content: [ContentBlock::Text] }`；将 `Message::Assistant` 转为 `AnthropicMessage { role: "assistant", content }` 其中 content 包含 `ContentBlock::Text`（如果 content 非空）和每个 tool_call 对应的 `ContentBlock::ToolUse`；将连续的 `Message::Tool` 合并为单个 `AnthropicMessage { role: "user", content: [ContentBlock::ToolResult, ...] }`
  - [ ] 实现 `Provider::chat()`：构建 `AnthropicRequest`，使用 `reqwest::Client` 发送 POST 请求到 `{api_base}/v1/messages`，设置 `x-api-key`、`anthropic-version: 2023-06-01`、`content-type: application/json` 请求头，以及 `extra_headers` 中的自定义头；用 `tokio::time::timeout` 包装请求；解析 `AnthropicResponse`，从 `content` 数组中提取文本和 tool_use blocks，转换为 `Message::assistant()` 或 `Message::assistant_with_tools()`
  - [ ] 实现 `Provider::bind_tools()`：将 `Vec<ToolDefinition>` 转换为 `Vec<AnthropicTool>`（`name` → `name`，`description` → `description`，`parameters` → `input_schema`），存储为 `Arc<Vec<AnthropicTool>>`
  - [ ] 创建 `crates/provider/src/anthropic/tests.rs` 文件，在 `mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 编写测试：`convert_messages` 正确提取 system 消息为独立参数
  - [ ] 编写测试：`convert_messages` 正确转换 user 和 assistant 消息
  - [ ] 编写测试：`convert_messages` 正确将带 tool_calls 的 assistant 消息转换为含 text + tool_use blocks 的 content 数组
  - [ ] 编写测试：`convert_messages` 正确将连续的 `Message::Tool` 合并为单个 user 消息中的多个 `tool_result` blocks
  - [ ] 编写测试：`convert_messages` 处理空消息列表
  - [ ] 编写测试：`bind_tools` 正确将 `ToolDefinition` 转换为 `AnthropicTool`（验证 `parameters` → `input_schema` 映射）
  - [ ] 编写测试：`AnthropicResponse` 反序列化纯文本响应
  - [ ] 编写测试：`AnthropicResponse` 反序列化含 tool_use 的响应
  - [ ] 编写测试：`AnthropicLike::new` 使用默认 api_base
  - [ ] 编写测试：`AnthropicLike::new` 使用自定义 api_base
  - [ ] 运行 `cargo test -p nanobot-provider` 验证全部通过

### 4. ✅ 实现 AnyProvider 枚举包装器

- 优先级: P0
- 依赖项: 2, 3
- 涉及文件: `crates/provider/src/any/mod.rs`, `crates/provider/src/any/tests.rs`
- 验收标准: `AnyProvider` 实现 `Provider` trait；`AnyProvider::from_config()` 根据 `ProvidersConfig` 枚举变体正确选择 provider
- 风险/注意点: `AnyProvider` 必须派生 `Clone`（`Provider` trait 要求）
- 步骤:
  - [ ] 创建 `crates/provider/src/any/mod.rs` 文件
  - [ ] 定义 `AnyProvider` 枚举：`OpenAI(OpenAILike)` 和 `Anthropic(AnthropicLike)` 两个变体，派生 `Clone`
  - [ ] 为 `AnyProvider` 实现 `Provider` trait：`chat()` 和 `bind_tools()` 通过 `match self` 委托到具体变体
  - [ ] 实现 `AnyProvider::from_config(config: &NanobotConfig) -> Result<Self>`：match `config.providers`，`ProvidersConfig::Custom(ref pc)` → `OpenAILike::new(pc, &config.agents.defaults.model)`，`ProvidersConfig::Anthropic(ref pc)` → `AnthropicLike::new(pc, &config.agents.defaults.model)`
  - [ ] 创建 `crates/provider/src/any/tests.rs` 文件，在 `mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 编写测试：`ProvidersConfig::Anthropic` 时 `from_config` 创建 `AnyProvider::Anthropic` 变体
  - [ ] 编写测试：`ProvidersConfig::Custom` 时 `from_config` 创建 `AnyProvider::OpenAI` 变体
  - [ ] 运行 `cargo test -p nanobot-provider` 验证全部通过

### 5. ✅ 更新 provider crate 导出

- 优先级: P0
- 依赖项: 3, 4
- 涉及文件: `crates/provider/src/lib.rs`
- 验收标准: `nanobot_provider::AnyProvider` 和 `nanobot_provider::AnthropicLike` 可从外部 crate 访问
- 风险/注意点: 无
- 步骤:
  - [ ] 在 `lib.rs` 中添加 `mod anthropic;` 和 `mod any;`
  - [ ] 添加 `pub use anthropic::AnthropicLike;` 和 `pub use any::AnyProvider;`
  - [ ] 更新模块文档注释，说明支持 OpenAI 和 Anthropic 两种协议
  - [ ] 运行 `cargo check -p nanobot-provider` 验证编译通过

### 6. ✅ 二进制 crate 适配：将 OpenAILike 替换为 AnyProvider

- 优先级: P0
- 依赖项: 4, 5
- 涉及文件: `crates/nanobot/src/commands/agent/mod.rs`, `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: `cargo build` 通过；所有 `OpenAILike` 引用替换为 `AnyProvider`；现有功能不受影响
- 风险/注意点: `gateway/mod.rs` 中 `OpenAILike` 出现在多处类型标注中（`ServicesContext<OpenAILike>`、`HeartbeatService<OpenAILike>`、`AgentLoop<OpenAILike>` 等），需全部替换；`init_provider` 方法的返回类型也需更改
- 步骤:
  - [ ] 在 `agent/mod.rs` 中：将 `use nanobot_provider::OpenAILike;` 改为 `use nanobot_provider::AnyProvider;`
  - [ ] 在 `agent/mod.rs` 中：将 `OpenAILike::from_config(&config)?` 改为 `AnyProvider::from_config(&config)?`
  - [ ] 在 `agent/mod.rs` 中：将函数签名中所有 `OpenAILike` 类型标注替换为 `AnyProvider`（`run_once` 和 `run_interactive` 的 `provider` 参数）
  - [ ] 在 `gateway/mod.rs` 中：将 `use nanobot_provider::{OpenAILike, Provider};` 改为 `use nanobot_provider::{AnyProvider, Provider};`
  - [ ] 在 `gateway/mod.rs` 中：将 `init_provider` 返回类型从 `Result<OpenAILike>` 改为 `Result<AnyProvider>`，内部调用从 `OpenAILike::from_config` 改为 `AnyProvider::from_config`
  - [ ] 在 `gateway/mod.rs` 中：将所有泛型实例化中的 `OpenAILike` 替换为 `AnyProvider`（`ServicesContext<AnyProvider>`、`Arc<AgentLoop<AnyProvider>>`、`HeartbeatService<AnyProvider>` 等）
  - [ ] 运行 `cargo build` 验证编译通过
  - [ ] 运行 `cargo test` 验证所有现有测试通过

### 7. ✅ 代码质量检查

- 优先级: P1
- 依赖项: 1-6
- 涉及文件: 所有修改和新增的文件
- 验收标准: `cargo +nightly fmt`、`cargo clippy -- -D warnings -D clippy::uninlined_format_args`、`cargo test`、`cargo doc --no-deps` 全部通过
- 风险/注意点: 无
- 步骤:
  - [ ] 运行 `cargo +nightly fmt` 格式化代码
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 修复所有 lint 警告
  - [ ] 运行 `cargo test` 确认所有测试通过
  - [ ] 运行 `cargo doc --no-deps` 确认文档生成无警告

## 实现建议

- `AnthropicLike` 的结构与 `OpenAILike` 对称：同样使用 `Arc<Vec<...>>` 存储转换后的工具定义，同样使用 `tokio::time::timeout` 包装请求
- Anthropic API 的 `content` 字段在请求中可以是字符串或 content block 数组。为简化实现，统一使用 content block 数组格式（`[{"type": "text", "text": "..."}]`）
- `convert_messages` 函数中处理连续 `Message::Tool` 合并时，使用一个临时 `Vec<ContentBlock>` 收集 tool_result blocks，遇到非 Tool 消息时 flush 为一个 `user` 消息
- `reqwest::Client` 应在 `AnthropicLike::new()` 中创建一次并复用（连接池），不要每次 `chat()` 调用都创建新 client
- Anthropic 响应中 tool call 的 `input` 字段是已解析的 JSON 对象，转换为内部 `ToolCall` 时需要 `serde_json::to_string()` 序列化为字符串存入 `arguments` 字段
- `ProvidersConfig` 枚举使用 serde 默认的外部标签格式，与当前 JSON 结构 `{"custom":{...}}` 天然兼容，无需额外的 serde 属性
