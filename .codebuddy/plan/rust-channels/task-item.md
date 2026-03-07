# 实施计划：Rust 通道框架

## 目标
实现一个灵活、可扩展的 Rust 通道框架，支持多种聊天平台集成，提供异步消息处理、权限控制和消息路由等功能。

## 任务清单

- [x] 1. 设置项目结构和依赖
   - 在 `crates/` 下创建 `channels` crate
   - 配置 Cargo.toml 添加必要依赖：tokio、serde、thiserror、anyhow、tracing、reqwest、async-trait 等
   - 为钉钉通道添加钉钉 SDK 或相关 HTTP 客户端依赖
   - _需求：1、7、8_

- [x] 2. 定义核心通道抽象
   - 创建 `src/traits/mod.rs` 文件
   - 定义 `Channel` trait 包含 `start()`、`stop()`、`send()`、`is_running()` 方法
   - 定义通道配置 trait 或使用泛型配置结构
   - _需求：1_

- [x] 3. 实现消息类型系统
   - 创建 `src/messages/mod.rs` 文件
   - 定义 `InboundMessage` 结构体包含 channel、sender_id、chat_id、content、media、metadata 字段
   - 定义 `OutboundMessage` 结构体包含 channel、chat_id、content、media、metadata 字段
   - 为消息类型实现 serde 的 Serialize 和 Deserialize
   - _需求：3_

- [x] 4. 实现错误处理
   - 创建 `src/error/mod.rs` 文件
   - 使用 thiserror 定义 `ChannelError` 枚举类型
   - 包含启动失败、发送失败、配置错误、API 错误等变体
   - 实现错误到 std::error::Error 的转换
   - _需求：7_

- [x] 5. 实现配置管理
   - 创建 `src/config/mod.rs` 文件
   - 定义 `ChannelConfig` 结构体支持 enabled、token、allow_from、proxy、reply_to_message 字段
   - 实现 YAML/JSON 配置解析功能
   - 添加配置验证逻辑
   - _需求：8_

- [x] 6. 实现通道管理器
   - 创建 `src/manager/mod.rs` 文件
   - 定义 `ChannelManager` 结构体
   - 实现 `new()` 方法读取配置并创建启用的通道
   - 实现 `start_all()` 方法并发启动所有通道
   - 实现 `stop_all()` 方法停止所有通道并清理资源
   - 实现 `route_message()` 方法根据 channel 字段路由消息
   - 实现 `get_status()` 方法返回所有通道运行状态
   - _需求：2_

- [x] 7. 实现钉钉通道
   - 创建 `src/dingtalk/mod.rs` 文件
   - 实现 `DingTalk` 结构体包含 client_id、client_secret、access_token 等字段
   - 实现 Stream Mode (WebSocket) 连接和自动重连逻辑
   - 实现消息接收处理和 CallbackHandler
   - 实现 Access Token 获取和自动刷新机制（提前 60 秒）
   - 实现 `send()` 方法使用 HTTP API 发送 Markdown 消息
   - 维护后台任务集合防止 GC 回收
   - _需求：5_

- [x] 8. 添加权限控制
   - 在通道 trait 实现中添加权限检查逻辑
   - 实现 `check_permission()` 方法验证发送者是否在 allow_from 列表中
   - 处理发送者 ID 包含分隔符的情况
   - 拒绝未授权消息并记录警告日志
   - _需求：4_

- [x] 9. 添加日志记录
   - 在所有关键操作点添加 tracing 日志
   - 记录通道启动/停止、消息接收/发送、错误等事件
   - 支持调试模式的详细日志输出
   - _需求：9_

- [x] 10. 编写单元测试和集成测试
   - 为消息类型编写单元测试
   - 为错误处理编写单元测试
   - 为配置解析编写单元测试
   - 为通道管理器编写集成测试
   - 为钉钉通道编写模拟测试
   - _需求：10_

## ✅ 实施完成

所有任务已完成！Rust 通道框架已成功实现，包含以下功能：

### 核心功能
- ✅ 统一的通道抽象接口（Channel trait）
- ✅ 完善的消息类型系统（InboundMessage/OutboundMessage）
- ✅ 健壮的错误处理机制（ChannelError）
- ✅ 灵活的配置管理（YAML/JSON 支持）
- ✅ 强大的通道管理器（ChannelManager）

### 钉钉通道实现
- ✅ 使用 dingtalk-stream SDK 实现 Stream Mode
- ✅ 自动管理 Access Token（SDK 内置）
- ✅ 支持发送 Markdown 消息
- ✅ 自动重连机制
- ✅ 权限控制功能

### 代码质量
- ✅ 完整的单元测试覆盖
- ✅ 零 clippy 警告
- ✅ 详细的文档注释
- ✅ 全面的日志记录

### 测试状态
- ✅ 所有单元测试通过
- ✅ 所有集成测试通过
- ✅ 代码编译通过
