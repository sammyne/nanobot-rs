# 需求文档

## 引言

本需求文档描述为 `nanobot-rs` 项目实现独立记忆管理 crate 的功能需求。该 crate 参考HKUDS/nanobot Python 版本的记忆存储模块设计，提供双层记忆系统：**长期记忆 (MEMORY.md)** 和 **历史日志(HISTORY.md)**，通过 LLM 辅助将对话历史压缩为持久化的记忆摘要。

记忆管理 crate 的核心目标是：
1. 提供独立、可复用的记忆管理能力
2. 解决对话上下文无限增长导致的 LLM token 消耗问题
3. 保留关键决策、用户偏好和重要事件信息
4. 提供 grep 可搜索的历史日志记录

## 需求

### 需求 1：独立 Crate 结构

**用户故事：** 作为项目架构师，我希望记忆管理功能作为独立 crate存在，以便实现模块解耦和代码复用。

#### 验收标准

1. WHEN 创建记忆管理模块 THEN 系统 SHALL 在 `crates/memory` 目录下创建独立 crate
2. WHEN 配置 crate 依赖 THEN 系统 SHALL 在 `Cargo.toml` 中声明必要的依赖项（tokio、serde、anyhow等）
3. WHEN 暴露公共 API THEN 系统 SHALL 通过 `lib.rs` 导出 `MemoryStore` 结构体及相关方法

### 需求 2：记忆存储结构

**用户故事：** 作为记忆管理使用者，我希望有一个独立的记忆存储模块，以便将对话历史压缩为持久化的记忆文件。

#### 验收标准

1. WHEN 创建 MemoryStore 实例 THEN 系统 SHALL 在指定 workspace 下创建 `memory/` 目录
2. WHEN 创建 MemoryStore 实例 THEN 系统 SHALL 初始化 `MEMORY.md` 和 `HISTORY.md` 文件路径
3. IF `MEMORY.md` 文件不存在 THEN `read_long_term()` 方法 SHALL 返回空字符串
4. WHEN 调用 `write_long_term(content)` THEN 系统 SHALL 将内容写入 `MEMORY.md` 文件
5. WHEN 调用 `append_history(entry)` THEN 系统 SHALL 将条目追加到 `HISTORY.md` 文件末尾，每个条目以双换行分隔
6. WHEN 调用 `get_memory_context()` THEN 系统 SHALL 返回格式化的长期记忆内容，以 `## Long-term Memory` 标题开头

### 需求 3：记忆整合触发机制

**用户故事：** 作为记忆管理使用者，我希望在对话历史达到一定长度时自动触发记忆整合，以便控制 LLM 上下文大小。

#### 验收标准

1. IF 对话消息数量超过 `memory_window` 配置值 THEN 系统 SHALL 触发记忆整合
2. WHEN 触发记忆整合 THEN 系统 SHALL 保留最近 `memory_window / 2` 条消息在会话中
3. WHEN 触发记忆整合 THEN 系统 SHALL 将 `last_consolidated` 到保留边界之间的旧消息发送给 LLM 处理
4. IF `archive_all` 参数为 true THEN 系统 SHALL 处理所有消息并将返回值设为 0
5. IF 会话消息数量小于等于保留数量 THEN 系统 SHALL 跳过整合并返回输入的 last_consolidated 值
6. IF 自上次整合以来没有新消息 THEN 系统 SHALL 跳过整合并返回输入的 last_consolidated 值

### 需求 4：LLM 辅助记忆整合

**用户故事：** 作为记忆管理组件，我希望通过 LLM 工具调用方式生成记忆摘要，以便自动提取关键信息。

#### 验收标准

1. WHEN 调用 `consolidate()` 方法 THEN 系统 SHALL 接收 6 个参数：`messages: &[Message]`、`last_consolidated: usize`、`provider: &str`、`model: &str`、`archive_all: bool`、`memory_window: usize`，与 Python 版本对齐
2. WHEN 调用 `consolidate()` 方法 THEN 系统 SHALL 返回 `Result<usize, Error>`，其中成功值为新的 last_consolidated 索引
3. WHEN 发送 LLM 请求 THEN 系统 SHALL 使用传入的 provider 和 model 参数构建 LLM 客户端
4. WHEN 发送 LLM 请求 THEN 系统 SHALL 构建包含当前长期记忆和待处理对话的提示词
5. WHEN 发送 LLM 请求 THEN 系统 SHALL 提供 `save_memory` 工具定义，包含 `history_entry` 和 `memory_update` 两个参数
6. IF LLM 返回 `save_memory` 工具调用 THEN 系统 SHALL 解析工具参数
7. WHEN LLM 返回 `history_entry` 参数 THEN 系统 SHALL 将其追加到 HISTORY.md
8. WHEN LLM 返回 `memory_update` 参数 AND 内容与当前记忆不同 THEN 系统 SHALL 更新 MEMORY.md
9. IF LLM 未调用 `save_memory` 工具 THEN 系统 SHALL 记录警告并返回错误

### 需求 5：工具定义规范

**用户故事：** 作为 LLM 提供者，我希望 `save_memory` 工具有清晰的参数描述，以便正确生成记忆摘要。

#### 验收标准

1. WHEN 定义 `save_memory` 工具 THEN 系统 SHALL 使用 OpenAI 兼容的函数调用格式
2. `history_entry` 参数描述 SHALL 说明：生成 2-5 句话的段落摘要，以 `[YYYY-MM-DD HH:MM]` 开头，包含便于 grep 搜索的详细信息
3. `memory_update` 参数描述 SHALL 说明：返回完整的更新后长期记忆（Markdown 格式），包含所有已有事实和新事实，若无新信息则返回原内容

### 需求 6：错误处理与日志

**用户故事：** 作为运维人员，我希望记忆整合过程有完善的错误处理和日志记录，以便排查问题。

#### 验收标准

1. WHEN LLM 调用失败 THEN 系统 SHALL 捕获异常、记录错误日志并返回 Err
2. WHEN 工具参数解析失败 THEN 系统 SHALL 记录警告并返回 Err
3. WHEN 文件操作失败 THEN 系统 SHALL 捕获异常并返回 Err
4. WHEN 记忆整合开始 THEN 系统 SHALL 记录 info 级别日志，包含待处理消息数量
5. WHEN 记忆整合完成 THEN 系统 SHALL 记录 info 级别日志，包含当前消息总数和新的 last_consolidated 值

### 需求 7：AgentLoop 集成

**用户故事：** 作为 AgentLoop 用户，我希望记忆整合在消息处理后自动执行，以便无需手动管理记忆。

#### 验收标准

1. WHEN AgentLoop 创建时 THEN 系统 SHALL 初始化 MemoryStore 实例，使用 config.workspace 作为存储路径
2. WHEN `process_direct()` 或 `handle_message()` 完成消息处理后 THEN 系统 SHALL 检查是否需要记忆整合
3. WHEN 需要记忆整合 THEN 系统 SHALL 调用 `consolidate()` 方法，传入 `&session.messages`、`session.last_consolidated`、`&config.provider`、`&config.model`、`archive_all`、`config.memory_window`
4. WHEN consolidate 成功返回 THEN 系统 SHALL 更新 session.last_consolidated 为返回值
5. WHEN 构建消息上下文 THEN 系统 SHALL 将长期记忆内容注入到系统消息中
6. IF 记忆整合失败 THEN 系统 SHALL 记录错误但不影响正常消息处理流程
