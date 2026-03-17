# 实施计划

- [ ] 1. 扩展 AgentLoop 结构支持整合状态追踪
   - 在 `AgentLoop` 结构中添加 `consolidating: Mutex<HashSet<String>>` 字段
   - 在 `AgentLoop::new` 方法中初始化该字段为空集合
   - 添加必要的 `use` 语句（`std::sync::Mutex`, `std::collections::HashSet`）
   - _需求：4.1、4.2_

- [ ] 2. 修改整合触发条件判断逻辑
   - 在 `process_message` 中实现新的条件判断：`len(session.messages) - session.last_consolidated >= memory_window`
   - 组合检查消息窗口条件和整合状态条件
   - 确保两个条件都满足时才触发整合
   - _需求：1.1、1.2、3.1、3.2_

- [ ] 3. 集成整合状态追踪到整合流程
   - 在调用 `try_consolidate` 前直接操作 `consolidating` 字段标记整合开始（插入会话ID）
   - 在整合完成后（成功或失败）直接操作 `consolidating` 字段清除状态（移除会话ID）
   - 确保错误情况下也能正确清除状态（使用 `defer` 或 `match` 模式）
   - _需求：2.1、2.3、3.3_

- [ ] 4. 添加单元测试验证整合触发条件
   - 测试消息数量未达到阈值时不触发整合
   - 测试消息数量达到阈值时触发整合
   - 测试整合进行中时拒绝新的整合请求
   - _需求：1.1、1.2、2.2_

- [ ] 5. 添加单元测试验证状态管理
   - 测试整合状态标记和清除的正确性
   - 测试整合失败时状态仍能正确清除
   - 测试 `last_consolidated` 值在失败时保持不变
   - _需求：2.3、2.4_

- [ ] 6. 添加并发安全性测试
   - 测试多线程并发访问 `consolidating` 状态的正确性
   - 验证 `Mutex` 能有效防止并发整合
   - _需求：2.2、技术说明-线程安全评估_
