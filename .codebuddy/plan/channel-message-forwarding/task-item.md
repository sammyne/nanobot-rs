# 实施计划

## 任务清单

- [ ] 1. 扩展 ChannelManager 结构体和构造函数
   - 添加 `outbound_rx: mpsc::Receiver<OutboundMessage>` 字段
   - 添加 `inbound_tx: mpsc::Sender<InboundMessage>` 字段
   - 添加 `outbound_task_handle: Option<JoinHandle<()>>` 字段用于跟踪监听任务
   - 添加 `channel_task_handles: HashMap<String, JoinHandle<()>>` 字段用于跟踪各通道启动任务
   - 修改 `new` 方法签名，接受这两个 channel 端点作为参数
   - _需求：1.1、1.2、1.3_

- [ ] 2. 实现非阻塞通道启动机制
   - 修改 `start_all` 方法，为每个通道创建独立的 tokio::spawn 任务
   - 在每个任务中调用 `channel.start().await`
   - 将任务句柄保存到 `channel_task_handles` 中
   - `start_all` 方法立即返回，不等待任务完成
   - 添加错误日志记录启动失败的通道
   - _需求：2.1、2.2、2.3、2.4_

- [ ] 3. 实现出站消息监听与转发后台任务
   - 在 `start_all` 中启动一个 tokio 任务监听 `outbound_rx`
   - 实现消息路由逻辑：根据 `msg.channel` 查找对应通道并调用 `send` 方法
   - 添加目标通道不存在的警告日志
   - 处理接收端关闭的情况，正确退出任务
   - 将监听任务句柄保存到 `outbound_task_handle` 字段
   - _需求：3.1、3.2、3.3、3.5_

- [ ] 4. 实现优雅停止机制
   - 修改 `stop_all` 方法，先 drop `outbound_rx` 触发监听任务退出
   - 等待 `outbound_task_handle` 任务完成（使用 `join` 或 `abort`）
   - 停止所有通道并等待启动任务完成
   - 清理 `channel_task_handles`
   - _需求：3.4_

- [ ] 5. 扩展 DingTalk 结构体支持入站消息发送
   - 在 `DingTalk` 结构体中添加 `inbound_tx: mpsc::Sender<InboundMessage>` 字段
   - 修改 `DingTalk::new` 方法接受该参数
   - _需求：4.1_

- [ ] 6. 实现 DingTalk 入站消息发送逻辑
   - 在 `process_message` 回调中，将钉钉消息转换为 `InboundMessage`
   - 通过 `inbound_tx.send()` 发送消息到 channel
   - 处理发送失败的情况，记录错误日志
   - _需求：4.2、4.3、4.4、4.5_

- [ ] 7. 实现通道创建时的消息端点注入
   - 修改 `add_dingtalk_channel` 方法，将 `inbound_tx` 克隆后传递给 DingTalk
   - 更新相关单元测试
   - _需求：5.1、5.2_
