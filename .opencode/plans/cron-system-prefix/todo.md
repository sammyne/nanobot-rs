# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/nanobot/src/commands/gateway/mod.rs` | 修改 | cron 回调中包装消息文本，添加系统前缀 |

## 任务列表

### 1. ✅ 在 cron 回调中为消息添加系统前缀

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/nanobot/src/commands/gateway/mod.rs`
- 验收标准: `setup_cron_callback` 闭包中传给 `process_direct()` 的第一个参数从 `&payload.message` 变为包含 `[Scheduled Task]` 前缀的格式化字符串
- 风险/注意点: 闭包内 `job.name` 在 `let payload = job.payload;` 之后不可用（`job` 的其他字段已被 move），需在解构 payload 前提取 `job.name`
- 信心评估: 5（改动位置明确，已读过源码）
- 步骤:
  - [ ] 在 `setup_cron_callback` 闭包内，`let payload = job.payload;` 之前，提取 `let job_name = job.name.clone();`
  - [ ] 在 `let session_key = ...;` 之后，构造 `let reminder_note = format!("[Scheduled Task] Timer finished.\n\nScheduled task '{}' has been triggered.\nScheduled instruction: {}", job_name, payload.message);`
  - [ ] 将 `process_direct(&payload.message, ...)` 改为 `process_direct(&reminder_note, ...)`
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 确认无警告
  - [ ] 运行 `cargo test` 确认现有测试通过

## 实现建议

- `job.name` 需要在 `job.payload` move 之前 clone 出来，因为 `CronJob` 的字段在解构后不可再访问
- 前缀格式与上游 Python 版 `on_cron_job` 回调中的 `reminder_note` 对齐（英文，便于 LLM 理解）
