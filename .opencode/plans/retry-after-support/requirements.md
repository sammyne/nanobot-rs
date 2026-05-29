# 需求

## 目标与背景

将上游 PR #2761 + 直接提交 #85-86 的 Retry-After 支持迁移到 Rust 版。当前 `AutoRetryProvider` 使用固定指数退避（1s, 2s, 4s），忽略 provider 返回的 `Retry-After` 头，导致 429 限流时可能过早重试。

**现状不足**：
- Anthropic 429 响应包含 `retry-after` 头（如 `retry-after: 20`），但 Rust 版不解析

### async-openai 的限制（备忘）

`async-openai` v0.28 的 429 处理存在以下问题（源码 `client.rs` 的 `execute_raw` 方法）：

1. **内置 backoff 重试**：429 在 `execute_raw` 内部被 `backoff` crate 拦截并重试，不会传播到 `create_byot` 调用方
2. **Retry-After 硬编码为 None**：`backoff::Error::Transient { err, retry_after: None }` 中 `retry_after` 始终为 `None`，未从响应头提取
3. **HTTP 头信息丢失**：`response.bytes().await` 消费了响应体后，HTTP 头不再可访问
4. **双重重试风险**：`async-openai` 内置 backoff + 我们的 `AutoRetryProvider` 形成双层重试，可能放大请求量

因此本次不改动 OpenAI provider。

## 方案比较（强制）

### 方案 1: 仅 Anthropic provider ✅ 已选定

- 思路: Anthropic provider 从 429 响应头提取 `retry-after`，AutoRetryProvider 优先使用该值
- 优点: 改动集中，Anthropic 获得精确等待
- 缺点: OpenAI 仍用 async-openai 内置 backoff（不解析 Retry-After）
- 工作量估算: S

### 方案 2: 同时改动 OpenAI provider

- 思路: 禁用 async-openai 内置 backoff 或绕过其 HTTP 层
- 优点: 统一重试策略
- 缺点: async-openai 不暴露 Retry-After 头，改动大
- 工作量估算: L

### 推荐

方案 1。

## 功能需求列表

### 核心功能

- `ProviderError::RateLimit` 新增 `retry_after: Option<Duration>` 字段
- Anthropic provider：429 时从响应头提取 `retry-after`
- `AutoRetryProvider`：重试等待时优先使用 `retry_after` 值，回退到指数退避

## 非功能需求

- 现有测试通过
- 新增 retry-after 相关测试

## 边界与不做事项

- 不改动 OpenAI provider（async-openai 内置 backoff 已处理 429）
- 不支持 HTTP-date 格式的 retry-after（仅支持整数秒）

## 待确认事项

无
