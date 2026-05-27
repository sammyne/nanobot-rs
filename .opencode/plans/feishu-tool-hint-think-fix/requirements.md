# 需求

## 目标与背景

两个独立但相关的小问题：

1. **空 think 标签泄漏**：某些模型（如 DeepSeek）会输出孤立的 `</think>` 标签（无匹配的 `<think>`）。当前 `strip_think` 正则 `<think>[\s\S]*?</think>` 只匹配成对标签，孤立的 `</think>` 会泄漏到最终回复中。

2. **飞书工具提示可读性差**：工具调用提示以普通 markdown 文本发送，多个工具调用挤在一行，不易阅读。Python 版将其格式化为代码块，每个工具调用独占一行。

## 方案比较（强制）

### 方案 1: 正则扩展 + feishu send 格式化（最小可行版 + 理想架构）

- 思路: `strip_think` 正则增加孤立 `</think>` 匹配；feishu `send()` 中检查 `is_tool_hint()` 并将内容包裹为代码块
- 优点: 改动最小，各自独立
- 缺点: 无
- 工作量估算: S

### 方案 2: 在 agent loop 层统一格式化

- 思路: `format_tool_hint` 直接输出 markdown 代码块格式
- 优点: 所有 channel 统一受益
- 缺点: 钉钉等其他 channel 可能不支持代码块渲染；代码块格式对 CLI 模式不友好
- 工作量估算: S

### 推荐

方案 1。think 标签修复是通用的，代码块格式化是飞书特有的 UX 优化，不应影响其他 channel。

## 功能需求列表

### 核心功能

1. `strip_think` 正则扩展：匹配孤立的 `</think>`（无 `<think>` 前缀）
2. 飞书 `send()` 中：当 `is_tool_hint()` 为 true 时，将内容格式化为代码块显示

## 非功能需求

- **向后兼容**：不影响其他 channel 的工具提示显示

## 边界与不做事项

- 不修改 `format_tool_hint` 的输出格式（保持 channel 无关）
- 不处理其他 channel 的工具提示格式化

## 待确认事项

- 无
