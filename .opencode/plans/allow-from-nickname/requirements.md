# 需求

## 目标与背景

当前 `allow_from` 只支持填写 ID（飞书 `ou_xxx`、钉钉 staff_id），用户需要先查找自己的 ID 才能配置，门槛较高。本需求让 `allow_from` 同时支持昵称/姓名，降低配置门槛。

## 两个 channel 的差异

| Channel | 昵称来源 | 解析时机 | 方案 |
|---------|---------|---------|------|
| 钉钉 | `msg.sender_nick` 在每条消息中直接可用 | 消息时 | `check_permission` 同时匹配 `sender_id` 和 `sender_nickname` |
| 飞书 | 事件中无昵称字段，需调用 contact API | 启动时 | `new()` 中将非 `ou_` 前缀的 `allow_from` 条目通过搜索 API 解析为 `open_id`，缓存到内存 |

## 方案比较（强制）

### 方案 1: 各 channel 独立实现（推荐）

- 思路:
  - 钉钉：`check_permission` 新增 `sender_nick` 参数，同时匹配 ID 和昵称
  - 飞书：`new()` 中解析 `allow_from`，非 `ou_` 前缀条目调用 contact 搜索 API 转为 open_id，结果缓存到 `resolved_allow_from: Vec<String>` 字段
- 优点: 钉钉零 API 调用；飞书仅启动时调用一次；运行时 `check_permission` 保持纯内存匹配
- 缺点: 飞书搜索 API 需要 `search:user` 或 `contact:user.base:readonly` scope
- 工作量估算: M

### 方案 2: 统一在消息时调用 API 解析

- 思路: 每条消息到达时，调用 API 获取发送者昵称，再匹配 `allow_from`
- 优点: 实现简单
- 缺点: 每条消息增加一次 HTTP 请求，延迟不可接受
- 工作量估算: S（但性能差）

### 推荐

方案 1。

## 功能需求列表

### 核心功能

#### 钉钉

1. `check_permission` 签名从 `(&self, sender_id: &str)` 改为 `(&self, sender_id: &str, sender_nickname: &str)`
2. 匹配逻辑：`allow_from` 中任一条目等于 `sender_id` 或 `sender_nickname` 即通过
3. 调用处传入 `sender_nickname`

#### 飞书

1. `Feishu` 新增字段 `resolved_allow_from: Vec<String>`，存储解析后的 ID 列表
2. `new()` 中遍历 `config.allow_from`：
   - `"*"` → 原样保留
   - 以 `ou_` 开头 → 视为 open_id，原样保留
   - 其他 → 视为姓名，调用飞书搜索 API（`POST /open-apis/search/v1/user`）解析为 open_id
   - 解析失败 → warn 日志，原样保留（可能是未知格式的 ID）
3. `check_permission` 改为匹配 `resolved_allow_from` 而非 `config.allow_from`

### 扩展功能

- 无

## 非功能需求

- **性能**: 飞书 API 调用仅在启动时执行一次，运行时零开销
- **容错**: 搜索 API 失败或无结果时 warn 日志 + 保留原始条目，不阻塞启动
- **测试要求**: 钉钉昵称匹配测试；飞书解析逻辑的单元测试（mock API 或测试辅助方法）

## 边界与不做事项

- 不处理飞书姓名重名的情况（搜索返回多个结果时取第一个，warn 日志提示）
- 不支持运行时动态刷新（修改 `allow_from` 需重启）
- 不修改配置 schema

## 假设与约束

- **技术假设**: 飞书 bot 应用有 `search:user` 或 `contact:user.base:readonly` scope
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 飞书搜索 API 的具体 scope 要求和 SDK 调用方式需在实现时确认
