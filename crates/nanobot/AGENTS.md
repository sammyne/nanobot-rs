# nanobot (binary crate)

CLI 入口，提供 onboard/agent/gateway/cron 子命令。

## 关键类型

- **`AgentCmd`** (clap) -- 交互式或单次 AI 对话模式；参数：`--message`, `--image`, `--session`
- **`GatewayCmd`** (clap) -- 启动完整后台服务（AgentLoop + ChannelManager + Cron + Heartbeat）；参数：`--port`, `--health-check-port`
- **`OnboardCmd`** (clap) -- 交互式初始化：创建配置、初始化工作空间
- **`CronCmd`** (clap) -- cron 任务 CLI 管理；子命令：`list`, `add`, `remove`, `enable`, `run`
- **`VERSION`** -- `"{cargo_version}-{git_rev}"` 常量

## 内部依赖

agent, channels, config, cron, heartbeat, provider, session, subagent, templates, utils
