# 需求

## 目标与背景

飞书 channel 收到用户消息后，bot 需要调用 LLM 和执行工具，响应延迟可达数秒到数十秒。在此期间用户没有任何反馈，无法判断 bot 是否收到消息。

上游 Python 版 nanobot（PR #1257）通过在 `_on_message()` 中立即给消息添加表情回应（默认 THUMBSUP）来解决这个问题，并将表情类型做成可配置项。Slack channel 已有同样的 `react_emoji` 配置。

nanobot-rs 的飞书 channel 当前没有任何消息签收机制。`feishu-sdk` v0.1.2 已提供 `im.v1.message_reaction.create` API 支持，`message_id` 在事件解析后可直接获取但当前未使用。

## 方案比较（强制）

### 方案 1: 飞书 channel 内部实现（最小可行版）

- 思路: 在 `FeishuConfig` 加 `react_emoji` 字段，在 `process_message()` 中解析出 `message_id` 后立即调用 reaction API，fire-and-forget
- 优点: 改动最小（2 个文件），与现有代码模式一致，不引入新抽象
- 缺点: 仅飞书 channel 有此能力，钉钉 channel 无法复用
- 工作量估算: S

### 方案 2: Channel trait 层面抽象 acknowledge 能力（理想架构）

- 思路: 在 `Channel` trait 中新增 `acknowledge(message_id)` 方法（默认空实现），各 channel 按需覆写；配置统一放在 `ChannelsConfig` 层级
- 优点: 架构统一，未来钉钉或其他 channel 加签收能力时有现成接口
- 缺点: 钉钉 Stream SDK 目前不支持 reaction API，抽象层暂时只有一个实现，属于过度设计；需要改 trait 定义影响所有 channel
- 工作量估算: M

### 推荐

推荐方案 1。当前只有飞书 channel 需要且能支持 reaction，钉钉 SDK 无此能力。按 AGENTS.md "Simplicity First" 原则，不为单一实现创建抽象。未来如果钉钉或新 channel 也需要签收能力，再提取公共接口不迟。

## 功能需求列表

### 核心功能

- `FeishuConfig` 新增 `react_emoji` 字段，类型 `String`，默认值 `"THUMBSUP"`，通过 `config.json` 的 `channels.feishu.reactEmoji` 配置
- 收到消息后，在 `process_message()` 中提取 `message_id`，调用飞书 reaction API 添加配置的表情
- reaction 调用采用 fire-and-forget 模式：spawn 独立 task，失败仅 `warn!` 日志，不阻塞消息处理流程
- `react_emoji` 为空字符串时跳过 reaction（允许用户禁用此功能）

### 扩展功能

- 无

## 非功能需求

- **性能**: reaction API 调用不得阻塞消息处理主流程，必须异步 fire-and-forget
- **安全**: 无额外安全要求，复用现有飞书客户端认证
- **兼容性**: `react_emoji` 默认值 `"THUMBSUP"` 保持与上游一致；字段缺省时 serde default 保证向后兼容
- **可维护性**: 遵循项目现有的 fire-and-forget 错误处理模式（参考图片下载失败的处理方式）
- **测试要求**: 为 reaction 逻辑添加单元测试

## 边界与不做事项

- 不为 Channel trait 添加 acknowledge 抽象
- 不为钉钉 channel 实现 reaction 功能
- 不实现 reaction 的删除或列表功能
- 不验证 `react_emoji` 值是否为合法的飞书表情类型（飞书 API 会返回错误，由 warn 日志体现）

## 假设与约束

- **技术假设**: `feishu-sdk` v0.1.2 的 `im_v1_reaction().create()` API 在当前认证模式下可用，不需要额外的权限申请
- **资源约束**: 无
- **环境约束**: 无

## 待确认事项

- 无
