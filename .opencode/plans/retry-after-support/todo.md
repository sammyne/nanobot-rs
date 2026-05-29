# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | ProviderError::RateLimit 新增 retry_after 字段 |
| `crates/provider/src/anthropic/mod.rs` | 修改 | 429 时从响应头提取 retry-after |
| `crates/provider/src/auto_retry/mod.rs` | 修改 | 优先使用 retry_after 等待 |
| `crates/provider/src/auto_retry/tests.rs` | 修改 | 新增 retry-after 测试 |

## 任务列表

### 1. ProviderError::RateLimit 新增 retry_after

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`
- 验收标准: `ProviderError::RateLimit` 携带 `retry_after: Option<Duration>`
- 信心评估: 5
- 步骤:
  - [ ] `ProviderError::RateLimit` 从 `RateLimit(String)` 改为 `RateLimit { message: String, retry_after: Option<Duration> }`
  - [ ] 更新 `is_transient()` 保持 RateLimit 返回 true
  - [ ] 修复所有构造 `RateLimit` 的地方
  - [ ] `cargo check -p nanobot-provider` 验证

### 2. Anthropic provider 提取 retry-after

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: 429 响应时从 HTTP 头提取 retry-after 值
- 信心评估: 4
- 步骤:
  - [ ] 在 429 错误处理分支中，从 `response.headers()` 提取 `retry-after` 头
  - [ ] 解析为秒数（支持整数格式，如 `retry-after: 20`）
  - [ ] 构造 `ProviderError::RateLimit { message, retry_after: Some(Duration::from_secs(n)) }`
  - [ ] `cargo check -p nanobot-provider` 验证

### 3. AutoRetryProvider 使用 retry_after

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/auto_retry/mod.rs`, `crates/provider/src/auto_retry/tests.rs`
- 验收标准: 重试等待时优先使用 ProviderError 中的 retry_after 值
- 信心评估: 5
- 步骤:
  - [ ] 在重试分支中，从 `ProviderError::RateLimit { retry_after, .. }` 提取 retry_after
  - [ ] 如果有 retry_after，使用该值作为等待时间；否则回退到指数退避
  - [ ] 新增测试：有 retry_after 时使用指定等待时间
  - [ ] 新增测试：无 retry_after 时回退到指数退避
  - [ ] `cargo test -p nanobot-provider` 验证

## 实现建议

- `retry-after` 头解析：只需支持整数秒格式（`retry-after: 20`），HTTP-date 格式暂不支持
- AutoRetryProvider 的等待逻辑：`let delay = retry_after.unwrap_or_else(|| Duration::from_secs(1 << attempt));`
- OpenAI provider 不做改动：async-openai 内置 backoff 处理 429，虽然不解析 Retry-After 头，但基本的指数退避已覆盖
