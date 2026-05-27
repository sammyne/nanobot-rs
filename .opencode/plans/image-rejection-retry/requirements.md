# 需求

## 目标与背景

用户在聊天中发送图片后，agent 将图片 base64 编码放入消息发给 LLM。如果配置的模型不支持视觉输入（如纯文本模型、某些 OpenAI 兼容 API），API 返回错误，对话中断。

Python 版本（#75 主动过滤 → #76 反应式重试）最终采用反应式方案：先正常发送，检测到图片拒绝错误后自动移除图片重试。不需要维护"哪些模型支持视觉"的列表。

**Rust 版本现状**：
- `AutoRetryProvider` 仅重试瞬态错误（429 限流、5xx 服务端错误、超时）
- 图片拒绝属于 `ProviderError::Api`（400 Bad Request），被视为永久错误，直接返回
- 用户看到错误消息，对话中断

## 方案比较（强制）

### 方案 1: AutoRetryProvider 中添加图片拒绝检测（最小可行版 + 理想架构）

- 思路: 在 `AutoRetryProvider::chat()` 的错误处理中，瞬态错误重试之后，检测图片拒绝错误模式，strip images 后重试一次
- 优点: 逻辑集中在 provider 层，对上层透明；与现有重试机制自然融合
- 缺点: 需要 clone + 修改 messages（`chat` 接收 `&[Message]`，需要构造新 Vec）
- 工作量估算: S

### 方案 2: Agent loop 层捕获并重试

- 思路: 在 `re_act()` 中捕获图片拒绝错误，strip images 后重新调用 LLM
- 优点: 不修改 provider 层
- 缺点: agent loop 不应关心 provider 层的错误恢复；逻辑分散
- 工作量估算: S

### 推荐

方案 1。图片拒绝重试是 provider 层的错误恢复策略，与瞬态错误重试同级。

## 功能需求列表

### 核心功能

1. `ProviderError` 新增 `is_image_unsupported(&self) -> bool` 方法，基于错误消息中的关键词检测
2. `strip_images(messages: &[Message]) -> Option<Vec<Message>>` 函数：将 `ContentPart::Image` 替换为 `ContentPart::Text { text: "[image omitted]" }`，无图片时返回 `None`
3. `AutoRetryProvider::chat()` 中：瞬态重试循环结束后，若错误匹配图片拒绝，strip images 后重试一次

## 非功能需求

- **fail-safe**：strip 后重试仍失败则返回原始错误
- **日志**：图片拒绝检测和重试时 warn 级别日志

## 边界与不做事项

- 不做主动的模型视觉能力检测（不维护 supports_vision 列表）
- 图片重试仅一次，不进入指数退避循环

## 待确认事项

- 无
