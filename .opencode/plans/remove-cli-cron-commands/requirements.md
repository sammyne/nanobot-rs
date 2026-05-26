# 需求

## 目标与背景

对应上游 HKUDS/nanobot 直接提交 c05cb2e（#14），移除 CLI cron 子命令中的 `list`、`add`、`remove`，仅保留 `enable` 和 `run`。

**为什么移除**：cron 任务的增删查应由 Agent 通过 CronTool 在对话中完成，CLI 手动管理与 Agent 驱动的设计理念冲突，且容易导致状态不一致（CLI 添加的任务缺少对话上下文）。`enable` 和 `run` 保留是因为它们是运维操作（紧急禁用/手动触发），不涉及任务创建。

## 方案比较（强制）

### 方案 1: 直接删除代码（最小可行版）

- 思路: 从 `cron/mod.rs` 中删除 `ListCmd`、`AddCmd`、`RemoveCmd` 及其辅助函数，更新枚举和 match 分支，清理无用依赖和测试
- 优点: 改动最小，一次性完成，无遗留代码
- 缺点: 无
- 工作量估算: S

### 方案 2: 标记废弃后延迟删除（理想架构）

- 思路: 先用 `#[deprecated]` 标记三个子命令，打印警告信息引导用户使用 CronTool，下个版本再删除
- 优点: 给用户过渡期
- 缺点: nanobot-rs 目前没有外部用户依赖 CLI cron 命令，过渡期无意义；增加维护负担
- 工作量估算: M

### 推荐

方案 1。没有外部用户依赖这些 CLI 命令，直接删除最干净。

## 功能需求列表

### 核心功能

- 移除 `CronSubcommand::List` 变体及 `ListCmd` 实现
- 移除 `CronSubcommand::Add` 变体及 `AddCmd` 实现
- 移除 `CronSubcommand::Remove` 变体及 `RemoveCmd` 实现
- 更新 `CronCmd::run()` 的 match 分支，仅保留 `Enable` 和 `Run`
- 移除不再使用的辅助函数 `format_schedule`（仅被 List/Add 使用）
- 保留 `format_time`（仍被 `EnableCmd` 使用）
- 移除不再使用的 import：`CronSchedule`
- 移除 `tests.rs` 中与 `AddCmd` 相关的测试（`test_build_schedule_*` 和 `test_format_schedule`）
- 保留 `tests.rs` 中 `test_format_time` 测试

### 扩展功能

- 无

## 非功能需求

- **可维护性**：清理因删除产生的无用 import 和依赖
  - `nanobot_utils::strings::truncate` 仅在 `ListCmd` 中使用，删除后 `nanobot-utils` 在 nanobot crate 中无其他引用，从 `Cargo.toml` 移除该依赖
  - `chrono` 仍被 gateway/tests.rs 和 `format_time` 使用，保留
- **测试要求**：`cargo test -p nanobot` 通过，`cargo clippy` 无警告

## 边界与不做事项

- 不修改 `CronService`、`CronStorage` 的公共 API（`add_job`/`remove_job`/`list_jobs` 仍被 CronTool 和 gateway 使用）
- 不修改 `CronTool`（Agent 侧的 cron 管理不受影响）
- 不移除 `init_cron_service`（仍被 `EnableCmd`、`RunCmd`、gateway、agent 使用）

## 假设与约束

- **技术假设**：`nanobot-utils` 在 nanobot crate 中除 `ListCmd` 外无其他使用点（已通过 grep 确认）
- **资源约束**：无

## 待确认事项

- 无
