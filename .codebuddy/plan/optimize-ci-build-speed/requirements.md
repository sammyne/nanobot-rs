# 需求文档

## 引言
当前的GitHub Actions release workflow使用QEMU模拟arm64架构进行Docker镜像构建，这种方式存在以下问题：
- QEMU模拟导致构建速度较慢，影响CI/CD效率
- 增加了不必要的计算资源消耗
- 延长了发布流程的时间

本需求旨在通过使用GitHub自托管或托管的arm64 runner，直接在真实的arm64硬件上构建arm64镜像，从而显著提升构建速度。

## 需求

### 需求 1：支持多架构并行构建

**用户故事：** 作为一名【DevOps工程师】，我希望【能够在不同的runner上并行构建不同架构的Docker镜像】，以便【提升整体构建效率】

#### 验收标准

1. WHEN 【触发release workflow】 THEN 【系统】 SHALL 【为amd64和arm64架构分别创建独立的构建job】
2. WHEN 【多个构建job同时执行】 THEN 【系统】 SHALL 【确保每个job在对应架构的runner上运行】
3. WHEN 【amd64构建完成】 THEN 【系统】 SHALL 【标记amd64镜像为可推送】
4. WHEN 【arm64构建完成】 THEN 【系统】 SHALL 【标记arm64镜像为可推送】

### 需求 2：使用GitHub提供的arm64 runner

**用户故事：** 作为一名【项目维护者】，我希望【利用GitHub提供的arm64 runner资源】，以便【无需自建基础设施即可实现高效构建】

#### 验收标准

1. IF 【GitHub提供托管的arm64 runner】 THEN 【系统】 SHALL 【配置arm64构建job使用ubuntu-latest-arm64】
2. IF 【GitHub未提供托管arm64 runner】 THEN 【系统】 SHALL 【支持配置自托管arm64 runner】
3. WHEN 【arm64 runner不可用】 THEN 【系统】 SHALL 【提供清晰的错误提示信息】

### 需求 3：合并多架构镜像

**用户故事：** 作为一名【用户】，我希望【获得一个包含所有架构的统一Docker镜像】，以便【在不同平台上都可以直接拉取使用】

#### 验收标准

1. WHEN 【所有架构的镜像构建完成】 THEN 【系统】 SHALL 【创建一个多架构的manifest镜像】
2. WHEN 【manifest创建成功】 THEN 【系统】 SHALL 【推送manifest到镜像仓库】
3. WHEN 【manifest推送完成】 THEN 【系统】 SHALL 【输出完整的镜像拉取命令】

### 需求 4：构建缓存优化

**用户故事：** 作为一名【开发者】，我希望【利用构建缓存加速重复构建】，以便【进一步提升构建效率】

#### 验收标准

1. WHEN 【构建镜像时】 THEN 【系统】 SHALL 【使用GitHub Actions Cache缓存构建层】
2. WHEN 【缓存命中时】 THEN 【系统】 SHALL 【复用缓存的构建层】
3. WHEN 【缓存未命中时】 THEN 【系统】 SHALL 【执行完整构建并更新缓存】

### 需求 5：向后兼容性

**用户故事：** 作为一名【现有用户】，我希望【新的构建流程保持与现有workflow的兼容性】，以便【不影响现有的使用方式】

#### 验收标准

1. WHEN 【用户推送Git Tag】 THEN 【系统】 SHALL 【保持现有的版本一致性校验逻辑】
2. WHEN 【用户手动触发workflow】 THEN 【系统】 SHALL 【保持现有的参数输入逻辑】
3. WHEN 【其他workflow调用release workflow】 THEN 【系统】 SHALL 【保持现有的输入参数接口】

### 需求 6：构建结果输出

**用户故事：** 作为一名【开发者】，我希望【能够清晰地查看构建结果和镜像信息】，以便【确认构建成功并获取镜像信息】

#### 验收标准

1. WHEN 【所有构建完成】 THEN 【系统】 SHALL 【输出构建成功的摘要信息】
2. WHEN 【输出构建信息时】 THEN 【系统】 SHALL 【包含镜像名称、标签、支持架构等信息】
3. WHEN 【构建失败时】 THEN 【系统】 SHALL 【输出详细的错误信息】
