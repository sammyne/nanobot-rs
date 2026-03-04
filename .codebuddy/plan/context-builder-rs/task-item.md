# 实施计划

- [ ] 1. 创建 crate 结构和错误类型定义
   - 在 `crates/context/` 目录创建新 crate，配置 `Cargo.toml` 依赖（thiserror、tokio、base64 等）
   - 定义 `ContextError` 错误类型，包含 `Io`、`InvalidPath`、`MediaType` 变体
   - 在 `lib.rs` 中导出核心公共 API
   - _需求：9.1、9.2、9.3、9.4、9.5、10.1、10.4_

- [ ] 2. 实现 ContextBuilder 核心结构
   - 定义 `ContextBuilder` 结构体，持有 `workspace: PathBuf` 和 `memory: MemoryStore`
   - 实现 `new()` 方法，接受 workspace 路径并初始化 MemoryStore
   - 路径不存在时返回错误而非 panic
   - _需求：1.1、1.2、1.3_

- [ ] 3. 实现核心身份信息生成
   - 实现 `build_core_identity()` 方法，生成 nanobot 介绍、运行时信息（OS、架构）
   - 包含工作空间绝对路径、记忆文件路径（MEMORY.md、HISTORY.md）
   - 包含工具调用指南
   - _需求：4.1、4.2、4.3、4.4、4.5_

- [ ] 4. 实现系统提示词构建
   - 实现 `build_system_prompt()` 方法，用 `---` 分隔符连接各部分
   - 组装核心身份 + 记忆上下文 + 技能摘要
   - 当记忆存储有内容时添加 `# Memory` 章节
   - _需求：3.1、3.2、3.3、3.4、3.5_

- [ ] 5. 实现运行时上下文注入
   - 实现 `inject_runtime_context()` 方法，添加当前时间（YYYY-MM-DD HH:MM (Weekday)）
   - 支持可选的 channel 和 chat_id 信息注入
   - 支持字符串和多媒体数组两种消息格式
   - _需求：5.1、5.2、5.3、5.4、5.5_

- [ ] 6. 实现媒体文件处理
   - 实现 `encode_image_to_base64()` 函数，检测 MIME 类型并编码为 base64
   - 构建 `data:{mime};base64,{data}` 格式
   - 文件不存在或非图片类型时跳过
   - _需求：7.1、7.2、7.3、7.4_

- [ ] 7. 实现消息列表构建
   - 实现 `build_messages()` 方法，返回 `Vec<Message>`
   - 按顺序添加：system 消息 → 历史消息 → 当前用户消息（带运行时上下文）
   - 支持媒体文件编码并放在文本内容之前
   - _需求：6.1、6.2、6.3、6.4、6.5_

- [ ] 8. 实现消息追加辅助方法
   - 实现 `append_tool_result()` 方法，创建 tool 角色消息
   - 实现 `append_assistant_message()` 方法，支持 content、tool_calls、reasoning_content 参数
   - _需求：8.1、8.2、8.3_

- [ ] 9. 编写单元测试
   - 创建 `tests.rs` 文件，测试各模块功能
   - 测试系统提示词构建、运行时上下文注入、媒体文件编码
   - 测试函数不使用 `test_` 前缀
   - _需求：10.2、10.3_
