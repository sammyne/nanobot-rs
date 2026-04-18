# Skill 元信息配置指南

本文档介绍如何为 nanobot 项目中的 skill 配置元信息，帮助开发者正确设置 skill 的依赖、描述和安装方式。

## 概述

Skill 元信息通过 YAML 前置元数据（frontmatter）定义在 `SKILL.md` 文件顶部，用于描述 skill 的依赖要求、人类可读描述、平台特定配置等信息。

## 基本结构

```yaml
---
description: "技能的简短描述"
always: false
requires:
  bins: []
  env: []
metadata:
  nanobot:
    emoji: null
    always: false
    requires:
      bins: []
      env: []
    install: []
---
```

## 字段说明

### 顶层字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `description` | 字符串 | `""` | 技能的人类可读描述 |
| `always` | 布尔值 | `false` | 是否始终加载到上下文 |
| `requires` | 依赖对象 | 默认值 | 技能运行时依赖 |
| `metadata` | 元数据对象 | `null` | 平台特定配置 |

### 依赖要求（requires）

定义技能运行时的依赖要求。

```yaml
requires:
  bins:          # 必需的 CLI 工具（需在 PATH 中可用）
    - git
    - rustc
  env:           # 必需的环境变量
    - OPENAI_API_KEY
```

### 平台特定元数据（metadata）

支持两个平台：`nanobot` 和 `openclaw`。平台特定配置会覆盖顶层同名配置。

```yaml
metadata:
  nanobot:       # nanobot 平台配置
    emoji: "🚀"
    always: false
    requires: {}
    install: []
  openclaw:      # openclaw 平台配置（与 nanobot 结构相同）
    emoji: null
    always: false
    requires: {}
    install: []
```

### nanobot/openclaw 平台配置

平台特定的元数据配置。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `emoji` | 字符串或 null | `null` | 技能的 emoji 图标 |
| `always` | 布尔值 | `false` | 是否始终加载到上下文（优先级高于顶层 `always`） |
| `requires` | 依赖对象或 null | `null` | 平台特定的依赖要求（优先级高于顶层 `requires`） |
| `install` | 安装方式列表 | `[]` | 可用的安装方式列表 |

### 安装方式（install）

定义技能的安装方式。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `id` | 字符串 | 必需 | 安装方式的唯一标识符 |
| `kind` | 字符串 | 必需 | 包管理器类型（如 `"brew"`, `"apt"`, `"npm"`） |
| `formula` | 字符串或 null | `null` | Homebrew formula 名称（仅 `brew` 类型） |
| `package` | 字符串或 null | `null` | apt 包名称（仅 `apt` 类型） |
| `bins` | 字符串列表 | `[]` | 安装后提供的可执行文件列表 |
| `label` | 字符串 | 必需 | 人类可读的安装选项标签 |

## 配置示例

### 最小配置

```yaml
---
description: "我的技能"
---
```

### 完整配置示例

```yaml
---
description: "Rust 项目分析工具"
always: false
requires:
  bins:
    - rustc
    - cargo
  env:
    - CARGO_HOME
metadata:
  nanobot:
    emoji: "🦀"
    always: false
    requires:
      bins:
        - rust-analyzer
    install:
      - id: brew
        kind: brew
        formula: rustup
        bins:
          - rustup
          - rustc
          - cargo
        label: "通过 Homebrew 安装"
      - id: apt
        kind: apt
        package: rustup
        bins:
          - rustup
        label: "通过 APT 安装"
---
```

## 配置优先级

当顶层和平台特定配置同时存在时，**平台特定配置优先级更高**：

1. 平台的 `always` 设置 > 顶层的 `always`
2. 平台的 `requires` 设置 > 顶层的 `requires`

## 字段继承规则

对于 `requires` 字段：
- 若平台的 `requires` 存在 → 使用平台特定配置
- 否则 → 使用顶层配置

## 完整 SKILL.md 示例

```markdown
---
description: "代码格式化工具"
requires:
  bins:
    - rustfmt
metadata:
  nanobot:
    emoji: "✨"
    always: false
    install:
      - id: rustup
        kind: brew
        formula: rustup
        bins:
          - rustfmt
        label: "通过 rustup 安装"
---
# Rustfmt

代码格式化工具...
```

## 注意事项

1. YAML 前置元数据必须以 `---` 开头和结尾
2. `id`、`kind` 和 `label` 在安装方式中是必需字段
3. `emoji` 为 `null` 时不会显示图标
4. 平台特定配置可以省略，此时使用顶层对应值
