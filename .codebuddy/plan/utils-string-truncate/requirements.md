# 需求文档

## 引言
本需求旨在创建一个通用的 `utils` crate，其中包含一个 `strings` 子模块，提供安全的 UTF-8 字符串截取功能。该功能将解决当前代码中使用字节下标截取字符串时可能出现的 UTF-8 边界问题导致的 panic。通过封装这个通用功能，可以在整个项目中的多个 crate 之间复用，提高代码的可维护性和安全性。

## 需求

### 需求 1: 创建 utils crate 基础结构

**用户故事：** 作为一名开发者，我希望有一个独立的 `utils` crate 作为项目的工具库，以便组织和复用通用功能。

#### 验收标准

1. WHEN 项目需要添加工具函数 THEN 系统 SHALL 创建 `crates/utils` 目录结构
2. WHEN 创建 utils crate THEN 系统 SHALL 包含 `Cargo.toml` 文件，声明为库类型
3. WHEN 创建 utils crate THEN 系统 SHALL 在工作空间配置中添加成员
4. IF utils crate 被创建 THEN 系统 SHALL 包含 `src/strings.rs` 子模块文件
5. IF utils crate 被创建 THEN 系统 SHALL 在 `lib.rs` 中声明 `strings` 为公开模块

### 需求 2: 实现安全的 UTF-8 字符串截取函数

**用户故事：** 作为一名开发者，我希望有一个 `truncate` 方法能够安全地截取 UTF-8 字符串到指定字符长度，以便避免在处理中文等多字节字符时出现 panic。

#### 验收标准

1. WHEN 调用 `truncate` 函数 THEN 系统 SHALL 接受字符串引用和最大字符数作为参数
2. WHEN 输入字符串长度小于等于最大字符数 THEN 系统 SHALL 返回原始字符串的引用
3. WHEN 执行截取操作 THEN 系统 SHALL 确保截断位置在合法的 UTF-8 字符边界上
4. WHEN 使用 `char_indices()` 遍历 THEN 系统 SHALL 正确计算字符位置而非字节位置
5. WHEN 处理空字符串 THEN 系统 SHALL 正确返回空字符串

### 需求 3: 在 session crate 中应用 truncate 方法

**用户故事：** 作为一名开发者，我希望将 `session.rs` 中的手动截取逻辑替换为调用 `utils::strings::truncate` 方法，以便简化代码并提高可维护性。

#### 验收标准

1. WHEN 在 session crate 中截取工具结果 THEN 系统 SHALL 调用 `utils::strings::truncate` 方法
2. WHEN 替换截取逻辑 THEN 系统 SHALL 移除手动实现的字符边界遍历代码
3. IF session crate 使用 truncate THEN 系统 SHALL 在 `Cargo.toml` 中添加 `utils` 依赖
4. WHEN 截取逻辑被替换 THEN 系统 SHALL 保持原有的 `TOOL_RESULT_MAX_CHARS` 常量定义

### 需求 4: 替换其他 crate 中的不安全字符串截取

**用户故事：** 作为一名开发者，我希望在其他使用字节下标截取字符串的地方也应用安全的 `truncate` 方法，以便消除项目中所有的 UTF-8 边界问题风险。

#### 验收标准

1. WHEN 搜索代码库中所有使用字符串下标截取的位置 THEN 系统 SHALL 识别所有潜在的不安全截取
2. WHEN 发现使用 `&s[..n]` 形式的截取 THEN 系统 SHALL 评估是否需要替换为 `truncate` 方法
3. IF 截取可能跨越 UTF-8 字符边界 THEN 系统 SHALL 使用 `truncate` 方法替换
4. WHEN 替换截取逻辑时 THEN 系统 SHALL 确保相关 crate 添加 `utils` 依赖
5. IF 截取逻辑被替换 THEN 系统 SHALL 验证行为与原有实现一致

### 需求 5: 为 truncate 函数添加单元测试

**用户故事：** 作为一名开发者，我希望 `truncate` 函数有完整的测试覆盖，以便确保在各种边界情况下都能正确工作。

#### 验收标准

1. WHEN 编写测试 THEN 系统 SHALL 使用表驱动测试模式
2. WHEN 测试空字符串 THEN 系统 SHALL 验证返回空字符串
3. WHEN 测试短字符串 THEN 系统 SHALL 验证返回原始字符串
4. WHEN 测试长字符串 THEN 系统 SHALL 验证正确截取和添加标记
5. WHEN 测试多字节字符 THEN 系统 SHALL 包含中文字符等 UTF-8 字符测试用例
6. WHEN 测试边界情况 THEN 系统 SHALL 包含截断位置正好在字符边界的情况
7. WHEN 测试截断标记 THEN 系统 SHALL 验证截断标记正确添加

### 需求 6: 文档和代码质量

**用户故事：** 作为一名开发者，我希望 `truncate` 函数有清晰的文档说明，以便其他开发者理解其用途和使用方法。

#### 验收标准

1. WHEN 定义 truncate 函数 THEN 系统 SHALL 添加详细的文档注释
2. WHEN 编写文档 THEN 系统 SHALL 说明函数的安全性和 UTF-8 处理方式
3. WHEN 编写文档 THEN 系统 SHALL 包含使用示例
4. WHEN 代码通过 clippy 检查 THEN 系统 SHALL 不产生警告
5. WHEN 格式化代码 THEN 系统 SHALL 遵循项目的 Rust 代码风格规范
