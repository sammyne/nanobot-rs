# nanobot

[![Build](https://github.com/sammyne/nanobot-rs/actions/workflows/build.yml/badge.svg)](https://github.com/sammyne/nanobot-rs/actions/workflows/build.yml)

[HKUDS/nanobot](https://github.com/HKUDS/nanobot) 的 Rust 版实现。

## 1. 快速开始

### 1.1 安装

#### 方式 1：从 GitHub Release 下载（推荐）

访问 [Releases 页面](https://github.com/sammyne/nanobot-rs/releases) 下载对应平台的预编译二进制文件：

| 平台 | 下载链接 |
|------|---------|
| Linux (x86_64) | `nanobot-x.x.x-x86_64-linux` |
| macOS (Apple Silicon) | `nanobot-x.x.x-aarch64-macos` |

下载后将文件重命名为 `nanobot` 并添加执行权限：

```bash
# 以 v1.4.0 为例
# Linux
curl -L https://github.com/sammyne/nanobot-rs/releases/download/1.4.0/nanobot-1.4.0-x86_64-linux -o nanobot
chmod +x nanobot

# macOS (Apple Silicon)
curl -L https://github.com/sammyne/nanobot-rs/releases/download/1.4.0/nanobot-1.4.0-aarch64-macos -o nanobot
chmod +x nanobot
```

#### 方式 2：从源码构建

确保已安装 Rust >= 1.93，然后构建项目：

```bash
cargo build --release -p nanobot
```

二进制文件位于 `target/release/nanobot`（或 `target/debug/nanobot` 如果不使用 `--release`）。

### 1.2 初始化配置

可通过环境变量预设 LLM 配置：

```bash
# 指定使用兼容 OpenAI API 的模型服务提供商接口
export NANOBOT_AGENTS__DEFAULTS__PROVIDER=custom
# 兼容 OpenAI API 的 base URL（如 OpenRouter）
export NANOBOT_PROVIDERS__CUSTOM__API_BASE=https://api.openrouter.ai/v1
# 访问 base URL 所需的 API Key
export NANOBOT_PROVIDERS__CUSTOM__API_KEY=sk-or-v1-xxxx
# Agent 使用的默认模型
export NANOBOT_AGENTS__DEFAULTS__MODEL=anthropic/claude-opus-4-5
```

然后运行 `onboard` 命令初始化配置和工作空间：

```bash
nanobot onboard
```

此命令会：
1. 读取环境变量（如已设置）初始化 LLM Provider 配置
2. 在 `~/.nanobot/` 下创建配置文件 `config.json`
3. 在工作空间中生成模板文件（AGENTS.md、SOUL.md、TOOLS.md 等）

### 1.3 使用方式 1：命令行

启动交互式对话：

```bash
nanobot agent
```

或发送单条消息：

```bash
nanobot agent -m "你好" --session "cli:direct"
```

样例输出如下
```bash
你好！我是 nanobot 🐱，一个帮助你处理各种任务的 AI 助手。

我可以帮你：

- 📁 管理文件（读取、写入、编辑）
- 💻 执行 Shell 命令
- 🔍 搜索网络信息
- ⏰ 设置提醒和定时任务
- 💬 回答问题

有什么我可以帮你的吗？
```

### 1.4 使用方式 2：借助 IM 工具接入

#### 1.4.1 配置 IM 通道

<details>
<summary><b>钉钉（DingTalk）</b></summary>

使用 **Stream Mode** —— 无需公网 IP。

**1. 创建钉钉机器人**

- 访问 [钉钉开放平台](https://open-dev.dingtalk.com/)
- 创建新应用 → 添加 **机器人** 能力
- **配置**：开启 **Stream Mode**
- **权限**：添加发送消息所需的权限
- 从"凭证"中获取 **AppKey**（Client ID）和 **AppSecret**（Client Secret）
- 发布应用

**2. 配置**

在 `~/.nanobot/config.json` 中添加：

```json
{
  "channels": {
    "dingtalk": {
      "enabled": true,
      "clientId": "YOUR_APP_KEY",
      "clientSecret": "YOUR_APP_SECRET",
      "allowFrom": ["YOUR_STAFF_ID"]
    }
  }
}
```

> `allowFrom`：添加你的员工 ID。使用 `["*"]` 允许所有用户。

</details>

<details>
<summary><b>飞书（Feishu）</b></summary>

使用 **WebSocket 长连接** —— 无需公网 IP。

**1. 创建飞书机器人**

- 访问 [飞书开放平台](https://open.feishu.cn/app)
- 创建新应用 → 开启 **机器人** 能力
- **权限**：
  - `im:message`（发送消息）和 `im:message.p2p_msg:readonly`（接收消息）
  - **Streaming 回复**（默认开启）：添加 **`cardkit:card:write`**
  - 若**无法**添加 `cardkit:card:write`，可在配置中设置 `"streaming": false`
- **事件**：添加 `im.message.receive_v1`（接收消息）
  - 选择 **长连接** 模式
- 从"凭证与基础信息"获取 **App ID** 和 **App Secret**
- 发布应用

**2. 配置**

在 `~/.nanobot/config.json` 中添加：

```json
{
  "channels": {
    "feishu": {
      "enabled": true,
      "appId": "cli_xxx",
      "appSecret": "xxx",
      "allowFrom": ["ou_YOUR_OPEN_ID"]
    }
  }
}
```

> `allowFrom`：添加你的 open_id（在日志中可找到）。使用 `["*"]` 允许所有用户

**3. 运行**

```bash
nanobot gateway
```

> 提示：飞书使用 WebSocket 接收消息，无需配置 webhook 或公网 IP！

</details>

#### 1.4.2 使用 `gateway` 命令启动后台服务

```bash
# 使用 onboard 生成的默认配置启动
nanobot gateway
# 或指定自定义端口
nanobot gateway --port 18790
# 或指定健康检查可用端口
nanobot gateway --health-check-port 7860
```

启动成功后，在 IM 软件内，给配置好的 IM 机器人发送指令即可。

## 温馨提示
- 魔塔社区的 moonshot/Kimi-K2.5 模型的工具调用问题，会导致响应截断
