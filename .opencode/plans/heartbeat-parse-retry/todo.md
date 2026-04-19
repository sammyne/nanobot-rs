# TODO

## 任务列表

### 1. 添加最大重试次数常量

- 优先级: P0
- 依赖项: 无 ✅ 已完成
- 风险/注意点: 定义为 `const MAX_PARSE_RETRIES: u8 = 1;`，放置在 HEARTBEAT_TOOL 之后

### 2. 重构 decide 方法实现重试循环

- 优先级: P0
- 依赖项: 1 ✅ 已完成
- 风险/注意点: 
  - 将原有的单次调用改为 for 循环
  - 使用 `attempt < MAX_PARSE_RETRIES` 判断是否继续重试
  - 保留 `options` 变量在循环外部定义

### 3. 添加错误反馈消息构造逻辑

- 优先级: P0
- 依赖项: 2 ✅ 已完成
- 风险/注意点:
  - 错误消息格式应清晰说明期望格式：
    ```
    Invalid arguments format. Error: <具体解析错误>.
    Expected JSON format:
      - To skip: "skip"
      - To run tasks: {"run": {"tasks": "<task description>"}}
    Please retry with a valid format.
    ```
  - 不暴露原始 `arguments` 内容，保护安全性
  - 使用 `Message::tool()` 构造反馈消息
  - 同时将 LLM 的文本回复加入上下文

### 4. 更新日志级别和内容

- 优先级: P1
- 依赖项: 2 ✅ 已完成
- 风险/注意点:
  - 使用 `warn!` 记录可恢复错误（非最终失败）
  - 使用 `error!` 记录最终失败
  - 包含尝试次数信息便于调试

### 5. 添加单元测试验证重试行为

- 优先级: P1
- 依赖项: 2, 3 ✅ 已完成
- 风险/注意点:
  - MockProvider 需要能够模拟解析失败场景
  - 测试验证重试次数和最终降级行为

## 实现建议

- **技术栈**：基于现有 `crates/heartbeat/src/service.rs`
- **可复用的现有模块和组件**：
  - `nanobot_provider::Message::tool()` - 构造工具消息
  - `nanobot_provider::Message::assistant()` - 构造助手消息
  - `ToolCall::parse_arguments()` - 解析工具参数
  - `tracing::{warn!, error!}` - 日志记录
- **代码风格**：遵循 AGENTS.md 中的 Rust 格式化规范，使用 `cargo +nightly fmt`
- **提交前检查**：`cargo clippy -- -D warnings`