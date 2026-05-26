# 需求

## 目标与背景

当前飞书和钉钉 channel 的 `check_permission()` 在 `allow_from` 为空时返回 `true`（允许所有人）。这意味着 channel 启用但未配置 `allow_from` 时，任何人都可以与 bot 交互，存在安全隐患。

上游 HKUDS/nanobot PR #1403 将默认行为改为 deny-by-default。同时上游 PR #1677 修复了 `|` 分割导致的 allowlist 绕过漏洞。本需求合并实现两个修复。

## 方案比较（强制）

### 方案 1: 各 channel 独立修改 check_permission（最小可行版）

- 思路: 分别修改飞书和钉钉的 `check_permission()`，空 `allow_from` 返回 `false` + warn 日志，移除 `|` 分割逻辑改为精确匹配
- 优点: 改动最小，不引入新抽象
- 缺点: 两个 channel 的权限逻辑仍然各自维护，存在重复
- 工作量估算: S

### 方案 2: 提取共享 check_permission 到 trait 默认实现（理想架构）

- 思路: 在 `Channel` trait 或新增 trait 中提供默认的 `check_permission()` 实现，各 channel 复用
- 优点: 消除重复，新增 channel 自动获得正确行为
- 缺点: 需要重构 trait 层级，`allow_from` 字段需要通过 trait 方法暴露
- 工作量估算: M

### 推荐

方案 1。当前仅 2 个 channel，提取 trait 过度设计。后续新增 channel 时再考虑抽象。

## 功能需求列表

### 核心功能

1. **deny-by-default**: 飞书和钉钉的 `check_permission()` 在 `allow_from` 为空时返回 `false`，并输出 warn 日志提示管理员配置
2. **移除 `|` 分割**: `check_permission()` 改为 `sender_id` 精确匹配 `allow_from` 列表，移除 `|` 分割逻辑
3. **通配符支持**: `allow_from` 包含 `"*"` 时允许所有人（提供显式的"允许所有人"配置方式，降低 deny-by-default 的使用门槛）
4. **更新测试**: 调整现有测试以匹配新行为，新增 deny-by-default 和通配符测试

### 扩展功能

- 无（昵称匹配作为后续独立需求）

## 非功能需求

- **兼容性**: 破坏性变更 — 现有未配置 `allow_from` 的用户升级后 bot 将拒绝所有消息。需在 warn 日志中明确提示解决方案（配置 `allow_from` 或添加 `"*"`）
- **测试要求**: 覆盖空列表拒绝、通配符允许、精确匹配、不匹配拒绝

## 边界与不做事项

- 不提取共享 trait（方案 2）
- 不实现昵称匹配（后续独立需求）
- 不修改配置 schema（`allow_from` 字段类型不变）

## 假设与约束

- **技术假设**: 飞书 sender_id 格式为 `ou_xxx`，钉钉 sender_id 为 staff_id 字符串，两者均不含 `|` 字符
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 无
