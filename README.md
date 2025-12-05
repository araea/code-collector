# 📦 Code Collector

[<img alt="github" src="https://img.shields.io/badge/github-araea/code__collector-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/araea/code-collector)
[<img alt="crates.io" src="https://img.shields.io/crates/v/code-collector.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/code-collector)

Rust 命令行工具，递归收集项目源代码并整合至 Markdown 文件，便于统一查阅和 AI 分析。

---

## 功能特点

- 多语言源代码自动识别与高亮（Rust、Python、JavaScript、C++、Java、Go 等）
- 递归遍历目录，内置智能忽略无关文件夹（`.git`、`node_modules`、`target` 等）
- **支持自定义忽略特定目录名或特定文件名**
- 支持跳过大文件、二进制文件、指定扩展名过滤
- 交互式与命令行两种使用方式
- 生成整合目录树和文件目录（TOC）
- 输出包含详细统计信息（行数、文件数、类型分布）
- 友好的彩色终端输出和进度提示

---

## 安装

确保已安装 Rust 和 Cargo：

```sh
cargo install --git https://github.com/araea/code-collector.git
```

或直接克隆源码编译：

```sh
git clone https://github.com/araea/code-collector.git
cd code-collector
cargo build --release
```

---

## 使用说明

### 交互模式

直接运行无需参数：

```sh
code-collector
```

程序将引导输入目标目录、输出文件名、忽略规则等配置。

### 命令行模式

简洁示例：

```sh
code-collector -Path ./my_project
```

完整参数示例：

```sh
code-collector -Path ./my_project -OutFile code.md -MaxBytes 1048576 -SkipExts "exe dll" -IgnoreDirs "tests docs" -IgnoreFiles "package-lock.json cargo.lock" -NoTree
```

#### 参数说明

| 参数            | 说明                                     | 默认值           |
| --------------- | ---------------------------------------- | ---------------- |
| `-Path`         | 目标目录路径（必填）                     | N/A              |
| `-OutFile`      | 输出 Markdown 文件名                     | `all-in-one.md`  |
| `-MaxBytes`     | 最大处理文件大小（字节）                 | `1048576` (1MB)  |
| `-SkipExts`     | 额外跳过的文件扩展名（空格分隔）         | 二进制扩展名列表 |
| `-IncludeExts`  | 白名单过滤，仅包含指定扩展名文件         | 全部文件         |
| `-IgnoreDirs`   | **额外忽略的目录名**（空格分隔）         | 内置忽略列表     |
| `-IgnoreFiles`  | **额外忽略的文件名**（空格分隔）         | 无               |
| `-NoTree`       | 不生成目录树                             | 生成             |
| `-NoToc`        | 不生成文件目录（TOC）                    | 生成             |
| `-h, --help`    | 显示帮助信息                             |                  |

---

## 输出示例

输出文件结构：

````markdown
# 📦 my_project - Code Collection

> 由 Code Collector 自动生成

| 属性 | 值 |
| ---- | -- |
| 生成时间 | 2024-06-05 18:00:00 |
| 源目录 | ./my_project |
| 文件数量 | 42 |

## 📂 目录结构

```
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── utils.rs
└── README.md
```

## 📑 文件目录

| # | 文件 | 类型 | 行数 | 大小 |
| - | ---- | ---- | ---- | ---- |
| 1 | src/main.rs | rust | 250 | 12.3 KB |
| 2 | README.md | markdown | 45 | 3.1 KB |

## 📄 文件内容

### src/main.rs

> 📏 250 行 | 💾 12.3 KB

```rust
// 文件内容...
```

...

## 📊 统计信息

### 文件类型分布

| 扩展名 | 文件数 |
| ------ | ------ |
| rs     | 15     |
| md     | 5      |
| toml   | 2      |

### 汇总

- 处理文件: 22
- 总代码行数: 15000
- 总大小: 1.2 MB
- 跳过（过大）: 3
- 跳过（二进制）: 10
- 跳过（编码问题）: 0
````

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
