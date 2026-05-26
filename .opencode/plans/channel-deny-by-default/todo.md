# TODO

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/channels/src/feishu/mod.rs` | 修改 | check_permission: deny-by-default + 移除 `\|` 分割 + 通配符 |
| `crates/channels/src/feishu/tests.rs` | 修改 | 更新权限测试 |
| `crates/channels/src/dingtalk/mod.rs` | 修改 | check_permission: deny-by-default + 移除 `\|` 分割 + 通配符 |
| `crates/channels/src/dingtalk/tests.rs` | 修改 | 更新权限测试 |

## 任务列表

### 1. ✅ 修改飞书 check_permission

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/feishu/mod.rs`
- 验收标准: 空 allow_from 返回 false + warn 日志；`"*"` 允许所有人；精确匹配 sender_id；无 `|` 分割
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `allow_from.is_empty()` 分支从 `return true` 改为 `warn!(...); return false`
  - [ ] 新增 `allow_from.contains("*")` 检查，返回 `true`
  - [ ] 移除 `sender_id.contains('|')` 及 `split('|')` 分支，仅保留 `allow_from.contains(&sender_id.to_string())`

### 2. ✅ 更新飞书权限测试

- 优先级: P0
- 依赖项: 1
- 涉及文件: `crates/channels/src/feishu/tests.rs`
- 验收标准: 覆盖空列表拒绝、通配符允许、精确匹配、不匹配拒绝
- 风险/注意点: 现有测试 `permission_check_empty_whitelist` 和 `permission_check_with_whitelist` 需要调整断言
- 信心评估: 5
- 步骤:
  - [ ] `permission_check_empty_whitelist`: 断言从 `assert!(true)` 改为 `assert!(!channel.check_permission(...))`
  - [ ] `permission_check_with_whitelist`: 移除 `|` 分割相关断言（`user1|extra` 不再匹配）
  - [ ] 新增测试 `permission_check_wildcard`: `allow_from: ["*"]` 允许所有人

### 3. ✅ 修改钉钉 check_permission

- 优先级: P0
- 依赖项: 无
- 涉及文件: `crates/channels/src/dingtalk/mod.rs`
- 验收标准: 与飞书相同的行为
- 风险/注意点: 无
- 信心评估: 5
- 步骤:
  - [ ] `allow_from.is_empty()` 分支从 `return true` 改为 `warn!(...); return false`
  - [ ] 新增 `allow_from.contains("*")` 检查，返回 `true`
  - [ ] 移除 `sender_id.contains('|')` 及 `split('|')` 分支，仅保留 `allow_from.contains(&sender_id.to_string())`

### 4. ✅ 更新钉钉权限测试

- 优先级: P0
- 依赖项: 3
- 涉及文件: `crates/channels/src/dingtalk/tests.rs`
- 验收标准: 覆盖空列表拒绝、通配符允许、精确匹配、不匹配拒绝
- 风险/注意点: 现有测试 `permission_check` 中 `|` 分割断言需要调整
- 信心评估: 5
- 步骤:
  - [ ] 移除 `user1|extra` 相关断言
  - [ ] 新增空列表拒绝断言
  - [ ] 新增通配符测试
  - [ ] 运行 `cargo clippy --all-targets -- -D warnings -D clippy::uninlined_format_args` 确认无警告
  - [ ] 运行 `cargo test` 确认所有测试通过

## 实现建议

- warn 日志格式建议：`"Channel {name} has no allow_from configured — blocking all access. Add allowed sender IDs or \"*\" to enable access."`
- 两个 channel 的 `check_permission()` 逻辑完全相同，后续可考虑提取为共享函数
