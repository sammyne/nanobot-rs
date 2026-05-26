# 需求

## 目标与背景

当前 `Provider::chat()` 调用在遇到瞬态错误（429 限流、5xx 服务端错误、超时）时直接失败，没有重试机制。

现状分析：
- **OpenAI 路径**：`async-openai` 库内部已有 `backoff` 重试（429 和 5xx），无需额外处理。
- **Anthropic 路径**：使用原生 `reqwest`，**完全没有重试逻辑**。429 和 5xx 直接返回 `ProviderError::Api(String)`。
- **`ProviderError` 枚举**：只有 `Api(String)` / `Timeout` / `Config(String)` 三个变体，无法区分限流、服务端错误等瞬态故障。

对应上游 PR：HKUDS/nanobot#1512（feat: add LLM retry with exponential backoff for transient errors）。

## 方案比较（强制）

### 方案 1: AutoRetryProvider 装饰器包装 AnthropicLike（最小可行版）

- 思路: 新增 `AutoRetryProvider<P: Provider>` 包装 Provider，在 `chat()` 中实现重试循环。扩展 `ProviderError` 以支持瞬态错误识别。在 `AnyProvider` 中仅对 `AnthropicLike` 使用 `AutoRetryProvider` 包装，`OpenAILike` 保持原样。
- 优点:
  - 不修改 `AgentLoop`，调用方无感知
  - OpenAI 路径不会双重重试
  - 重试逻辑与具体 provider 实现解耦
- 缺点: 无明显缺点
- 工作量估算: S

### 方案 2: 在 Provider trait 中添加默认方法（理想架构）

- 思路: 在 `Provider` trait 上添加 `fn chat_with_retry()` 默认实现，内部调用 `self.chat()` 并重试。
- 优点:
  - 所有 Provider 自动获得重试能力
  - 不需要额外的包装类型
- 缺点:
  - 污染 trait 接口，混合了策略和能力
  - 默认方法中无法灵活配置重试参数（除非加更多 trait 方法）
  - 调用方需要显式选择调用 `chat()` 还是 `chat_with_retry()`
- 工作量估算: S

### 推荐

方案 1。装饰器模式符合单一职责原则，不污染 Provider trait，仅对需要重试的 Anthropic 路径生效，OpenAI 路径保持原样避免双重重试。

## 功能需求列表

### 核心功能

1. **扩展 `ProviderError`**：新增 `RateLimit(String)` 和 `ServerError(String)` 变体，添加 `is_transient()` 方法
2. **Anthropic provider 错误分类**：根据 HTTP 状态码返回对应的 `ProviderError` 变体（429 → `RateLimit`，5xx → `ServerError`）
3. **`AutoRetryProvider<P>`**：实现 `Provider` trait，在 `chat()` 中封装指数退避重试（最多 3 次，间隔 1s/2s/4s），仅对 `is_transient()` 为 true 的错误重试
4. **调整 `AnyProvider`**：将 `Anthropic` 变体的内部类型从 `AnthropicLike` 改为 `AutoRetryProvider<AnthropicLike>`，`OpenAI` 变体保持 `OpenAILike` 不变

### 扩展功能

- 无

## 非功能需求

- **性能**：重试间隔使用 `tokio::time::sleep`，不阻塞其他任务
- **安全**：无新增安全考量
- **兼容性**：对 `AgentLoop` 等调用方透明，`AnyProvider` 仍然实现 `Provider` trait
- **可维护性**：重试逻辑集中在 `auto_retry` 模块，与具体 provider 实现解耦
- **测试要求**：使用 mock provider 测试重试行为（瞬态错误重试成功、永久错误立即返回、重试次数耗尽）

## 边界与不做事项

- 不修改 `AgentLoop::call_llm()` 或其他调用方
- 不修改 `Provider` trait 签名
- 不修改 OpenAI provider 的错误处理（async-openai 内部已有重试）
- 不添加可配置的重试策略（固定 3 次 + 指数退避）

## 假设与约束

- **技术假设**：`anyhow::Error` 支持 `downcast_ref::<ProviderError>()` 来识别错误类型
- **资源约束**：无
- **环境约束**：无

## 待确认事项

- 无
