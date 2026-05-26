# 需求

## 目标与背景

当前 cron 任务触发时，`setup_cron_callback` 将 `payload.message`（用户原始调度指令）原样传给 `process_direct()`。LLM 无法区分这是系统定时触发还是用户主动查询，可能将定时提醒内容误解为新的调度指令（例如"每 10 秒提醒我"被 LLM 再次调用 cron tool 创建新任务）。

上游 HKUDS/nanobot PR #1371 通过为 cron 消息添加 `[Scheduled Task]` 前缀解决此问题。本需求仅迁移前缀部分（不含 message tool 发送追踪和回复压制逻辑）。

## 方案比较（强制）

### 方案 1: 在 cron 回调中包装消息文本（最小可行版）

- 思路: 在 `setup_cron_callback` 闭包内，将 `payload.message` 包装为带系统前缀的 `reminder_note`，传给 `process_direct()`
- 优点: 改动极小（1 处，~5 行），零新依赖，不影响 cron crate 公共 API
- 缺点: 前缀格式硬编码在 binary crate 中，不可配置
- 工作量估算: S

### 方案 2: 在 CronService 内部包装消息（理想架构）

- 思路: 在 `crates/cron/src/service/mod.rs` 的 `execute_job()` 中包装消息，让所有调用方自动获得前缀
- 优点: 所有 cron 执行路径统一处理
- 缺点: `execute_job()` 只调用 `JobCallback`，不直接接触消息内容；需要修改 `JobCallback` 签名或 `CronJob` 结构，改动范围扩大
- 工作量估算: M

### 推荐

方案 1。改动集中在唯一的 cron 回调注册点（`setup_cron_callback`），与上游实现位置一致（Python 版也在 `on_cron_job` 回调中包装）。cron crate 保持通用，不耦合消息格式。

## 功能需求列表

### 核心功能

- cron 任务触发时，传给 `process_direct()` 的消息文本添加系统前缀，格式为：
  ```
  [Scheduled Task] Timer finished.

  Scheduled task '{job_name}' has been triggered.
  Scheduled instruction: {original_message}
  ```

### 扩展功能

- 无

## 非功能需求

- **性能**: 无影响（仅 `format!` 一次字符串拼接）
- **安全**: 无影响
- **兼容性**: 向后兼容，现有 cron 任务行为不变（仅 LLM 看到的消息文本变化）
- **可维护性**: 前缀格式与上游 Python 版对齐，便于后续同步
- **测试要求**: 无需新增测试（改动在 binary crate 的回调闭包中，无可单独测试的公共 API 变更）

## 边界与不做事项

- 不实现 message tool 的 `_sent_in_turn` 追踪和回复压制逻辑（上游 PR #1371 的另一半）
- 不修改 cron crate 的公共 API 或 `CronJob`/`CronPayload` 结构
- 不添加前缀格式的配置化能力

## 假设与约束

- **技术假设**: `job.name` 和 `payload.message` 均为非空字符串（由 `CronTool.add_job()` 的参数校验保证）
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 无
