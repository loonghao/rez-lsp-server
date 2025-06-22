# Rez LSP Extension for VSCode

This extension provides Language Server Protocol (LSP) support for [Rez package manager](https://github.com/AcademySoftwareFoundation/rez) in Visual Studio Code.

## Features

- **Intelligent Code Completion**: Auto-complete package names and versions in `requires` lists
- **Syntax Highlighting**: Enhanced syntax highlighting for Rez package.py files
- **Hover Information**: Get detailed information about packages on hover
- **Diagnostics**: Real-time error checking and validation
- **Go to Definition**: Navigate to package definitions (coming soon)

## Requirements

- Visual Studio Code 1.74.0 or higher
- Rez LSP Server binary (included or separately installed)
- Rez package manager environment

## Installation

### From VSCode Marketplace (Coming Soon)

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Rez LSP"
4. Click Install

### Development Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/loonghao/rez-lsp-server.git
   cd rez-lsp-server/vscode-extension
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Compile the extension:
   ```bash
   npm run compile
   ```

4. Open in VSCode and press F5 to launch Extension Development Host

## Configuration

The extension can be configured through VSCode settings:

### Settings

- `rezLsp.serverPath`: Path to the Rez LSP server executable
  - Default: `"rez-lsp-server"`
  - Example: `"/usr/local/bin/rez-lsp-server"`

- `rezLsp.trace.server`: Trace communication with the server
  - Options: `"off"`, `"messages"`, `"verbose"`
  - Default: `"off"`

- `rezLsp.packagePaths`: Additional package paths to search
  - Type: Array of strings
  - Default: `[]`
  - Example: `["/custom/packages", "/shared/packages"]`

- `rezLsp.enableDiagnostics`: Enable diagnostic messages
  - Type: Boolean
  - Default: `true`

### Example Configuration

Add to your VSCode `settings.json`:

```json
{
  "rezLsp.serverPath": "/path/to/rez-lsp-server",
  "rezLsp.trace.server": "messages",
  "rezLsp.packagePaths": [
    "/studio/packages",
    "/shared/packages"
  ],
  "rezLsp.enableDiagnostics": true
}
```

## Environment Setup

Set the `REZ_PACKAGES_PATH` environment variable before starting VSCode:

### Windows
```cmd
set REZ_PACKAGES_PATH=C:\rez\packages;C:\shared\packages
code .
```

### Linux/macOS
```bash
export REZ_PACKAGES_PATH="/opt/rez/packages:/shared/packages"
code .
```

## Usage

1. Open a Rez project containing `package.py` files
2. The extension will automatically activate when it detects package.py files
3. Start typing in a `requires` list to see code completion
4. Hover over package names to see detailed information

## Commands

- `Rez: Restart LSP Server` - Restart the language server
- `Rez: Show Output Channel` - Show the LSP server output logs

## Troubleshooting

### Server Not Starting

1. Check the "Rez LSP" output channel for errors
2. Verify the `rezLsp.serverPath` setting
3. Ensure the LSP server binary exists and is executable

### No Code Completion

1. Verify `REZ_PACKAGES_PATH` is set correctly
2. Check that package.py files exist in the configured paths
3. Enable verbose logging to see detailed information

### Performance Issues

1. Large package repositories may take time to scan initially
2. Check the output channel for scan completion messages
3. Consider reducing the number of package paths

## Development

### Building from Source

1. Install Rust and Cargo
2. Build the LSP server:
   ```bash
   cargo build --release
   ```
3. The binary will be at `target/release/rez-lsp-server`

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test with the Extension Development Host
5. Submit a pull request

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Links

- [Main Repository](https://github.com/loonghao/rez-lsp-server)
- [Issue Tracker](https://github.com/loonghao/rez-lsp-server/issues)
- [Rez Documentation](https://rez.readthedocs.io/)
