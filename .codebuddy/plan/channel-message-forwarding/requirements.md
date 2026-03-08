# 需求文档

## 引言

本需求文档描述了为 `ChannelManager` 引入异步消息转发机制的功能增强。该功能旨在：

1. **出站消息转发**：`ChannelManager` 通过注入的出站消息接收端监听 `OutboundMessage`，并自动路由到相应通道
2. **入站消息传递**：`DingTalk` 通道通过注入的入站消息发送端将接收到的消息发送给响应接收端
3. **非阻塞通道启动**：每个通道的 `start` 方法在独立的 tokio 任务中执行，避免相互阻塞

## 需求

### 需求 1：ChannelManager 构造函数参数扩展

**用户故事：** 作为一名开发者，我希望 ChannelManager 能够接收外部的消息通道端点，以便实现消息的依赖注入和生命周期解耦。

#### 验收标准

1. WHEN 创建 ChannelManager 实例 THEN 系统 SHALL 接受 `mpsc::Receiver<OutboundMessage>` 类型的出站消息接收端作为构造参数
2. WHEN 创建 ChannelManager 实例 THEN 系统 SHALL 接受 `mpsc::Sender<InboundMessage>` 类型的入站消息发送端作为构造参数
3. WHEN ChannelManager 构造完成 THEN 系统 SHALL 持有上述两个通道端点用于后续消息处理

### 需求 2：ChannelManager 非阻塞通道启动

**用户故事：** 作为一名开发者，我希望 ChannelManager 能够以非阻塞方式启动所有通道，以便各通道能够并发运行而不会相互阻塞。

#### 验收标准

1. WHEN ChannelManager 调用 `start_all` 方法 THEN 系统 SHALL 为每个通道创建一个独立的 tokio 任务
2. WHEN 为通道创建 tokio 任务 THEN 系统 SHALL 在该任务中调用通道的 `start` 方法
3. IF 某个通道的 `start` 方法阻塞 THEN 系统 SHALL 不影响其他通道的启动和运行
4. WHEN 所有通道启动任务创建完成 THEN 系统 SHALL `start_all` 方法立即返回，不等待任何通道的 `start` 方法完成

### 需求 3：ChannelManager 出站消息监听与转发

**用户故事：** 作为一名开发者，我希望 ChannelManager 能够自动监听并转发出站消息，以便上层模块无需手动调用路由方法即可发送消息到目标通道。

#### 验收标准

1. WHEN ChannelManager 启动所有通道（`start_all`）THEN 系统 SHALL 启动一个后台任务监听注入的出站消息接收端
2. WHEN 后台任务收到 OutboundMessage THEN 系统 SHALL 根据消息的 `channel` 字段将消息路由到对应的通道
3. IF 目标通道不存在 THEN 系统 SHALL 记录警告日志并忽略该消息
4. WHEN ChannelManager 停止所有通道（`stop_all`）THEN 系统 SHALL 先停止后台监听任务，再停止各通道
5. WHEN 出站消息接收端关闭 THEN 系统 SHALL 正确退出后台监听任务

### 需求 4：DingTalk 入站消息发送机制

**用户故事：** 作为一名开发者，我希望 DingTalk 通道在收到消息时能够通过注入的发送端将消息发送出去，以便 ChannelManager 或其他组件能够接收并处理这些消息。

#### 验收标准

1. WHEN 创建 DingTalk 通道 THEN 系统 SHALL 接受 `mpsc::Sender<InboundMessage>` 类型的发送端（从 ChannelManager 注入）
2. WHEN DingTalk 收到聊天消息（`process_message` 被调用）THEN 系统 SHALL 将消息转换为 `InboundMessage` 类型
3. WHEN 转换完成后 THEN 系统 SHALL 通过注入的 `mpsc::Sender` 将 `InboundMessage` 发送到 channel
4. IF channel 发送失败（如接收端已关闭）THEN 系统 SHALL 记录错误日志但不影响通道运行
5. WHEN DingTalk 停止运行 THEN 系统 SHALL 不再尝试发送消息到已关闭的 channel

### 需求 5：通道创建与消息端点注入

**用户故事：** 作为一名开发者，我希望 ChannelManager 在创建通道时能够正确注入消息发送端，以便通道能够正常发送入站消息。

#### 验收标准

1. WHEN ChannelManager 创建 DingTalk 通道 THEN 系统 SHALL 将构造函数接收的入站消息发送端传递给 DingTalk
2. WHEN ChannelManager 持有入站消息发送端 THEN 系统 SHALL 在创建各通道时统一注入该发送端
3. IF 通道不需要入站消息发送功能 THEN 系统 SHALL 不强制要求注入发送端

