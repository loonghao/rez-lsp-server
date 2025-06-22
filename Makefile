# Makefile for Rez LSP Server
# Provides unified build commands across platforms

.PHONY: help build build-release extension install package test clean dev-setup

# Default target
help:
	@echo "Rez LSP Server Build System"
	@echo "============================"
	@echo ""
	@echo "Available targets:"
	@echo "  build         Build LSP server in debug mode"
	@echo "  build-release Build LSP server in release mode"
	@echo "  extension     Build VSCode extension"
	@echo "  install       Install VSCode extension for development"
	@echo "  package       Create distribution packages"
	@echo "  test          Run all tests"
	@echo "  clean         Clean build artifacts"
	@echo "  dev-setup     Complete development environment setup"
	@echo "  help          Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make build-release extension"
	@echo "  make dev-setup"
	@echo "  make test"

# Build LSP server in debug mode
build:
	@echo "🔧 Building LSP server (debug)..."
	cargo build
	@echo "✅ LSP server built successfully"

# Build LSP server in release mode
build-release:
	@echo "🔧 Building LSP server (release)..."
	cargo build --release
	@echo "✅ LSP server built successfully"

# Build VSCode extension
extension:
	@echo "🔧 Building VSCode extension..."
	cd vscode-extension && npm install
	cd vscode-extension && npm run compile
	@echo "✅ VSCode extension built successfully"

# Install VSCode extension for development
install: extension
	@echo "🔧 Installing VSCode extension for development..."
	cd vscode-extension && code --install-extension . --force
	@echo "✅ VSCode extension installed"

# Create distribution packages
package: build-release extension
	@echo "🔧 Creating distribution packages..."
	cd vscode-extension && npm run package
	@echo "✅ Distribution packages created"

# Run all tests
test:
	@echo "🔧 Running tests..."
	cargo test
	@echo "✅ All tests passed"

# Clean build artifacts
clean:
	@echo "🔧 Cleaning build artifacts..."
	cargo clean
	rm -rf vscode-extension/out
	rm -rf vscode-extension/node_modules
	rm -f vscode-extension/*.vsix
	@echo "✅ Build artifacts cleaned"

# Complete development environment setup
dev-setup: build extension install
	@echo "🎉 Development environment setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Set REZ_PACKAGES_PATH environment variable"
	@echo "  2. Open a Rez project in VSCode"
	@echo "  3. Test the extension with package.py files"

# Platform-specific targets
ifeq ($(OS),Windows_NT)
    # Windows-specific commands
    SHELL := powershell.exe
    .SHELLFLAGS := -NoProfile -Command
    
    dev-install:
		.\scripts\dev-install.ps1
    
    build-all:
		.\scripts\build-all.ps1 -Release -Extension
else
    # Unix-like systems
    dev-install:
		./scripts/dev-install.sh
    
    build-all:
		./scripts/build-all.sh --release --extension
endif
