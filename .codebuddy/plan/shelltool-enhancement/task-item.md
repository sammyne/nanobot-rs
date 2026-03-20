# 实施计划

- [ ] 1. 添加依赖和定义配置结构体
   - 在 Cargo.toml 中添加 `regex` 依赖
   - 定义 `ShellToolOptions` 结构体，包含 `deny_patterns`、`allow_patterns`、`restrict_to_workspace`、`path_append`、`timeout`、`workspace` 字段
   - 为 `ShellToolOptions` 实现 `Default` trait，提供默认拒绝模式列表
   - _需求：1.1、1.2、2.1、3.1、4.1_

- [ ] 2. 实现正则表达式拒绝模式功能
   - 在 `ShellTool` 中添加 `deny_patterns` 字段（`Vec<Regex>` 类型）
   - 实现默认拒绝模式列表（包含 `rm -rf`、`dd if=`、`mkfs`、fork bomb 等危险命令的正则表达式）
   - 实现 `check_deny_patterns` 方法，使用正则匹配检查命令是否匹配任一拒绝模式
   - _需求：1.1、1.2、1.3、1.4_

- [ ] 3. 实现允许模式白名单功能
   - 在 `ShellTool` 中添加 `allow_patterns` 字段（`Option<Vec<Regex>>` 类型）
   - 实现 `check_allow_patterns` 方法，检查命令是否匹配白名单模式
   - IF 配置了允许模式且命令不匹配任何模式，返回错误
   - _需求：2.1、2.2、2.3、2.4_

- [ ] 4. 实现工作空间路径限制功能
   - 在 `ShellTool` 中添加 `restrict_to_workspace` 和 `workspace_path` 字段
   - 实现 `detect_path_traversal` 方法，检测 `../`、`..\\` 等路径遍历尝试
   - 实现 `validate_paths_in_workspace` 方法，验证命令中的绝对路径是否在工作空间内
   - _需求：3.1、3.2、3.3、3.4、3.5_

- [ ] 5. 实现 PATH 环境变量扩展功能
   - 在 `ShellTool` 中添加 `path_append` 字段
   - 实现 `build_env_with_path` 方法，将 `path_append` 追加到 PATH 环境变量
   - 使用系统正确的路径分隔符（Unix 为 `:`，Windows 为 `;`）
   - _需求：4.1、4.2、4.3_

- [ ] 6. 实现绝对路径提取辅助方法
   - 实现 `extract_windows_absolute_paths` 方法，提取 Windows 风格绝对路径（如 `C:\...`）
   - 实现 `extract_posix_absolute_paths` 方法，提取 POSIX 风格绝对路径（如 `/...`）
   - 正确处理带引号和空格的路径
   - _需求：5.1、5.2、5.3_

- [ ] 7. 实现统一的安全守卫方法
   - 创建 `security_guard` 方法，整合所有安全检查
   - 按顺序执行：拒绝模式检查 → 允许模式检查 → 工作空间限制检查
   - 任一检查失败时返回相应的错误信息
   - _需求：6.1、6.2、6.3、6.4_

- [ ] 8. 更新 ShellTool 的 execute 方法
   - 在命令执行前调用 `security_guard` 方法
   - 应用 PATH 环境变量扩展
   - 确保所有安全检查通过后才执行命令
   - _需求：6.1、4.2_

- [ ] 9. 添加单元测试
   - 为拒绝模式功能编写测试用例（测试默认模式和自定义模式）
   - 为允许模式功能编写测试用例（测试白名单匹配和拒绝）
   - 为工作空间限制功能编写测试用例（测试路径遍历检测和路径验证）
   - 为绝对路径提取方法编写测试用例（测试 Windows 和 POSIX 路径提取）
   - _需求：1.1-1.4、2.1-2.4、3.1-3.5、5.1-5.3_

- [ ] 10. 更新文档和示例代码
   - 更新 README 文档，说明新增的配置选项和安全特性
   - 添加使用示例代码，展示如何配置和使用各项安全功能
   - _需求：1-6_
