# 发布指南

本文档描述了如何为 `things-cost` 项目创建和发布新版本。

## 发布流程

### 1. 准备工作

确保所有更改都已提交到仓库：

```bash
git status
git add .
git commit -m "Your commit message"
git push origin main
```

### 2. 更新版本号

编辑 `Cargo.toml` 文件，更新 `version` 字段：

```toml
[package]
name = "things-cost"
version = "0.1.1"  # 更新版本号
```

### 3. 创建发布标签

```bash
# 创建并推送标签
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1
```

### 4. 自动发布

GitHub Actions 会自动：
- 运行所有测试
- 为多个平台构建二进制文件
- 创建 GitHub Release
- 上传预编译的二进制文件
- 发布到 crates.io（如果配置了令牌）

## 手动构建

### 使用脚本构建

#### Linux/macOS
```bash
chmod +x scripts/build-release.sh
./scripts/build-release.sh
```

#### Windows
```cmd
scripts\build-release.bat
```

### 手动构建特定目标

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS ARM
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## 支持的平台

- **Linux**: x86_64, aarch64 (glibc 和 musl)
- **macOS**: x86_64, aarch64
- **Windows**: x86_64

## 发布到 crates.io

要发布到 crates.io，需要：

1. 在 [crates.io](https://crates.io) 注册账户
2. 获取 API 令牌
3. 在 GitHub Secrets 中设置 `CARGO_REGISTRY_TOKEN`

然后发布流程会自动处理。

## 验证发布

发布后，请验证：

1. GitHub Release 页面是否正确创建
2. 所有平台的二进制文件是否可用
3. 二进制文件是否可以正常运行
4. crates.io 页面是否正确更新（如果发布）

## 故障排除

### 构建失败

- 确保所有依赖项都是最新的
- 检查 Rust 工具链是否最新
- 验证跨平台编译工具是否安装

### 发布失败

- 检查 GitHub Secrets 配置
- 验证 crates.io 令牌权限
- 检查版本号是否已存在

## 版本管理

我们遵循 [语义化版本控制](https://semver.org/)：

- **主版本号**: 不兼容的 API 更改
- **次版本号**: 向后兼容的功能性新增
- **修订号**: 向后兼容的问题修正