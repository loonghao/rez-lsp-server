#!/usr/bin/env pwsh
# Development installation script for Rez LSP Server

param(
    [switch]$Force,
    [switch]$Help
)

if ($Help) {
    Write-Host "Rez LSP Server Development Installation" -ForegroundColor Blue
    Write-Host ""
    Write-Host "This script sets up a complete development environment:" -ForegroundColor Yellow
    Write-Host "  • Builds the LSP server in debug mode"
    Write-Host "  • Builds and installs the VSCode extension"
    Write-Host "  • Configures VSCode settings for development"
    Write-Host "  • Sets up test environment"
    Write-Host ""
    Write-Host "Usage: .\scripts\dev-install.ps1 [options]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Cyan
    Write-Host "  -Force    Force reinstallation even if already installed"
    Write-Host "  -Help     Show this help message"
    exit 0
}

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "🔧 $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Cyan
}

function Find-VSCode {
    # Try to find VSCode executable in various locations
    $possiblePaths = @()

    # Standard PATH lookup
    $pathCommand = Get-Command code -ErrorAction SilentlyContinue
    if ($pathCommand) {
        $possiblePaths += $pathCommand.Source
    }

    # Add common installation paths
    $possiblePaths += @(
        "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd",
        "$env:PROGRAMFILES\Microsoft VS Code\bin\code.cmd",
        "${env:PROGRAMFILES(X86)}\Microsoft VS Code\bin\code.cmd",

        # Insiders version
        "$env:LOCALAPPDATA\Programs\Microsoft VS Code Insiders\bin\code-insiders.cmd",
        "$env:PROGRAMFILES\Microsoft VS Code Insiders\bin\code-insiders.cmd",

        # Portable versions (common locations)
        "C:\VSCode\bin\code.cmd",
        "D:\VSCode\bin\code.cmd",

        # User-specific installations
        "$env:USERPROFILE\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd"
    )

    foreach ($path in $possiblePaths) {
        if ($path -and (Test-Path $path)) {
            Write-Info "Found VSCode at: $path"
            return $path
        }
    }

    # Try registry lookup for installed versions
    try {
        $registryPaths = @(
            "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
            "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
            "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
        )

        foreach ($regPath in $registryPaths) {
            $items = Get-ItemProperty $regPath -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -like "*Visual Studio Code*" }

            foreach ($item in $items) {
                if ($item.InstallLocation) {
                    $codePath = Join-Path $item.InstallLocation "bin\code.cmd"
                    if (Test-Path $codePath) {
                        Write-Info "Found VSCode via registry at: $codePath"
                        return $codePath
                    }
                }
            }
        }
    }
    catch {
        Write-Info "Registry lookup failed: $($_.Exception.Message)"
    }

    return $null
}

function Test-VSCodeInstallation {
    param([string]$CodePath)

    try {
        $version = & $CodePath --version 2>$null
        if ($LASTEXITCODE -eq 0 -and $version) {
            Write-Info "VSCode version: $($version[0])"
            return $true
        }
    }
    catch {
        Write-Info "Failed to get VSCode version: $($_.Exception.Message)"
    }

    return $false
}

Write-Host "🚀 Rez LSP Server Development Setup" -ForegroundColor Blue
Write-Host "=====================================" -ForegroundColor Blue
Write-Host ""

# Check prerequisites
Write-Step "Checking prerequisites..."

# Check Rust
$rustCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $rustCheck) {
    Write-Error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}

# Check Node.js
$nodeCheck = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCheck) {
    Write-Error "Node.js not found. Please install Node.js from https://nodejs.org/"
    exit 1
}

# Check VSCode
Write-Info "Looking for VSCode installation..."
$codePath = Find-VSCode
if (-not $codePath) {
    Write-Error "VSCode not found. Please install VSCode from https://code.visualstudio.com/"
    Write-Info "Searched locations:"
    Write-Info "  • PATH environment variable"
    Write-Info "  • Standard installation directories"
    Write-Info "  • Windows Registry"
    exit 1
}

if (-not (Test-VSCodeInstallation $codePath)) {
    Write-Error "VSCode found but not working properly at: $codePath"
    exit 1
}

Write-Success "All prerequisites found"

# Build LSP Server
Write-Step "Building LSP server..."
$buildResult = cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build LSP server"
    exit 1
}
Write-Success "LSP server built successfully"

# Build and install VSCode extension
Write-Step "Building VSCode extension..."

Push-Location "vscode-extension"
try {
    # Install dependencies
    Write-Info "Installing npm dependencies..."
    $npmInstall = npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install npm dependencies"
        exit 1
    }
    
    # Compile TypeScript
    Write-Info "Compiling TypeScript..."
    $npmCompile = npm run compile
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to compile TypeScript"
        exit 1
    }
    
    Write-Success "VSCode extension built successfully"
    
    # Package extension first
    Write-Step "Packaging VSCode extension..."

    # Install vsce if not available
    $vsceCheck = Get-Command vsce -ErrorAction SilentlyContinue
    if (-not $vsceCheck) {
        Write-Info "Installing vsce..."
        npm install -g vsce
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to install vsce"
            exit 1
        }
    }

    # Package the extension
    $packageResult = vsce package --out rez-lsp-extension.vsix
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to package VSCode extension"
        exit 1
    }

    # Install extension
    Write-Step "Installing VSCode extension for development..."
    $vsixPath = Join-Path (Get-Location) "rez-lsp-extension.vsix"
    Write-Info "Using VSCode at: $codePath"
    Write-Info "Installing extension from: $vsixPath"

    $installResult = & $codePath --install-extension $vsixPath --force
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install VSCode extension"
        Write-Info "You can manually install the extension by:"
        Write-Info "  1. Open VSCode"
        Write-Info "  2. Press Ctrl+Shift+P"
        Write-Info "  3. Type 'Extensions: Install from VSIX'"
        Write-Info "  4. Select the VSIX file: $vsixPath"
        exit 1
    }
    
    Write-Success "VSCode extension installed"
    
}
finally {
    Pop-Location
}

# Create development configuration
Write-Step "Creating development configuration..."

$configDir = "$env:APPDATA\Code\User"
if (-not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

$settingsPath = "$configDir\settings.json"
$currentDir = Get-Location

# Read existing settings or create new
$settings = @{}
if (Test-Path $settingsPath) {
    try {
        $settingsContent = Get-Content $settingsPath -Raw
        $settings = $settingsContent | ConvertFrom-Json -AsHashtable
    }
    catch {
        Write-Info "Could not parse existing settings.json, creating new configuration"
        $settings = @{}
    }
}

# Add Rez LSP settings
$settings["rezLsp.serverPath"] = "$currentDir\target\debug\rez-lsp-server.exe"
$settings["rezLsp.trace.server"] = "verbose"
$settings["rezLsp.enableDiagnostics"] = $true

# Convert back to JSON and save
$settingsJson = $settings | ConvertTo-Json -Depth 10
$settingsJson | Set-Content $settingsPath -Encoding UTF8

Write-Success "Development configuration created"

# Set up test environment
Write-Step "Setting up test environment..."

$testPackagesPath = "$currentDir\test_packages"
if (Test-Path $testPackagesPath) {
    Write-Info "Test packages already exist"
}
else {
    Write-Info "Test packages not found, you may need to create them manually"
}

Write-Success "Test environment configured"

# Run basic tests
Write-Step "Running basic tests..."
$testResult = cargo test --lib
if ($LASTEXITCODE -ne 0) {
    Write-Error "Some tests failed, but installation continues"
}
else {
    Write-Success "All tests passed"
}

# Summary
Write-Host ""
Write-Host "🎉 Development environment setup complete!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host ""

Write-Host "What's been configured:" -ForegroundColor Yellow
Write-Host "  ✅ LSP server built in debug mode"
Write-Host "  ✅ VSCode extension installed for development"
Write-Host "  ✅ VSCode settings configured for development"
Write-Host "  ✅ Test environment prepared"
Write-Host ""

Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Set environment variable:"
Write-Host "     `$env:REZ_PACKAGES_PATH = '$testPackagesPath'"
Write-Host ""
Write-Host "  2. Open a Rez project in VSCode:"
Write-Host "     code ."
Write-Host ""
Write-Host "  3. Test the extension:"
Write-Host "     • Open a package.py file"
Write-Host "     • Try code completion in requires list"
Write-Host "     • Check 'Rez LSP' output channel"
Write-Host ""

Write-Host "Useful commands:" -ForegroundColor Magenta
Write-Host "  • Rebuild: cargo build"
Write-Host "  • Run tests: cargo test"
Write-Host "  • Test LSP: node test_lsp_client.js"
Write-Host "  • View logs: Check 'Rez LSP' output in VSCode"
Write-Host ""

Write-Host "Happy coding! 🚀" -ForegroundColor Green
