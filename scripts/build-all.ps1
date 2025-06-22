#!/usr/bin/env pwsh
# Build script for Rez LSP Server and VSCode extension

param(
    [switch]$Release,
    [switch]$Extension,
    [switch]$Install,
    [switch]$Package,
    [switch]$Test,
    [switch]$Clean,
    [switch]$Help
)

if ($Help) {
    Write-Host "Rez LSP Server Build Script" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\scripts\build-all.ps1 [options]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Green
    Write-Host "  -Release    Build in release mode"
    Write-Host "  -Extension  Build VSCode extension"
    Write-Host "  -Install    Install VSCode extension for development"
    Write-Host "  -Package    Create distribution packages"
    Write-Host "  -Test       Run all tests"
    Write-Host "  -Clean      Clean build artifacts"
    Write-Host "  -Help       Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\scripts\build-all.ps1 -Release -Extension"
    Write-Host "  .\scripts\build-all.ps1 -Install"
    Write-Host "  .\scripts\build-all.ps1 -Test"
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
        "$env:PROGRAMFILES\Microsoft VS Code Insiders\bin\code-insiders.cmd"
    )

    foreach ($path in $possiblePaths) {
        if ($path -and (Test-Path $path)) {
            return $path
        }
    }

    return $null
}

# Clean build artifacts
if ($Clean) {
    Write-Step "Cleaning build artifacts..."
    
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
    }
    
    if (Test-Path "vscode-extension/out") {
        Remove-Item -Recurse -Force "vscode-extension/out"
    }
    
    if (Test-Path "vscode-extension/node_modules") {
        Remove-Item -Recurse -Force "vscode-extension/node_modules"
    }
    
    Write-Success "Build artifacts cleaned"
}

# Build LSP Server
Write-Step "Building Rez LSP Server..."

$buildArgs = @("build")
if ($Release) {
    $buildArgs += "--release"
}
if ($Extension) {
    $env:BUILD_VSCODE_EXTENSION = "1"
}

$buildResult = & cargo @buildArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build LSP server"
    exit 1
}

Write-Success "LSP server built successfully"

# Build VSCode Extension
if ($Extension -or $Install -or $Package) {
    Write-Step "Building VSCode extension..."
    
    Push-Location "vscode-extension"
    
    try {
        # Install dependencies
        Write-Host "Installing npm dependencies..."
        $npmInstall = npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to install npm dependencies"
            exit 1
        }
        
        # Compile TypeScript
        Write-Host "Compiling TypeScript..."
        $npmCompile = npm run compile
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to compile TypeScript"
            exit 1
        }
        
        Write-Success "VSCode extension built successfully"
        
        # Package extension
        if ($Package) {
            Write-Step "Packaging VSCode extension..."
            
            # Install vsce if not available
            $vsceCheck = Get-Command vsce -ErrorAction SilentlyContinue
            if (-not $vsceCheck) {
                Write-Host "Installing vsce..."
                npm install -g vsce
            }
            
            $packageResult = vsce package
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Failed to package VSCode extension"
                exit 1
            }
            
            Write-Success "VSCode extension packaged successfully"
        }
        
        # Install extension for development
        if ($Install) {
            Write-Step "Installing VSCode extension for development..."

            # Find VSCode
            $codePath = Find-VSCode
            if (-not $codePath) {
                Write-Error "VSCode not found. Please install VSCode or add it to PATH"
                exit 1
            }

            # Get the extension path
            $extensionPath = Get-Location

            # Install extension
            Write-Host "Using VSCode at: $codePath"
            $installResult = & $codePath --install-extension $extensionPath --force
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Failed to install VSCode extension"
                exit 1
            }

            Write-Success "VSCode extension installed for development"
        }
        
    }
    finally {
        Pop-Location
    }
}

# Run tests
if ($Test) {
    Write-Step "Running tests..."
    
    # Run Rust tests
    Write-Host "Running Rust tests..."
    $testResult = cargo test
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Rust tests failed"
        exit 1
    }
    
    # Run VSCode extension tests if available
    if (Test-Path "vscode-extension/src/test") {
        Write-Host "Running VSCode extension tests..."
        Push-Location "vscode-extension"
        try {
            $extTestResult = npm test
            if ($LASTEXITCODE -ne 0) {
                Write-Error "VSCode extension tests failed"
                exit 1
            }
        }
        finally {
            Pop-Location
        }
    }
    
    Write-Success "All tests passed"
}

# Summary
Write-Host ""
Write-Host "🎉 Build completed successfully!" -ForegroundColor Green
Write-Host ""

if ($Release) {
    $binaryPath = "target/release/rez-lsp-server.exe"
    if (Test-Path $binaryPath) {
        Write-Host "LSP Server binary: $binaryPath" -ForegroundColor Cyan
    }
}

if ($Extension -or $Package) {
    $vsixFiles = Get-ChildItem "vscode-extension/*.vsix" -ErrorAction SilentlyContinue
    if ($vsixFiles) {
        Write-Host "VSCode extension package: $($vsixFiles[0].FullName)" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  • Test the LSP server: cargo run"
Write-Host "  • Install extension: .\scripts\build-all.ps1 -Install"
Write-Host "  • Run tests: .\scripts\build-all.ps1 -Test"
