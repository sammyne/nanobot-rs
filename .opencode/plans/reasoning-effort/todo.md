# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/config/src/schema/agent.rs` | 修改 | 定义 `ReasoningEffort` 枚举，`AgentDefaults` 新增字段 |
| `crates/provider/src/base/mod.rs` | 修改 | `Options` 新增 `reasoning_effort` 字段 |
| `crates/provider/src/openai/mod.rs` | 修改 | 传递 `reasoning_effort` 到 builder |
| `crates/agent/src/loop/mod.rs` | 修改 | `call_llm()` 从 config 构造 Options |
| `crates/subagent/src/manager.rs` | 修改 | `run_subagent()` 从字段构造 Options |

## 任务列表

### ✅ 1. 定义 ReasoningEffort 枚举并添加 config 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/agent.rs`
- 验收标准: `ReasoningEffort` 枚举可从 config crate 导出；`AgentDefaults` 包含 `reasoning_effort: Option<ReasoningEffort>`；JSON `"reasoningEffort": "low"` 能正确反序列化
- 风险/注意点: 枚举需 `Copy + Clone + Serialize + Deserialize`；字段需 `#[serde(default)]` 以兼容无此字段的旧配置
- 信心评估: 5
- 步骤:
  - [ ] 在 `agent.rs` 中定义 `ReasoningEffort` 枚举（Low/Medium/High），derive `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`，`#[serde(rename_all = "lowercase")]`
  - [ ] `AgentDefaults` 新增 `#[serde(default)] pub reasoning_effort: Option<ReasoningEffort>` 字段
  - [ ] 确保 `ReasoningEffort` 从 config crate 的公共 API 导出

### ✅ 2. Options 新增 reasoning_effort 字段

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/base/mod.rs`
- 验收标准: `Options` 包含 `reasoning_effort: Option<ReasoningEffort>`，保持 `Copy` trait；`Default` 实现中为 `None`
- 风险/注意点: provider crate 已依赖 config crate，可直接使用 `nanobot_config::ReasoningEffort`
- 信心评估: 5
- 步骤:
  - [ ] `Options` 新增 `pub reasoning_effort: Option<nanobot_config::ReasoningEffort>` 字段
  - [ ] `Default` impl 中设为 `None`

### ✅ 3. OpenAI provider 传递 reasoning_effort

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/provider/src/openai/mod.rs`
- 验收标准: 当 `options.reasoning_effort` 为 `Some` 时，builder 设置对应的 `async_openai::types::ReasoningEffort`
- 风险/注意点: 需要从 `nanobot_config::ReasoningEffort` 映射到 `async_openai::types::ReasoningEffort`（同名不同类型），通过 `impl From` 转换
- 信心评估: 5
- 步骤:
  - [ ] 实现 `impl From<nanobot_config::ReasoningEffort> for async_openai::types::ReasoningEffort`
  - [ ] 在 `chat()` 方法中，`builder.temperature(...)` 之后，检查 `options.reasoning_effort`
  - [ ] `Some` 时调用 `builder.reasoning_effort(re.into())`

### ✅ 4. 修复 AgentLoop::call_llm() 的 Options 构造

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/agent/src/loop/mod.rs`
- 验收标准: `call_llm()` 从 `self.config` 构造 `Options`，包含 max_tokens、temperature、reasoning_effort
- 风险/注意点: `self.config.max_tokens` 是 `usize`，`Options.max_tokens` 是 `u16`，需要 `as u16` 转换；`self.config.temperature` 是 `f64`，`Options.temperature` 是 `f32`
- 信心评估: 5
- 步骤:
  - [ ] 将 `let options = nanobot_provider::Options::default();` 替换为从 `self.config` 构造的 `Options`

### ✅ 5. 修复 SubagentManager::run_subagent() 的 Options 构造

- 优先级: P0
- 依赖项: 2
- 涉及文件: `crates/subagent/src/manager.rs`
- 验收标准: `run_subagent()` 从 `self.temperature` 和 `self.max_tokens` 构造 `Options`，`reasoning_effort` 为 `None`
- 风险/注意点: SubagentManager 的 `max_tokens` 是 `u32`，需 `as u16`
- 信心评估: 5
- 步骤:
  - [ ] 将 `let options = nanobot_provider::Options::default();` 替换为从 `self.temperature`、`self.max_tokens` 构造的 `Options`

### ✅ 6. 单元测试

- 优先级: P1
- 依赖项: 1
- 涉及文件: `crates/config/src/schema/tests.rs`（或对应测试文件）
- 验收标准: 测试 ReasoningEffort 的 serde 序列化/反序列化，测试 AgentDefaults 含 reasoning_effort 的反序列化
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 测试 `"low"` / `"medium"` / `"high"` 反序列化为对应枚举值
  - [ ] 测试无 `reasoningEffort` 字段时默认为 `None`
  - [ ] 测试无效值（如 `"invalid"`）反序列化失败

### ✅ 7. 全量验证

- 优先级: P0
- 依赖项: 1-6
- 涉及文件: 全部
- 验收标准: `cargo +nightly fmt`、`cargo clippy -- -D warnings -D clippy::uninlined_format_args`、`cargo test` 全部通过
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 运行 `cargo +nightly fmt`
  - [ ] 运行 `cargo clippy -- -D warnings -D clippy::uninlined_format_args`
  - [ ] 运行 `cargo test`

## 实现建议

- `nanobot_config::ReasoningEffort` → `async_openai::types::ReasoningEffort` 通过 `From` trait 转换：
  ```rust
  impl From<nanobot_config::ReasoningEffort> for async_openai::types::ReasoningEffort {
      fn from(re: nanobot_config::ReasoningEffort) -> Self {
          match re {
              nanobot_config::ReasoningEffort::Low => Self::Low,
              nanobot_config::ReasoningEffort::Medium => Self::Medium,
              nanobot_config::ReasoningEffort::High => Self::High,
          }
      }
  }
  ```
  调用处：`builder.reasoning_effort(re.into())`
- AgentLoop 的 Options 构造：
  ```rust
  let options = nanobot_provider::Options {
      max_tokens: self.config.max_tokens as u16,
      temperature: self.config.temperature as f32,
      reasoning_effort: self.config.reasoning_effort,
  };
  ```
