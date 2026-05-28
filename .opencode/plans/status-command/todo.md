# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | 新增 `Usage`、`MeteredMessage`（含 `Deref<Target=Message>`）；`Provider::chat()` 返回类型改为 `Result<MeteredMessage>` |
| `crates/provider/src/anthropic/mod.rs` | 修改 | 解析 API 响应中的 usage 字段，返回 `MeteredMessage` |
| `crates/provider/src/openai/mod.rs` | 修改 | 读取 response.usage 字段，返回 `MeteredMessage` |
| `crates/agent/src/loop/mod.rs` | 修改 | AgentLoop 新增 `start_time`、`last_usage` 字段；`call_llm()` 从 `MeteredMessage` 提取 usage；注册 `/status` 命令 |
| `crates/agent/src/cmd/mod.rs` | 修改 | 新增 `status` 模块声明和导出 |
| `crates/agent/src/cmd/status/mod.rs` | 新增 | StatusCmd 实现 |
| `crates/agent/src/cmd/status/tests.rs` | 新增 | StatusCmd 测试 |
| `crates/agent/src/cmd/help/mod.rs` | 修改 | `/help` 文本新增 `/status` 说明 |

## 任务列表

### 1. ✅ Provider 层新增 Usage、MeteredMessage 并修改 chat() 返回类型

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`、`crates/provider/src/anthropic/mod.rs`、`crates/provider/src/openai/mod.rs`
- 验收标准: `cargo check -p nanobot-provider` 通过
- 风险/注意点: `MeteredMessage` 通过 `Deref<Target=Message>` 使调用方透明兼容，subagent 等处 `provider.chat()` 的返回值仍可直接当 `Message` 用，无需改动；Anthropic 的 usage 在响应顶层 `{"usage": {"input_tokens": N, "output_tokens": N}}`；OpenAI 的 usage 在 `response.usage` 字段
- 信心评估: 4
- 步骤:
  - [ ] 在 `base/mod.rs` 中新增 `#[derive(Debug, Clone, Default)] pub struct Usage { pub input_tokens: u64, pub output_tokens: u64 }`
  - [ ] 在 `base/mod.rs` 中新增 `MeteredMessage { pub message: Message, pub usage: Option<Usage> }`，实现 `Deref<Target=Message>` 和 `DerefMut`
  - [ ] 为 `MeteredMessage` 实现 `From<Message>`（usage 为 None），方便无 usage 场景构造
  - [ ] `Provider::chat()` 返回类型从 `Result<Message>` 改为 `Result<MeteredMessage>`
  - [ ] Anthropic provider：`AnthropicResponse` 新增 `usage: Option<AnthropicUsage>` 字段（`AnthropicUsage { input_tokens: u64, output_tokens: u64 }`）；`chat()` 返回 `MeteredMessage { message, usage }`
  - [ ] OpenAI provider：从 `response.usage` 读取 `prompt_tokens` 和 `completion_tokens`，构建 `MeteredMessage { message, usage }`
  - [ ] 运行 `cargo check -p nanobot-provider` 验证通过

### 2. ✅ AgentLoop 适配并记录运行时状态

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `cargo check -p nanobot-agent` 通过
- 风险/注意点: `call_llm()` 返回 `MeteredMessage`，通过 Deref 大部分代码无需改动；只需在 `re_act()` 中额外提取 `.usage` 存入 `last_usage`
- 信心评估: 5
- 步骤:
  - [ ] `AgentLoop` 新增字段：`start_time: std::time::Instant`、`last_usage: std::sync::Mutex<Option<Usage>>`
  - [ ] `new()` 中初始化 `start_time: Instant::now()`、`last_usage: Mutex::new(None)`
  - [ ] `call_llm()` 返回类型改为 `Result<MeteredMessage>`
  - [ ] `re_act()` 中 `call_llm()` 调用处：从 `MeteredMessage` 提取 `.usage` 存入 `self.last_usage`；后续代码通过 Deref 继续当 `Message` 使用
  - [ ] 运行 `cargo check -p nanobot-agent` 验证通过（subagent 通过 Deref 自动兼容，无需改动）

### 3. ✅ 实现 StatusCmd 并注册

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/agent/src/cmd/mod.rs`、`crates/agent/src/cmd/status/mod.rs`、`crates/agent/src/cmd/status/tests.rs`、`crates/agent/src/cmd/help/mod.rs`、`crates/agent/src/loop/mod.rs`
- 验收标准: `cargo test -p nanobot-agent` 通过；`/status` 命令返回包含版本、模型、运行时长等信息的文本
- 风险/注意点: StatusCmd 需要访问 AgentLoop 的 `config`、`start_time`、`last_usage`、`sessions`；通过构造函数传入所需引用（参照 NewCmd/StopCmd 模式）
- 信心评估: 5
- 步骤:
  - [ ] 创建 `crates/agent/src/cmd/status/mod.rs`，定义 `StatusCmd` 结构体，持有 `model: String`、`start_time: Instant`、`last_usage: Option<Usage>`、`session_message_count: usize`
  - [ ] 实现 `Command` trait：构建状态文本，包含版本号（`env!("CARGO_PKG_VERSION")`）、模型名称、最近 token 用量（in/out，None 时显示 "N/A"）、会话消息数、运行时长（格式化为 `Xd Xh Xm Xs`）
  - [ ] 创建 `crates/agent/src/cmd/status/tests.rs`，测试状态文本包含关键字段
  - [ ] 在 `cmd/mod.rs` 中添加 `mod status;` 和 `pub use status::StatusCmd;`
  - [ ] 在 `loop/mod.rs` 的 `try_handle_cmd()` 中添加 `"status"` 分支：从 `self` 读取 config/start_time/last_usage/sessions，构造 `StatusCmd` 并调用
  - [ ] 更新 `help/mod.rs` 中的帮助文本，添加 `/status — Show bot runtime status`
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

### 4. ✅ 全量验证

- 优先级: P0
- 依赖项: 1-3
- 涉及文件: 无
- 验收标准: `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 通过；`cargo test` 全工作空间通过
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`
  - [ ] 修复发现的问题

## 实现建议

- `MeteredMessage` 通过 `Deref<Target=Message>` 使 subagent 等调用方零改动：原来 `let response = provider.chat(...)?; response.content()` 继续工作
- `Usage` 和 `MeteredMessage` 放在 `provider/base/mod.rs`，与 `Message`、`ToolCall` 等类型同级
- StatusCmd 直接持有所需数据的快照（不持有 Arc 引用），在 `try_handle_cmd()` 中构造时从 AgentLoop 读取当前值
- 运行时长格式化：`let secs = self.start_time.elapsed().as_secs(); format!("{}d {}h {}m {}s", secs/86400, secs%86400/3600, secs%3600/60, secs%60)`
- 版本号使用 `env!("CARGO_PKG_VERSION")` 编译时嵌入
