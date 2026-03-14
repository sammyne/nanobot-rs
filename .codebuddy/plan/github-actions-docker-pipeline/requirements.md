# 需求文档

## 引言

为 nanobot-rs 项目设计一个 GitHub Actions CI/CD 流水线，实现基于 Git Tag 推送和手动触发两种方式的 Docker 镜像构建与推送功能。镜像将推送到阿里云容器镜像服务（杭州区域），标签格式为 `${tag}-${git-commit-id}`，确保每次构建都有唯一且可追溯的镜像版本标识。

## 需求

### 需求 1：Git Tag 触发构建

**用户故事：** 作为一名开发者，我希望在推送 Git Tag 时自动触发镜像构建和推送，以便实现版本的自动化发布流程。

#### 验收标准

1. WHEN 推送 Git Tag 到仓库 THEN 系统 SHALL 自动触发 Docker 镜像构建工作流
2. WHEN 工作流被 Tag 触发 THEN 系统 SHALL 使用 Tag 名称作为镜像标签的前缀
3. WHEN 镜像构建完成 THEN 系统 SHALL 将镜像推送到 `registry.cn-hangzhou.aliyuncs.com/sammyne/nanobot:${tag}-${commit-id}`

### 需求 2：手动触发构建

**用户故事：** 作为一名开发者，我希望能够手动触发镜像构建，以便在需要时进行测试或临时发布。

#### 验收标准

1. WHEN 用户在 GitHub Actions 页面手动触发工作流 THEN 系统 SHALL 显示输入参数表单，包含 tag 输入框
2. IF 用户未填写 tag 参数 THEN 系统 SHALL 使用 `latest` 作为默认标签前缀
3. WHEN 手动触发构建完成 THEN 系统 SHALL 将镜像推送到指定的阿里云镜像仓库

### 需求 3：镜像构建与标签生成

**用户故事：** 作为一名开发者，我希望镜像标签包含 Git 提交 ID，以便快速定位镜像对应的源代码版本。

#### 验收标准

1. WHEN 镜像构建开始 THEN 系统 SHALL 获取当前 Git 提交的短哈希值（前 7 位）
2. WHEN 生成镜像标签 THEN 系统 SHALL 按照格式 `${tag}-${commit-id}` 组合标签
3. IF 构建过程中获取 commit ID 失败 THEN 系统 SHALL 中止构建并报告错误

### 需求 4：阿里云镜像仓库认证

**用户故事：** 作为一名开发者，我希望使用 GitHub Secrets 安全地存储镜像仓库凭证，以便流水线能够安全地推送镜像。

#### 验收标准

1. WHEN 工作流执行推送操作 THEN 系统 SHALL 从 GitHub Secrets 读取阿里云镜像仓库的用户名和密码
2. IF 必要的 Secrets 未配置 THEN 系统 SHALL 在工作流启动时进行预检查并给出明确的配置指引
3. WHEN 推送完成 THEN 系统 SHALL 使用 `docker logout` 清理本地凭证，避免残留

### 需求 5：构建优化与缓存

**用户故事：** 作为一名开发者，我希望构建过程能够利用 Docker 层缓存，以便加快构建速度并节省 CI 资源。

#### 验收标准

1. WHEN 构建 Docker 镜像 THEN 系统 SHALL 启用 BuildKit 以支持高级缓存特性
2. IF 使用 GitHub Actions 缓存 THEN 系统 SHALL 缓存 cargo-chef 产生的依赖层
3. WHEN 缓存命中 THEN 系统 SHALL 跳过已缓存的构建步骤，减少构建时间

### 需求 6：构建结果通知

**用户故事：** 作为一名开发者，我希望在构建完成后看到清晰的状态信息，以便了解构建结果和镜像详情。

#### 验收标准

1. WHEN 构建成功 THEN 系统 SHALL 在工作流日志中输出完整的镜像名称和标签
2. WHEN 构建失败 THEN 系统 SHALL 明确标记失败步骤并保留详细日志便于排查
3. WHEN 工作流完成 THEN 系统 SHALL 设置构建状态（成功/失败）供 GitHub Actions 页面展示
