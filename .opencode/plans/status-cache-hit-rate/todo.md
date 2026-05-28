# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | Usage 新增 cached_tokens 字段 |
| `crates/provider/src/anthropic/mod.rs` | 修改 | 提取 cache_read_input_tokens |
| `crates/provider/src/openai/mod.rs` | 修改 | 提取 prompt_tokens_details.cached_tokens |
| `crates/agent/src/cmd/status/mod.rs` | 修改 | 显示缓存命中率 |
| `crates/agent/src/cmd/status/tests.rs` | 修改 | 新增缓存率测试 |

## 任务列表

### 1. Usage 新增 cached_tokens 字段

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`
- 验收标准: `cargo check -p nanobot-provider` 通过
- 信心评估: 5
- 步骤:
  - [x] 在 `Usage` 结构体中添加 `pub cached_tokens: Option<u64>`
  - [x] 确保 `Default` derive 仍然正常（Option 默认 None）
  - [x] `cargo check -p nanobot-provider` 验证

### 2. Anthropic provider 提取 cached_tokens

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: Anthropic 响应中的 cache_read_input_tokens 被正确提取到 Usage.cached_tokens
- 信心评估: 5
- 步骤:
  - [x] 在 `AnthropicUsage` 结构体中添加 `cache_read_input_tokens: Option<u64>`
  - [x] 在 `parse_response` 中将 `cache_read_input_tokens` 映射到 `Usage.cached_tokens`
  - [x] 更新现有测试验证 cached_tokens 提取
  - [x] `cargo test -p nanobot-provider` 验证

### 3. OpenAI provider 提取 cached_tokens

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/openai/mod.rs`
- 验收标准: OpenAI 响应中的 prompt_tokens_details.cached_tokens 被正确提取
- 信心评估: 3（需确认 async-openai 库是否暴露 prompt_tokens_details）
- 步骤:
  - [x] 检查 async-openai 库的 CompletionUsage 结构体是否有 prompt_tokens_details
  - [x] 如果有，提取 cached_tokens；如果没有，跳过此步骤
  - [x] `cargo check -p nanobot-provider` 验证

### 4. /status 命令显示缓存命中率

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/agent/src/cmd/status/mod.rs`, `crates/agent/src/cmd/status/tests.rs`
- 验收标准: /status 输出中当 cached_tokens 存在时显示 "(XX% cached)"
- 信心评估: 5
- 步骤:
  - [x] 修改 StatusCmd::run 中的 tokens 行，当 cached_tokens > 0 时追加 "(XX% cached)"
  - [x] 缓存率计算: cached_tokens * 100 / input_tokens
  - [x] 新增测试：有 cached_tokens 时显示百分比
  - [x] 新增测试：无 cached_tokens 时不显示
  - [x] `cargo test -p nanobot-agent` 验证

## 实现建议

- 缓存率格式与上游一致: `"12487 in / 265 out (82% cached)"`
- Anthropic 的 `cache_read_input_tokens` 是 prompt cache 读取的 token 数，直接作为 cached_tokens
- OpenAI 的 `prompt_tokens_details` 可能在 async-openai 库中未暴露，需要检查
