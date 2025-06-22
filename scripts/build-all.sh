#!/bin/bash
# Build script for Rez LSP Server and VSCode extension

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
RELEASE=false
EXTENSION=false
INSTALL=false
PACKAGE=false
TEST=false
CLEAN=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE=true
            shift
            ;;
        --extension)
            EXTENSION=true
            shift
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --package)
            PACKAGE=true
            shift
            ;;
        --test)
            TEST=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            echo -e "${BLUE}Rez LSP Server Build Script${NC}"
            echo ""
            echo -e "${YELLOW}Usage: ./scripts/build-all.sh [options]${NC}"
            echo ""
            echo -e "${GREEN}Options:${NC}"
            echo "  --release    Build in release mode"
            echo "  --extension  Build VSCode extension"
            echo "  --install    Install VSCode extension for development"
            echo "  --package    Create distribution packages"
            echo "  --test       Run all tests"
            echo "  --clean      Clean build artifacts"
            echo "  --help       Show this help message"
            echo ""
            echo -e "${CYAN}Examples:${NC}"
            echo "  ./scripts/build-all.sh --release --extension"
            echo "  ./scripts/build-all.sh --install"
            echo "  ./scripts/build-all.sh --test"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

function write_step() {
    echo -e "${BLUE}🔧 $1${NC}"
}

function write_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

function write_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Clean build artifacts
if [ "$CLEAN" = true ]; then
    write_step "Cleaning build artifacts..."
    
    rm -rf target
    rm -rf vscode-extension/out
    rm -rf vscode-extension/node_modules
    
    write_success "Build artifacts cleaned"
fi

# Build LSP Server
write_step "Building Rez LSP Server..."

BUILD_ARGS="build"
if [ "$RELEASE" = true ]; then
    BUILD_ARGS="$BUILD_ARGS --release"
fi
if [ "$EXTENSION" = true ]; then
    export BUILD_VSCODE_EXTENSION=1
fi

if ! cargo $BUILD_ARGS; then
    write_error "Failed to build LSP server"
    exit 1
fi

write_success "LSP server built successfully"

# Build VSCode Extension
if [ "$EXTENSION" = true ] || [ "$INSTALL" = true ] || [ "$PACKAGE" = true ]; then
    write_step "Building VSCode extension..."
    
    cd vscode-extension
    
    # Install dependencies
    echo "Installing npm dependencies..."
    if ! npm install; then
        write_error "Failed to install npm dependencies"
        exit 1
    fi
    
    # Compile TypeScript
    echo "Compiling TypeScript..."
    if ! npm run compile; then
        write_error "Failed to compile TypeScript"
        exit 1
    fi
    
    write_success "VSCode extension built successfully"
    
    # Package extension
    if [ "$PACKAGE" = true ]; then
        write_step "Packaging VSCode extension..."
        
        # Install vsce if not available
        if ! command -v vsce &> /dev/null; then
            echo "Installing vsce..."
            npm install -g vsce
        fi
        
        if ! vsce package; then
            write_error "Failed to package VSCode extension"
            exit 1
        fi
        
        write_success "VSCode extension packaged successfully"
    fi
    
    # Install extension for development
    if [ "$INSTALL" = true ]; then
        write_step "Installing VSCode extension for development..."
        
        # Get the extension path
        EXTENSION_PATH=$(pwd)
        
        # Install extension
        if ! code --install-extension "$EXTENSION_PATH" --force; then
            write_error "Failed to install VSCode extension"
            exit 1
        fi
        
        write_success "VSCode extension installed for development"
    fi
    
    cd ..
fi

# Run tests
if [ "$TEST" = true ]; then
    write_step "Running tests..."
    
    # Run Rust tests
    echo "Running Rust tests..."
    if ! cargo test; then
        write_error "Rust tests failed"
        exit 1
    fi
    
    # Run VSCode extension tests if available
    if [ -d "vscode-extension/src/test" ]; then
        echo "Running VSCode extension tests..."
        cd vscode-extension
        if ! npm test; then
            write_error "VSCode extension tests failed"
            exit 1
        fi
        cd ..
    fi
    
    write_success "All tests passed"
fi

# Summary
echo ""
echo -e "${GREEN}🎉 Build completed successfully!${NC}"
echo ""

if [ "$RELEASE" = true ]; then
    BINARY_PATH="target/release/rez-lsp-server"
    if [ -f "$BINARY_PATH" ]; then
        echo -e "${CYAN}LSP Server binary: $BINARY_PATH${NC}"
    fi
fi

if [ "$EXTENSION" = true ] || [ "$PACKAGE" = true ]; then
    VSIX_FILE=$(find vscode-extension -name "*.vsix" | head -1)
    if [ -n "$VSIX_FILE" ]; then
        echo -e "${CYAN}VSCode extension package: $VSIX_FILE${NC}"
    fi
fi

echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  • Test the LSP server: cargo run"
echo "  • Install extension: ./scripts/build-all.sh --install"
echo "  • Run tests: ./scripts/build-all.sh --test"
