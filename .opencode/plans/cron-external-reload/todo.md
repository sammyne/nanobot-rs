# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/cron/src/storage/mod.rs` | 修改 | 添加 mtime 追踪 + `reload_if_changed()` |
| `crates/cron/src/storage/tests.rs` | 修改 | 新增外部修改重载测试 |
| `crates/cron/src/service/mod.rs` | 修改 | timer tick 中调用 `reload_if_changed()` |

## 任务列表

### 1. ✅ CronStorage 添加 mtime 追踪和 reload_if_changed

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/cron/src/storage/mod.rs`
- 验收标准: `reload_if_changed()` 在文件 mtime 变化时重载内存 store，mtime 不变时不操作
- 风险/注意点: `last_mtime` 需要用 `RwLock` 包裹以支持 `&self` 方法签名；初始值 `SystemTime::UNIX_EPOCH` 确保首次调用必定触发重载；文件不存在时不应 panic
- 信心评估: 5
- 步骤:
  - [ ] `CronStorage` 新增字段 `last_mtime: RwLock<std::time::SystemTime>`，初始值 `std::time::SystemTime::UNIX_EPOCH`
  - [ ] `load()` 中加载文件成功后，通过 `tokio::fs::metadata().await?.modified()?` 获取 mtime 并存入 `last_mtime`
  - [ ] `save()` 中写入文件后，获取新 mtime 并更新 `last_mtime`
  - [ ] 新增 `pub async fn reload_if_changed(&self)` 方法：获取文件 mtime → 与 `last_mtime` 比较 → 不一致时读取文件 → 反序列化 → 替换 `self.store` → 更新 `last_mtime`；任何错误仅 warn 日志，不影响现有内存数据

### 2. ✅ 新增 storage 重载测试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/cron/src/storage/tests.rs`
- 验收标准: 测试验证外部修改 jobs.json 后 `reload_if_changed()` 能正确重载
- 风险/注意点: 需要 sleep 确保 mtime 差异（文件系统 mtime 精度为 1 秒）
- 信心评估: 4（mtime 精度可能导致 flaky，需 sleep 1s+）
- 步骤:
  - [ ] 新增测试 `reload_if_changed_picks_up_external_modification`：创建 storage → 添加 job → save → sleep 1.1s → 外部写入新 jobs.json（不同内容）→ 调用 `reload_if_changed()` → 验证内存 store 已更新
  - [ ] 新增测试 `reload_if_changed_noop_when_unchanged`：创建 storage → save → 调用 `reload_if_changed()` → 验证内存 store 不变

### 3. ✅ timer tick 中调用 reload_if_changed

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/cron/src/service/mod.rs`
- 验收标准: timer tick 循环在 `storage.list_jobs()` 之前调用 `storage.reload_if_changed().await`
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] 在 `arm_timer()` 闭包的 timer tick 循环中，`let jobs = storage.list_jobs(true).await;`（第 148 行）之前插入 `storage.reload_if_changed().await;`
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 确认无警告
  - [ ] 运行 `cargo test` 确认所有测试通过

## 实现建议

- `last_mtime` 使用 `RwLock<SystemTime>` 初始值 `UNIX_EPOCH`，任何真实文件 mtime 都比它新，首次 `reload_if_changed()` 必定触发重载，无需处理 `None` 分支
- `reload_if_changed()` 中的错误处理：文件不存在 → 跳过（可能被删除）；读取/解析失败 → warn 日志 + 保持现有数据
- 测试中 sleep 1.1 秒确保 mtime 差异，避免文件系统精度导致 flaky
