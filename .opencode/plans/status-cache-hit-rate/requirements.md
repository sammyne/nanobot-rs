# 需求

## 目标与背景

将上游 PR #2645 的 /status 缓存命中率显示功能迁移到 Rust 版。当前 Rust 版 `/status` 命令只显示 `input_tokens` 和 `output_tokens`，不显示 prompt cache 命中率。上游已在 /status 中添加 "82% cached" 等缓存率信息，帮助用户了解 prompt 缓存效果。

**现状不足**：用户无法判断 Anthropic prompt cache 是否生效，无法评估缓存优化的实际效果。

## 方案比较（强制）

### 方案 1: 仅扩展 Usage 结构体（最小可行版）✅ 已选定

- 思路: 在 `Usage` 中添加 `cached_tokens: Option<u64>`，各 provider 提取该字段，/status 显示缓存率
- 优点: 改动最小，不改变现有架构
- 缺点: 无
- 工作量估算: S

### 方案 2: 通用 usage 字典（理想架构）

- 思路: 将 Usage 改为 HashMap<String, u64>，支持任意 provider 特有字段
- 优点: 未来扩展性好
- 缺点: 破坏类型安全，过度设计
- 工作量估算: M

### 推荐

方案 1。

## 功能需求列表

### 核心功能

- `Usage` 结构体新增 `cached_tokens: Option<u64>` 字段
- Anthropic provider：从 `cache_read_input_tokens` 提取 cached_tokens
- OpenAI provider：从 `prompt_tokens_details.cached_tokens` 提取 cached_tokens
- `/status` 命令：当 cached_tokens 存在时显示缓存命中率百分比

## 非功能需求

- 测试：新增 cached_tokens 提取和 /status 格式化的测试

## 边界与不做事项

- 不累积多轮 cached_tokens（只显示最近一次调用的缓存率，与上游一致）

## 待确认事项

无
