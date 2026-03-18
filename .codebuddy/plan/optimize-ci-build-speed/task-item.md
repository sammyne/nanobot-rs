# 实施计划

- [ ] 1. 分析并重构现有workflow结构
   - 读取并分析当前release.yml的整体结构
   - 识别需要拆分的部分（checkout、login、build、push等）
   - 保留现有的版本一致性校验逻辑和参数输入接口
   - _需求：5.1、5.2、5.3_

- [ ] 2. 创建独立的amd64构建job
   - 在workflow中新增amd64-build job，使用ubuntu-latest runner
   - 从原workflow中提取并复用Dockerfile构建逻辑
   - 配置构建输出为临时标签（如：amd64-${{ inputs.version }}）
   - _需求：1.1、1.3_

- [ ] 3. 创建独立的arm64构建job
   - 在workflow中新增arm64-build job，使用ubuntu-latest-arm64或自托管runner
   - 从原workflow中提取并复用Dockerfile构建逻辑
   - 配置构建输出为临时标签（如：arm64-${{ inputs.version }}）
   - _需求：1.2、1.4、2.1、2.2、2.3_

- [ ] 4. 配置多架构job并行执行
   - 将amd64-build和arm64-build设置为并行执行（移除相互依赖）
   - 确保两个job都依赖于workflow的版本校验和参数校验阶段
   - 在job中添加对应的runner类型配置（runs-on字段）
   - _需求：1.1、1.2_

- [ ] 5. 为每个架构构建job添加缓存配置
   - 在amd64-build job中配置GitHub Actions Cache用于Docker层缓存
   - 在arm64-build job中配置GitHub Actions Cache用于Docker层缓存
   - 使用不同的cache key以避免跨架构缓存冲突
   - _需求：4.1、4.2、4.3_

- [ ] 6. 创建manifest创建和推送job
   - 新增manifest job，依赖于amd64-build和arm64-build的完成
   - 使用docker buildx create、docker buildx build --push命令创建多架构manifest
   - 推送最终的镜像（如：${{ inputs.version }}）到镜像仓库
   - 清理临时标签
   - _需求：3.1、3.2_

- [ ] 7. 添加构建结果输出
   - 在workflow结束阶段添加步骤，输出构建成功的摘要信息
   - 使用GitHub Actions的job outputs或environment variables传递镜像信息
   - 输出内容包括：镜像名称、版本标签、支持的架构列表
   - 添加错误处理，在失败时输出详细错误信息
   - _需求：3.3、6.1、6.2、6.3_

- [ ] 8. 测试并验证workflow
   - 通过手动触发workflow测试完整流程
   - 验证amd64和arm64镜像是否正确构建
   - 验证manifest是否正确创建并包含所有架构
   - 验证缓存是否正常工作
   - _需求：全部需求_
