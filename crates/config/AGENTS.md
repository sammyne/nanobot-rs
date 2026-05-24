# config crate

统一的配置加载、验证和管理（~/.nanobot/config.json）。

## 关键类型

- **`Config`** -- 顶层配置，字段：`providers`, `agents`, `channels`, `gateway`, `tools`
  - `load() -> Result<Option<Config>>` -- 从 `~/.nanobot/config.{json,yaml,yml}` 加载
  - `save()` -- 持久化到磁盘
  - `from_env()` -- 仅从环境变量创建
  - `validate()` -- 校验所有字段
- **`ProvidersConfig`** (enum) -- `Custom(ProviderConfig)` | `Anthropic(ProviderConfig)`
- **`ProviderConfig`** -- `api_key`, `api_base`, `extra_headers`
- **`AgentDefaults`** -- `workspace`, `model`, `max_tokens`, `temperature`, `max_tool_iterations`, `memory_window`
- **`ChannelsConfig`** -- `dingtalk: Option<DingTalkConfig>`, `feishu: Option<FeishuConfig>`, `send_tool_hints`, `send_progress`
- **`GatewayConfig`** -- `host`, `port`, `heartbeat: Option<HeartbeatConfig>`
- **`HeartbeatConfig`** -- `enabled`, `interval_s`
- **`McpServerConfig`** (enum) -- `Stdio { command, args, env }` | `Http { url, headers, tool_timeout }`
- **`ToolsConfig`** -- `mcp_servers`, `restrict_to_workspace`, `exec: Option<ExecToolConfig>`
- **`ConfigError`** (enum) -- 配置错误
- `resolve_config_path() -> Option<PathBuf>` -- 按优先级查找配置文件
- 静态路径：`HOME`, `NANOBOT_HOME_DIR`, `CONFIG_PATH`, `DEFAULT_WORKSPACE_PATH`

## 内部依赖

utils
