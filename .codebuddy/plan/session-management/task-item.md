# 实施计划

- [x] 1. 创建 session crate 项目结构
   - 在 `crates/session` 目录下创建新 crate
   - 配置 `Cargo.toml`：声明必要依赖（serde、chrono、tracing、parking_lot）
   - 在 workspace 根目录 `Cargo.toml` 中添加 members 并统一管理依赖版本
   - _需求：1.1_

- [x] 2. 实现 Session 数据模型和核心接口
   - 创建 `src/session.rs` 模块
   - 定义 `Session` 结构体：`key`、`messages`、`created_at`、`updated_at`、`metadata`、`last_consolidated` 字段
   - 实现 `Session::add_message`、`Session::get_history`、`Session::clear` 方法
   - 在 `get_history` 中实现 max_messages 限制和首条 user 消息对齐逻辑
   - _需求：1.2、3.1、3.2、4.1、4.2、4.3、5.1、5.2_

- [x] 3. 实现 SessionManager 持久化和缓存机制
   - 创建 `src/manager.rs` 模块
   - 定义 `SessionManager` 结构体：`workspace`、`sessions_dir`、`cache` 字段
   - 实现 `get_session_path` 方法生成 JSONL 文件路径
   - 实现 `load` 方法从 JSONL 文件加载会话（含元数据行解析）
   - 实现 `save` 方法持久化会话到 JSONL 文件（元数据行 + 消息行）
   - 实现 `get_or_create`、`invalidate` 方法管理内存缓存
   - 实现错误处理：文件损坏时记录警告并返回空会话
   - _需求：1.2、1.3、1.4、2.1、2.2、2.3、2.4、3.3、3.4_

- [x] 4. 实现会话列表查询功能
   - 在 `src/manager.rs` 中实现 `list_sessions` 方法
   - 遍历 sessions 目录解析每个 JSONL 文件的元数据行
   - 按 `updated_at` 降序排列返回结果
   - _需求：6.1、6.2、6.3_

- [x] 5. 编写单元测试
   - 在 `tests/` 目录下创建独立测试文件（session_test.rs、manager_test.rs）
   - 为 `Session::get_history` 编写测试（验证 max_messages 和 user 对齐）
   - 为 `SessionManager` 持久化流程编写测试（创建、保存、加载、缓存）
   - 为 `list_sessions` 编写测试（验证排序和字段完整性）
   - _需求：1.2、1.3、2.1、2.2、4.1、4.2、6.3_

- [x] 6. 导出公开接口和添加文档
   - 在 `src/lib.rs` 中导出 `Session`、`SessionManager` 及相关类型
   - 为所有公开接口添加 rustdoc 文档注释
   - _需求：7.1_

- [x] 7. 重构 agent crate 集成 session crate
   - 在 `crates/agent/Cargo.toml` 中添加对 `session` crate 的依赖
   - 修改 `AgentLoop` 结构体：将 `sessions: HashMap` 替换为 `sessions: SessionManager`（字段名保持不变）
   - 重构 `get_or_create_session` 和 `update_session` 方法调用 `SessionManager` API
   - 在 `new` 构造函数内初始化 `SessionManager`
   - _需求：7.1、7.2、7.3_

- [x] 5. 添加单元测试和集成测试
   - 为 `Session` 的 `get_history` 方法编写测试（验证 max_messages 和 user 对齐）
   - 为 `SessionManager` 的持久化流程编写测试（创建、保存、加载、缓存）
   - 为 `list_sessions` 编写测试（验证排序和字段完整性）
   - _需求：1.1、1.2、1.3、2.1、2.2、4.1、4.2、6.3_

- [x] 6. 更新模块公开接口和文档
   - 在 `crates/agent/src/lib.rs` 中导出 `Session` 和 `SessionManager`
   - 为公开接口添加 rustdoc 文档注释
   - _需求：7.1_

---

## 方案 A 实施：Session.messages 类型调整为 Vec<Message>

**实施日期**: 2026-03-02

### 变更摘要

将 `Session.messages` 的元素类型从 `Vec<serde_json::Value>` 调整为 `Vec<nanobot_provider::Message>`。

### 完成的任务

| 状态 | 任务 | 文件 |
|:----:|------|------|
| ✅ | 更新 session/Cargo.toml | 添加 nanobot-provider 依赖 |
| ✅ | 更新 session/src/session.rs | 修改 messages 类型、add_message 和 get_history 方法 |
| ✅ | 更新 session/src/manager.rs | 调整 load 方法中的消息解析逻辑 |
| ✅ | 更新 session/src/lib.rs | 导出 Message 类型，更新文档示例 |
| ✅ | 更新 session 测试文件 | 适配 Message 类型 API |
| ✅ | 简化 agent crate | 移除 get_or_create_session 和 update_session 中的 JSON 转换代码 |

### 收益

- **类型安全**: 编译期捕获消息格式错误
- **代码简化**: 移除 agent crate 中的序列化/反序列化样板代码
- **性能提升**: 消除运行时 JSON 转换开销
