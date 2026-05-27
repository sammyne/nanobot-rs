# 需求

## 目标与背景

heartbeat 的 Phase 1 `decide()` 让 LLM 判断 HEARTBEAT.md 中是否有需要执行的任务。但 prompt 中没有当前时间信息，LLM 无法判断"每天 9 点执行"这类时间相关任务是否到期。

context builder 的 `inject_runtime_context` 已有时间注入逻辑，但 heartbeat 自建消息不经过 ContextBuilder。

## 方案比较（强制）

### 方案 1: decide() 中直接注入时间（最小可行版 + 理想架构）

- 思路: 在 `decide()` 的 user message 中追加当前时间，复用 context builder 的格式
- 优点: 一行改动，无新依赖（chrono 加到 heartbeat Cargo.toml 即可）
- 缺点: 时间格式与 context builder 不共享（但格式简单，重复可接受）
- 工作量估算: S

### 方案 2: 提取共享时间格式化函数到 utils crate

- 思路: 将时间格式化提取到 nanobot-utils，heartbeat 和 context 共用
- 优点: DRY
- 缺点: utils 需要新增 chrono 依赖；过度设计
- 工作量估算: S

### 推荐

方案 1。一行时间格式化不值得提取共享函数。

## 功能需求列表

### 核心功能

1. heartbeat `decide()` 的 user message 中注入当前时间（格式：`Current Time: YYYY-MM-DD HH:MM (Weekday) (timezone)`）

## 边界与不做事项

- 不提取共享时间格式化函数
- 不修改 context builder

## 待确认事项

- 无
