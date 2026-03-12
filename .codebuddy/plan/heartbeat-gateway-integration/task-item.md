## 实施计划

- [x] 1. 实现 pick_heartbeat_target 辅助函数
   - 在 gateway 命令模块中创建 pick_heartbeat_target 函数
   - 实现从已启用渠道中选择目标逻辑
   - 实现最近更新会话的优先选择逻辑
   - 添加默认值返回 ("cli", "direct")
   - _需求：4.1、4.2、4.3、4.4_

- [x] 2. 在 GatewayCmd 中初始化 HeartbeatService
   - 在 GatewayCmd 结构体中添加 HeartbeatService 字段
   - 实现 HeartbeatService 的初始化逻辑
   - 从配置文件读取 heartbeat 配置
   - 设置空的 on_execute 和 on_notify 回调
   - _需求：1.1、1.2、1.3、1.4、1.5_

- [x] 3. 实现 on_execute 回调函数
   - 创建 on_execute 回调闭包
   - 实现调用 AgentLoop::process_direct 的逻辑
   - 使用 "heartbeat" 作为 session_key
   - 调用 pick_heartbeat_target 获取目标渠道
   - 处理并返回执行结果
   - _需求：2.1、2.2、2.3、2.4、2.5_

- [x] 4. 实现 on_notify 回调函数
   - 创建 on_notify 回调闭包
   - 实现调用 pick_heartbeat_target 获取通知目标
   - 实现 "cli" 目标的跳过逻辑
   - 实现通过消息总线发送 OutboundMessage
   - 添加任务执行结果到消息内容
   - _需求：3.1、3.2、3.3、3.4、3.5_

- [x] 5. 配置 HeartbeatService 回调
   - 将 on_execute 回调绑定到 HeartbeatService
   - 将 on_notify 回调绑定到 HeartbeatService
   - _需求：1.4、1.5、2.1、3.1_

- [x] 6. 实现 HeartbeatService 启动逻辑
   - 在 CronService 启动后启动 HeartbeatService
   - 使用异步方式启动服务
   - 添加禁用状态的日志记录
   - 添加启动成功的日志记录（包含心跳间隔）
   - _需求：5.1、5.2、5.3、5.4_

- [x] 7. 实现 HeartbeatService 停止逻辑
   - 在关闭信号处理中添加 HeartbeatService 停止逻辑
   - 确保在其他服务关闭前停止
   - 调用 stop 方法并记录日志
   - _需求：6.1、6.2、6.3_

- [x] 8. 实现启动状态显示
   - 添加 HeartbeatService 配置信息显示
   - 显示心跳间隔
   - 显示禁用状态信息
   - _需求：7.1、7.2、7.3_

- [x] 9. 添加错误处理和日志
   - 为 HeartbeatService 初始化添加错误处理
   - 为回调函数执行添加错误处理
   - 添加关键的调试日志
   - _需求：技术约束 4、5_

- [x] 10. 测试集成功能
   - 编写单元测试验证 pick_heartbeat_target 逻辑
   - 测试 HeartbeatService 初始化
   - 测试回调函数的正确执行
   - 验证服务启动和关闭流程
   - _需求：成功标准 1-6_

## 实施完成

✅ 所有任务已完成！

**实现内容：**
- `pick_heartbeat_target` 函数：智能选择心跳通知的目标渠道
- `on_execute` 回调：执行心跳任务并返回结果
- `on_notify` 回调：发送心跳任务结果通知
- HeartbeatService 初始化和启动/停止逻辑
- 完善的单元测试（11个测试用例全部通过）

**测试覆盖：**
- 选择最近更新的启用渠道
- 跳过内部渠道（cli、system）
- 跳过未启用的渠道
- 空会话列表返回默认值
- 只有内部渠道时返回默认值
- 无启用的外部渠道时返回默认值
- 处理空 chat_id 的会话
- 处理无效的 session_key 格式

**文件修改：**
- `crates/cli/src/commands/gateway/mod.rs`：主要功能实现
- `crates/cli/src/commands/gateway/tests.rs`：单元测试