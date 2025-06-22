# Multi-platform Build Script
# This script builds the Rez LSP Server for multiple platforms

param(
    [string[]]$Targets = @(),
    [switch]$All,
    [switch]$Release,
    [switch]$Clean,
    [switch]$Help
)

if ($Help) {
    Write-Host "Multi-platform Build Script" -ForegroundColor Blue
    Write-Host ""
    Write-Host "This script builds the Rez LSP Server for multiple platforms."
    Write-Host ""
    Write-Host "Usage: .\scripts\build-multiplatform.ps1 [options]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Parameters:" -ForegroundColor Cyan
    Write-Host "  -Targets <targets>    Specific targets to build (space-separated)"
    Write-Host "  -All                  Build for all supported platforms"
    Write-Host "  -Release              Build in release mode (optimized)"
    Write-Host "  -Clean                Clean before building"
    Write-Host "  -Help                 Show this help message"
    Write-Host ""
    Write-Host "Supported Targets:" -ForegroundColor Yellow
    Write-Host "  x86_64-unknown-linux-gnu      Linux x64"
    Write-Host "  aarch64-unknown-linux-gnu     Linux ARM64"
    Write-Host "  x86_64-pc-windows-msvc        Windows x64"
    Write-Host "  aarch64-pc-windows-msvc       Windows ARM64"
    Write-Host "  x86_64-apple-darwin           macOS x64"
    Write-Host "  aarch64-apple-darwin          macOS ARM64"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Magenta
    Write-Host "  .\scripts\build-multiplatform.ps1 -All -Release"
    Write-Host "  .\scripts\build-multiplatform.ps1 -Targets 'x86_64-pc-windows-msvc','aarch64-pc-windows-msvc'"
    Write-Host "  .\scripts\build-multiplatform.ps1 -Targets 'x86_64-unknown-linux-gnu' -Release"
    Write-Host ""
    exit 0
}

# Colors for output
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

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

# Define supported targets
$SupportedTargets = @{
    "x86_64-unknown-linux-gnu" = @{
        "name" = "Linux x64"
        "cross" = $false
        "extension" = ""
    }
    "aarch64-unknown-linux-gnu" = @{
        "name" = "Linux ARM64"
        "cross" = $true
        "extension" = ""
    }
    "x86_64-pc-windows-msvc" = @{
        "name" = "Windows x64"
        "cross" = $false
        "extension" = ".exe"
    }
    "aarch64-pc-windows-msvc" = @{
        "name" = "Windows ARM64"
        "cross" = $false
        "extension" = ".exe"
    }
    "x86_64-apple-darwin" = @{
        "name" = "macOS x64"
        "cross" = $IsLinux -or $IsWindows
        "extension" = ""
    }
    "aarch64-apple-darwin" = @{
        "name" = "macOS ARM64"
        "cross" = $IsLinux -or $IsWindows
        "extension" = ""
    }
}

Write-Host "🚀 Multi-platform Build Script" -ForegroundColor Blue
Write-Host "===============================" -ForegroundColor Blue
Write-Host ""

# Determine targets to build
if ($All) {
    $TargetsToBuild = $SupportedTargets.Keys
    Write-Info "Building for all supported platforms"
} elseif ($Targets.Count -gt 0) {
    $TargetsToBuild = $Targets
    Write-Info "Building for specified targets: $($Targets -join ', ')"
} else {
    # Default to current platform
    if ($IsWindows) {
        $TargetsToBuild = @("x86_64-pc-windows-msvc")
    } elseif ($IsLinux) {
        $TargetsToBuild = @("x86_64-unknown-linux-gnu")
    } elseif ($IsMacOS) {
        $TargetsToBuild = @("x86_64-apple-darwin")
    } else {
        Write-Error "Unable to determine current platform"
        exit 1
    }
    Write-Info "Building for current platform: $($TargetsToBuild -join ', ')"
}

Write-Host ""

# Validate targets
foreach ($target in $TargetsToBuild) {
    if (-not $SupportedTargets.ContainsKey($target)) {
        Write-Error "Unsupported target: $target"
        Write-Info "Supported targets: $($SupportedTargets.Keys -join ', ')"
        exit 1
    }
}

# Check prerequisites
Write-Step "Checking prerequisites..."

if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Cargo.toml not found. Please run this script from the project root."
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo not found. Please install Rust."
    exit 1
}

Write-Success "Prerequisites check passed"

# Clean if requested
if ($Clean) {
    Write-Step "Cleaning previous builds..."
    cargo clean
    Write-Success "Clean completed"
}

# Install required targets and tools
Write-Step "Installing required targets and tools..."

$needsCross = $false
foreach ($target in $TargetsToBuild) {
    $targetInfo = $SupportedTargets[$target]
    
    # Add target
    Write-Info "Adding target: $target ($($targetInfo.name))"
    rustup target add $target
    
    if ($targetInfo.cross) {
        $needsCross = $true
    }
}

# Install cross if needed
if ($needsCross) {
    Write-Info "Installing cross for cross-compilation..."
    if (-not (Get-Command cross -ErrorAction SilentlyContinue)) {
        cargo install cross --git https://github.com/cross-rs/cross
    }
}

Write-Success "Targets and tools installed"

# Build for each target
$BuildResults = @{}
$BuildMode = if ($Release) { "release" } else { "debug" }

Write-Step "Building for $($TargetsToBuild.Count) target(s) in $BuildMode mode..."

foreach ($target in $TargetsToBuild) {
    $targetInfo = $SupportedTargets[$target]
    
    Write-Host ""
    Write-Info "Building for $target ($($targetInfo.name))..."
    
    $buildArgs = @("build", "--target", $target)
    if ($Release) {
        $buildArgs += "--release"
    }
    
    $startTime = Get-Date
    
    try {
        if ($targetInfo.cross) {
            Write-Info "Using cross-compilation..."
            & cross @buildArgs
        } else {
            Write-Info "Using native compilation..."
            & cargo @buildArgs
        }
        
        if ($LASTEXITCODE -eq 0) {
            $endTime = Get-Date
            $duration = $endTime - $startTime
            
            # Check if binary exists
            $binaryPath = "target/$target/$BuildMode/rez-lsp-server$($targetInfo.extension)"
            if (Test-Path $binaryPath) {
                $fileSize = (Get-Item $binaryPath).Length
                $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
                
                $BuildResults[$target] = @{
                    "success" = $true
                    "duration" = $duration
                    "size" = $fileSizeMB
                    "path" = $binaryPath
                }
                
                Write-Success "Build completed for $target in $($duration.TotalSeconds.ToString('F1'))s (${fileSizeMB}MB)"
            } else {
                $BuildResults[$target] = @{
                    "success" = $false
                    "error" = "Binary not found at expected path"
                }
                Write-Error "Binary not found at $binaryPath"
            }
        } else {
            $BuildResults[$target] = @{
                "success" = $false
                "error" = "Build failed with exit code $LASTEXITCODE"
            }
            Write-Error "Build failed for $target"
        }
    } catch {
        $BuildResults[$target] = @{
            "success" = $false
            "error" = $_.Exception.Message
        }
        Write-Error "Build error for $target`: $_"
    }
}

# Summary
Write-Host ""
Write-Host "🎉 Build Summary" -ForegroundColor Green
Write-Host "================" -ForegroundColor Green
Write-Host ""

$successCount = 0
$totalSize = 0

foreach ($target in $TargetsToBuild) {
    $result = $BuildResults[$target]
    $targetInfo = $SupportedTargets[$target]
    
    if ($result.success) {
        $successCount++
        $totalSize += $result.size
        Write-Host "✅ $target ($($targetInfo.name))" -ForegroundColor Green
        Write-Host "   Duration: $($result.duration.TotalSeconds.ToString('F1'))s" -ForegroundColor Gray
        Write-Host "   Size: $($result.size)MB" -ForegroundColor Gray
        Write-Host "   Path: $($result.path)" -ForegroundColor Gray
    } else {
        Write-Host "❌ $target ($($targetInfo.name))" -ForegroundColor Red
        Write-Host "   Error: $($result.error)" -ForegroundColor Gray
    }
    Write-Host ""
}

Write-Host "Results: $successCount/$($TargetsToBuild.Count) successful" -ForegroundColor Cyan
Write-Host "Total size: $([math]::Round($totalSize, 2))MB" -ForegroundColor Cyan

if ($successCount -eq $TargetsToBuild.Count) {
    Write-Host ""
    Write-Success "All builds completed successfully!"
    
    if ($Release) {
        Write-Host ""
        Write-Host "📦 Release binaries are ready for distribution!" -ForegroundColor Yellow
        Write-Host "You can find them in the target/<platform>/release/ directories." -ForegroundColor Yellow
    }
} else {
    Write-Host ""
    Write-Warning "Some builds failed. Check the errors above."
    exit 1
}
