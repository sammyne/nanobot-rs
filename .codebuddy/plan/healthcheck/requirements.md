# 需求文档

## 引言

本功能旨在为 nanobot gateway 服务提供一个极简的 HTTP 健康检查端点。该端点基于 TCP 实现，对所有 HTTP 请求返回 200 状态码，用于支持 Kubernetes、Docker 等容器编排平台的服务探活（liveness/readiness probe）。

## 需求

### 需求 1：HTTP 健康检查服务

**用户故事：** 作为一名运维工程师，我希望 gateway 服务提供一个 HTTP 健康检查端点，以便容器编排平台能够检测服务是否存活。

#### 验收标准

1. WHEN gateway 服务启动且配置了健康检查端口 THEN 系统 SHALL 同时启动一个独立的 HTTP 健康检查服务
2. WHEN gateway 服务启动且未配置健康检查端口 THEN 系统 SHALL 不启动健康检查服务
3. WHEN HTTP 健康检查服务收到任意路径的请求 THEN 系统 SHALL 返回 HTTP 200 状态码
4. WHEN HTTP 健康检查服务收到请求 THEN 系统 SHALL 返回响应体 `OK`
5. IF 健康检查端口被占用 THEN 系统 SHALL 记录错误日志并继续运行主服务

### 需求 2：配置支持

**用户故事：** 作为一名系统管理员，我希望能够配置健康检查服务的端口，以便避免与其它服务端口冲突。

#### 验收标准

1. WHEN 用户在配置文件中指定 `gateway.health_check_port` THEN 系统 SHALL 使用该端口启动健康检查服务
2. WHEN 用户未配置健康检查端口 THEN 系统 SHALL 不启动健康检查服务（无默认端口）
3. WHEN 用户通过命令行参数 `--health-check-port` 指定端口 THEN 系统 SHALL 优先使用命令行参数值

### 需求 3：服务生命周期管理

**用户故事：** 作为一名运维工程师，我希望健康检查服务与主服务同步启动和关闭，以便确保服务状态的一致性。

#### 验收标准

1. WHEN gateway 主服务启动完成 THEN 系统 SHALL 在后台启动健康检查服务（如已配置）
2. WHEN gateway 主服务关闭 THEN 系统 SHALL 同时关闭健康检查服务（通过 tokio runtime 机制）
3. WHEN 健康检查服务启动失败 THEN 系统 SHALL 记录错误日志但不影响主服务运行

### 需求 4：极简实现

**用户故事：** 作为一名开发者，我希望健康检查服务实现极简，以便最小化资源消耗和维护成本。

#### 验收标准

1. WHEN 实现健康检查服务 THEN 系统 SHALL 仅使用 tokio 的异步 TCP API（不引入额外 HTTP 框架）
2. WHEN 处理请求 THEN 系统 SHALL 仅解析 HTTP 请求行，忽略所有请求头和请求体
3. WHEN 返回响应 THEN 系统 SHALL 返回最小化的 HTTP 响应（状态行 + 简单响应体）
