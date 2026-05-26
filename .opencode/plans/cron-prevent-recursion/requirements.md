# 需求

## 目标与背景

当 cron 任务触发时，agent 通过 `process_direct()` 执行，拥有所有工具的完整访问权限，包括 cron tool 本身。如果原始调度消息类似"每 10 秒提醒我"，agent 可能将 cron payload 解释为新的调度指令，调用 `cron add` 创建新任务，形成无限递归。

PR #108 添加的 `[Scheduled Task]` 前缀降低了 LLM 误解的概率，但没有硬性阻断。上游 HKUDS/nanobot PR #1458 通过标志在 cron 执行期间阻断 `add` 操作。

## 方案比较（强制）

### 方案 1: CronTool 内部 AtomicBool 标志（最小可行版）

- 思路: `CronTool` 新增全局 `Arc<AtomicBool>` 标志
- 优点: 改动小
- 缺点: **并发不安全** — 多个 cron job 同时触发或 cron 与用户消息并发时，全局标志互相干扰
- 工作量估算: S
- **不推荐**

### 方案 2: ToolContext 传递 is_cron 标志（推荐）

- 思路: `ToolContext` 新增 `is_cron: bool` 字段。每次工具调用创建独立的 `ToolContext`（agent loop 第 259 行），cron 执行路径设置 `is_cron = true`。CronTool 在 `execute()` 中检查 `ctx.is_cron`
- 优点: 并发安全（每个执行流有独立 context）；无共享状态；任何工具都能感知 cron 上下文
- 缺点: 修改 `ToolContext` 公共结构
- 工作量估算: S

### 推荐

方案 2。并发安全，改动量与方案 1 相当。

## 功能需求列表

### 核心功能

1. `ToolContext` 新增 `scheduled: bool` 字段（默认 false）
2. `ToolContext::new()` 签名不变（scheduled 默认 false），新增 `ToolContext::scheduled()` 构造方法（scheduled = true）
3. `CronTool::execute()` 中 `CronArgs::Add` 分支检查 `ctx.scheduled`，为 true 时返回错误
4. `AgentLoop::re_act()` 新增 `scheduled: bool` 参数；ToolContext 在 re_act 入口创建一次（移出工具调用循环）；`process_message()` 中根据 `session_key.starts_with("cron:")` 判断传值；`process_system_message()` 和测试中的调用传 `false`
5. `list` 和 `remove` 操作不受影响

### 扩展功能

- 无

## 非功能需求

- **并发安全**: 每个执行流独立 ToolContext，无共享状态
- **测试要求**: 新增测试验证 cron context 下 add 被阻断，list/remove 不受影响
- **兼容性**: `ToolContext::new()` 签名不变，现有调用方无需修改

## 边界与不做事项

- 不影响 CLI 的 cron 命令
- 不影响 subagent 的工具执行

## 假设与约束

- 无

## 待确认事项

- 无
