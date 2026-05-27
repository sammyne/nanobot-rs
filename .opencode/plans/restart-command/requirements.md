# 需求

## 目标与背景

当前 Rust 版本支持 `/help`、`/new`、`/stop` 三个命令，缺少 `/restart` 命令。用户无法从消息渠道远程重启 agent 以重新加载配置。

对应上游 PR：HKUDS/nanobot#1751（feat(channels): add "restart" command）。

上游实现：使用 `python -m nanobot` + `os.execv` 原地重启进程（PR #1958 修复了 Windows 兼容性）。

## 方案

新增 `RestartCmd` 实现 `Command` trait，在 `try_handle_cmd` 中注册 `/restart`。执行时先发送 "Restarting..." 回复，然后通过 `std::process::Command` 重新启动当前进程（使用 `std::env::current_exe()` 获取可执行文件路径，`std::env::args()` 获取参数）。

## 功能需求列表

### 核心功能

1. 新增 `RestartCmd` 结构体，实现 `Command` trait
2. `try_handle_cmd` 中注册 `/restart` 命令
3. `/help` 输出中包含 `/restart` 说明
4. 重启实现：spawn 新进程后退出当前进程

## 边界与不做事项

- 不处理 Windows 特殊路径问题（Rust 的 `current_exe()` 已跨平台）
- 不实现优雅关闭（依赖现有的 SIGTERM 处理）

## 待确认事项

- 无
