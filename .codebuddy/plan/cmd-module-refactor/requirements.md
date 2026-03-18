# 需求文档

## 引言
本需求旨在重构 agent crate 中的命令处理逻辑，将分散在 loop 模块中的命令处理代码抽离到独立的 cmd 模块。通过为每个命令创建对应的结构体并实现 run 方法，实现命令的模块化设计，提高代码的可维护性和可扩展性。

## 需求

### 需求 1：创建 cmd 模块

**用户故事：** 作为一名开发者，我想要创建一个独立的 cmd 模块，以便将命令处理逻辑从 loop 模块中分离出来，提高代码组织性和可维护性。

#### 验收标准
1. WHEN 创建 cmd 模块时 THEN 系统 SHALL 在 crates/agent/src 目录下创建 cmd 子目录
2. WHEN 创建 cmd 模块时 THEN 系统 SHALL 在 cmd 目录下创建 mod.rs 文件作为模块入口
3. WHEN 创建 cmd 模块时 THEN 系统 SHALL 在 mod.rs 文件末尾添加 `#[cfg(test)] mod tests;` 引入测试模块
4. WHEN 创建 cmd 模块时 THEN 系统 SHALL 遵循项目的命名规范，使用 kebab-case 命名

### 需求 2：实现命令结构体基 trait

**用户故事：** 作为一名开发者，我想要定义一个统一的命令 trait，以便所有命令结构体都能实现相同的接口，方便在 loop 模块中统一调用。

#### 验收标准
1. WHEN 定义命令 trait 时 THEN 系统 SHALL 定义一个名为 Command 的 trait
2. WHEN 定义 Command trait 时 THEN 系统 SHALL 包含一个 async run 方法签名
3. WHEN 定义 Command trait 时 THEN 系统 SHALL run 方法接收必要的参数（如 AgentLoop、InboundMessage、session_key 等）
4. WHEN 定义 Command trait 时 THEN 系统 SHALL run 方法返回 Result<String, String> 表示命令执行结果（成功返回字符串，失败返回错误信息）

### 需求 3：实现 Help 命令

**用户故事：** 作为一名开发者，我想要将 /help 命令的逻辑封装到 HelpCmd 结构体中，以便命令逻辑独立且易于测试。

#### 验收标准
1. WHEN 创建 HelpCmd 结构体时 THEN 系统 SHALL 实现 Command trait
2. WHEN 创建 HelpCmd 时 THEN 系统 SHALL 在 cmd/help 子模块中定义，文件名为 help.rs
3. WHEN 执行 HelpCmd::run 时 THEN 系统 SHALL 返回帮助信息字符串
4. WHEN 执行 HelpCmd::run 时 THEN 系统 SHALL 返回的帮助信息与原有实现保持一致
5. WHEN 创建 cmd 模块时 THEN 系统 SHALL 在 mod.rs 中声明 pub mod help; 引入 help 子模块

### 需求 4：实现 New 命令

**用户故事：** 作为一名开发者，我想要将 /new 命令的逻辑封装到 NewCmd 结构体中，以便命令逻辑独立且易于测试。

#### 验收标准
1. WHEN 创建 NewCmd 结构体时 THEN 系统 SHALL 实现 Command trait
2. WHEN 创建 NewCmd 时 THEN 系统 SHALL 在 cmd/new 子模块中定义，文件名为 new.rs
3. WHEN 执行 NewCmd::run 时 THEN 系统 SHALL 执行记忆整合逻辑（archive_all=true）
4. WHEN 执行 NewCmd::run 时 THEN 系统 SHALL 清除当前会话
5. WHEN 执行 NewCmd::run 时 THEN 系统 SHALL 返回成功或错误消息
6. WHEN 执行 NewCmd::run 时 THEN 系统 SHALL 处理并发控制，避免重复整合
7. WHEN 创建 cmd 模块时 THEN 系统 SHALL 在 mod.rs 中声明 pub mod new; 引入 new 子模块

### 需求 5：重构 loop 模块的命令分发逻辑

**用户故事：** 作为一名开发者，我想要重构 loop 模块的 try_handle_cmd 方法，以便通过命令分发机制调用不同命令子模块中 Cmd 结构体的 run 方法，而不是直接处理命令逻辑。

#### 验收标准
1. WHEN 重构 try_handle_cmd 时 THEN 系统 SHALL 移除直接的命令处理逻辑
2. WHEN 重构 try_handle_cmd 时 THEN 系统 SHALL 根据命令名称从相应子模块构建对应的 Cmd 结构体实例
3. WHEN 重构 try_handle_cmd 时 THEN 系统 SHALL 调用 Cmd 结构体的 run 方法
4. WHEN 重构 try_handle_cmd 时 THEN 系统 SHALL 将 run 方法的返回值转换为 OutboundMessage
5. WHEN 重构 try_handle_cmd 时 THEN 系统 SHALL 保持对不支持命令的错误处理逻辑

### 需求 6：实现命令模块测试

**用户故事：** 作为一名开发者，我想要为每个命令子模块编写单元测试，以便确保命令逻辑的正确性。

#### 验收标准
1. WHEN 编写测试时 THEN 系统 SHALL 在 cmd/tests.rs 文件中创建集成测试
2. WHEN 编写 HelpCmd 测试时 THEN 系统 SHALL 验证返回的帮助信息内容
3. WHEN 编写 NewCmd 测试时 THEN 系统 SHALL 验证会话清除逻辑
4. WHEN 编写测试时 THEN 系统 SHALL 遵循项目的测试命名规范（不使用 test_ 前缀）
5. WHEN 编写测试时 THEN 系统 SHALL 在各命令子模块的文件中创建单元测试

### 需求 7：集成测试

**用户故事：** 作为一名开发者，我想要编写集成测试，以便验证命令分发和执行的完整流程。

#### 验收标准
1. WHEN 编写集成测试时 THEN 系统 SHALL 验证命令分发到正确命令结构体
2. WHEN 编写集成测试时 THEN 系统 SHALL 验证命令执行结果正确返回
3. WHEN 编写集成测试时 THEN 系统 SHALL 验证不支持命令的错误处理
