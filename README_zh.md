# Rez LSP 服务器

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![LSP](https://img.shields.io/badge/LSP-3.17-green.svg)](https://microsoft.github.io/language-server-protocol/)
[![Development Status](https://img.shields.io/badge/status-alpha-red.svg)](https://github.com/loonghao/rez-lsp-server)

中文文档 | [English](README.md)

> ⚠️ **开发状态**: 本项目正在积极开发中，目前处于 **alpha** 阶段。API、配置格式和功能可能会在没有通知的情况下发生变化。目前不建议在生产环境中使用。

为 [Rez 包管理器](https://github.com/AcademySoftwareFoundation/rez) 实现的语言服务器协议 (LSP)，为所有主流 IDE 中的 `package.py` 文件提供智能代码补全、依赖解析和语法验证功能。

## ✨ 特性

- 🔍 **智能包补全**: 智能的包名和版本补全
- 🔗 **依赖解析**: 实时依赖解析和冲突检测  
- 📝 **语法验证**: package.py 文件的语法高亮和验证
- 🌐 **跨 IDE 支持**: 支持 VSCode、PyCharm、Vim、Neovim 等
- ⚡ **快速发现**: 通过 `REZ_PACKAGES_PATH` 高效的本地包发现
- 🛠️ **高性能**: 使用 Rust 构建，速度快且可靠

## 🚀 快速开始

### 前置要求

- Rust 1.75+ 
- 已安装并配置的 Rez 包管理器

### 安装

```bash
# 克隆仓库
git clone https://github.com/loonghao/rez-lsp-server.git
cd rez-lsp-server

# 构建项目
cargo build --release

# 二进制文件将位于 target/release/rez-lsp-server
```

### IDE 设置

#### VSCode

1. 安装 Rez LSP 扩展（即将推出）
2. 扩展将自动检测并使用 LSP 服务器

#### Neovim

```lua
-- 添加到你的 Neovim 配置中
require'lspconfig'.rez_lsp.setup{
  cmd = { "/path/to/rez-lsp-server" },
  filetypes = { "python" },
  root_dir = function(fname)
    return require'lspconfig.util'.find_git_ancestor(fname) or vim.fn.getcwd()
  end,
}
```

## 🏗️ 架构

LSP 服务器采用模块化架构构建：

- **LSP 协议层**: 处理与 IDE 的通信
- **Rez 解析器**: 解析 package.py 文件和 Rez 语法
- **包发现**: 扫描本地包仓库
- **依赖解析器**: 解析包依赖和冲突
- **补全引擎**: 提供智能代码补全

## 🛠️ 开发

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
cargo run
```

服务器通过 stdin/stdout 使用 LSP 协议进行通信。

## 📝 贡献

我们欢迎贡献！请查看我们的[贡献指南](CONTRIBUTING.md)了解详情。

### 开发设置

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 如适用，添加测试
5. 运行 `cargo fmt` 和 `cargo clippy`
6. 提交 pull request

## 📄 许可证

本项目采用 Apache License 2.0 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Rez](https://github.com/AcademySoftwareFoundation/rez) - 这个 LSP 服务器支持的出色包管理器
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - 本项目使用的 LSP 框架
- [Academy Software Foundation](https://www.aswf.io/) - 维护 Rez 项目

## 🔗 链接

- [Rez 文档](https://rez.readthedocs.io/)
- [语言服务器协议规范](https://microsoft.github.io/language-server-protocol/)
- [问题跟踪器](https://github.com/loonghao/rez-lsp-server/issues)
