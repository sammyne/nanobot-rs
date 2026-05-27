# 需求

## 目标与背景

当前内存整合（`consolidate_internal`）调用 LLM 时使用硬编码的 `Options::default()`（max_tokens=4096, temperature=0.7），没有继承 agent 配置的 temperature、max_tokens、reasoning_effort。这导致整合 LLM 调用的行为与主循环不一致。

对应上游 PR：HKUDS/nanobot#1868（fix(memory): pass temperature, max_tokens and reasoning_effort to memory consolidation）。

## 方案

构造时传入 Options。`MemoryStore::new(workspace, options)` 和 `ContextBuilder::new(workspace, options)` 签名新增 `options` 参数，`consolidate_internal` 使用 `self.options` 而非 `Options::default()`。

优点：
- Options 从创建起就正确，不可能遗漏
- 无需 RwLock 内部可变性
- 简单直接

## 功能需求列表

### 核心功能

1. `MemoryStore` 新增 `options: nanobot_provider::Options` 字段，`new()` 签名改为 `new(workspace, options)`
2. `consolidate_internal` 中 `Options::default()` 替换为 `self.options`
3. `ContextBuilder::new()` 签名改为 `new(workspace, options)`，透传给 `MemoryStore::new()`
4. `AgentLoop::new()` 构造 Options 传入 `ContextBuilder::new()`
5. 所有测试调用方传入 `Options::default()`

## 边界与不做事项

- 不修改 `try_consolidate()` / `consolidate()` 签名
- 不引入内部可变性

## 待确认事项

- 无
