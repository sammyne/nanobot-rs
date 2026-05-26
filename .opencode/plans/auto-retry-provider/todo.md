# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/provider/src/base/mod.rs` | 修改 | `ProviderError` 新增 `RateLimit`/`ServerError` 变体 + `is_transient()` |
| `crates/provider/src/base/tests.rs` | 修改 | 新增 `is_transient()` 测试 |
| `crates/provider/src/anthropic/mod.rs` | 修改 | HTTP 429 → `RateLimit`，5xx → `ServerError` |
| `crates/provider/src/auto_retry/mod.rs` | 新增 | `AutoRetryProvider<P>` 实现 |
| `crates/provider/src/auto_retry/tests.rs` | 新增 | 重试行为测试 |
| `crates/provider/src/lib.rs` | 修改 | 导出 `AutoRetryProvider` |
| `crates/provider/src/any/mod.rs` | 修改 | `Anthropic` 变体改为 `AutoRetryProvider<AnthropicLike>` |
| `crates/provider/src/any/tests.rs` | 修改 | 适配新的变体类型 |
| `crates/provider/AGENTS.md` | 修改 | 文档更新 |

## 任务列表

### 1. ✅ 扩展 ProviderError 枚举

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/provider/src/base/mod.rs`, `crates/provider/src/base/tests.rs`
- 验收标准: `ProviderError` 包含 `RateLimit`/`ServerError` 变体，`is_transient()` 对 `Timeout`/`RateLimit`/`ServerError` 返回 true，对 `Api`/`Config` 返回 false
- 风险/注意点: 新增变体不影响现有 match 分支（现有代码通过 `?` 传播错误，不做 match）
- 信心评估: 5
- 步骤:
  - [ ] 在 `crates/provider/src/base/mod.rs` 的 `ProviderError` 枚举中新增 `RateLimit(String)` 变体（`#[error("请求限流: {0}")]`）
  - [ ] 新增 `ServerError(String)` 变体（`#[error("服务端错误: {0}")]`）
  - [ ] 为 `ProviderError` 添加 `pub fn is_transient(&self) -> bool` 方法，对 `Timeout | RateLimit(_) | ServerError(_)` 返回 true
  - [ ] 在 `crates/provider/src/base/tests.rs` 末尾新增 `is_transient` 测试函数，覆盖所有 5 个变体的返回值
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过

### 2. ✅ Anthropic provider 错误分类

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/anthropic/mod.rs`
- 验收标准: Anthropic provider 对 HTTP 429 返回 `ProviderError::RateLimit`，对 5xx 返回 `ProviderError::ServerError`，其他非成功状态码保持返回 `ProviderError::Api`
- 风险/注意点: 需要在现有的 `if !response.status().is_success()` 分支中细化判断，保持错误消息格式一致
- 信心评估: 5
- 步骤:
  - [ ] 在 `crates/provider/src/anthropic/mod.rs` 第 275-280 行的 `if !response.status().is_success()` 块中，将单一的 `ProviderError::Api` 替换为三路判断：
    - `status == 429` → `ProviderError::RateLimit(error_msg)`
    - `status.is_server_error()` → `ProviderError::ServerError(format!("HTTP {status}: {error_msg}"))`
    - 其他 → `ProviderError::Api(format!("HTTP {status}: {error_msg}"))` （保持不变）
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过
  - [ ] 运行 `cargo clippy -p nanobot-provider -- -D warnings -D clippy::uninlined_format_args` 验证无警告

### 3. ✅ 实现 AutoRetryProvider

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/provider/src/auto_retry/mod.rs`（新增）, `crates/provider/src/auto_retry/tests.rs`（新增）, `crates/provider/src/lib.rs`
- 验收标准: `AutoRetryProvider<P>` 实现 `Provider` trait；瞬态错误最多重试 3 次（间隔 1s/2s/4s）后返回最后一次错误；永久错误立即返回；成功时立即返回
- 风险/注意点: 需要通过 `anyhow::Error::downcast_ref::<ProviderError>()` 判断错误类型；如果 downcast 失败（非 ProviderError），应视为永久错误不重试
- 信心评估: 4
- 步骤:
  - [ ] 创建 `crates/provider/src/auto_retry/mod.rs`，定义 `AutoRetryProvider<P: Provider>` 结构体，字段：`inner: P`, `max_retries: usize`
  - [ ] 实现 `AutoRetryProvider::new(inner: P) -> Self`，默认 `max_retries = 3`
  - [ ] 实现 `Provider` trait 的 `chat()` 方法：循环 `0..=max_retries`，调用 `self.inner.chat()`，成功立即返回；失败时 downcast 为 `ProviderError`，若 `is_transient()` 且未达最大次数则 `tokio::time::sleep(Duration::from_secs(1 << attempt))` 后继续；否则立即返回错误
  - [ ] 实现 `Provider` trait 的 `bind_tools()` 方法：直接委托给 `self.inner.bind_tools(tools)`
  - [ ] 为 `AutoRetryProvider` 派生 `Clone`（`P: Provider` 已要求 `Clone`）
  - [ ] 在 `crates/provider/src/lib.rs` 中添加 `mod auto_retry;` 和 `pub use auto_retry::AutoRetryProvider;`
  - [ ] 创建 `crates/provider/src/auto_retry/tests.rs`，在 `mod.rs` 末尾添加 `#[cfg(test)] mod tests;`
  - [ ] 编写测试：定义 `MockProvider` 结构体（实现 `Provider` trait），通过 `Arc<AtomicUsize>` 计数调用次数，可配置前 N 次返回瞬态错误、之后返回成功
  - [ ] 测试用例 1：`transient_error_retries_then_succeeds` — mock 前 2 次返回 `ProviderError::RateLimit`，第 3 次成功，验证调用 3 次且返回成功
  - [ ] 测试用例 2：`permanent_error_no_retry` — mock 返回 `ProviderError::Api`，验证只调用 1 次
  - [ ] 测试用例 3：`retries_exhausted_returns_last_error` — mock 始终返回 `ProviderError::ServerError`，验证调用 4 次（1 + 3 重试）且返回错误
  - [ ] 测试用例 4：`non_provider_error_no_retry` — mock 返回 `anyhow!("unknown")`（非 ProviderError），验证只调用 1 次
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过

### 4. ✅ 调整 AnyProvider 集成 AutoRetryProvider

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/provider/src/any/mod.rs`, `crates/provider/src/any/tests.rs`
- 验收标准: `AnyProvider::Anthropic` 变体内部类型为 `AutoRetryProvider<AnthropicLike>`；`from_config()` 对 Anthropic 路径用 `AutoRetryProvider::new()` 包装；`OpenAI` 变体保持不变；现有测试适配后通过
- 风险/注意点: 变体类型变更后 `chat()` 和 `bind_tools()` 的 match 分支需要同步更新
- 信心评估: 5
- 步骤:
  - [ ] 在 `crates/provider/src/any/mod.rs` 中导入 `AutoRetryProvider`
  - [ ] 将 `Anthropic(AnthropicLike)` 变体改为 `Anthropic(AutoRetryProvider<AnthropicLike>)`
  - [ ] 在 `from_config()` 的 `ProvidersConfig::Anthropic` 分支中，将 `AnthropicLike::new(pc, model)?` 包装为 `AutoRetryProvider::new(AnthropicLike::new(pc, model)?)`
  - [ ] `chat()` 和 `bind_tools()` 的 `Self::Anthropic(p)` 分支无需修改（`AutoRetryProvider` 实现了 `Provider` trait）
  - [ ] 更新 `crates/provider/src/any/tests.rs` 中 `from_config_anthropic_creates_anthropic` 测试的 `matches!` 断言，适配新的变体类型
  - [ ] 运行 `cargo test -p nanobot-provider` 验证通过

### 5. ✅ 更新 provider crate 的 AGENTS.md

- 优先级: P1
- 依赖项: 4
- 涉及文件: `crates/provider/AGENTS.md`
- 验收标准: AGENTS.md 中记录 `AutoRetryProvider`、`ProviderError` 新增变体
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在关键类型列表中添加 `AutoRetryProvider<P: Provider>` — 装饰器，为内部 Provider 添加指数退避重试（仅瞬态错误）
  - [ ] 更新 `ProviderError` 描述，补充 `RateLimit`、`ServerError` 变体和 `is_transient()` 方法
  - [ ] 更新 `AnyProvider` 描述，说明 `Anthropic` 变体使用 `AutoRetryProvider<AnthropicLike>`

## 实现建议

- `AutoRetryProvider` 的重试间隔使用 `1 << attempt` 秒（即 1s, 2s, 4s），与上游 Python 版本一致
- 测试中使用 `tokio::time::pause()` 加速 sleep，避免测试实际等待秒级延迟
- `downcast_ref::<ProviderError>()` 失败时（非 ProviderError 的 anyhow 错误）视为永久错误，不重试
