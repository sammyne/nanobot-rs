# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/utils/src/strings.rs` | 修改 | 新增 `estimate_tokens(text)` 文本 token 估算函数 |
| `crates/utils/src/strings/tests.rs` | 新增 | 拆分现有内联测试 + 新增 token 估算测试 |
| `crates/provider/src/base/mod.rs` | 修改 | `Message` 新增 `estimate_tokens()` 方法 |
| `crates/provider/src/base/tests.rs` | 修改 | 新增 message token 估算测试 |
| `crates/config/src/schema/agent.rs` | 修改 | `AgentDefaults` 新增 `max_input_tokens` 字段 |
| `crates/config/src/schema/tests.rs` | 修改 | 新增配置测试 |
| `crates/session/src/session.rs` | 修改 | `get_history()` 增加 token 预算参数 |
| `crates/session/tests/session.rs` | 修改 | 适配新签名 + 新增 token 截断测试 |
| `crates/agent/src/loop/mod.rs` | 修改 | 传递 `max_input_tokens` 到 `get_history()`；ReAct 循环内 token 检查；整合触发改为 token-based |
| `crates/agent/src/loop/tests.rs` | 修改 | 适配新签名 |
| `crates/memory/src/store.rs` | 修改 | `should_consolidate()` 增加 token-based 触发路径 |

## 任务列表

### 1. ✅ 文本 token 估算函数

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/utils/src/strings.rs`, `crates/utils/src/strings/tests.rs`
- 验收标准: `estimate_tokens(text)` 返回基于字节长度的 token 估算值；空字符串返回 0；ASCII 文本约 4 字节/token；中文字符约 1 字符/token
- 风险/注意点: 当前 `strings.rs` 的测试内联在文件中（违反项目规范），需要一并拆分到 `tests.rs`
- 信心评估: 5
- 步骤:
  - [ ] 将 `crates/utils/src/strings.rs` 中的 `#[cfg(test)] mod tests { ... }` 内联测试块拆分到 `crates/utils/src/strings/tests.rs`（需要将 `strings.rs` 改为 `strings/mod.rs` 目录形式）
  - [ ] 在 `crates/utils/src/strings/mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 新增 `pub fn estimate_tokens(text: &str) -> usize` 函数：`text.len() / 4`（空字符串返回 0）
  - [ ] 在 `tests.rs` 中新增测试：空字符串→0，"hello"(5 bytes)→1，100 字节 ASCII→25，中文字符串验证
  - [ ] 运行 `cargo test -p nanobot-utils` 验证通过

### 2. ✅ Message::token_len() 方法

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/base/mod.rs`, `crates/provider/src/base/tests.rs`
- 验收标准: `Message::token_len()` 返回消息的 token 估算值，包含角色开销（4 token）+ 内容 token + 工具调用 token
- 风险/注意点: provider crate 需要依赖 utils crate（检查 Cargo.toml 确认已有依赖）
- 信心评估: 5
- 步骤:
  - [ ] 在 `Message` 的 `impl` 块中新增 `pub fn token_len(&self) -> usize` 方法
  - [ ] 实现逻辑：角色开销 4 + `estimate_tokens(content)` + 对 Assistant 消息的 `tool_calls` 每个累加 `estimate_tokens(name) + estimate_tokens(arguments)` + 对 thinking 字段 `estimate_tokens(json_string)`
  - [ ] 在 `tests.rs` 中新增测试：system 消息、user 消息、assistant 带工具调用消息、tool 消息
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过

### 3. ✅ 新增 `max_input_tokens` 配置项

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/config/src/schema/agent.rs`, `crates/config/src/schema/tests.rs`
- 验收标准: `AgentDefaults` 包含 `max_input_tokens: usize` 字段，默认值 128000，serde 序列化/反序列化正确，旧配置文件兼容（缺失字段使用默认值）
- 风险/注意点: 使用 `#[serde(default = "default_max_input_tokens")]` 确保向后兼容
- 信心评估: 5
- 步骤:
  - [ ] 在 `crates/config/src/schema/agent.rs` 的 `AgentDefaults` 中新增字段 `#[serde(default = "default_max_input_tokens")] pub max_input_tokens: usize`
  - [ ] 新增 `fn default_max_input_tokens() -> usize { 128_000 }`
  - [ ] 在 `Default` impl 中添加 `max_input_tokens: default_max_input_tokens()`
  - [ ] 在 `crates/config/src/schema/tests.rs` 中新增测试：默认值为 128000、自定义值反序列化、旧配置兼容
  - [ ] 运行 `cargo test -p nanobot-config` 验证通过

### 4. ✅ Session::get_history() 增加 token 预算

- 优先级: P0
- 依赖项: 1, 2
- 涉及文件: `crates/session/src/session.rs`, `crates/session/tests/session.rs`
- 验收标准: `get_history()` 新增 `max_tokens: usize` 参数；当 `max_tokens > 0` 时，从最新消息向前累加 token，超出预算时停止（保持 user-alignment）；`max_tokens == 0` 时行为与原来一致（仅按消息计数截断）
- 风险/注意点: 签名变更会影响所有调用方（agent loop、tests），需要同步更新
- 信心评估: 4
- 步骤:
  - [ ] 修改 `get_history()` 签名：`pub fn get_history(&self, max_messages: usize, max_tokens: usize, buf: &mut Vec<Message>) -> usize`
  - [ ] 在现有的消息计数截断逻辑之后，增加 token 预算检查：如果 `max_tokens > 0`，从 `final_start` 向后扫描，累加每条消息的 `estimate_tokens()`，找到使总 token 不超过 `max_tokens` 的起始位置
  - [ ] 确保 token 截断后仍保持 user-alignment（起始消息为 User 类型）
  - [ ] 更新 `crates/session/tests/session.rs` 中所有 `get_history()` 调用，传入 `max_tokens: 0`（保持原行为）
  - [ ] 新增测试：token 预算截断（构造大消息验证截断行为）、token 预算为 0 时等价于原行为
  - [ ] 运行 `cargo test -p nanobot-session` 验证通过

### 5. ✅ Agent loop 集成 token 预算

- 优先级: P0
- 依赖项: 3, 4
- 涉及文件: `crates/agent/src/loop/mod.rs`, `crates/agent/src/loop/tests.rs`
- 验收标准: `process_message()` 传递 `max_input_tokens` 到 `get_history()`；ReAct 循环每次 `call_llm()` 前检查总 token，超出时从历史部分丢弃旧消息；整合触发改为 token-based
- 风险/注意点: 整合触发条件变更需要同步修改 `try_consolidate()` 函数签名
- 信心评估: 4
- 步骤:
  - [ ] 在 `process_message()` 中，将 `session.get_history(self.config.memory_window, &mut history)` 改为 `session.get_history(self.config.memory_window, self.config.max_input_tokens, &mut history)`
  - [ ] 在 `re_act()` 方法中，进入主循环前记录 `turn_start = messages.len()`（系统提示 + 历史 + 用户消息的总数，即当前轮次的起始边界）
  - [ ] 在每次 `call_llm()` 调用前，新增 token 检查：计算 `messages` 总 token（`messages.iter().map(|m| m.token_len()).sum::<usize>()`），如果超过 `self.config.max_input_tokens`，从 index 1 开始逐条移除消息（`messages.remove(1)`），每次移除后 `turn_start -= 1`，直到总 token 降到预算内或 `turn_start <= 1`（仅剩系统提示）
  - [ ] 修改 `try_consolidate()` 函数：将触发条件从 `messages.len() - last_consolidated >= memory_window` 改为基于 token 的条件——计算未整合消息的总 token，当超过 `max_input_tokens / 2` 时触发
  - [ ] 更新 `try_consolidate()` 签名，新增 `max_input_tokens: usize` 参数，在 `process_message()` 调用处传入
  - [ ] 更新 `crates/agent/src/loop/tests.rs` 中所有受影响的测试（`get_history` 调用签名变更、`try_consolidate` 签名变更）
  - [ ] 运行 `cargo test -p nanobot-agent` 验证通过

### 6. ✅ 更新 AGENTS.md 文档

- 优先级: P1
- 依赖项: 5
- 涉及文件: `crates/config/AGENTS.md`, `crates/session/AGENTS.md`, `crates/agent/AGENTS.md`
- 验收标准: 文档反映新增的 `max_input_tokens` 配置、token-based 历史截断、ReAct 循环 token 检查
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `crates/config/AGENTS.md`：在 `AgentDefaults` 描述中添加 `max_input_tokens` 字段
  - [ ] `crates/session/AGENTS.md`：更新 `get_history()` 方法签名说明
  - [ ] `crates/agent/AGENTS.md`：在 `re_act()` 描述中说明 token 检查行为

## 实现建议

- Token 估算使用 `text.len() / 4`（字节数除以 4），简单高效，后续可替换为精确 tokenizer
- `get_history()` 的 token 截断应从最新消息向前扫描（保留最近的上下文），而非从最旧消息向后扫描
- ReAct 循环内的 token 检查应保留系统提示（index 0）和当前轮次的所有消息（`turn_start_idx..`），只从历史部分（`1..turn_start_idx`）丢弃
- 整合触发的 token 阈值使用 `max_input_tokens / 2`，与上游的 `compression_start_ratio = 0.5` 对齐
- `memory_window` 配置保留作为消息计数的硬上限，与 token 预算取较严格的限制
