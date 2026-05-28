# 需求

## 目标与背景

将上游 #74（`210643e`）的 reasoning_content 提取功能迁移到 Rust 版。部分 OpenAI 兼容模型（Kimi、DeepSeek-R1、MiMo）在响应中返回 `reasoning_content` 字段，包含模型的思考过程。当前 Rust 版 OpenAI provider 使用 `async-openai` 库的强类型响应，该库不包含 `reasoning_content` 字段，导致思考过程被静默丢弃。

**方案**：启用 `async-openai` 的 `byot`（Bring Your Own Types）feature，使用 `create_byot()` 获取 `serde_json::Value` 响应，从原始 JSON 中提取 `reasoning_content`。

## 方案比较（强制）

### 方案 1: 使用 async-openai byot feature ✅ 已选定

- 思路: 启用 `byot` feature，用 `create_byot()` 替代 `create()`，响应类型改为 `serde_json::Value`，手动提取所有字段
- 优点: 不绕过 async-openai，复用其 HTTP/重试/认证逻辑；能提取任意非标准字段
- 缺点: 需要手动解析 JSON 响应（失去强类型保证）
- 工作量估算: M

### 方案 2: 绕过 async-openai 用 reqwest 直接调用

- 思路: 不用 async-openai 的 chat API，直接用 reqwest 发 HTTP 请求
- 优点: 完全控制请求和响应
- 缺点: 需要重写 HTTP 调用、认证、重试逻辑
- 工作量估算: L

### 推荐

方案 1。

## 功能需求列表

### 核心功能

- 启用 `async-openai` 的 `byot` feature
- `OpenAILike::chat()` 改用 `create_byot()` 获取 `Value` 响应
- 从 JSON 响应中提取 `reasoning_content`
- 有 `reasoning_content` 时存入 `Message::Assistant { thinking }` 字段（与 Anthropic thinking 复用同一字段）

## 非功能需求

- 向后兼容：无 `reasoning_content` 时行为不变
- 现有测试通过

## 边界与不做事项

- 不实现流式 reasoning_content 提取（当前 Rust 版不支持流式）

## 待确认事项

无
