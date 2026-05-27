# 需求

## 目标与背景

当前 `extract_absolute_paths()` 只匹配 `/xxx`（POSIX）和 `C:\xxx`（Windows）开头的路径，不检测 `~` 开头的路径。当 `restrict_to_workspace = true` 时，LLM 可以通过 `~` 路径绕过工作空间限制：

```bash
cat ~/.nanobot/config.json   # 读取含 API key 的配置文件 — 不会被拦截
cat ~/../../etc/passwd       # 路径遍历 — 不会被拦截
```

Shell 会自动将 `~` 展开为用户主目录，实际访问工作空间外的路径。

对应上游 PR：
- HKUDS/nanobot#1827（fix(exec): enforce workspace guard for home-expanded paths）
- HKUDS/nanobot#1845（fix: detect tilde paths in restrictToWorkspace shell guard）

两个 PR 修复同一个问题，合并实现。

## 方案

在 `extract_absolute_paths()` 中新增 `~` 路径提取，在 `validate_paths_in_workspace()` 中对 `~` 路径执行 `expanduser` 展开后再做边界检查。

## 功能需求列表

### 核心功能

1. 新增 `extract_tilde_paths(cmd)` 函数，匹配 `~` 或 `~/xxx` 模式的路径
2. `extract_absolute_paths()` 中合并 tilde 路径提取结果
3. `validate_paths_in_workspace()` 中对 `~` 开头的路径先 `expanduser` 展开为绝对路径，再做 `starts_with(workspace)` 检查

## 非功能需求

- **测试要求**：`extract_tilde_paths` 单元测试；`validate_paths_in_workspace` 集成测试覆盖 `~` 路径拦截

## 边界与不做事项

- 不修改 deny/allow 模式逻辑
- 不修改路径遍历检测逻辑（`../` 检测已独立处理）

## 待确认事项

- 无
