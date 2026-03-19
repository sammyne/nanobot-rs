# GitHub Action 调研报告

## 调研目标

寻找现成的 GitHub Action 来实现以下功能：
1. 从 Git 提交历史生成 CHANGELOG.md
2. 创建 GitHub Release 并填充发布说明
3. 支持 Conventional Commits 格式
4. 自动推送到仓库

## 调研结果

### 1. **release-drafter/release-drafter** ⭐⭐⭐⭐⭐

**GitHub**: https://github.com/release-drafter/release-drafter  
**Stars**: 3.2k+

**功能特点**：
- ✅ 自动从 PR 和提交中提取变更
- ✅ 支持标签分类（feat、fix、bug等）
- ✅ 自动生成 Release Notes
- ✅ 支持模板自定义
- ✅ 可以作为 draft 保存，也可以直接发布
- ✅ 高度可配置

**优点**：
- 非常成熟，维护活跃
- 配置简单，开箱即用
- 支持自定义分类和模板
- 可以与现有 workflow 无缝集成

**缺点**：
- 主要基于 PR，对直接提交到 main 的支持较弱
- 不直接生成 CHANGELOG.md 文件（需要额外步骤）

**示例配置**：
```yaml
- name: Draft Release
  uses: release-drafter/release-drafter@v6
  with:
    config-name: release-drafter.yml
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

### 2. **softprops/action-gh-release** ⭐⭐⭐⭐⭐

**GitHub**: https://github.com/softprops/action-gh-release  
**Stars**: 3.5k+

**功能特点**：
- ✅ 创建 GitHub Release
- ✅ 支持上传文件（二进制、压缩包等）
- ✅ 支持自动生成 Release Notes
- ✅ 支持草稿模式
- ✅ 支持预发布标记

**优点**：
- 功能全面，支持文件上传
- 可以自动生成 Release Notes（基于 GitHub 的自动生成功能）
- 维护活跃，社区支持好

**缺点**：
- 不生成 CHANGELOG.md 文件
- Release Notes 的格式化能力有限

**示例配置**：
```yaml
- name: Create Release
  uses: softprops/action-gh-release@v2
  with:
    body: ${{ steps.changelog.outputs.changelog }}
    draft: false
    prerelease: false
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

### 3. **orhun/git-cliff-action** ⭐⭐⭐⭐⭐

**GitHub**: https://github.com/orhun/git-cliff-action  
**Stars**: 9k+ (git-cliff 本身)

**功能特点**：
- ✅ 基于 git-cliff 工具
- ✅ 高度可定制的 CHANGELOG 生成
- ✅ 支持 Conventional Commits
- ✅ 支持自定义模板
- ✅ 可以生成文件或输出到变量
- ✅ Rust 编写，性能优秀

**优点**：
- **完美支持 Conventional Commits**
- **可以生成 CHANGELOG.md 文件**
- **可以输出到变量用于 Release Notes**
- 配置灵活，模板功能强大
- 维护活跃

**缺点**：
- 需要配置文件（cliff.toml）
- 学习曲线稍陡

**示例配置**：
```yaml
- name: Generate CHANGELOG
  uses: orhun/git-cliff-action@v4
  id: changelog
  with:
    config: cliff.toml
    args: --tag ${{ github.ref_name }}

- name: Create Release
  uses: softprops/action-gh-release@v2
  with:
    body: ${{ steps.changelog.outputs.content }}
```

---

### 4. **metcalfc/changelog-generator** ⭐⭐⭐

**GitHub**: https://github.com/metcalfc/changelog-generator  
**Stars**: 100+

**功能特点**：
- ✅ 从 Git 历史生成 CHANGELOG
- ✅ 支持标签范围
- ✅ 输出到文件或变量

**优点**：
- 轻量级
- 配置简单

**缺点**：
- 功能相对简单
- 不支持 Conventional Commits 分类
- 维护不够活跃

---

### 5. **github-changelog-generator/github-changelog-generator** ⭐⭐⭐

**GitHub**: https://github.com/github-changelog-generator/github-changelog-generator  
**Stars**: 7k+

**功能特点**：
- ✅ 基于 Ruby 的经典工具
- ✅ 从 PR 和 Issues 生成 CHANGELOG
- ✅ 支持标签过滤

**优点**：
- 功能成熟
- 社区使用广泛

**缺点**：
- 需要 Ruby 环境
- 配置复杂
- 主要基于 PR 和 Issues，不是基于提交

---

## 推荐方案

### 方案一：git-cliff + softprops/action-gh-release（推荐）⭐⭐⭐⭐⭐

**理由**：
1. **git-cliff** 完美支持 Conventional Commits，可以生成符合 Keep a Changelog 格式的 CHANGELOG.md
2. **softprops/action-gh-release** 用于创建 GitHub Release
3. 两者结合可以满足所有需求

**实施步骤**：
1. 创建 `cliff.toml` 配置文件
2. 在 release workflow 中添加 git-cliff-action
3. 使用 softprops/action-gh-release 创建 Release
4. 将生成的 CHANGELOG.md 推送到仓库

**优点**：
- ✅ 完全满足需求
- ✅ 高度可定制
- ✅ 维护活跃
- ✅ 性能优秀

**缺点**：
- 需要维护配置文件

---

### 方案二：release-drafter + 自定义脚本

**理由**：
1. release-drafter 可以自动生成 Release Notes
2. 使用自定义脚本将 Release Notes 追加到 CHANGELOG.md

**实施步骤**：
1. 配置 release-drafter.yml
2. 在 workflow 中使用 release-drafter
3. 添加自定义步骤将内容追加到 CHANGELOG.md
4. 推送 CHANGELOG.md

**优点**：
- ✅ release-drafter 成熟稳定
- ✅ 配置相对简单

**缺点**：
- 需要编写自定义脚本
- 对直接提交的支持较弱

---

## 最终推荐

**推荐使用方案一：git-cliff + softprops/action-gh-release**

这个方案可以完美满足所有需求：
1. ✅ 从 Git 提交历史生成 CHANGELOG.md
2. ✅ 支持 Conventional Commits 格式
3. ✅ 创建 GitHub Release 并填充发布说明
4. ✅ 自动推送到仓库
5. ✅ 高度可定制
6. ✅ 维护活跃，社区支持好

## 下一步行动

如果采用推荐方案，需要：
1. 创建 `cliff.toml` 配置文件
2. 修改 `.github/workflows/release.yml`，添加 changelog 生成和 Release 创建步骤
3. 测试 workflow 是否正常工作
