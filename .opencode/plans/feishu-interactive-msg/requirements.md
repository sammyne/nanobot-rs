# 需求

## 目标与背景

飞书的交互式卡片消息（`message_type: "interactive"`）是一种常见的消息格式，用于发送结构化内容（带标题、按钮、链接、表格等）。当用户在飞书中转发卡片消息或通过卡片与 bot 交互时，bot 需要能提取其中的文本内容。

nanobot-rs 当前只支持 `text` 和 `image` 两种消息类型（`feishu/mod.rs:124`），interactive 消息被直接忽略。HKUDS/nanobot PR #1323 修复了 Python 版中 interactive 消息提取的双层嵌套遍历问题。

Python 版的 interactive 消息提取涉及两个递归函数：
- `_extract_interactive_content(content)` — 提取 title、双层嵌套 elements、card、header
- `_extract_element_content(element)` — 按 tag 类型提取单个元素内容（markdown、div、a、button、img、note、column_set、plain_text 等）

## 方案比较（强制）

### 方案 1: 完整移植 Python 版提取逻辑（理想架构）

- 思路: 将 `_extract_interactive_content()` 和 `_extract_element_content()` 完整移植为 Rust 函数，支持所有 tag 类型
- 优点:
  - 与 Python 版功能完全对齐
  - 覆盖所有卡片元素类型
- 缺点:
  - 代码量较大（Python 版两个函数约 80 行）
  - 部分 tag 类型（column_set、note 等）使用频率低
- 工作量估算: M

### 方案 2: 最小可行版——提取核心文本（最小可行版）

- 思路: 只提取 interactive 消息中最常见的文本内容（title、header、markdown/plain_text 元素），跳过复杂嵌套结构
- 优点:
  - 代码量小
  - 覆盖 80% 的实际使用场景
- 缺点:
  - button、column_set 等元素中的文本会丢失
  - 后续可能需要补充
- 工作量估算: S

### 推荐

方案 1。interactive 消息的元素类型有限且明确，完整移植一次到位，避免后续反复补充。Python 版的逻辑清晰，移植成本可控。

## 功能需求列表

### 核心功能

1. 在消息类型白名单中添加 `"interactive"`
2. 实现 `extract_interactive_content(content: &Value) -> String`，递归提取 title、elements（双层嵌套）、card、header
3. 实现 `extract_element_content(element: &Value) -> Vec<String>`，按 tag 类型提取：markdown/lark_md、div、a、button、img、note、column_set、plain_text，以及 fallback（遍历 elements 子数组）

### 扩展功能

- 无

## 非功能需求

- **兼容性**：content 字段可能是 JSON 字符串或已解析的 JSON 对象，需兼容两种情况
- **健壮性**：对缺失字段静默跳过；对非预期类型（如 elements 不是数组、element 不是对象等）打印 `warn!` 日志后跳过，不 panic

## 边界与不做事项

- 不处理 share_chat、share_user、share_calendar_event 等其他非文本消息类型（保持现有忽略行为）
- 不修改消息发送逻辑（发送 interactive 消息已有实现）

## 假设与约束

- **技术假设**：飞书 interactive 消息的 elements 是二维数组（行列表，每行是元素列表），符合飞书官方文档
- **数据格式**：content 字段是 JSON 字符串，需先 `serde_json::from_str` 解析

## 待确认事项

无
