# 需求

## 目标与背景

当前 `CronScheduleArgs` 的文档描述过于简略，导致 AI Agent 在处理"每天指定时刻"类型的调度请求时，错误地使用 `Every { every_seconds: 86400 }`（每 86400 秒）而非 `Cron` 枚举值。

**核心问题**：`Every` 是 **interval-based**（基于时间间隔）调度，适用于"每 N 秒执行一次"；而"每天 8:00 AM"是 **time-based**（基于时钟时刻）调度，应该用 `Cron` 枚举值。文档未清晰区分这两种语义，导致 AI 误用。

## 功能需求列表

### 核心功能

1. **为 `CronScheduleArgs` 枚举添加总览文档**
   - 说明三种调度方式的适用场景
   - 明确区分 interval-based (`Every`) 和 time-based (`Cron`) 调度

2. **优化 `Every` 变体的文档注释**
   - 强调其 interval-based 语义
   - 说明从作业启动时间开始计算间隔，不对齐到时钟时刻
   - 提供反例：若在 8:05 AM 启动 `every_seconds: 86400`，下次执行是次日 8:05 AM 而非 8:00 AM

3. **优化 `Cron` 变体的文档注释**
   - 强调其 time-based 语义，适用于"每天/每周/每月固定时刻"
   - 添加常用 cron 表达式示例（如 `"0 8 * * *"` 表示每天 8:00 AM）

4. **优化 `CronArgs::Add` 中 `schedule` 字段的文档**
   - 添加选择指导：time-based 场景用 `cron`，interval-based 场景用 `every`

5. **优化 `CronTool::description()` 方法的返回字符串**
   - 更明确地说明三种 schedule variant 的适用场景
   - 强调 `cron` 用于"每天指定时刻"

## 非功能需求

- **可读性**：文档应简洁清晰，AI Agent 能正确理解并遵循
- **一致性**：文档风格与代码库其他部分保持一致
- **测试**：无功能代码变更，无需新增测试

## 边界与不做事项

- **不做**：不修改 `CronScheduleArgs` 枚举结构（不新增变体）
- **不做**：不修改 `handle_add` 等业务逻辑
- **不做**：不修改现有测试用例
- **边界**：仅修改 `crates/cron/src/tool/mod.rs` 中的文档注释

## 假设与约束

- **技术假设**：纯文档修改，不涉及编译或运行时行为
- **资源约束**：仅涉及单个文件的注释修改
- **环境约束**：无

## 修改清单

| 位置 | 修改类型 | 说明 |
|------|---------|------|
| `CronScheduleArgs` 枚举定义上方 | 新增 | 添加总览文档 |
| `CronScheduleArgs::Every` 变体 (line 21-22) | 修改 | 强调 interval-based 语义 |
| `CronScheduleArgs::Cron` 变体 (line 23-28) | 修改 | 添加使用示例 |
| `CronArgs::Add.schedule` 字段 (line 41-42) | 修改 | 添加选择指导 |
| `CronTool::description()` (line 158-159) | 修改 | 丰富场景说明 |
