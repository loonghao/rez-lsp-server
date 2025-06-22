# Rez LSP Server

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![LSP](https://img.shields.io/badge/LSP-3.17-green.svg)](https://microsoft.github.io/language-server-protocol/)
[![CI](https://github.com/loonghao/rez-lsp-server/workflows/CI/badge.svg)](https://github.com/loonghao/rez-lsp-server/actions)
[![Release](https://github.com/loonghao/rez-lsp-server/workflows/Release/badge.svg)](https://github.com/loonghao/rez-lsp-server/actions)
[![VSCode Marketplace](https://img.shields.io/visual-studio-marketplace/v/loonghao.rez-lsp)](https://marketplace.visualstudio.com/items?itemName=loonghao.rez-lsp)
[![Development Status](https://img.shields.io/badge/status-alpha-red.svg)](https://github.com/loonghao/rez-lsp-server)

[中文文档](README_zh.md) | English

> ⚠️ **Development Status**: This project is in active development and is considered **alpha** software. APIs, configuration formats, and functionality may change without notice. Use in production environments is not recommended at this time.

A Language Server Protocol (LSP) implementation for [Rez package manager](https://github.com/AcademySoftwareFoundation/rez), providing intelligent code completion, dependency resolution, and syntax validation for `package.py` files across all major IDEs.

## ✨ Features

### 🎯 Core LSP Features
- 🔍 **Smart Package Completion**: Intelligent package name and version completion
- 🔗 **Dependency Resolution**: Real-time dependency resolution and conflict detection
- 📝 **Syntax Validation**: Advanced Python and Rez-specific validation
- 🎯 **Go to Definition**: Navigate to package definitions with Ctrl+Click
- 🔍 **Find References**: Find all references to packages across your workspace
- 📋 **Document Symbols**: Outline view of package.py structure
- 🌍 **Workspace Symbols**: Search for packages across your entire workspace
- 💡 **Hover Information**: Rich tooltips with package details

### 🔧 Advanced Features
- ⚡ **Performance Monitoring**: Built-in metrics collection and profiling
- 🗄️ **Multi-level Caching**: Intelligent caching with TTL support
- 🔄 **Incremental Updates**: Efficient handling of file changes
- 🛡️ **Smart Suggestions**: Automatic fix suggestions for common issues
- 🌐 **Cross-IDE Support**: Works with VSCode, PyCharm, Vim, Neovim, and more
- 🛠️ **High Performance**: Built with Rust for speed and reliability

### 🎮 LSP Server Commands (VSCode)
- 🔄 **Restart Server**: Restart the LSP server without reloading VSCode
- 🛑 **Stop Server**: Stop the LSP server
- 🔄 **Reload Workspace**: Reload workspace configuration and package cache
- 📋 **Open Logs**: View detailed LSP server logs and diagnostics
- 🔨 **Rebuild Dependencies**: Rebuild build dependencies and refresh cache
- ⚙️ **Toggle Check on Save**: Enable/disable diagnostics when saving files
- 📊 **Server Status**: Interactive status bar with quick access to all commands

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ 
- Rez package manager installed and configured

### Installation

#### Option 1: One-Click Development Setup (Recommended)

For a complete development environment setup:

```bash
# Clone the repository
git clone https://github.com/loonghao/rez-lsp-server.git
cd rez-lsp-server

# Windows
.\scripts\dev-install.ps1

# Linux/macOS
./scripts/dev-install.sh
```

This will:
- Build the LSP server
- Build and install the VSCode extension
- Configure VSCode settings for development
- Set up the test environment
- Run basic tests

#### Option 2: Manual Installation

```bash
# Build the project
cargo build --release

# The binary will be available at target/release/rez-lsp-server
```

#### Build System

We provide a unified build system that can build both the LSP server and VSCode extension:

```bash
# Windows
.\scripts\build-all.ps1 -Help                    # Show all options
.\scripts\build-all.ps1 -Release -Extension      # Build everything
.\scripts\build-all.ps1 -Install                 # Install VSCode extension
.\scripts\build-all.ps1 -Test                    # Run all tests

# Linux/macOS
./scripts/build-all.sh --help                    # Show all options
./scripts/build-all.sh --release --extension     # Build everything
./scripts/build-all.sh --install                 # Install VSCode extension
./scripts/build-all.sh --test                    # Run all tests

# Using Make (cross-platform)
make help                                         # Show all targets
make build-release extension                      # Build everything
make install                                      # Install VSCode extension
make test                                         # Run all tests
make dev-setup                                    # Complete dev setup
```

### IDE Setup

#### VSCode

##### Option 1: Install from Marketplace (Coming Soon)
1. Install the Rez LSP extension from the VSCode marketplace
2. The extension will automatically detect and use the LSP server

##### Option 2: Development/Testing Setup

For testing the latest development version:

1. **Build the LSP Server**:
   ```bash
   cargo build --release
   # The binary will be at target/release/rez-lsp-server
   ```

2. **Install VSCode Extension Dependencies**:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   ```

3. **Configure VSCode Settings**:
   Add to your VSCode `settings.json`:
   ```json
   {
     "rezLsp.serverPath": "/path/to/rez-lsp-server/target/release/rez-lsp-server",
     "rezLsp.trace.server": "verbose",
     "rezLsp.enableDiagnostics": true,
     "rezLsp.showStatusBarItem": true,
     "rezLsp.checkOnSave": true,
     "rezLsp.packagePaths": []
   }
   ```

4. **Test the Extension**:
   - Open the `vscode-extension` folder in VSCode
   - Press `F5` to launch a new Extension Development Host window
   - Open a Rez project with `package.py` files
   - Test code completion by typing in a `requires` list

5. **Environment Setup**:
   Set the `REZ_PACKAGES_PATH` environment variable:
   ```bash
   # Windows
   set REZ_PACKAGES_PATH=C:\path\to\your\rez\packages

   # Linux/macOS
   export REZ_PACKAGES_PATH=/path/to/your/rez/packages
   ```

6. **Verify Installation**:
   - Open a `package.py` file
   - Check the "Rez LSP" output channel for server logs
   - Try code completion in the `requires` field

#### Neovim

```lua
-- Add to your Neovim configuration
require'lspconfig'.rez_lsp.setup{
  cmd = { "/path/to/rez-lsp-server" },
  filetypes = { "python" },
  root_dir = function(fname)
    return require'lspconfig.util'.find_git_ancestor(fname) or vim.fn.getcwd()
  end,
}
```

## 🏗️ Architecture

The LSP server is built with a modular architecture:

- **LSP Protocol Layer**: Handles communication with IDEs
- **Rez Parser**: Parses package.py files and Rez syntax
- **Package Discovery**: Scans local package repositories
- **Dependency Resolver**: Resolves package dependencies and conflicts
- **Completion Engine**: Provides intelligent code completion

## 🛠️ Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run
```

The server communicates via stdin/stdout using the LSP protocol.

## 🔧 Troubleshooting

### VSCode Extension Issues

1. **Server Not Starting**:
   - Check the "Rez LSP" output channel for error messages
   - Verify the `rezLsp.serverPath` setting points to the correct executable
   - Ensure the LSP server binary has execute permissions
   - Use `Rez LSP: Restart Server` command to restart the server
   - Check the status bar for server status indicators

2. **No Code Completion**:
   - Verify `REZ_PACKAGES_PATH` environment variable is set
   - Check that package.py files exist in the configured paths
   - Enable verbose logging with `"rezLsp.trace.server": "verbose"`

3. **Performance Issues**:
   - Large package repositories may take time to scan initially
   - Check the output channel for scan completion messages
   - Consider reducing the number of package paths

### Common Configuration Issues

1. **Environment Variables**:
   ```bash
   # Verify Rez environment
   echo $REZ_PACKAGES_PATH  # Linux/macOS
   echo %REZ_PACKAGES_PATH%  # Windows
   ```

2. **Package Discovery**:
   - Ensure package directories follow Rez structure: `package_name/version/package.py`
   - Check file permissions on package directories
   - Verify package.py files contain valid Python syntax

3. **LSP Server Logs**:
   - Enable debug logging: `export REZ_LSP_DEBUG=true`
   - Check server output for detailed error messages
   - Use `Rez LSP: Open Logs` command to view logs
   - Click the status bar item for quick access to server commands

## 📝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Rez](https://github.com/AcademySoftwareFoundation/rez) - The amazing package manager this LSP server supports
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - The LSP framework used in this project
- [Academy Software Foundation](https://www.aswf.io/) - For maintaining Rez

## 🔗 Links

- [Rez Documentation](https://rez.readthedocs.io/)
- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)
- [Issue Tracker](https://github.com/loonghao/rez-lsp-server/issues)
