# 实施计划：Rust 版 AgentLoop 实现

## 任务清单

- [ ] 1. 设置模块结构和依赖配置
   - 创建 `crates/agent` 目录结构（Cargo.toml、src/lib.rs、src/loop.rs、src/tests.rs）
   - 在 Cargo.toml 中添加依赖：`nano_config`、`nano_provider`、`anyhow`、`tracing`
   - 在 lib.rs 中设置模块声明和 re-export
   - _需求：7.1、7.2_

- [ ] 2. 实现 AgentLoop 核心结构定义
   - 定义 `AgentLoop` 结构体，包含 `provider`、`config`、`messages` 等字段
   - 实现 `new()` 构造函数，接受 `provider` 和 `config` 参数（均为必选）
   - 初始化内部状态（消息历史等）
   - _需求：1.1、1.2_

- [ ] 3. 实现消息上下文构建功能
   - 实现 `build_context()` 方法，构建消息列表
   - 支持 `Message::user()`、`Message::assistant()`、`Message::system()` 三种角色
   - 实现消息内容截断逻辑（基于配置的最大长度）
   - _需求：2.1、2.2、2.3_

- [ ] 4. 实现 LLM 调用与响应处理
   - 实现 `call_llm()` 方法，使用 provider 和 config.model 调用 LLM
   - 实现响应解析逻辑
   - 添加超时控制（使用 tokio::time::timeout）
   - 实现错误信息的友好返回
   - _需求：3.1、3.2、3.3、3.4_

- [ ] 5. 实现迭代循环控制（参考 Python 版 `_run_agent_loop`）
   - 实现 `run()` 方法，接收初始消息列表，返回最终响应内容
   - 使用 while 循环结构，迭代计数器从 0 递增到 `config.max_tool_iterations`
   - 每次迭代：调用 `provider.chat()`，检查响应是否包含工具调用
   - 若有工具调用：添加 assistant 消息、执行工具、添加工具结果消息，继续循环
   - 若无工具调用：提取最终内容并跳出循环
   - 达到最大迭代次数时返回提示消息（与 Python 版保持一致）：
     ```
     I reached the maximum number of tool call iterations ({max_iterations}) without completing the task. You can try breaking the task into smaller steps.
     ```
   - _需求：4.1、4.2、4.3、4.4_

- [ ] 6. 添加错误处理与日志
   - 使用 `anyhow::Result` 作为返回类型
   - 在关键操作处添加 `.context()` 语义化错误上下文
   - 使用 `tracing` 添加结构化日志（初始化、LLM 调用、响应等）
   - 日志使用中文描述
   - _需求：5.1、5.2、5.3_

- [ ] 7. 编写单元测试
   - 创建 `tests.rs` 文件，定义 `Case` 和 `Expect` 结构体
   - 编写测试用例：`agent_loop_new_ok`（验证构造成功）
   - 编写测试用例：`agent_loop_build_context_ok`（验证上下文构建）
   - 编写测试用例：`agent_loop_iteration_limit_reached`（验证迭代限制）
   - 使用表驱动测试风格
   - _需求：6.1、6.2、6.3_
