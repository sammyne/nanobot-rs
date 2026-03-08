# 需求文档

## 引言

Gateway 命令是 nanobot 的核心服务启动命令，负责初始化并协调所有后台服务的运行。该命令参考 Python 版实现，在 Rust 版本中实现等效功能，包括：启动 AgentLoop 和 ChannelManager，并提供优雅的启动和关闭机制。

## 需求

### 需求 1：Gateway 命令基础功能

**用户故事：** 作为一名运维人员，我希望通过命令行启动 nanobot gateway 服务，以便让 AI 助手能够接收和处理来自各渠道的消息。

#### 验收标准

1. WHEN 用户执行 `nanobot gateway` 命令 THEN 系统 SHALL 显示启动信息并初始化服务
2. WHEN 用户指定 `--port` 或 `-p` 参数 THEN 系统 SHALL 在指定端口启动服务（默认端口 18790）
3. WHEN 用户指定 `--verbose` 或 `-v` 参数 THEN 系统 SHALL 启用详细日志输出
4. IF 配置文件不存在 THEN 系统 SHALL 报错并提示用户先运行 onboard 命令
5. WHEN 服务启动成功 THEN 系统 SHALL 显示已启用的通道列表

### 需求 2：核心组件初始化

**用户故事：** 作为一名开发者，我希望 gateway 命令能够正确初始化所有核心组件，以便服务能够正常运行。

#### 验收标准

1. WHEN gateway 启动 THEN 系统 SHALL 加载配置文件（Config）
2. WHEN 配置加载成功 THEN 系统 SHALL 初始化 LLM Provider（OpenAILike）
3. WHEN Provider 初始化完成 THEN 系统 SHALL 创建 SessionManager 实例
4. WHEN SessionManager 创建完成 THEN 系统 SHALL 创建 AgentLoop 实例
5. WHEN AgentLoop 创建完成 THEN 系统 SHALL 创建 ChannelManager 实例并加载所有已启用的通道
6. IF 没有 API Key 配置 THEN 系统 SHALL 显示错误信息并退出

### 需求 3：并发服务运行与优雅关闭

**用户故事：** 作为一名运维人员，我希望 gateway 能够并发运行所有服务并支持优雅关闭，以便保证服务的稳定性和数据的完整性。

#### 验收标准

1. WHEN 所有组件初始化完成 THEN 系统 SHALL 并发启动 AgentLoop 和 ChannelManager
2. WHEN 用户按下 Ctrl+C THEN 系统 SHALL 捕获中断信号并启动优雅关闭流程
3. WHEN 优雅关闭启动 THEN 系统 SHALL 按顺序停止：AgentLoop -> ChannelManager
4. WHEN 所有服务停止完成 THEN 系统 SHALL 显示关闭完成信息并退出
5. IF 服务启动过程中发生错误 THEN 系统 SHALL 记录错误并尝试清理已启动的服务

### 需求 4：状态输出与日志

**用户故事：** 作为一名运维人员，我希望 gateway 能够清晰地输出服务状态信息，以便监控服务的运行状态。

#### 验收标准

1. WHEN gateway 启动 THEN 系统 SHALL 显示 logo 和启动信息
2. WHEN 通道加载完成 THEN 系统 SHALL 显示已启用的通道名称列表
3. WHEN 启用 verbose 模式 THEN 系统 SHALL 输出详细的调试日志
