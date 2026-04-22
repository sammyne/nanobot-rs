# 需求

## 目标与背景

当前 `CronTool::description()` 方法返回的描述过于简略（`"Schedule reminders and recurring tasks. Actions: add, list, remove."`），无法有效引导 LLM 在收到用户请求时正确选择 action。

**典型问题**：用户发送"每天早上 10:00 帮我整理 harness 工程相关的新闻"（期望 `add` action + `cron` schedule），但 LLM 错误地调用了 `list` 或 `remove` action。

**核心原因**：`CronScheduleArgs` 各变体的使用场景（`cron` 用于 time-based，`every` 用于 interval-based）已在 JSON Schema 的 docstring 中有清晰说明，但 action 层面的选择指导缺失。

## 功能需求列表

### 核心功能

1. **改进 `CronTool::description()` 方法**
   - 扩展 action 描述，明确说明每个 action 的适用场景
   - 使用触发词（trigger phrases）引导 LLM 正确识别用户意图
   - 不重复 `CronScheduleArgs` JSON Schema 中已有的 schedule 变体技术细节（schedule 变体的描述由 `schemars` 根据 docstring 自动生成，已包含 `"0 8 * * *"` 等示例）

## 非功能需求

- **可读性**：description 应简洁、分层，便于 LLM 快速理解
- **一致性**：description 风格与项目中其他 Tool 保持一致
- **测试**：`cargo test -p nanobot-cron` 和 `cargo clippy` 均应通过

## 边界与不做事项

- **不做**：不修改 `CronScheduleArgs` 的 docstring（已在 commit `681d8d9` 中改进）
- **不做**：不修改 `handle_add`、`handle_list`、`handle_remove` 的业务逻辑
- **不做**：不修改 JSON Schema 生成逻辑
- **边界**：仅修改 `crates/cron/src/tool/mod.rs` 中 `CronTool::description()` 的返回值

## 假设与约束

- **技术假设**：`description()` 的内容会通过 `ToolDefinition` 传递给 LLM，影响 tool calling 决策
- **资源约束**：仅涉及单个方法的返回值修改
- **环境约束**：无特殊要求

## 修改清单

| 位置 | 修改类型 | 说明 |
|------|---------|------|
| `crates/cron/src/tool/mod.rs` line 177-179 | 修改 | 替换 `description()` 返回值为更详细的 action 选择指导 |
