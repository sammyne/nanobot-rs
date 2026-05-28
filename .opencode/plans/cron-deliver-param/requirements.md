# 需求

## 目标与背景

将上游 cron deliver 参数功能迁移到 Rust 版（对应上游 6 条直接提交 #43-48）。

当前 Rust 版 `CronTool` 在创建任务时硬编码 `deliver: true`，LLM 无法创建"静默任务"（执行但不发送通知）。数据层 `CronPayload.deliver` 字段和 gateway 中 `if payload.deliver` 的判断逻辑已存在，只是工具层没有暴露给 LLM。

**应用场景**：
- `deliver: true`（默认）— 任务执行后，结果经 evaluator 评估后发送给用户（如"每天 8 点提醒我喝水"）
- `deliver: false` — 静默执行，结果不发送（如"每小时检查一次系统状态并写入日志"）

## 方案比较（强制）

### 方案 1: 在 CronArgs 中暴露 deliver 参数（最小可行版）✅ 已选定

- 思路: 在 `CronArgs::Add` 和 `CronArgsSchema` 中添加 `deliver` 字段（默认 `true`），传入 `handle_add`
- 优点: 改动最小，数据层已支持
- 缺点: 无
- 工作量估算: S

### 方案 2: 不做

- 思路: 保持硬编码 `true`
- 优点: 零改动
- 缺点: LLM 无法创建静默任务
- 工作量估算: 无

### 推荐

方案 1。

## 功能需求列表

### 核心功能

- `CronArgs::Add` 新增 `deliver: bool` 字段，默认 `true`
- `CronArgsSchema` 新增 `deliver: bool` 字段，默认 `true`
- `handle_add` 接受 `deliver` 参数，传入 `add_job`（替代硬编码 `true`）
- `execute` match arm 传递 `deliver`

## 非功能需求

- 向后兼容：不传 `deliver` 时默认 `true`，与上游行为一致
- 现有测试通过

## 边界与不做事项

- 不修改 `CronPayload` 结构体（已有 `deliver` 字段）
- 不修改 gateway 中的 deliver 判断逻辑（已正确实现）

## 待确认事项

无
