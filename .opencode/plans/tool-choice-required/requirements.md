# 需求

## 目标与背景

内存整合时，LLM 被要求调用 `save_memory` 工具保存摘要，但 `tool_choice` 默认为 auto，LLM 有时会返回纯文本而不调用工具，导致整合静默失败（`last_consolidated` 不前进，历史消息无限增长）。

虽然已有 `raw_archive` 降级策略兜底，但从源头强制 LLM 调用工具更可靠。

对应上游 PR：HKUDS/nanobot#1909（fix: force save_memory in consolidation）。

## 方案

在 `Options` 中新增 `tool_choice: Option<ToolChoice>` 字段，默认 `None`（不传 = API 默认 auto）。整合调用时设为 `Some(ToolChoice::Required)`。OpenAI/Anthropic provider 在构建请求时传递该参数。

对现有代码零影响：所有现有的 `Options::default()` 和 `call_llm()` 都不需要改。

## 功能需求列表

### 核心功能

1. 新增 `ToolChoice` 枚举：`Auto`、`Required`、`Named(String)`
2. `Options` 新增 `tool_choice: Option<ToolChoice>` 字段，默认 `None`
3. OpenAI provider：`chat()` 中根据 `options.tool_choice` 设置请求的 `tool_choice` 字段
4. Anthropic provider：`chat()` 中根据 `options.tool_choice` 设置请求的 `tool_choice` 字段
5. 整合调用（`consolidate_internal`）：构造 Options 时设置 `tool_choice: Some(ToolChoice::Required)`

## 边界与不做事项

- 不修改 `Provider` trait 签名
- 不修改 `AgentLoop::call_llm()`（保持 `None` = auto）

## 待确认事项

- 无
