# Release 问题排查指南

## 为什么 GitHub Actions 执行完成后没有生成新的 Release？

### 常见原因

#### 1. 标签格式不正确
- **问题**: 工作流只对 `v*` 标签触发，但标签格式不正确
- **检查**: 确保标签格式为 `vX.Y.Z`（例如 `v0.1.0`）
- **正确示例**: `git tag -a v0.1.0 -m "Release v0.1.0"`

#### 2. 工作流配置问题
- **问题**: 工作流文件语法错误或配置不正确
- **检查**:
  - 确保 `.github/workflows/release.yml` 文件存在且语法正确
  - 检查 YAML 缩进和格式
  - 验证 `on.push.tags` 配置

#### 3. 权限问题
- **问题**: GitHub Actions 没有创建 release 的权限
- **检查**:
  - 确保仓库设置中启用了 Actions
  - 检查是否有足够的权限创建 release
  - 验证 `GITHUB_TOKEN` 权限

#### 4. 工作流执行失败
- **问题**: 工作流中的某个作业失败
- **检查**:
  - 查看 GitHub Actions 运行日志
  - 检查测试是否通过
  - 检查构建是否成功

#### 5. 跨平台编译问题
- **问题**: 跨平台编译需要额外的工具链
- **解决方案**:
  - 使用 SQLite 捆绑版本（已配置）
  - 在 GitHub Actions 中安装交叉编译工具

### 调试步骤

#### 1. 检查标签
```bash
# 查看当前标签
git tag -l

# 创建测试标签
git tag -a v0.1.0-test -m "Test release"
git push origin v0.1.0-test
```

#### 2. 检查工作流文件
```bash
# 验证 YAML 语法
./scripts/validate-workflows.sh

# 测试本地构建
./scripts/test-release.sh
```

#### 3. 检查 GitHub Actions 日志
1. 访问仓库的 Actions 页面
2. 找到对应的 release 工作流运行
3. 检查每个作业的执行状态
4. 查看详细的错误日志

#### 4. 简化测试
创建一个简单的测试工作流：

```yaml
# .github/workflows/test-release.yml
name: Test Release

on:
  push:
    tags:
      - 'test-*'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ github.ref }}
          name: Test Release
          body: This is a test release
          draft: true
          prerelease: true
```

### 解决方案

#### 1. 使用简化的工作流
我们已经创建了 `release-simple.yml`，它更可靠：
- 每个平台独立创建 release
- 避免复杂的 artifact 下载/上传
- 更清晰的错误处理

#### 2. 确保 SQLite 配置正确
在 `Cargo.toml` 中：
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

#### 3. 验证工作流触发
```bash
# 创建测试标签
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

#### 4. 检查 GitHub 设置
1. 仓库 Settings → Actions → General
2. 确保 "Allow all actions and reusable workflows" 启用
3. 检查 "Workflow permissions" 设置为 "Read and write permissions"

### 成功发布的标准流程

1. **准备发布**
   ```bash
   # 更新版本号
   vim Cargo.toml  # version = "0.1.0"

   # 提交更改
   git add .
   git commit -m "Release v0.1.0"
   git push origin main
   ```

2. **创建标签**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

3. **监控发布**
   - 访问 GitHub Actions 页面
   - 查看 release 工作流执行状态
   - 检查是否创建了新的 release

4. **验证发布**
   - 访问 Releases 页面
   - 确认所有平台的二进制文件都已上传
   - 测试下载的二进制文件

### 故障排除工具

- `scripts/validate-workflows.sh` - 验证工作流配置
- `scripts/test-release.sh` - 测试本地构建
- `scripts/build-release.sh` - 手动构建脚本

如果问题仍然存在，请检查 GitHub Actions 的详细日志并查看具体的错误信息。