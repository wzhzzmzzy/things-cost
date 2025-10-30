# 物品日均花费统计

简单统计一下当前物品的日均消费成本，例如，一台手机是 2020 年 1 月 1 日花费 3000 元购买，到 2022 年 1 月 1 日，其日均成本为 3000 / (366 + 365)

## 应用架构设计

该应用采用三层架构设计：

### 架构层次

- **数据层** (`src/database.rs`): 保存在遵从 *nix XDG 标准的目录下的 SQLite 数据库，向应用层暴露数据获取接口。应用层不需要关心数据是如何存储的。
- **应用层** (`src/services.rs`): 处理业务逻辑，从数据层获取数据、结合今日日期换算成具体的条目，计算日均成本。
- **视图层** (`src/cli.rs`): 处理用户交互，调用应用层的 Service，展示和增加、修改条目。

### 数据模型

- **物品模型** (`src/models.rs`): 定义物品的数据结构和业务逻辑方法

### 项目结构

```
src/
├── main.rs          # 程序入口点
├── lib.rs           # 模块声明
├── models.rs        # 数据模型定义
├── database.rs      # 数据层实现
├── services.rs      # 应用层服务
└── cli.rs           # CLI 视图层
```

## 技术栈

- **语言**: Rust 2024 Edition
- **数据库**: SQLite (通过 rusqlite)
- **CLI 框架**: clap
- **表格显示**: tabled
- **日期处理**: chrono
- **序列化**: serde
- **配置文件**: XDG 标准目录

## 应用数据设计

### 1. 物品模型

物品有如下字段：

- `id`: 唯一标识符 (可选，数据库自动生成)
- `name`: 物品名称
- `start_date`: 起始日期 (NaiveDate)
- `discard_date`: 弃用日期 (可选 NaiveDate)
- `price`: 价格 (f64)
- `currency`: 币种 (String，默认 CNY)

### 2. 业务逻辑

- **日均成本计算**: `price / (end_date - start_date).num_days()`
- **使用天数计算**: 从起始日期到弃用日期或今日的天数
- **数据持久化**: SQLite 数据库存储在 XDG 数据目录

## CLI 命令

### 添加物品
```bash
things-cost add "物品名" 起始日期 价格 [币种] [弃用日期]
```

### 列出物品
```bash
things-cost list
```

### 更新物品
```bash
things-cost update ID [--name] [--start-date] [--discard-date] [--price] [--currency]
```

### 删除物品
```bash
things-cost delete ID
```

## 测试

项目包含完整的单元测试，覆盖所有核心功能模块。运行测试：

```bash
cargo test
```

## 开发要求

- 开发前编写技术文档
- 开发时注意项目架构合理性和可测性，并且需要开发单元测试部分
- 使用 anyhow 处理项目中出现的各类异常
