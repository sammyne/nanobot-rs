# 需求

## 目标与背景

provider crate 中多处类型转换存在过深嵌套（7-10 层缩进），主要原因是 `ContentPart` 和 `UserContent` 到各 provider 格式的转换逻辑内联在大 match 块中。通过提取独立的 `From`/`Into` 实现，可以将每个转换分支简化为一行调用。

## 方案比较（强制）

### 方案 1: 提取 From 实现 ✅ 已选定

- 思路: 为 `ContentPart` 和 `UserContent` 到各 provider 类型的转换实现 `From` trait
- 优点: 惯用 Rust 模式，每个转换独立可测，主函数扁平化
- 缺点: 无
- 工作量估算: S

### 方案 2: 提取辅助函数

- 思路: 提取为 `fn convert_content_part(...)` 等函数
- 优点: 也能减少嵌套
- 缺点: 不如 `From` 惯用，无法用 `.into()` 语法
- 工作量估算: S

### 推荐

方案 1。

## 功能需求列表

### A. OpenAI provider（openai/mod.rs）

- `From<&ContentPart> for ChatCompletionRequestUserMessageContentPart`
- `From<&UserContent> for ChatCompletionRequestUserMessageContent`
- 简化 `TryFrom<&Message>` 的 `User` 分支为一行

### B. Anthropic provider（anthropic/mod.rs）

- `From<&ContentPart> for ContentBlock`（Anthropic 格式）
- 简化 `convert_messages()` 的 `User` 分支中 `Parts` 处理

### C. Provider base（base/mod.rs）

- 简化 `strip_images()` 中双层 `.map(match)` 为辅助方法或 `From` 实现

## 非功能需求

- 纯重构，行为不变
- 现有测试全部通过

## 边界与不做事项

- 不重构 DingTalk/Feishu 图片下载链（错误处理模式，不适合 From）
- 不重构 agent loop 的 tokio::select（控制流嵌套，不适合 From）

## 待确认事项

无
