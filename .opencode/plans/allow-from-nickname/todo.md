# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/channels/src/dingtalk/mod.rs` | 修改 | check_permission 新增 sender_nick 参数 |
| `crates/channels/src/dingtalk/tests.rs` | 修改 | 新增昵称匹配测试 |
| `crates/channels/src/feishu/mod.rs` | 修改 | 新增 resolved_allow_from 字段 + 启动时解析 |
| `crates/channels/src/feishu/tests.rs` | 修改 | 新增解析逻辑测试 |

## 任务列表

### 1. ✅ 钉钉 check_permission 支持昵称匹配

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/dingtalk/mod.rs`, `crates/channels/src/dingtalk/tests.rs`
- 验收标准: `check_permission("staff_123", "张三")` 在 `allow_from: ["张三"]` 时返回 true
- 风险/注意点: 调用处需同时传入 sender_id 和 sender_nickname
- 信心评估: 5
- 步骤:
  - [ ] `check_permission` 签名改为 `(&self, sender_id: &str, sender_nickname: &str)`
  - [ ] 匹配逻辑：`allow_from` 中任一条目 == sender_id 或 == sender_nickname
  - [ ] 调用处 `process_message` 中传入 `sender_nickname`
  - [ ] 新增测试：昵称匹配通过、昵称不匹配拒绝、ID 和昵称混合配置

### 2. ✅ 飞书启动时解析 allow_from 中的昵称为 open_id

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/feishu/mod.rs`, `crates/channels/src/feishu/tests.rs`
- 验收标准: `allow_from: ["张三"]` 在启动时被解析为 `resolved_allow_from: ["ou_xxx"]`；`allow_from: ["ou_abc"]` 原样保留
- 风险/注意点: 搜索 API 需要特定 scope；解析失败不应阻塞启动
- 信心评估: 3（需确认飞书 SDK 搜索 API 调用方式）
- 步骤:
  - [ ] `Feishu` 新增字段 `resolved_allow_from: Vec<String>`
  - [ ] 新增 `async fn resolve_allow_from(client: &Client, allow_from: &[String]) -> Vec<String>` 辅助方法
  - [ ] 遍历 allow_from：`"*"` 和 `ou_` 前缀原样保留；其他调用搜索 API 解析
  - [ ] `new()` 中调用 `resolve_allow_from` 初始化字段
  - [ ] `check_permission` 改为匹配 `self.resolved_allow_from`
  - [ ] Clone impl 中包含 `resolved_allow_from`
  - [ ] 新增测试：ou_ 前缀原样保留、通配符保留、解析失败保留原始值
  - [ ] 运行 `cargo clippy` + `cargo test` 确认通过

## 实现建议

- 钉钉改动极小：签名加一个参数 + 匹配条件加一个 `||`
- 飞书搜索 API：优先尝试 `POST /open-apis/search/v1/user`（需 `search:user` scope），如果 SDK 不直接支持，可用 `client.request()` 通用方法构造请求
- 解析失败时保留原始条目并 warn，不 panic 不阻塞
