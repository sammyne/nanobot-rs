# 需求

## 目标与背景

`inject_runtime_context()`（`crates/context/src/builder/mod.rs:338`）在每次构建 LLM 请求时，将运行时上下文（当前时间、channel、chat_id）以 `[Runtime Context]` 块的形式追加到用户消息末尾。这些信息仅对当前请求有意义。

`save_turn()`（`crates/session/src/session.rs:118`）保存用户消息时只做了图片剥离（`strip_images`），没有剥离 runtime context 块。导致每轮对话的 `[Runtime Context]` 在 session 历史中累积，回放时浪费 context window token。

上游 Python 版 PR #1222 修复了同一问题，但方案不同：Python 版将 runtime context 作为独立 user 消息注入，过滤时按整条消息跳过；nanobot-rs 将 runtime context 追加到用户消息文本内，需要从文本中剥离。

## 方案比较（强制）

### 方案 1: 在 save_turn 中剥离 runtime context 文本（最小可行版）

- 思路: 新增 `strip_runtime_context()` 函数，在 `save_turn()` 处理 `Message::User` 时，先 `strip_images` 再 `strip_runtime_context`，将 `\n\n[Runtime Context]\n...` 后缀从用户消息文本中移除
- 优点: 改动集中在 session crate 一个文件，与现有 `strip_images` 模式一致
- 缺点: `[Runtime Context]` 标记字符串在 context crate 和 session crate 中各出现一次，需保持同步
- 工作量估算: S

### 方案 2: 将 runtime context 改为独立消息注入，save_turn 按消息过滤（理想架构）

- 思路: 重构 `inject_runtime_context()` 为独立 user 消息（对齐 Python 版 PR #1126），`save_turn()` 按消息标记过滤
- 优点: 架构与上游对齐，过滤逻辑更干净
- 缺点: 改动范围大（context crate 重构 + session crate + agent crate 调用链），且 PR #1126 本身在 nanobot-rs 中已被评估为收益有限（第 5 条，已跳过）
- 工作量估算: M

### 推荐

推荐方案 1。改动最小，与现有 `strip_images` 模式完全一致，解决实际问题。标记字符串重复的风险可通过注释标注缓解。

## 功能需求列表

### 核心功能

- 新增 `strip_runtime_context()` 函数，从 `UserContent` 中移除 `[Runtime Context]` 块
  - `UserContent::Text`: 查找 `\n\n[Runtime Context]\n` 并截断其后所有内容
  - `UserContent::Parts`: 移除末尾以 `\n\n[Runtime Context]` 开头的文本 part
- `save_turn()` 中 `Message::User` 分支调用 `strip_runtime_context` 后再保存

### 扩展功能

- 无

## 非功能需求

- **性能**: 字符串查找为 O(n)，用户消息通常很短，无性能顾虑
- **兼容性**: 已持久化的历史消息不受影响（只影响新保存的消息）
- **测试要求**: 覆盖 Text 和 Parts 两种变体，以及无 runtime context 时的 no-op 场景

## 边界与不做事项

- 不重构 `inject_runtime_context()` 的注入方式（保持追加到用户消息内）
- 不清理已持久化的历史 session 文件中的 runtime context
- 不将 `[Runtime Context]` 标记提取为跨 crate 共享常量（遵循 Simplicity First）

## 假设与约束

- **技术假设**: `[Runtime Context]` 标记不会出现在用户正常输入中（极低概率，且误剥离影响可忽略）

## 待确认事项

- 无
