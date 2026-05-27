# 需求

## 目标与背景

当前 Rust 版本的文件系统和 Shell 工具功能较基础，存在以下问题：

1. **ReadFileTool 无分页**：只能读取整个文件（截断到 128K 字符），无法指定 offset/limit 读取特定区域
2. **EditFileTool 无回退匹配**：精确匹配失败时直接报错，没有模糊匹配或最佳匹配提示
3. **ListDirTool 无条目上限**：递归列出时可能返回海量条目，浪费 token
4. **ExecTool 只截断尾部**：大输出只保留前 10000 字符，丢失尾部（通常包含错误信息和退出码）

对应上游 PR：HKUDS/nanobot#1895（enhance: improve filesystem & shell tools with pagination, fallback matching, and smarter output）。

## 方案比较（强制）

### 方案 1: 逐项增强（最小可行版）

- 思路: 按优先级逐个增强工具，每个工具独立改动
- 优点: 可分步交付，风险可控
- 缺点: 无
- 工作量估算: M

### 方案 2: 全面重构工具层（理想架构）

- 思路: 引入统一的分页/截断框架，所有工具共享
- 优点: 一致性好
- 缺点: 过度设计，当前工具数量少不需要框架
- 工作量估算: L

### 推荐

方案 1。逐项增强，保持简单。

## 功能需求列表

### 核心功能

1. **ReadFileTool 分页**：新增 `offset`（起始行号，1-indexed）和 `limit`（最大行数）参数，输出带行号前缀（`{line_no}: {content}`）
2. **EditFileTool replace_all**：新增 `replace_all: bool` 参数，为 true 时替换所有匹配而非报错
3. **ListDirTool 条目上限**：新增 `max_entries` 参数（默认 200），超出时截断并提示总数
4. **ExecTool head+tail 截断**：大输出保留前 5000 + 后 5000 字符（中间省略），始终显示退出码

### 扩展功能

- 无

## 非功能需求

- **兼容性**：新参数均为可选，默认行为与现有一致
- **测试要求**：每个增强点至少一个测试用例

## 边界与不做事项

- 不引入 EditFileTool 的渐进式回退匹配（复杂度高，收益有限）
- 不提取公共分页框架
- 不修改 WriteFileTool

## 待确认事项

- 无
