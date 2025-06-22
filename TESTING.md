# Testing Guide for Rez LSP Server

This guide provides comprehensive instructions for testing the Rez LSP Server and its VSCode extension.

## Prerequisites

- Rust 1.75+ installed
- Node.js 16+ installed
- Visual Studio Code 1.74.0+
- Git

## Quick Test Setup

### 1. Build the LSP Server

```bash
# Clone the repository
git clone https://github.com/loonghao/rez-lsp-server.git
cd rez-lsp-server

# Build the server
cargo build --release

# Verify the build
./target/release/rez-lsp-server --help  # Linux/macOS
.\target\release\rez-lsp-server.exe --help  # Windows
```

### 2. Test Basic LSP Functionality

```bash
# Set up test environment
export REZ_PACKAGES_PATH="$(pwd)/test_packages"  # Linux/macOS
set REZ_PACKAGES_PATH=%CD%\test_packages  # Windows

# Test with the included test client
node test_lsp_client.js
```

Expected output should show:
- Server initialization
- Package discovery (3 packages found)
- Code completion with python, maya, houdini packages

### 3. Test VSCode Extension

#### Option A: Extension Development Host

1. **Prepare the Extension**:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   ```

2. **Open in VSCode**:
   ```bash
   code .
   ```

3. **Launch Extension Development Host**:
   - Press `F5` or use "Run Extension" from the debug panel
   - A new VSCode window will open with the extension loaded

4. **Test the Extension**:
   - Open the test_packages directory in the new window
   - Open `test_packages/python/3.9.0/package.py`
   - Try editing the `requires` list and look for code completion

#### Option B: Manual Configuration

1. **Configure VSCode Settings**:
   Add to your `settings.json`:
   ```json
   {
     "rezLsp.serverPath": "/full/path/to/rez-lsp-server/target/release/rez-lsp-server",
     "rezLsp.trace.server": "verbose",
     "rezLsp.enableDiagnostics": true
   }
   ```

2. **Install Extension Manually**:
   ```bash
   cd vscode-extension
   npm run compile
   code --install-extension .
   ```

## Detailed Testing Scenarios

### Scenario 1: Package Discovery

1. **Setup Custom Package Repository**:
   ```bash
   mkdir -p /tmp/test-rez-packages/my-package/1.0.0
   cat > /tmp/test-rez-packages/my-package/1.0.0/package.py << 'EOF'
   name = "my-package"
   version = "1.0.0"
   description = "Test package for LSP"
   authors = ["Test Author"]
   requires = ["python-3.7+"]
   tools = ["my-tool"]
   EOF
   ```

2. **Test Discovery**:
   ```bash
   export REZ_PACKAGES_PATH="/tmp/test-rez-packages"
   node test_lsp_client.js
   ```

3. **Verify**: Should show "my-package" in completion results

### Scenario 2: Code Completion

1. **Create Test File**:
   ```python
   # test_completion.py
   name = "test-completion"
   version = "1.0.0"
   requires = [
       "python-3.9",
       # Type here and test completion
   ]
   ```

2. **Test in VSCode**:
   - Open the file
   - Position cursor after the comma in requires list
   - Type `"` and look for completion suggestions
   - Should see available packages

### Scenario 3: Hover Information

1. **Test Hover**:
   - Open a package.py file
   - Hover over package names in the requires list
   - Should see package descriptions and version information

### Scenario 4: Error Handling

1. **Test Invalid Package Path**:
   ```bash
   export REZ_PACKAGES_PATH="/nonexistent/path"
   node test_lsp_client.js
   ```

2. **Test Invalid Package File**:
   Create a malformed package.py and test error reporting

## Performance Testing

### Large Repository Test

1. **Create Large Test Repository**:
   ```bash
   ./scripts/create-large-test-repo.sh  # Creates 1000+ test packages
   ```

2. **Measure Performance**:
   ```bash
   time node test_lsp_client.js
   ```

3. **Expected Results**:
   - Initial scan: < 5 seconds for 1000 packages
   - Completion response: < 100ms
   - Memory usage: < 100MB

## Troubleshooting Tests

### Common Issues

1. **Server Won't Start**:
   ```bash
   # Check binary exists and is executable
   ls -la target/release/rez-lsp-server
   
   # Test direct execution
   ./target/release/rez-lsp-server
   ```

2. **No Packages Found**:
   ```bash
   # Verify environment variable
   echo $REZ_PACKAGES_PATH
   
   # Check directory structure
   find $REZ_PACKAGES_PATH -name "package.py" | head -5
   ```

3. **VSCode Extension Issues**:
   - Check "Rez LSP" output channel
   - Verify extension is activated (check status bar)
   - Restart VSCode and try again

### Debug Mode

1. **Enable Debug Logging**:
   ```bash
   export REZ_LSP_DEBUG=true
   export RUST_LOG=debug
   ```

2. **Verbose LSP Tracing**:
   ```json
   {
     "rezLsp.trace.server": "verbose"
   }
   ```

## Automated Testing

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run specific test module
cargo test discovery

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration test suite
./scripts/run-integration-tests.sh

# Test specific scenarios
./scripts/test-completion.sh
./scripts/test-hover.sh
```

### CI/CD Testing

The project includes GitHub Actions for automated testing:
- Code formatting and linting
- Unit tests on multiple platforms
- Integration tests with sample packages
- Performance benchmarks

## Reporting Issues

When reporting issues, please include:

1. **Environment Information**:
   - Operating system and version
   - Rust version (`rustc --version`)
   - VSCode version
   - Extension version

2. **Reproduction Steps**:
   - Exact commands used
   - File contents
   - Environment variables

3. **Logs**:
   - LSP server output
   - VSCode extension logs
   - Error messages

4. **Expected vs Actual Behavior**:
   - What you expected to happen
   - What actually happened
   - Screenshots if applicable

## Contributing Test Cases

1. Fork the repository
2. Add test cases to the appropriate test suite
3. Update this testing guide if needed
4. Submit a pull request with test descriptions

For more information, see [CONTRIBUTING.md](CONTRIBUTING.md).
