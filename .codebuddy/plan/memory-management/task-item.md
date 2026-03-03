# 实施计划

- [ ] 1. 创建独立 memory crate 结构
   - 在 `crates/` 目录下创建 `memory` crate，配置 `Cargo.toml` 依赖（tokio、serde、anyhow、tracing等）
   - 创建 `lib.rs` 并导出公共 API（MemoryStore 结构体、Error 类型等）
   - _需求：1.1、1.2、1.3_

- [ ] 2. 实现 MemoryStore 核心结构和文件操作方法
   - 定义 `MemoryStore` 结构体，包含 workspace 路径、MEMORY.md 和 HISTORY.md 文件路径字段
   - 实现 `new(workspace: PathBuf)` 构造函数，创建 memory/ 目录并初始化文件路径
   - 实现 `read_long_term() -> Result<String>` 方法，读取 MEMORY.md 内容（不存在则返回空字符串）
   - 实现 `write_long_term(content: &str) -> Result<()>` 方法，写入 MEMORY.md
   - 实现 `append_history(entry: &str) -> Result<()>` 方法，追加条目到 HISTORY.md
   - 实现 `get_memory_context() -> String` 方法，格式化长期记忆为上下文字符串
   - _需求：2.1、2.2、2.3、2.4、2.5、2.6_

- [ ] 3. 实现记忆整合触发逻辑
   - 实现 `should_consolidate()` 私有方法，判断是否需要触发整合（消息数量检查）
   - 实现 `calculate_archive_range()` 私有方法，计算待归档消息的范围（last_consolidated 到保留边界）
   - _需求：3.1、3.2、3.3、3.4、3.5、3.6_

- [ ] 4. 实现 LLM 辅助记忆整合方法
   - 定义 `save_memory` 工具的 JSON Schema（包含 history_entry 和 memory_update 参数描述）
   - 构建 LLM 提示词模板（包含当前长期记忆和待处理对话内容）
   - 实现 `consolidate()` 异步方法，接收 6 个参数并调用 LLM
   - 解析 LLM 返回的工具调用，提取 history_entry 和 memory_update 参数
   - 根据 LLM 返回结果更新 MEMORY.md 和 HISTORY.md 文件
   - 返回新的 last_consolidated 索引值
   - _需求：4.1、4.2、4.3、4.4、4.5、4.6、4.7、4.8、4.9_

- [ ] 5. 添加错误处理和日志记录
   - 定义 MemoryError 枚举类型，包含文件操作错误、LLM 调用错误、工具解析错误等变体
   - 在关键操作点添加 tracing 日志（info 级别：整合开始/完成；error 级别：失败场景）
   - 确保所有错误使用 `?` 操作符正确传播，避免 panic
   - _需求：6.1、6.2、6.3、6.4、6.5_

- [ ] 6. 集成到 AgentLoop
   - 在 `crates/agent/src/loop.rs` 中添加 `memory_store: MemoryStore` 字段到 AgentLoop 结构体
   - 在 AgentLoop 构造函数中初始化 MemoryStore 实例
   - 在 `process_direct()` 或 `handle_message()` 方法末尾添加记忆整合调用逻辑
   - 在 Session 结构体中添加 `last_consolidated: usize` 字段
   - 在消息上下文构建时注入长期记忆内容到系统消息
   - 处理记忆整合失败场景（记录错误但不中断消息处理）
   - _需求：7.1、7.2、7.3、7.4、7.5、7.6_

- [ ] 7. 编写单元测试和集成测试
   - 为 MemoryStore 文件操作方法编写单元测试（测试读写、追加、不存在文件处理）
   - 为记忆整合触发逻辑编写单元测试（测试边界条件：消息数量不足、archive_all、无新消息等）
   - 编写集成测试验证完整记忆整合流程（mock LLM 响应）
   - _需求：2.x、3.x、4.x_
