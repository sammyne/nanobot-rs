# 实施计划

- [ ] 1. 添加依赖和导入语句
   - 在 `crates/context/Cargo.toml` 中添加 `nanobot-skills` 依赖项
   - 在 `builder.rs` 中添加 `use nanobot_skills::SkillsLoader;` 及相关类型导入
   - _需求：5.1、5.2_

- [ ] 2. 修改 ContextBuilder 结构体定义
   - 在 `ContextBuilder` 结构体中添加 `skills: SkillsLoader` 字段
   - 修改 `new()` 方法，初始化 `SkillsLoader` 实例并存储（优雅降级处理初始化失败）
   - 实现 `skills()` 方法返回 `SkillsLoader` 的不可变引用
   - _需求：1.1、1.2、1.3、1.4、6.1_

- [ ] 3. 实现 Active Skills 内容集成
   - 在 `build_system_prompt()` 方法中调用 `skills.get_always_skills()` 获取常驻技能列表
   - 调用 `skills.load_skills_for_context()` 加载技能完整内容
   - 生成 `# Active Skills` 章节并添加到系统提示中
   - _需求：2.1、2.2、2.3、2.4、2.5_

- [ ] 4. 实现 Skills 摘要信息集成
   - 在 `build_system_prompt()` 方法中调用 `skills.build_skills_summary()` 获取技能摘要
   - 生成 `# Skills` 章节，包含指导说明和 XML 格式技能列表
   - 处理空摘要情况，添加必要的错误处理
   - _需求：3.1、3.2、3.3、3.4、3.5、6.2_

- [ ] 5. 重构系统提示组装逻辑
   - 按照正确顺序组装系统提示：Memory Context → Active Skills → Skills Summary
   - 使用 `\n\n---\n\n` 分隔符连接各章节
   - 实现空章节的跳过逻辑
   - _需求：4.1、4.2、4.3_

- [ ] 6. 完善错误处理和日志记录
   - 为 skills 相关操作添加警告和错误级别日志
   - 实现优雅降级机制，确保部分失败不影响整体功能
   - 添加调试日志记录加载的技能数量等信息
   - _需求：6.1、6.2、6.3、6.4_

- [ ] 7. 验证现有功能兼容性
   - 确认 Memory Context 继续正常工作
   - 确认现有测试全部通过
   - 验证 `build_messages()` 等方法的行为一致性
   - _需求：7.1、7.2、7.3、7.4、7.5_
