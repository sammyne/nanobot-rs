# 需求

## 目标与背景

当前 `consolidate_internal` 解析 `save_memory` 工具调用的 payload 时，使用 `serde_json::Value` 手动提取字段，对缺失字段静默跳过。这会导致部分写入或整合"成功"但什么都没写。

对应上游 PR：HKUDS/nanobot#1810（fix(memory): validate save_memory payload before persisting）。

## 方案

定义 `SaveMemoryArgs` 结构体，使用 `parse_arguments::<SaveMemoryArgs>()` 反序列化。serde 自动验证必填字段，反序列化失败直接返回 `MemoryError::ToolParse`。

```rust
#[derive(Deserialize)]
struct SaveMemoryArgs {
    history_entry: String,
    memory_update: String,
}
```

替换现有的手动 `args.get("history_entry")` / `args.get("memory_update")` 逻辑。反序列化成功后再验证 `history_entry` 非空，然后一次性写入。

## 功能需求列表

### 核心功能

1. 新增 `SaveMemoryArgs` 结构体（`history_entry: String`, `memory_update: String`）
2. `consolidate_internal` 中用 `parse_arguments::<SaveMemoryArgs>()` 替代手动字段提取
3. 验证 `history_entry` trim 后非空
4. 仅在验证通过后才写入 HISTORY.md + MEMORY.md

## 边界与不做事项

- 不修改 `MemoryError` 枚举（复用 `ToolParse`）
- 不修改 `save_memory` 工具定义

## 待确认事项

- 无
