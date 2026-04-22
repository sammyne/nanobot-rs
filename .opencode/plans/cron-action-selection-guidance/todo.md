# TODO

## 任务列表

### 1. 修改 `CronTool::description()` 方法 ✅

- **优先级**: P0
- **依赖项**: 无
- **风险/注意点**: 无
- **说明**:
  - 文件：`crates/cron/src/tool/mod.rs`
  - 行号：177-179
  - 将 `"Schedule reminders and recurring tasks. Actions: add, list, remove."` 替换为包含触发词的分层描述：

```rust
fn description(&self) -> &str {
    "Schedule reminders and recurring tasks. Actions:\n\
    - add: Create a new scheduled job. Use when user asks to 'remind me', 'schedule', 'add a task', or 'set up a recurring task'.\n\
    - list: Show all scheduled jobs. Use when user asks to 'list tasks', 'show my reminders', or 'what tasks do I have'.\n\
    - remove: Delete a job by ID. Use when user asks to 'delete', 'cancel', or 'remove' a scheduled task."
}
```

- **完成备注**: 25 tests passed, clippy passed

## 实现建议

1. **修改内容**：`CronTool::description()` 仅修改返回值字符串，不涉及业务逻辑变更
2. **测试验证**：
   - `cargo test -p nanobot-cron` — 确保 cron 工具相关测试全部通过
   - `cargo clippy -- -D warnings` — 确保无 lint 警告
3. **代码风格**：description 采用分层列表格式，触发词使用单引号包裹，便于 LLM 识别
