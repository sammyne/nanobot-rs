# 需求文档

## 引言

本功能旨在自动化项目的发布流程，在 Git Tag 推送后自动生成变更历史并更新 CHANGELOG.md 文件。该功能将集成到现有的 release workflow 中，实现从 Git 提交历史中提取变更信息、生成结构化的变更日志、更新 CHANGELOG.md 文件并推送到仓库，同时在 GitHub Release 页面创建包含变更历史的发布说明。

该功能将遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范，从提交消息中提取变更类型（如 feat、fix、docs 等），并按照语义化版本进行分类展示。

## 需求

### 需求 1：提取提交历史并分类

**用户故事：** 作为一名项目维护者，我希望系统能够自动从 Git 提交历史中提取变更信息并按类型分类，以便生成结构化的变更日志。

#### 验收标准

1. WHEN Tag 推送触发 release workflow THEN 系统 SHALL 获取从上一个 Tag 到当前 Tag 之间的所有提交记录
2. WHEN 提取提交记录 THEN 系统 SHALL 解析提交消息中的 Conventional Commits 格式（如 `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `style:`, `perf:`）
3. IF 提交消息包含 breaking change 标记（`!` 或 `BREAKING CHANGE:`）THEN 系统 SHALL 将该提交标记为破坏性变更
4. WHEN 分类完成 THEN 系统 SHALL 将提交按以下类型分组：Features、Bug Fixes、Breaking Changes、Documentation、Performance Improvements、Refactoring、Others

### 需求 2：生成 CHANGELOG.md 内容

**用户故事：** 作为一名项目维护者，我希望系统能够生成符合标准的 CHANGELOG.md 内容，以便用户能够清晰了解每个版本的变更。

#### 验收标准

1. WHEN 生成 CHANGELOG 内容 THEN 系统 SHALL 遵循 [Keep a Changelog](https://keepachangelog.com/) 格式规范
2. WHEN 生成版本条目 THEN 系统 SHALL 包含版本号、发布日期和变更分类
3. IF CHANGELOG.md 文件已存在 THEN 系统 SHALL 在文件顶部插入新版本的变更内容
4. IF CHANGELOG.md 文件不存在 THEN 系统 SHALL 创建新文件并包含标准头部说明
5. WHEN 生成内容 THEN 系统 SHALL 为每个提交包含提交哈希（短格式）和提交消息摘要

### 需求 3：推送 CHANGELOG.md 到仓库

**用户故事：** 作为一名项目维护者，我希望系统能够自动将更新的 CHANGELOG.md 推送到仓库，以便变更历史能够与代码同步保存。

#### 验收标准

1. WHEN CHANGELOG.md 内容生成完成 THEN 系统 SHALL 配置 Git 用户信息（使用 `github-actions[bot]`）
2. WHEN 推送文件 THEN 系统 SHALL 创建一个提交，提交消息格式为 `chore: update CHANGELOG.md for v{version}`
3. WHEN 推送失败 THEN 系统 SHALL 实施重试机制（最多 3 次）
4. IF 推送成功 THEN 系统 SHALL 输出提交 SHA 和变更摘要

### 需求 4：创建 GitHub Release

**用户故事：** 作为一名项目维护者，我希望系统能够在 GitHub Release 页面自动创建发布说明，以便用户能够通过 GitHub 界面查看版本变更。

#### 验收标准

1. WHEN Tag 推送触发 workflow THEN 系统 SHALL 使用 GitHub API 创建 Release
2. WHEN 创建 Release THEN 系统 SHALL 将版本号作为标题（格式：`v{version}` 或 `{version}`）
3. WHEN 创建 Release THEN 系统 SHALL 将生成的变更历史作为发布说明内容
4. IF Release 已存在 THEN 系统 SHALL 更新现有 Release 的内容而不是创建新的
5. WHEN Release 创建成功 THEN 系统 SHALL 输出 Release URL

### 需求 5：集成到现有 workflow

**用户故事：** 作为一名项目维护者，我希望变更历史生成功能能够无缝集成到现有的 release workflow 中，以便保持发布流程的一致性。

#### 验收标准

1. WHEN release workflow 执行 THEN 变更历史生成 SHALL 在 Docker 镜像构建成功后执行
2. WHEN 变更历史生成失败 THEN 系统 SHALL 不影响 Docker 镜像的发布结果
3. WHEN workflow 执行 THEN 系统 SHALL 输出详细的执行日志，包括提取的提交数量、分类结果、生成的 CHANGELOG 内容预览
4. IF 这是首次发布（没有上一个 Tag）THEN 系统 SHALL 获取从仓库初始化到当前 Tag 的所有提交

### 需求 6：边界情况处理

**用户故事：** 作为一名项目维护者，我希望系统能够正确处理各种边界情况，以便发布流程的稳定性。

#### 验收标准

1. IF 两个 Tag 之间没有提交记录 THEN 系统 SHALL 生成包含 "No changes" 的变更日志
2. IF 提交消息不符合 Conventional Commits 格式 THEN 系统 SHALL 将该提交归类到 "Others" 分类
3. IF Git 历史获取失败 THEN 系统 SHALL 记录错误日志并跳过 CHANGELOG 更新步骤
4. IF CHANGELOG.md 文件格式异常 THEN 系统 SHALL 创建备份文件并重新生成标准格式
