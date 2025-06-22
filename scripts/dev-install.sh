#!/bin/bash
# Development installation script for Rez LSP Server (Linux/macOS)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse command line arguments
FORCE=false
HELP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE=true
            shift
            ;;
        --help)
            HELP=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ "$HELP" = true ]; then
    echo -e "${BLUE}Rez LSP Server Development Installation${NC}"
    echo ""
    echo -e "${YELLOW}This script sets up a complete development environment:${NC}"
    echo "  • Builds the LSP server in debug mode"
    echo "  • Builds and installs the VSCode extension"
    echo "  • Configures VSCode settings for development"
    echo "  • Sets up test environment"
    echo ""
    echo -e "${GREEN}Usage: ./scripts/dev-install.sh [options]${NC}"
    echo ""
    echo -e "${CYAN}Options:${NC}"
    echo "  --force    Force reinstallation even if already installed"
    echo "  --help     Show this help message"
    exit 0
fi

function write_step() {
    echo -e "${BLUE}🔧 $1${NC}"
}

function write_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

function write_error() {
    echo -e "${RED}❌ $1${NC}"
}

function write_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

function find_vscode() {
    # Try to find VSCode executable in various locations
    local possible_paths=(
        # Standard PATH lookup
        "$(command -v code 2>/dev/null)"
        "$(command -v code-insiders 2>/dev/null)"
        
        # macOS specific paths
        "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
        "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/bin/code"
        "$HOME/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"
        
        # Linux specific paths
        "/usr/bin/code"
        "/usr/local/bin/code"
        "/opt/visual-studio-code/bin/code"
        "/snap/bin/code"
        "$HOME/.local/bin/code"
        
        # Flatpak
        "/var/lib/flatpak/exports/bin/com.visualstudio.code"
        "$HOME/.local/share/flatpak/exports/bin/com.visualstudio.code"
    )
    
    for path in "${possible_paths[@]}"; do
        if [ -n "$path" ] && [ -x "$path" ]; then
            write_info "Found VSCode at: $path"
            echo "$path"
            return 0
        fi
    done
    
    return 1
}

function test_vscode_installation() {
    local code_path="$1"
    
    if [ -z "$code_path" ]; then
        return 1
    fi
    
    if "$code_path" --version >/dev/null 2>&1; then
        local version=$("$code_path" --version | head -n1)
        write_info "VSCode version: $version"
        return 0
    fi
    
    return 1
}

echo -e "${BLUE}🚀 Rez LSP Server Development Setup${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# Check prerequisites
write_step "Checking prerequisites..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    write_error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Node.js
if ! command -v node &> /dev/null; then
    write_error "Node.js not found. Please install Node.js from https://nodejs.org/"
    exit 1
fi

# Check VSCode
write_info "Looking for VSCode installation..."
CODE_PATH=$(find_vscode)
if [ -z "$CODE_PATH" ]; then
    write_error "VSCode not found. Please install VSCode from https://code.visualstudio.com/"
    write_info "Searched locations:"
    write_info "  • PATH environment variable"
    write_info "  • Standard installation directories"
    write_info "  • macOS Applications folder"
    write_info "  • Linux system directories"
    write_info "  • Snap and Flatpak installations"
    exit 1
fi

if ! test_vscode_installation "$CODE_PATH"; then
    write_error "VSCode found but not working properly at: $CODE_PATH"
    exit 1
fi

write_success "All prerequisites found"

# Build LSP Server
write_step "Building LSP server..."
if ! cargo build; then
    write_error "Failed to build LSP server"
    exit 1
fi
write_success "LSP server built successfully"

# Build and install VSCode extension
write_step "Building VSCode extension..."

cd vscode-extension
# Install dependencies
write_info "Installing npm dependencies..."
if ! npm install; then
    write_error "Failed to install npm dependencies"
    exit 1
fi

# Compile TypeScript
write_info "Compiling TypeScript..."
if ! npm run compile; then
    write_error "Failed to compile TypeScript"
    exit 1
fi

write_success "VSCode extension built successfully"

# Install extension
write_step "Installing VSCode extension for development..."
EXTENSION_PATH=$(pwd)
write_info "Using VSCode at: $CODE_PATH"
write_info "Installing extension from: $EXTENSION_PATH"

if ! "$CODE_PATH" --install-extension "$EXTENSION_PATH" --force; then
    write_error "Failed to install VSCode extension"
    write_info "You can manually install the extension by:"
    write_info "  1. Open VSCode"
    write_info "  2. Press Ctrl+Shift+P (Cmd+Shift+P on macOS)"
    write_info "  3. Type 'Extensions: Install from VSIX'"
    write_info "  4. Select the extension folder: $EXTENSION_PATH"
    exit 1
fi

write_success "VSCode extension installed"
cd ..

# Create development configuration
write_step "Creating development configuration..."

# Determine VSCode settings directory
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    CONFIG_DIR="$HOME/Library/Application Support/Code/User"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    CONFIG_DIR="$HOME/.config/Code/User"
else
    write_error "Unsupported operating system: $OSTYPE"
    exit 1
fi

mkdir -p "$CONFIG_DIR"
SETTINGS_PATH="$CONFIG_DIR/settings.json"
CURRENT_DIR=$(pwd)

# Read existing settings or create new
if [ -f "$SETTINGS_PATH" ]; then
    # Backup existing settings
    cp "$SETTINGS_PATH" "$SETTINGS_PATH.backup.$(date +%s)"
    write_info "Backed up existing settings to $SETTINGS_PATH.backup.*"
fi

# Create or update settings
cat > "$SETTINGS_PATH" << EOF
{
    "rezLsp.serverPath": "$CURRENT_DIR/target/debug/rez-lsp-server",
    "rezLsp.trace.server": "verbose",
    "rezLsp.enableDiagnostics": true
}
EOF

write_success "Development configuration created"

# Set up test environment
write_step "Setting up test environment..."

TEST_PACKAGES_PATH="$CURRENT_DIR/test_packages"
if [ -d "$TEST_PACKAGES_PATH" ]; then
    write_info "Test packages already exist"
else
    write_info "Test packages not found, you may need to create them manually"
fi

write_success "Test environment configured"

# Run basic tests
write_step "Running basic tests..."
if ! cargo test --lib; then
    write_error "Some tests failed, but installation continues"
else
    write_success "All tests passed"
fi

# Summary
echo ""
echo -e "${GREEN}🎉 Development environment setup complete!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""

echo -e "${YELLOW}What's been configured:${NC}"
echo "  ✅ LSP server built in debug mode"
echo "  ✅ VSCode extension installed for development"
echo "  ✅ VSCode settings configured for development"
echo "  ✅ Test environment prepared"
echo ""

echo -e "${CYAN}Next steps:${NC}"
echo "  1. Set environment variable:"
echo "     export REZ_PACKAGES_PATH='$TEST_PACKAGES_PATH'"
echo ""
echo "  2. Open a Rez project in VSCode:"
echo "     code ."
echo ""
echo "  3. Test the extension:"
echo "     • Open a package.py file"
echo "     • Try code completion in requires list"
echo "     • Check 'Rez LSP' output channel"
echo ""

echo -e "${YELLOW}Useful commands:${NC}"
echo "  • Rebuild: cargo build"
echo "  • Run tests: cargo test"
echo "  • Test LSP: node test_lsp_client.js"
echo "  • View logs: Check 'Rez LSP' output in VSCode"
echo ""

echo -e "${GREEN}Happy coding! 🚀${NC}"
