# 需求

## 目标与背景

当前 Rust 版本的上下文管理完全基于消息计数（`memory_window` 默认 100 条），存在以下问题：

1. **无法精确控制输入大小**：不同消息长度差异巨大，消息计数无法反映实际 token 消耗
2. **整合触发不精确**：基于消息计数的整合阈值无法反映实际上下文压力

对应上游 PR：HKUDS/nanobot#1704（refactor: implement token-based context compression mechanism）。

现状分析（Rust 版本）：
- `Session::get_history(memory_window)` 按消息计数截取最近 N 条
- `save_turn()` 对工具结果截断到 500 字符（仅持久化时）
- 无 token 计数能力
- 整合触发条件：`messages.len() - last_consolidated >= memory_window`

## 方案

严格对齐 Python 上游实现。Token 控制**仅通过整合机制**实现，ReAct 循环内不做 token 检查，`get_history()` 不增加 token 参数。

### Python 上游行为总结

1. **ReAct 循环**：消息无限累积，不做任何 token 裁剪
2. **整合入口** `maybe_consolidate_by_tokens(session)`：
   - 估算完整 prompt 的 token 数（系统提示 + 历史 + 工具定义 + 探针消息）
   - 如果 `estimated < context_window_tokens`，不整合
   - 目标：压缩到 `context_window_tokens / 2`
   - 最多 5 轮渐进式整合
3. **切割点选择** `pick_consolidation_boundary(session, tokens_to_remove)`：
   - 从 `last_consolidated` 向前扫描，在 user 消息边界处切割
   - 确保移除足够的 token
4. **整合执行**：
   - 提取 `messages[last_consolidated..boundary]` 作为 chunk
   - 调用 LLM 用 `save_memory` 工具做摘要 → 写入 MEMORY.md + HISTORY.md
   - 推进 `session.last_consolidated = boundary`（消息不删除，只移指针）
   - 持久化 session
5. **降级策略**：连续 3 次 LLM 整合失败后，原文转储到 HISTORY.md

## 功能需求列表

### 核心功能

1. **Token 估算工具**：`estimate_tokens(text) -> usize`（已实现于 PR #117）
2. **Message::token_len()**：消息级 token 估算（已实现于 PR #117）
3. **新增配置项**：`max_input_tokens: usize`（默认 128000）（已实现于 PR #117）
4. **整合触发改为 token-based**：`try_consolidate()` 中估算完整 prompt token，超过 `max_input_tokens` 时触发整合
5. **多轮渐进式整合**：最多 5 轮，每轮在 user 消息边界切割一批消息，调用 LLM 做摘要，推进 `last_consolidated`，直到 prompt token 降到 `max_input_tokens / 2` 以下
6. **切割点选择**：`pick_consolidation_boundary()` 从 `last_consolidated` 向前扫描，在 user 消息边界处切割，确保移除足够 token
7. **降级策略**：连续 3 次 LLM 整合失败后，原文转储到 HISTORY.md（`_raw_archive`）

### 需要从 PR #117 回退的改动

8. **移除 `get_history()` 的 `max_tokens` 参数**：恢复原签名 `get_history(max_messages, buf)`
9. **移除 ReAct 循环内的 token 检查**：移除 `turn_start` 和 `messages.remove(1)` 逻辑

## 非功能需求

- **性能**：token 估算应为 O(n) 字符扫描，不引入外部依赖
- **安全**：无新增安全考量
- **兼容性**：`memory_window` 配置保留作为消息计数的硬上限
- **可维护性**：token 估算逻辑集中在 utils crate
- **测试要求**：整合触发测试、多轮整合测试、user 边界切割测试、降级策略测试

## 边界与不做事项

- 不引入 `tiktoken-rs` 或其他外部 tokenizer（使用字节估算）
- 不在 ReAct 循环内做 token 检查（与 Python 对齐）
- 不修改 `get_history()` 签名（与 Python 对齐）
- 不修改系统提示的大小控制

## 假设与约束

- **技术假设**：1 token ~= 4 字节对英文/代码足够准确
- **资源约束**：无
- **环境约束**：无

## 待确认事项

- 无
