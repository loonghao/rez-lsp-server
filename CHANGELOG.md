# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.1.2](https://github.com/loonghao/rez-lsp-server/compare/v0.1.1...v0.1.2) - 2025-06-27

### Fixed

- resolve release-plz multiple triggers and VSCode extension publishing issues
- unify server binary naming across all platforms

### Other

- *(deps)* update dependency vite to v7

## [0.1.1](https://github.com/loonghao/rez-lsp-server/compare/v0.1.0...v0.1.1) - 2025-06-22

### Added

- optimize LSP server log output and remove binary tracking
- add LSP server commands and status bar monitoring

### Fixed

- resolve TypeScript compilation error with vite.config.ts
- replace npm ci with npm install in all workflows
- remove npm cache from ALL GitHub Actions workflows
- completely remove all npm caching to resolve persistent cache issues
- replace problematic setup-node cache with manual npm cache
- resolve GitHub Actions Node.js cache configuration issues
- resolve command registration timing and clippy warnings
- resolve VSCode extension command registration issues
- explicitly disable npm cache to prevent cache resolution errors
- remove npm cache configuration to resolve cache dependency error
- replace npm ci with npm install in GitHub Actions
- remove npm cache configuration from GitHub Actions
- remove pre-release flag from VSCode extension publishing
- improve status bar to match rust-analyzer style
- resolve VSCode extension publishing artifact download failures

### Other

- add extensive logging to diagnose VSCode extension issues
- update Node.js to v22 and fix npm caching
- replace webpack with Vite for faster builds
- optimize VSCode extension bundling with webpack
- *(deps)* update dependency node to v22
- *(deps)* update dependency typescript to v5
- *(deps)* update peter-evans/create-pull-request action to v7

### Security

- fix esbuild vulnerability by upgrading Vite to 6.3.5
## [Unreleased]

## [0.1.0] - 2024-12-22

### Added

#### Core LSP Features
- **Language Server Protocol Implementation**: Complete LSP server with support for all major IDEs
- **Intelligent Code Completion**: Auto-complete package names and versions in `requires` lists
- **Go to Definition**: Navigate to package definitions with Ctrl+Click
- **Find References**: Find all references to packages across workspace
- **Document Symbols**: Outline view showing package.py structure
- **Workspace Symbols**: Search for packages across entire workspace
- **Hover Information**: Rich tooltips with package details and documentation

#### Advanced Validation System
- **Python Syntax Validation**: Comprehensive checking for indentation, brackets, string literals
- **Rez-specific Validation**: Required fields, version formats, naming conventions
- **Real-time Diagnostics**: Live error reporting with severity levels
- **Smart Suggestions**: Automatic fix suggestions for common issues
- **Configurable Rules**: Customizable validation severity and rule sets

#### Performance & Optimization
- **Multi-level Caching**: Intelligent caching system with TTL support
- **Performance Monitoring**: Built-in metrics collection and analysis
- **Profiling System**: Detailed call tree analysis for performance optimization
- **Timer Utilities**: Performance measurement macros and utilities
- **Incremental Updates**: Efficient handling of file changes

#### Package Discovery & Resolution
- **Automatic Package Discovery**: Scans `REZ_PACKAGES_PATH` for available packages
- **Dependency Resolution**: Real-time dependency resolution and conflict detection
- **Version Constraint Parsing**: Support for complex version requirements
- **Conflict Detection**: Intelligent detection of package conflicts
- **Package Caching**: Efficient caching of package metadata

#### Developer Experience
- **Cross-platform Support**: Windows, Linux, and macOS compatibility
- **VSCode Integration**: Seamless VSCode extension with auto-installation
- **One-click Setup**: Complete development environment setup scripts
- **Unified Build System**: Cross-platform build scripts and Makefile
- **Comprehensive Testing**: 69 tests covering all major functionality

#### Build & Development Tools
- **Automated Build Scripts**: PowerShell and Bash scripts for all platforms
- **VSCode Extension**: Complete extension with syntax highlighting and LSP integration
- **Development Installation**: One-command setup for development environment
- **Dynamic VSCode Discovery**: Automatic detection of VSCode installations
- **Package Management**: VSIX packaging and installation support

### Technical Details

#### Architecture
- **Modular Design**: Clean separation of concerns with dedicated modules
- **Async/Await**: Full async support for non-blocking operations
- **Error Handling**: Comprehensive error handling with detailed messages
- **Configuration Management**: Flexible configuration system
- **Logging & Tracing**: Detailed logging for debugging and monitoring

#### Dependencies
- **Rust 1.75+**: Built with modern Rust for performance and safety
- **Tower-LSP**: LSP framework for robust protocol implementation
- **Tokio**: Async runtime for high-performance I/O
- **Serde**: Serialization for configuration and data exchange
- **Regex**: Pattern matching for syntax validation

#### Testing
- **Unit Tests**: 42 unit tests covering core functionality
- **Integration Tests**: 3 integration tests for end-to-end scenarios
- **Performance Tests**: 24 performance tests for optimization validation
- **Validation Tests**: Comprehensive validation system testing
- **Cross-platform Testing**: Tested on Windows, Linux, and macOS

### Documentation
- **Comprehensive README**: Detailed installation and usage instructions
- **API Documentation**: Complete Rust documentation with examples
- **Development Guide**: Setup instructions for contributors
- **Troubleshooting Guide**: Common issues and solutions
- **Architecture Overview**: System design and component interaction

### Known Limitations
- **Alpha Status**: This is an alpha release with potential breaking changes
- **Limited Python AST**: Basic Python parsing, full AST support planned
- **Package Repository**: Currently supports local packages only
- **IDE Support**: Primarily tested with VSCode, other IDEs may need configuration

### Breaking Changes
- None (initial release)

### Security
- **No Known Vulnerabilities**: Clean security audit
- **Safe Rust**: Memory-safe implementation
- **Input Validation**: Comprehensive input sanitization
- **Error Boundaries**: Proper error isolation
