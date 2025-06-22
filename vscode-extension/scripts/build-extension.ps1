# Build script for VSCode extension with embedded LSP server
param(
    [string]$Platform = "win32-x64",
    [switch]$Release = $false
)

Write-Host "🔨 Building Rez LSP Extension with embedded server..." -ForegroundColor Green

# Determine build configuration
$BuildConfig = if ($Release) { "release" } else { "debug" }
$CargoTarget = "../target/$BuildConfig"

Write-Host "📋 Configuration:" -ForegroundColor Cyan
Write-Host "  Platform: $Platform" -ForegroundColor Gray
Write-Host "  Build Config: $BuildConfig" -ForegroundColor Gray
Write-Host "  Cargo Target: $CargoTarget" -ForegroundColor Gray

# Create server directory in extension
$ServerDir = "server"
if (Test-Path $ServerDir) {
    Remove-Item -Path $ServerDir -Recurse -Force
}
New-Item -ItemType Directory -Path $ServerDir -Force | Out-Null

# Build Rust LSP server first
Write-Host "🦀 Building Rust LSP server..." -ForegroundColor Yellow
Push-Location ..
try {
    if ($Release) {
        cargo build --release
    } else {
        cargo build
    }
    
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

# Copy LSP server binary based on platform
Write-Host "📦 Copying LSP server binary..." -ForegroundColor Yellow

$ServerBinary = switch ($Platform) {
    "win32-x64" { "rez-lsp-server.exe" }
    "linux-x64" { "rez-lsp-server" }
    "darwin-x64" { "rez-lsp-server" }
    "darwin-arm64" { "rez-lsp-server" }
    default { "rez-lsp-server.exe" }
}

$SourcePath = "$CargoTarget/$ServerBinary"
$DestPath = "$ServerDir/$ServerBinary"

if (Test-Path $SourcePath) {
    Copy-Item -Path $SourcePath -Destination $DestPath -Force
    Write-Host "✅ Copied $ServerBinary to extension" -ForegroundColor Green
    
    # Get file size for info
    $FileSize = (Get-Item $DestPath).Length
    $FileSizeMB = [math]::Round($FileSize / 1MB, 2)
    Write-Host "   Size: $FileSizeMB MB" -ForegroundColor Gray
} else {
    Write-Host "❌ LSP server binary not found at: $SourcePath" -ForegroundColor Red
    Write-Host "   Make sure to build the Rust project first" -ForegroundColor Yellow
    exit 1
}

# Compile TypeScript
Write-Host "📝 Compiling TypeScript..." -ForegroundColor Yellow
npm run compile
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ TypeScript compilation failed" -ForegroundColor Red
    exit 1
}

# Run linting
Write-Host "🔍 Running linter..." -ForegroundColor Yellow
npm run lint
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Linting issues found" -ForegroundColor Yellow
}

# Package extension
Write-Host "📦 Packaging extension..." -ForegroundColor Yellow
$PackageName = "rez-lsp-extension-$Platform"
if ($Release) {
    $PackageName += "-release"
}
$PackageName += ".vsix"

# Use vsce to package
vsce package --out $PackageName --target $Platform

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Extension packaged successfully: $PackageName" -ForegroundColor Green
    
    # Show package info
    $PackageSize = (Get-Item $PackageName).Length
    $PackageSizeMB = [math]::Round($PackageSize / 1MB, 2)
    Write-Host "   Package size: $PackageSizeMB MB" -ForegroundColor Gray
    
    Write-Host ""
    Write-Host "🚀 Installation command:" -ForegroundColor Cyan
    Write-Host "   code --install-extension $PackageName" -ForegroundColor Gray
} else {
    Write-Host "❌ Extension packaging failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 Build complete!" -ForegroundColor Green
