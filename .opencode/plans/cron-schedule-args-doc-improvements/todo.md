# TODO

## 任务列表

### 1. 为 `CronScheduleArgs` 枚举添加总览文档 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 在枚举定义上方添加模块级文档，使用 `///` 语法
- 完成备注: 添加了 Rustdoc 格式的总览文档，清晰区分 interval-based 和 time-based 调度

### 2. 优化 `CronScheduleArgs::Every` 变体的文档注释 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 需要强调 interval-based 语义，并提供反例说明
- 完成备注: 强调其非 time-based 特性，并明确指出"每天 8 AM"场景应使用 Cron

### 3. 优化 `CronScheduleArgs::Cron` 变体的文档注释 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 添加 cron 表达式示例，使用 6 字段格式
- 完成备注: 添加了 `"0 8 * * *"` 等常用 cron 表达式示例

### 4. 优化 `CronArgs::Add` 中 `schedule` 字段的文档 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 简洁的选择指导说明
- 完成备注: 添加了 schedule variant 选择指导

### 5. 优化 `CronTool::description()` 方法的返回字符串 ✅

- 优先级: P0
- 依赖项: 无
- 风险/注意点: 保持描述简洁，但需明确三种 variant 的适用场景
- 完成备注: 更新描述，明确说明 cron/every/at 三种语义的适用场景

## 实现建议

- 所有修改均在 `crates/cron/src/tool/mod.rs` 文件中完成
- 纯文档修改，不涉及任何业务逻辑变更
- 修改后建议运行 `cargo doc --no-deps` 验证文档生成正常
- 文档风格应与项目其他部分保持一致（Rustdoc 标准格式）

## 验证结果

- ✅ `cargo doc --no-deps -p nanobot-cron` 构建成功
- ✅ `cargo clippy -- -D warnings -D clippy::uninlined_format_args` 检查通过
- ✅ `cargo test -p nanobot-cron` 全部 25 个测试通过
