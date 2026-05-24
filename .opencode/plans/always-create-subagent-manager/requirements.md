# 需求

## 目标与背景

nanobot-rs 的 `AgentLoop::new()` 接受 `Option<Arc<SubagentManager<P>>>` 参数，CLI 单次消息模式传入 `None`，导致 LLM 无法使用 `spawn` 工具。Python 版在 `AgentLoop.__init__` 中无条件创建 `SubagentManager`，不存在 None 的情况。

当前 `Option` 包裹导致下游代码（`StopCmd`、`handle_stop`）都需要处理 `None` 分支，增加了不必要的复杂度。应移除 `Option`，要求调用方总是提供 `SubagentManager`。

## 方案比较

### 方案 1: 移除 Option，调用方总是创建 SubagentManager

- 思路: 修改 `AgentLoop::new()` 签名，`subagent_manager` 参数从 `Option<Arc<SubagentManager<P>>>` 改为 `Arc<SubagentManager<P>>`；同步移除 struct 字段和 StopCmd 中的 Option 包裹；所有调用方（生产代码 + 测试）总是传入 SubagentManager
- 优点: 消除所有 `if let Some(ref manager)` 分支，代码更简洁；与 Python 版行为一致
- 缺点: 测试中 19 处传 `None` 的调用需要改为创建 SubagentManager（已有 `mock_subagent_manager` 辅助函数）

### 推荐

推荐方案 1。

## 功能需求列表

### 核心功能

- `AgentLoop::new()` 的 `subagent_manager` 参数从 `Option<Arc<SubagentManager<P>>>` 改为 `Arc<SubagentManager<P>>`
- `AgentLoop` struct 的 `subagent_manager` 字段从 `Option<Arc<SubagentManager<P>>>` 改为 `Arc<SubagentManager<P>>`
- `StopCmd` 的 `subagent_manager` 字段从 `Option<Arc<SubagentManager<P>>>` 改为 `Arc<SubagentManager<P>>`
- `handle_stop()` 中移除 `if let Some(ref manager)` 分支，直接调用
- CLI 单次消息模式（`run_once()`）创建 SubagentManager
- 测试中所有传 `None` 的调用改为使用 `mock_subagent_manager()`

## 非功能需求

- **测试要求**：所有现有测试通过，无回归

## 边界与不做事项

- 不修改 `cron_service` 参数的 Option（cron 确实是可选的）

## 假设与约束

- 测试中已有 `mock_subagent_manager` 辅助函数可复用

## 待确认事项

无
