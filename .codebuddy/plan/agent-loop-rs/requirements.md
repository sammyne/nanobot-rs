# 需求文档：Rust 版 AgentLoop 实现

## 引言

本需求文档描述在 Rust 中实现 AgentLoop 核心模块的详细需求。AgentLoop 是 nanobot 的核心处理引擎，负责接收消息、构建上下文、调用 LLM 并返回响应。

本实现参考 HKUDS 版 Python 实现，但仅聚焦于核心功能：**provider 和 agent.defaults 配置支持**，暂不包含完整的工具系统、session 管理、memory consolidation 等高级特性。

**依赖复用：**
- `AgentDefaults` 结构复用 `nano_config` crate 中已定义的版本
- `Message` 消息类型复用 `nano_provider` crate 中已定义的版本

## 需求

### 需求 1：AgentLoop 核心结构定义

**用户故事：** 作为开发者，我希望 AgentLoop 拥有清晰的结构定义，以便能够正确初始化和管理 agent 运行状态。

#### 验收标准

1. WHEN 创建 AgentLoop 实例 THEN 系统 SHALL 接受以下配置参数：
   - `provider`: LLM 提供者实例（实现 Provider trait，来自 `nano_provider` crate）
   - `config`: AgentDefaults 配置结构（来自 `nano_config` crate，**必选项**）

2. WHEN AgentLoop 创建完成 THEN 系统 SHALL 初始化内部状态，包括运行标志、消息历史等。

---

### 需求 2：消息上下文构建

**用户故事：** 作为开发者，我希望 AgentLoop 能够正确构建消息上下文，以便 LLM 能够理解对话历史。

#### 验收标准

1. WHEN 接收到用户消息 THEN 系统 SHALL 构建包含系统提示和用户消息的消息列表（使用 `nano_provider::Message`）。

2. WHEN 构建消息上下文 THEN 系统 SHALL 支持以下消息角色（通过 `Message::user()`、`Message::assistant()`、`Message::system()` 构造）：
   - `system`: 系统提示消息
   - `user`: 用户消息
   - `assistant`: 助手响应消息

3. IF 消息内容超过最大长度限制 THEN 系统 SHALL 进行适当的截断处理。

---

### 需求 3：LLM 调用与响应处理

**用户故事：** 作为开发者，我希望 AgentLoop 能够正确调用 LLM 并处理响应，以便完成对话交互。

#### 验收标准

1. WHEN 调用 LLM THEN 系统 SHALL 使用配置的 provider 和 `config.model` 参数。

2. WHEN LLM 返回响应 THEN 系统 SHALL 解析响应内容并返回给调用方。

3. IF LLM 调用超时 THEN 系统 SHALL 返回超时错误，而非无限等待。

4. IF LLM 调用失败 THEN 系统 SHALL 返回包含错误信息的响应，便于用户理解问题。

---

### 需求 4：迭代循环控制

**用户故事：** 作为开发者，我希望 AgentLoop 能够控制迭代次数，以便避免无限循环消耗资源。

#### 验收标准

1. WHEN 开始处理消息 THEN 系统 SHALL 初始化迭代计数器为 0。

2. WHEN 每次调用 LLM THEN 系统 SHALL 递增迭代计数器。

3. IF 迭代次数达到 `config.max_tool_iterations` 限制 THEN 系统 SHALL 停止迭代并返回提示消息。

4. WHEN 迭代正常完成 THEN 系统 SHALL 返回最终内容。

---

### 需求 5：错误处理与日志

**用户故事：** 作为开发者，我希望 AgentLoop 具有完善的错误处理和日志记录，以便于调试和运维。

#### 验收标准

1. WHEN 发生任何错误 THEN 系统 SHALL 使用 `anyhow::Result` 返回错误，并通过 `.context()` 添加语义化上下文。

2. WHEN 执行关键操作 THEN 系统 SHALL 使用 `tracing` 记录结构化日志，包括：
   - 初始化开始/完成
   - LLM 调用开始/完成
   - 响应长度等关键信息

3. WHEN 日志输出 THEN 系统 SHALL 使用中文描述，保持与项目规范一致。

---

### 需求 6：单元测试

**用户故事：** 作为开发者，我希望 AgentLoop 具有完善的单元测试覆盖，以便确保代码质量和可维护性。

#### 验收标准

1. WHEN 实现 AgentLoop THEN 系统 SHALL 提供单元测试文件 `tests.rs`，通过 `#[cfg(test)]` 引入。

2. WHEN 编写测试用例 THEN 系统 SHALL 使用表驱动测试（Table-Driven Tests）风格，定义 `Case` 和 `Expect` 结构体。

3. WHEN 测试函数命名 THEN 系统 SHALL 使用描述性命名（如 `agent_loop_process_message_ok`），不带 `test_` 前缀。

---

### 需求 7：模块结构与导出

**用户故事：** 作为开发者，我希望 AgentLoop 模块结构清晰，以便于其他模块引用和扩展。

#### 验收标准

1. WHEN 组织代码结构 THEN 系统 SHALL 采用以下文件结构：
   ```
   crates/agent/
   ├── Cargo.toml
   └── src/
       ├── lib.rs          # 模块入口和 re-export
       ├── loop.rs         # AgentLoop 核心实现
       └── tests.rs        # 单元测试
   ```

2. WHEN 其他 crate 引用 agent 模块 THEN 系统 SHALL 能够直接访问 `AgentLoop` 类型，并通过 re-export 访问 `AgentDefaults`（来自 `nano_config`）和 `Message`（来自 `nano_provider`）。

3. WHEN 定义公共 API THEN 系统 SHALL 仅暴露必要的类型和函数，内部实现保持私有。
