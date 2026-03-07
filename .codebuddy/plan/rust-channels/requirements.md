# 需求文档：Rust 通道框架

## 引言
本文档定义了 nanobot-rs 项目的通道框架需求。该框架需要提供一个灵活、可扩展的架构，用于集成多种聊天平台（Telegram、Discord、WhatsApp 等）到 nanobot 系统。框架需要支持异步消息处理、权限控制、消息路由等功能，并遵循 Rust 的最佳实践，包括类型安全、零成本抽象和内存安全。

## 需求

### 需求 1：基础通道抽象

**用户故事：** 作为一名【开发者】，我希望【定义一个抽象的通道 trait】，以便【为不同的聊天平台实现统一的接口】。

#### 验收标准

1. WHEN 【开发者创建通道实现】 THEN 【系统】 SHALL 【要求实现 `Channel` trait】
2. WHEN 【`Channel` trait 被定义】 THEN 【系统】 SHALL 【包含以下方法】：
   - `async fn start(&mut self) -> Result<()>` - 启动通道
   - `async fn stop(&mut self) -> Result<()>` - 停止通道
   - `async fn send(&self, msg: OutboundMessage) -> Result<()>` - 发送消息
3. IF 【通道运行状态被查询】 THEN 【系统】 SHALL 【提供 `is_running(&self) -> bool` 方法】
4. WHEN 【通道配置被传递】 THEN 【系统】 SHALL 【接受泛型配置类型或使用动态配置】
5. WHEN 【消息总线被传递】 THEN 【系统】 SHALL 【提供向总线发布消息的能力】

### 需求 2：通道管理器

**用户故事：** 作为一名【系统用户】，我希望【有一个通道管理器来协调所有通道】，以便【统一管理和控制多个聊天平台】。

#### 验收标准

1. WHEN 【通道管理器被初始化】 THEN 【系统】 SHALL 【读取配置并创建启用的通道】
2. WHEN 【`start_all()` 被调用】 THEN 【系统】 SHALL 【并发启动所有通道】
3. WHEN 【`stop_all()` 被调用】 THEN 【系统】 SHALL 【停止所有通道并清理资源】
4. WHEN 【出站消息到达】 THEN 【系统】 SHALL 【根据消息的 channel 字段路由到对应通道】
5. IF 【目标通道不存在】 THEN 【系统】 SHALL 【记录警告日志】
6. WHEN 【通道启动失败】 THEN 【系统】 SHALL 【记录错误并继续启动其他通道】
7. WHEN 【`get_status()` 被调用】 THEN 【系统】 SHALL 【返回所有通道的运行状态】

### 需求 3：消息类型定义

**用户故事：** 作为一名【开发者】，我希望【定义统一的消息类型】，以便【在通道和消息总线之间传递消息】。

#### 验收标准

1. WHEN 【入站消息被创建】 THEN 【系统】 SHALL 【包含以下字段】：
   - `channel: String` - 通道名称
   - `sender_id: String` - 发送者标识
   - `chat_id: String` - 聊天标识
   - `content: String` - 消息文本
   - `media: Vec<String>` - 媒体文件路径列表
   - `metadata: HashMap<String, Value>` - 元数据
2. WHEN 【出站消息被创建】 THEN 【系统】 SHALL 【包含以下字段】：
   - `channel: String` - 目标通道
   - `chat_id: String` - 目标聊天
   - `content: String` - 消息文本
   - `media: Vec<String>` - 媒体文件路径
   - `metadata: HashMap<String, Value>` - 元数据
3. WHEN 【消息被序列化/反序列化】 THEN 【系统】 SHALL 【支持 serde】

### 需求 4：权限控制

**用户故事：** 作为一名【管理员】，我希望【控制哪些用户可以使用 bot】，以便【保护系统安全】。

#### 验收标准

1. WHEN 【入站消息到达】 THEN 【系统】 SHALL 【检查发送者是否在允许列表中】
2. IF 【允许列表为空】 THEN 【系统】 SHALL 【允许所有发送者】
3. IF 【发送者不在允许列表中】 THEN 【系统】 SHALL 【拒绝消息并记录警告】
4. WHEN 【发送者 ID 包含分隔符（如 `|`）】 THEN 【系统】 SHALL 【分别检查每个部分】

### 需求 5：钉钉通道实现

**用户故事：** 作为一名【用户】，我希望【通过钉钉与 bot 交互】，以便【使用企业级的沟通协作平台】。

#### 验收标准

1. WHEN 【钉钉通道被启动】 THEN 【系统】 SHALL 【使用 Stream Mode (WebSocket) 建立与钉钉开放平台的连接】
2. WHEN 【钉钉 Stream Client 初始化】 THEN 【系统】 SHALL 【使用 Client ID 和 Client Secret 进行认证】
3. WHEN 【Stream 连接断开】 THEN 【系统】 SHALL 【自动尝试重连（最多 5 秒延迟）】
4. WHEN 【接收到文本消息】 THEN 【系统】 SHALL 【通过 CallbackHandler 解析并转发到消息总线】
5. WHEN 【消息被解析】 THEN 【系统】 SHALL 【提取 sender_id、sender_name 和 content 字段】
6. WHEN 【发送消息】 THEN 【系统】 SHALL 【使用 HTTP API 获取 Access Token】
7. WHEN 【Access Token 过期】 THEN 【系统】 SHALL 【自动刷新 Access Token（提前 60 秒）】
8. WHEN 【发送消息】 THEN 【系统】 SHALL 【使用 oToMessages/batchSend API 发送 Markdown 消息】
9. WHEN 【消息处理失败】 THEN 【系统】 SHALL 【返回 OK 状态码以避免钉钉服务端重试】
10. WHEN 【通道运行时】 THEN 【系统】 SHALL 【维护后台任务集合以防止 GC 回收未完成的任务】
11. WHEN 【通道停止】 THEN 【系统】 SHALL 【取消所有后台任务并关闭 HTTP 客户端】
12. WHEN 【发送者未在 allow_from 列表】 THEN 【系统】 SHALL 【拒绝消息并记录警告日志】

### 需求 6：可扩展性

**用户故事：** 作为一名【开发者】，我希望【轻松添加新的通道实现】，以便【支持更多聊天平台】。

#### 验收标准

1. WHEN 【开发者添加新通道】 THEN 【系统】 SHALL 【只需实现 `Channel` trait】
2. WHEN 【新通道被添加】 THEN 【系统】 SHALL 【自动被通道管理器识别】
3. WHEN 【通道需要额外依赖】 THEN 【系统】 SHALL 【通过 feature flag 可选启用】

### 需求 7：错误处理

**用户故事：** 作为一名【运维人员】，我希望【系统能妥善处理错误】，以便【保持系统稳定运行】。

#### 验收标准

1. WHEN 【通道启动失败】 THEN 【系统】 SHALL 【返回 `Result` 类型的错误】
2. WHEN 【消息发送失败】 THEN 【系统】 SHALL 【记录错误但不崩溃】
3. WHEN 【通道运行时发生错误】 THEN 【系统】 SHALL 【捕获并记录错误】
4. IF 【错误是致命的】 THEN 【系统】 SHALL 【尝试优雅关闭通道】
5. WHEN 【外部 API 调用失败】 THEN 【系统】 SHALL 【使用 `thiserror` 定义清晰的错误类型】

### 需求 8：配置管理

**用户故事：** 作为一名【用户】，我希望【通过配置文件控制通道行为】，以便【灵活调整系统】。

#### 验收标准

1. WHEN 【配置被加载】 THEN 【系统】 SHALL 【支持 YAML/JSON 格式】
2. WHEN 【通道配置包含】 THEN 【系统】 SHALL 【支持以下字段】：
   - `enabled: bool` - 是否启用
   - `token: String` - API 令牌
   - `allow_from: Vec<String>` - 允许的用户列表
   - `proxy: Option<String>` - 代理配置
   - `reply_to_message: bool` - 是否回复消息
3. WHEN 【配置无效】 THEN 【系统】 SHALL 【提供清晰的错误信息】
4. WHEN 【配置更改】 THEN 【系统】 SHALL 【支持热重载（可选）】

### 需求 9：日志记录

**用户故事：** 作为一名【运维人员】，我希望【系统提供详细的日志】，以便【调试和监控系统】。

#### 验收标准

1. WHEN 【通道启动】 THEN 【系统】 SHALL 【记录启动信息】
2. WHEN 【通道停止】 THEN 【系统】 SHALL 【记录停止信息】
3. WHEN 【消息被接收】 THEN 【系统】 SHALL 【记录消息摘要】
4. WHEN 【消息被发送】 THEN 【系统】 SHALL 【记录发送成功/失败】
5. WHEN 【错误发生】 THEN 【系统】 SHALL 【记录错误详情】
6. WHEN 【调试模式启用】 THEN 【系统】 SHALL 【记录详细的调试信息】

### 需求 10：测试支持

**用户故事：** 作为一名【新开发者】，我希望【有清晰的文档和示例】，以便【快速理解和使用框架】。

#### 验收标准

1. WHEN 【框架被发布】 THEN 【系统】 SHALL 【包含 API 文档】
2. WHEN 【开发者查看文档】 THEN 【系统】 SHALL 【提供添加新通道的指南】
3. WHEN 【示例被查看】 THEN 【系统】 SHALL 【包含至少一个完整的通道实现示例】
4. WHEN 【配置被查看】 THEN 【系统】 SHALL 【提供配置文件示例】
