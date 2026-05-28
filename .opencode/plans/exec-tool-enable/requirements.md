# 需求

## 目标与背景

为 `ExecToolConfig` 新增 `enable` 开关（默认 true），允许用户在安全敏感场景下完全禁用 shell 执行工具。对齐上游 HKUDS/nanobot PR #1824。

当前 ExecTool 总是注册到 ToolRegistry，无法通过配置禁用。在某些部署场景（如仅需文件读写、不允许执行命令）下，需要能关闭 shell 执行能力。

## 方案比较（强制）

### 方案 1: 在 ExecToolConfig 加 enable 字段（最小可行版 / 推荐）

- 思路: `ExecToolConfig` 新增 `disabled: bool = false`，`ToolRegistry::new()` 中根据该字段决定是否注册 ExecTool
- 优点: 改动最小（~5 行），与上游一致
- 缺点: 无
- 工作量估算: S

### 方案 2: 在 ToolsConfig 层面控制（理想架构）

- 思路: 在 `ToolsConfig` 中添加通用的工具启用/禁用机制，支持按名称禁用任意工具
- 优点: 更通用
- 缺点: 过度设计，当前只有 ExecTool 需要禁用
- 工作量估算: M

### 推荐

方案 1。只有 ExecTool 有安全敏感性需要禁用开关，不需要通用机制。

## 功能需求列表

### 核心功能

1. `ExecToolConfig` 新增 `disabled: bool`，默认 `false`
2. `ToolRegistry::new()` 中当 `disabled` 为 `true` 时跳过 ExecTool 注册

## 非功能需求

- 向后兼容：不配置 `disabled` 时行为不变（默认 false）
- 测试：验证 `disabled=true` 时 ToolRegistry 不包含 shell 工具

## 边界与不做事项

- 不实现通用的工具启用/禁用机制
- 不影响文件系统工具（ReadFile/WriteFile/EditFile/ListDir）

## 待确认事项

无
