# cron crate

Cron 定时任务调度、存储和执行。

## 架构

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   CronTool   │     │ CronService  │     │ CronStorage  │
│  (Tool trait) │     │  调度逻辑     │     │  持久化       │
│              │     │              │     │              │
│ add/list/    │────►│ arm_timer()  │◄───►│ jobs.json    │
│ remove       │     │              │     │ RwLock<Store>│
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │ tokio::spawn    │
                   │ 循环:            │
                   │  sleep → 执行   │
                   │  → 重新计算唤醒  │
                   └─────────────────┘
```

任务变更（add/remove/enable）后自动 re-arm 定时器。

## 关键类型

- **`CronService`** -- 管理 cron 任务，基于定时器的执行循环
  - `new(store_path) -> Result<Self>` -- 从磁盘加载任务
  - `start()` / `stop()` -- 启动/停止执行循环
  - `set_on_job_callback(callback)` -- 设置任务触发时的回调
  - `add_job(...)` / `remove_job()` / `enable_job()` / `list_jobs()` -- CRUD
- **`CronJob`** -- `id`, `name`, `enabled`, `schedule`, `payload`, `state`, 时间戳, `delete_after_run`
- **`CronSchedule`** (enum) -- `At { at_ms }` | `Every { every_ms }` | `Cron { expr, tz }`
- **`CronPayload`** -- `kind`, `message`, `deliver`, `channel`, `to`
- **`CronJobState`** -- `next_run_at_ms`, `last_run_at_ms`, `last_status`, `last_error`
- **`CronStore`** -- `version`, `jobs: Vec<CronJob>`（持久化格式）
- **`CronStorage`** -- JSON 文件后端，`RwLock<CronStore>`
- **`CronTool`** -- 实现 `Tool` trait，注册为 "cron"，支持 add/list/remove 操作
- **`JobCallback`** -- `Arc<dyn Fn(CronJob) -> Pin<Box<dyn Future<Output = Result<String, String>>>>>`
- `is_valid_timezone(tz) -> bool` -- 校验 IANA 时区字符串

## 内部依赖

tools
