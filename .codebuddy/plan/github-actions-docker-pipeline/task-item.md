# 实施计划

- [x] 1. 创建 GitHub Actions 工作流文件基础结构
   - 创建 `.github/workflows/dockerize.yml` 文件
   - 配置工作流名称为 `dockerize`
   - _需求：1.1、2.1_

- [x] 2. 实现 Git Tag 触发机制
   - 配置 `on: push: tags` 触发器，监听所有 tag 推送
   - 实现 tag 名称提取逻辑（从 `github.ref` 中提取）
   - _需求：1.1、1.2_

- [x] 3. 实现手动触发机制
   - 配置 `on: workflow_dispatch` 输入参数表单
   - 实现 tag 参数处理逻辑，未填写时使用 `latest` 作为默认值
   - _需求：2.1、2.2_

- [x] 4. 实现镜像构建与标签生成逻辑
   - 使用 `git rev-parse --short HEAD` 获取 Git 提交短哈希值（7位）
   - 组合生成 `${tag}-${commit-id}` 格式的完整镜像标签
   - 添加 commit ID 获取失败时的错误处理
   - _需求：3.1、3.2、3.3_

- [x] 5. 配置阿里云镜像仓库认证
   - 用户名固定为 `黎康鲤`，无需配置为 Secret
   - 添加 Secret 配置预检查步骤，仅验证 `ACR_PASSWORD`
   - 实现 `docker login` 登录阿里云镜像仓库
   - 在推送完成后添加 `docker logout` 清理凭证
   - _需求：4.1、4.2、4.3_

- [x] 6. 配置构建优化与缓存
   - 设置 `DOCKER_BUILDKIT=1` 环境变量启用 BuildKit
   - 配置 GitHub Actions 缓存，缓存 cargo-chef 依赖层
   - 优化 Dockerfile 以支持层缓存复用
   - _需求：5.1、5.2、5.3_

- [x] 7. 实现镜像构建与推送
   - 使用 `docker build` 构建镜像，指定完整标签
   - 使用 `docker push` 推送镜像到阿里云镜像仓库
   - _需求：1.3、2.3_

- [x] 8. 添加构建结果通知与文档
   - 在构建成功时输出完整的镜像名称和标签信息
   - 在构建失败时明确标记失败步骤并保留详细日志
   - 更新项目 README，添加 GitHub Actions 使用说明和 Secrets 配置指南
   - _需求：6.1、6.2、6.3、4.2_
