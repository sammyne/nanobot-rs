# 需求

## 目标与背景

对应上游 HKUDS/nanobot PR #1400（#31），添加 SIGTERM 信号处理，使 systemd `systemctl stop` 能触发优雅关闭。

**现状问题**：gateway 仅监听 `ctrl_c`（SIGINT），systemd 默认发 SIGTERM 停止服务，进程不响应，最终被 SIGKILL 强杀，无法优雅关闭。

## 方案比较（强制）

### 方案 1: tokio::select! 合并信号（最小可行版）

- 思路: 用 `tokio::select!` 同时等待 ctrl_c 和 SIGTERM，任一触发即开始关闭
- 优点: 改动最小，一个 `select!` 替换一个 `await`
- 缺点: 无
- 工作量估算: S

### 方案 2: CancellationToken 全局信号（理想架构）

- 思路: 创建全局 CancellationToken，在信号 handler 中 cancel，所有服务监听 token
- 优点: 更灵活的关闭协调
- 缺点: 过度设计，当前架构已有顺序关闭逻辑
- 工作量估算: M

### 推荐

方案 1。当前关闭逻辑已经是顺序的（stop heartbeat → abort agent → stop cron → stop channels），只需让信号触发点支持 SIGTERM。

## 功能需求列表

### 核心功能

- gateway 同时监听 SIGINT（ctrl_c）和 SIGTERM，任一触发即开始优雅关闭
- 使用 `#[cfg(unix)]` 条件编译，非 Unix 平台回退到仅 ctrl_c

### 扩展功能

- 无

## 非功能需求

- **兼容性**：`#[cfg(unix)]` 保证 Windows 编译不报错（虽然当前不支持 Windows）

## 边界与不做事项

- 不处理 SIGHUP（上游做了但 nanobot-rs 无 reload 逻辑）
- 不处理 SIGPIPE（Rust 默认忽略 SIGPIPE）
- 不修改 agent 命令（交互式模式不需要 SIGTERM）

## 待确认事项

- 无
