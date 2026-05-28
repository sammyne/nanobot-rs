# 需求

## 目标与背景

将上游 Python 版的 Hook 生命周期钩子系统迁移到 Rust 版。当前 Rust 版仅有一个简单的 `ProgressTracker` trait（`track(content, is_tool_hint)`），只在 `re_act` 循环中工具调用前的一个点被调用。上游已扩展为 6 个生命周期钩子点 + CompositeHook 组合器，支持在 agent 循环的关键节点注入自定义行为。

**现状不足**：
- 只有进度通知一个扩展点，无法在迭代前后、工具执行前、最终内容处理等环节注入逻辑
- 后续功能（如 token usage 追踪、`<think>` 标签剥离、Dream 记忆整合触发）都需要在不同生命周期点插入行为，目前只能硬编码在 `re_act` 中

## 方案比较（强制）

### 方案 1: 最小侵入式 — Hook trait + LoopHook 适配（最小可行版）

- 思路: 保留 ProgressTracker，在内部包装为 LoopHook
- 优点: 对外 API 不变
- 缺点: 两套 trait 并存，概念冗余
- 工作量估算: S

### 方案 2: 完全替换 — 用 Hook 取代 ProgressTracker（理想架构）✅ 已选定

- 思路: 删除 `ProgressTracker` trait 和 `ChannelProgressTracker`，统一为 `Hook`。所有调用方直接构造 hook 实例。
- 优点: 概念统一，无冗余抽象
- 缺点: 破坏现有公共 API，需修改 nanobot binary crate
- 工作量估算: M

### 推荐

方案 2（用户已选定）。

## 功能需求列表

### 核心功能

- 定义 `Hook` trait，包含 4 个生命周期方法（均有默认空实现）：
  - `before_iteration` — 每次 LLM 调用前
  - `before_execute_tools` — 工具执行前（发送 tool hint）
  - `after_iteration` — 每轮迭代后（用于 usage 追踪等）
  - `finalize_content` — 最终内容变换（用于 strip `<think>` 等）
- 定义 `HookCtx` 结构体，携带当前迭代的上下文信息
- 实现 `CompositeHook`，将多个 hook 组合调用，异步方法有错误隔离
- 实现 `LoopHook`，直接持有 `mpsc::Sender<OutboundMessage>` + channel/chat_id，在 `before_execute_tools` 中发送思考内容和工具提示
- 删除 `ProgressTracker` trait、`ChannelProgressTracker` 和整个 `progress` 模块
- 重构 `re_act` 方法，在各生命周期点调用 hook
- 修改 `process_direct` 签名，接受 `Option<Arc<dyn Hook>>` 替代 `Option<Arc<dyn ProgressTracker>>`
- 修改 nanobot binary crate 的 `AgentCmd`，直接构造 hook 实例

## 非功能需求

- **可维护性**: hook 模块独立，测试文件分离到 `tests.rs`
- **测试要求**: 新增 hook 模块单元测试；现有 agent 测试全部通过

## 边界与不做事项

- 不实现流式输出接口（`on_stream`/`on_stream_end`），不预留
- 不修改 gateway 命令中的 hook 构造（gateway 走 `run()` 方法，内部自动构造 `LoopHook`）

## 假设与约束

- **技术假设**: `Hook` trait 使用 `async_trait` 宏
- **影响范围**: agent crate（主要）+ nanobot binary crate（AgentCmd 调用处）

## 待确认事项

无
