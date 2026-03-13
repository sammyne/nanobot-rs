# 需求文档

## 引言

本功能旨在将已实现的 `SpawnTool` 集成到 `AgentLoop` 中，使主代理能够通过调用 `spawn` 工具创建后台子代理来处理复杂或耗时的任务。

`SpawnTool` 允许主代理将复杂任务委托给独立的子代理执行，子代理完成后会将结果报告回主代理。这种设计使得主代理可以并行处理多个独立任务，提高了系统的并发能力和用户体验。

## 背景

- `SpawnTool` 已在 `nanobot-subagent` crate 中实现完成
- `SubagentManager` 已实现子代理的创建、管理和监控功能
- `AgentLoop` 当前已支持 `CronTool` 的注册模式
- 子代理通过 `mpsc::Sender<InboundMessage>` 向主代理报告完成结果

## 需求

### 需求 1：为 AgentLoop 添加 SubagentManager 支持

**用户故事：** 作为开发者，我希望 AgentLoop 能够持有 SubagentManager 实例，以便可以创建和管理子代理任务。

#### 验收标准

1. WHEN 调用 AgentLoop::new THEN 系统 SHALL 接受必选参数 `Arc<SubagentManager<P>>`
2. WHEN AgentLoop 存储字段初始化 THEN 系统 SHALL 将 SubagentManager 存储为 AgentLoop 的字段
3. WHEN AgentLoop 创建 SpawnTool 时 THEN 系统 SHALL 使用注入的 SubagentManager 实例

### 需求 2：在 AgentLoop 中注册 SpawnTool

**用户故事：** 作为开发者，我希望 AgentLoop 在初始化时自动注册 SpawnTool，以便 LLM 可以调用 `spawn` 工具。

#### 验收标准

1. WHEN AgentLoop 初始化且 SubagentManager 存在 THEN 系统 SHALL 创建 SpawnTool 实例
2. WHEN SpawnTool 创建完成 THEN 系统 SHALL 将其注册到 ToolRegistry 中
3. WHEN 注册完成后 THEN 系统 SHALL 更新 Provider 绑定的工具定义列表
4. WHEN 工具定义更新后 THEN 系统 SHALL 在 LLM 调用时包含 spawn 工具的描述

### 需求 3：建立子代理完成通知机制

**用户故事：** 作为开发者，我希望子代理完成任务后能够通知主代理，以便主代理可以将结果反馈给用户。

#### 验收标准

1. WHEN SubagentManager 创建时 THEN 系统 SHALL 接收一个消息总线发送端用于发送完成通知
2. WHEN 子代理完成任务 THEN 系统 SHALL 通过消息总线发送包含结果的 InboundMessage
3. WHEN AgentLoop 的 run 循环收到子代理完成消息 THEN 系统 SHALL 处理该消息并将结果发送到出站通道

### 需求 4：更新 AgentLoop 构造函数签名

**用户故事：** 作为开发者，我希望 AgentLoop 的构造函数接受必选的 SubagentManager 参数，以便能够使用子代理功能。

#### 验收标准

1. WHEN 调用 AgentLoop::new THEN 系统 SHALL 接受必选参数 `Arc<SubagentManager<P>>`
2. WHEN 调用 AgentLoop::new_direct THEN 系统 SHALL 接受必选参数 `Arc<SubagentManager<P>>`
3. WHEN AgentLoop 初始化完成 THEN 系统 SHALL 使用 SubagentManager 创建 SpawnTool 并注册到工具注册表

### 需求 5：更新依赖配置

**用户故事：** 作为开发者，我希望 agent crate 依赖 subagent crate，以便可以使用相关类型。

#### 验收标准

1. WHEN 编译 agent crate THEN 系统 SHALL 成功解析 nanobot-subagent 依赖
2. WHEN 编译通过 THEN 系统 SHALL 能在 agent crate 中使用 `SpawnTool` 和 `SubagentManager` 类型

### 需求 6：更新调用方代码

**用户故事：** 作为开发者，我希望所有调用 AgentLoop::new 的地方都能正确创建并传入 SubagentManager 参数，以便功能正常工作。

#### 验收标准

1. WHEN CLI agent 命令调用 AgentLoop::new THEN 系统 SHALL 先创建 SubagentManager 并传入
2. WHEN gateway 命令调用 AgentLoop::new THEN 系统 SHALL 先创建 SubagentManager 并传入
3. WHEN 创建 SubagentManager THEN 系统 SHALL 使用正确的 inbound_tx 参数用于子代理通知
4. WHEN 编译完成后 THEN 系统 SHALL 通过 `cargo clippy --all-targets --all-features -- -D warnings -D clippy::uninlined_format_args` 检查

## 技术考虑

### 架构模式

参考 `CronTool` 的集成模式，采用依赖注入设计：
- `AgentLoop::new()` 接受必选的 `Arc<SubagentManager<P>>` 参数
- SubagentManager 由调用方创建，AgentLoop 只负责使用
- 工具定义在注册后自动同步到 Provider

### 依赖注入优势

- **职责分离**：AgentLoop 专注代理循环，调用方负责基础设施创建
- **灵活性**：调用方可以自定义 SubagentManager 的配置参数
- **可测试性**：便于在测试中注入 Mock 的 SubagentManager

### 泛型约束

`SpawnTool<P>` 的泛型参数 `P` 需要满足与 `AgentLoop<P>` 相同的约束：
- `P: Provider + Clone + Send + Sync + 'static`

## 成功标准

1. 编译通过，所有依赖正确配置
2. AgentLoop::new 接受必选的 `Arc<SubagentManager<P>>` 参数
3. spawn 工具始终对 LLM 可用
4. 子代理完成任务后，主代理能够收到通知
5. 所有调用方（CLI agent、gateway）正确创建并传入 SubagentManager
