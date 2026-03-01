# Tools 功能需求文档

## 引言

本文档定义了 nanobot-agent 的工具（Tools）功能需求。工具系统允许 AI Agent 与外部环境交互，执行文件操作和命令执行任务。该功能将 Agent 从纯对话能力扩展为能够实际操作环境的智能助手。

参考实现：`file:///github.com/sammyne/nanobot-rs/_nanobot/nanobot/agent/tools`

## 需求

### 需求 1：基础工具抽象层

**用户故事：** 作为一名开发者，我希望有一个统一的基础工具抽象层，以便实现新的工具时有一致的接口规范。

#### 验收标准

1.1. WHEN 开发者创建新工具时，THEN 系统 SHALL 提供 `Tool` trait，包含 `name()`、`description()`、`parameters()` 和 `execute()` 方法

1.2. WHEN 系统需要序列化工具信息时，THEN 系统 SHALL 提供 `Tool::to_schema()` 方法返回 OpenAI Function Calling 格式的工具定义

1.3. WHEN 调用工具时传入参数，THEN 系统 SHALL 使用 JSON Schema 验证参数合法性，返回详细的验证错误信息

1.4. WHEN 参数验证失败时，THEN 系统 SHALL 返回明确的错误信息，列出所有验证失败的字段

1.5. WHEN 工具执行时发生未捕获异常，THEN 系统 SHALL 捕获异常并返回格式化的错误信息，不应让 Agent 崩溃

---

### 需求 2：工具注册与管理

**用户故事：** 作为一名应用开发者，我希望能够动态注册和管理工具，以便灵活配置 Agent 的能力集。

#### 验收标准

2.1. WHEN 系统启动时，THEN 系统 SHALL 提供 `ToolRegistry` 用于管理工具实例

2.2. WHEN 需要添加工具时，THEN 系统 SHALL 支持 `register(tool: Box<dyn Tool>)` 方法注册工具

2.3. WHEN 需要移除工具时，THEN 系统 SHALL 支持 `unregister(name: &str)` 方法注销工具

2.4. WHEN Agent 需要获取可用工具列表时，THEN 系统 SHALL 通过 `get_definitions()` 返回所有工具的 OpenAI 格式定义

2.5. WHEN Agent 调用工具时，THEN 系统 SHALL 通过 `execute(name, params).await` 异步执行指定工具

2.6. WHEN 指定的工具不存在时，THEN 系统 SHALL 返回错误信息并列出所有可用工具名称

2.7. WHEN 请求工具列表时，THEN 系统 SHALL 提供 `tool_names()` 返回所有已注册工具名称

---

### 需求 3：文件系统工具集

**用户故事：** 作为一名用户，我希望 Agent 能够操作文件系统，以便读取、编辑和管理项目文件。

#### 验收标准

3.1. WHEN 用户需要查看文件内容时，THEN 系统 SHALL 提供 `read_file` 工具，支持相对路径（基于 workspace）和绝对路径

3.2. WHEN 用户需要创建或覆盖文件时，THEN 系统 SHALL 提供 `write_file` 工具，自动创建父目录

3.3. WHEN 用户需要修改文件部分内容时，THEN 系统 SHALL 提供 `edit_file` 工具，要求 old_text 完全匹配，支持模糊匹配提示

3.4. WHEN 用户需要查看目录结构时，THEN 系统 SHALL 提供 `list_dir` 工具，返回格式化的目录列表

3.5. WHEN 文件操作路径超出允许目录（allowed_dir）时，THEN 系统 SHALL 返回权限错误，阻止越权访问

3.6. WHEN 使用相对路径时，THEN 系统 SHALL 将路径解析为相对于 workspace 的绝对路径

3.7. WHEN 文件不存在或不是普通文件时，THEN `read_file` SHALL 返回明确的错误信息

3.8. WHEN `edit_file` 的 old_text 在文件中不存在时，THEN 系统 SHALL 提示差异信息

3.9. WHEN `edit_file` 的 old_text 出现多次时，THEN 系统 SHALL 警告用户并要求提供更唯一的上下文

---

### 需求 4：Shell 命令执行工具

**用户故事：** 作为一名用户，我希望 Agent 能够执行 Shell 命令，以便进行系统操作和自动化任务。

#### 验收标准

4.1. WHEN 用户需要执行命令时，THEN 系统 SHALL 提供 `shell` 工具，支持设置工作目录、环境变量和超时

4.2. WHEN 命令包含危险操作（如 rm -rf /、格式化磁盘等）时，THEN 系统 SHALL 识别并阻止执行，返回安全警告

4.3. WHEN 命令执行超时时，THEN 系统 SHALL 终止进程并返回超时错误

4.4. WHEN 命令需要用户输入（stdin）时，THEN 系统 SHALL 自动拒绝并提示用户该命令需要交互

4.5. WHEN 命令执行完成时，THEN 系统 SHALL 返回退出码、stdout 和 stderr 的完整信息

4.6. WHEN 命令输出超出最大长度限制时，THEN 系统 SHALL 截断并标记为 "(truncated)"

4.7. WHEN 设置 `require_confirmation=true` 时，THEN 对于高风险命令 SHALL 要求用户确认后才执行

---

### 需求 5：配置与初始化

**用户故事：** 作为一名开发者，我希望工具系统能够根据配置文件自动初始化，以便简化部署。

#### 验收标准

5.1. WHEN 系统启动时，THEN 系统 SHALL 从 `AgentDefaults` 读取工具相关配置（workspace、allowed_dir、timeout 等）

5.2. WHEN `allowed_dir` 和 `workspace` 都未配置时，THEN 系统 SHALL 使用当前工作目录作为默认值

5.3. WHEN 配置变更时，THEN 系统 SHALL 支持热重载或重启后生效新的工具配置

---

### 需求 6：错误处理与日志

**用户故事：** 作为一名运维人员，我希望工具系统有良好的错误处理和日志记录，以便排查问题。

#### 验收标准

6.1. WHEN 工具执行发生错误时，THEN 系统 SHALL 返回格式化的错误字符串，包含错误类型和描述

6.2. WHEN 工具系统内部发生错误时，THEN 系统 SHALL 使用 `tracing` 记录错误日志，包括工具名称、参数和执行上下文

6.3. WHEN 参数验证失败时，THEN 错误信息 SHALL 指明具体哪个字段验证失败，期望值是什么

6.4. WHEN 工具执行超时时，THEN 错误信息 SHALL 包含超时时间配置和执行时长

6.5. WHEN 权限错误发生时，THEN 错误信息 SHALL 说明受限制的路径和允许的操作范围

---

### 需求 7：测试支持

**用户故事：** 作为一名开发者，我希望工具系统易于测试，以便保证代码质量。

#### 验收标准

7.1. WHEN 编写单元测试时，THEN 系统 SHALL 提供 `MockTool` 实现便于测试工具调用逻辑

7.2. WHEN 测试工具注册表时，THEN `ToolRegistry` SHALL 支持空注册和批量操作测试

7.3. WHEN 测试文件系统工具时，THEN 工具 SHALL 接受临时目录路径作为 workspace 参数

7.4. WHEN 测试 Shell 工具时，THEN 工具 SHALL 支持超时设置便于测试边界情况

7.5. WHEN 测试参数验证时，THEN `Tool::validate_params` SHALL 返回详细的错误列表便于断言

