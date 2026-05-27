# 需求

## 目标与背景

MCP 服务器可能暴露大量工具（如 GitHub MCP 有 30+ 工具），但用户往往只需要其中几个。当前 Rust 版本的 `connect()` 函数无条件注册所有工具，导致 LLM 上下文中工具列表过长，影响工具选择准确性和 token 消耗。

Python 版本（#62/#63/#64）在 MCP 配置中新增 `enabledTools` 字段，连接时按配置过滤。

## 方案比较（强制）

### 方案 1: 每个变体内联 enabled_tools 字段（最小可行版 + 理想架构）

- 思路: 在 `McpServerConfig::Stdio` 和 `McpServerConfig::Http` 两个变体中各加 `enabled_tools: Vec<String>` 字段，`connect()` 中过滤
- 优点: 改动最小，配置直觉（每个服务器独立控制），向后兼容（默认空 = 全部启用）
- 缺点: 两个变体重复同一字段
- 工作量估算: S

### 方案 2: 提取公共配置结构体

- 思路: 将 `enabled_tools`、`tool_timeout` 等公共字段提取到 `McpServerCommon` 结构体，用 `#[serde(flatten)]` 嵌入
- 优点: 消除字段重复
- 缺点: `#[serde(untagged)]` + `#[serde(flatten)]` 组合在 serde 中有已知问题，可能导致反序列化歧义；过度设计
- 工作量估算: M

### 推荐

方案 1。一个字段的重复不值得引入 flatten 的复杂性。

## 功能需求列表

### 核心功能

1. `McpServerConfig` 两个变体新增 `enabled_tools: Vec<String>` 字段（`#[serde(default)]`，camelCase 序列化为 `enabledTools`）
2. `connect()` 中 `list_tools()` 后按 `enabled_tools` 过滤：空列表 = 全部启用，非空 = 仅保留列表中的工具
3. 过滤时按原始工具名匹配（非 `mcp_server_tool` 前缀名）

## 非功能需求

- **向后兼容**：不设置 `enabledTools` 时行为不变
- **可观测性**：过滤时 debug 日志记录跳过的工具名

## 边界与不做事项

- 不做 `disabledTools`（黑名单模式）
- 不做通配符/正则匹配

## 待确认事项

- 无
