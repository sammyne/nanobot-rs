# 实施计划

- [ ] 1. 实现目标路由信息解析函数
   - 创建私有方法解析系统消息的 chat_id 字段
   - 支持冒号分隔符分割 channel 和 chat_id
   - 处理无分隔符情况，使用默认值 "cli" 作为 channel
   - 构建会话 key（格式："{channel}:{chat_id}"）
   - _需求：1.1、1.2、1.3_

- [ ] 2. 实现 process_system_message 私有方法
   - 定义方法签名，接收 InboundMessage 参数，返回 OutboundMessage
   - 调用路由解析函数获取目标 channel 和 chat_id
   - 获取或创建会话状态
   - 设置工具上下文
   - 构建消息历史
   - 调用 agent 核心处理逻辑
   - 保存会话状态
   - 返回带有目标路由信息的 OutboundMessage
   - _需求：2.1、2.2、2.3、2.4_

- [ ] 3. 集成到 process_message 方法
   - 在 process_message 方法入口处添加 channel 判断逻辑
   - 当 channel 为 "system" 时调用 process_system_message
   - 当 channel 不为 "system" 时保持现有处理逻辑不变
   - 确保返回值正确传递
   - _需求：3.1、3.2、3.3_

- [ ] 4. 编写单元测试
   - 测试路由解析函数的各种边界情况（有分隔符、无分隔符、空字符串）
   - 测试 process_system_message 方法的完整流程
   - 测试 process_message 对系统消息的正确路由
   - _需求：1.1、1.2、2.1、3.1_
