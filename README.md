# Things Cost - 物品日均花费统计工具

> 此项目完全由 DeepSeek 编写

一个简单的命令行工具，用于统计物品的日均消费成本。

## 功能特性

- 📊 跟踪物品购买日期、价格和使用时长
- 📈 计算每日平均成本
- 💾 数据持久化存储（SQLite）
- 🖥️ 美观的表格显示
- 🔧 完整的 CRUD 操作

## 安装

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/your-username/things-cost.git
cd things-cost

# 构建并安装
cargo install --path .
```

### 使用预编译二进制文件

从 [Releases](https://github.com/your-username/things-cost/releases) 页面下载对应平台的二进制文件。

#### Linux

```bash
# 下载并安装
curl -L https://github.com/wzhzzmzzy/things-cost/releases/latest/download/things-cost-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv things-cost /usr/local/bin/
```

#### macOS

```bash
# 下载并安装
curl -L https://github.com/wzhzznzzy/things-cost/releases/latest/download/things-cost-x86_64-apple-darwin.tar.gz | tar xz
sudo mv things-cost /usr/local/bin/
```

#### Windows

1. 从 Releases 页面下载 `things-cost-x86_64-pc-windows-msvc.zip`
2. 解压并将 `things-cost.exe` 添加到 PATH 环境变量

### 使用 cargo-binstall

```bash
cargo binstall things-cost
```

## 使用方法

### 添加物品

```bash
things-cost add "iPhone 12" 2020-10-01 3000 CNY
```

### 列出所有物品

```bash
things-cost list
```

### 更新物品信息

```bash
things-cost update 1 --name "iPhone 12 Pro" --price 3500
```

### 删除物品

```bash
things-cost delete 1
```

## 示例

```bash
# 添加几个物品
things-cost add "MacBook Pro" 2021-01-01 12000 CNY
things-cost add "iPhone" 2020-10-01 3000 CNY 2023-12-31

# 查看统计
things-cost list
```

输出示例：

```
+----+-------------+------------+------------+--------------+----------+-------------+
| ID | 名称        | 起始日期   | 弃用日期   | 价格         | 使用天数 | 日均成本    |
+----+-------------+------------+------------+--------------+----------+-------------+
| 2  | iPhone      | 2020-10-01 | 2023-12-31 | 3000.00 CNY  | 1156     | 2.5952 CNY  |
+----+-------------+------------+------------+--------------+----------+-------------+
| 1  | MacBook Pro | 2021-01-01 | -          | 12000.00 CNY | 1234     | 9.7245 CNY  |
+----+-------------+------------+------------+--------------+----------+-------------+
```

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 运行

```bash
cargo run -- --help
```

## 许可证

本项目采用双重许可证：

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

您可以选择其中任意一个许可证。

