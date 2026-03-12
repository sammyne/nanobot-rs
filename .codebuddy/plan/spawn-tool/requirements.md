# 需求文档

## 引言

本功能旨在在 nanobot-rs 项目的 subagent crate 中实现 Rust 版本的 `SpawnTool`，参考 Python 版本的实现（`_nanobot/nanobot/agent/tools/spawn.py`）。

`SpawnTool` 是一个工具（Tool），允许主代理生成后台子代理来处理复杂或耗时的任务。子代理完成后会将结果报告回主代理。

## 设计决策

采用泛型方案 `SpawnTool<P: Provider>` 而非动态分发（`Arc<dyn Spawn>`），理由如下：
- 性能更优：泛型在编译期单态化，无运行时开销
- 类型安全：编译期确保类型正确
- 符合 Rust 最佳实践：在所有权和生命周期明确的场景下优先使用泛型

## 需求

### 需求 1：实现 SpawnTool<P> 泛型结构体

**用户故事：** 作为开发者，我希望 SpawnTool 能够持有子代理管理器并管理来源上下文信息。

#### 验收标准

1. WHEN 创建 SpawnTool<P> 实例 THEN 系统 SHALL 接受 `Arc<SubagentManager<P>>` 作为参数
2. WHEN SpawnTool 创建时 THEN 系统 SHALL 设置默认的 origin_channel 为 "cli" 和 origin_chat_id 为 "direct"
3. WHEN 调用 set_context 方法 THEN 系统 SHALL 更新 origin_channel 和 origin_chat_id 值

### 需求 2：为 SpawnTool<P> 实现 Tool trait

**用户故事：** 作为开发者，我希望 SpawnTool 实现 Tool trait，以便它可以被注册到 ToolRegistry 中并被 LLM 调用。

#### 验收标准

1. WHEN 调用 name() 方法 THEN 系统 SHALL 返回 "spawn" 字符串
2. WHEN 调用 description() 方法 THEN 系统 SHALL 返回描述子代理功能的英文描述
3. WHEN 调用 parameters() 方法 THEN 系统 SHALL 返回包含 task（必需）和 label（可选）参数的 JSON Schema
4. WHEN 调用 execute() 方法 THEN 系统 SHALL 调用 SubagentManager<P> 的 spawn 方法并返回启动状态消息

### 需求 3：更新模块导出

**用户故事：** 作为开发者，我希望 SpawnTool 能从 subagent crate 的公开 API 中访问。

#### 验收标准

1. WHEN SpawnTool 实现完成 THEN 系统 SHALL 在 lib.rs 中导出 SpawnTool
2. IF 用户引用 nanobot-subagent crate THEN 系统 SHALL 能访问 `SpawnTool` 类型

### 需求 4：添加必要的依赖

**用户故事：** 作为开发者，我希望所有必要的依赖都已正确配置。

#### 验收标准

1. WHEN 编译项目 THEN 系统 SHALL 成功编译，所有依赖正确链接
2. IF 需要新依赖 THEN 系统 SHALL 已添加到 Cargo.toml 中

## 技术考虑

### 泛型约束

`SpawnTool<P>` 的泛型参数 `P` 需要满足以下约束：
- `P: Provider + Clone + Send + Sync + 'static`

### 参数 Schema

参考 Python 版本，参数 Schema 应包含：
- `task` (string, required): 子代理需要完成的任务
- `label` (string, optional): 任务的简短标签（用于显示）

### 上下文传递

`SpawnTool` 需要记录来源上下文（channel 和 chat_id），这用于子代理完成时通知主代理。这些值可以通过 `set_context` 方法更新。
