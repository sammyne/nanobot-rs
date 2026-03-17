# 实施计划

- [ ] 1. 重构 try_consolidate 方法签名
   - 将 session 参数从 `&mut Session` 改为 `&Session`
   - 修改返回类型为 `Result<Option<usize>>`
   - 移除方法内部对 session 字段的修改
   - _需求：1.1、1.2、1.3、2.1_

- [ ] 2. 移除 try_consolidate 中的持久化调用
   - 删除 `self.sessions.save(&session)` 调用
   - 保留记忆整合的核心逻辑
   - 确保方法只返回新的 last_consolidated 值
   - _需求：2.2、2.3_

- [ ] 3. 完善 try_consolidate 错误处理
   - 确保内部错误被正确传播
   - 添加必要的错误日志记录
   - _需求：1.4、4.1_

- [ ] 4. 修改 process_message 调用方逻辑
   - 接收 try_consolidate 的返回值 `Option<usize>`
   - 当返回 Some(new_value) 时，更新 session.last_consolidated
   - _需求：3.1、3.2_

- [ ] 5. 在 process_message 中添加持久化逻辑
   - 在 session 状态更新后调用 `self.sessions.save(&session)`
   - 添加持久化失败的错误日志记录
   - 确保持久化失败不影响消息处理主流程
   - _需求：3.3、3.4、4.2、4.3_

- [ ] 6. 验证重构结果
   - 确保编译通过，无类型错误
   - 检查所有调用点的逻辑正确性
   - _需求：全部_
