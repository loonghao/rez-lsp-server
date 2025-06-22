# Release script for Rez LSP Server
# This script prepares and creates a release

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [switch]$DryRun,
    [switch]$Help
)

if ($Help) {
    Write-Host "Rez LSP Server Release Script" -ForegroundColor Blue
    Write-Host ""
    Write-Host "This script prepares and creates a release for the Rez LSP Server."
    Write-Host ""
    Write-Host "Usage: .\scripts\release.ps1 -Version <version> [options]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Parameters:" -ForegroundColor Cyan
    Write-Host "  -Version <version>    Version to release (e.g., '0.1.0')"
    Write-Host "  -DryRun              Perform a dry run without making changes"
    Write-Host "  -Help                Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\scripts\release.ps1 -Version '0.1.0'"
    Write-Host "  .\scripts\release.ps1 -Version '0.2.0' -DryRun"
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

# Validate version format
if (-not ($Version -match '^\d+\.\d+\.\d+$')) {
    Write-Error "Invalid version format. Expected format: x.y.z (e.g., 0.1.0)"
    exit 1
}

Write-Host "🚀 Rez LSP Server Release Process" -ForegroundColor Blue
Write-Host "=================================" -ForegroundColor Blue
Write-Host ""

if ($DryRun) {
    Write-Warning "DRY RUN MODE - No changes will be made"
    Write-Host ""
}

Write-Info "Release version: $Version"
Write-Host ""

# Step 1: Check prerequisites
Write-Step "Checking prerequisites..."

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Cargo.toml not found. Please run this script from the project root."
    exit 1
}

# Check if git is available
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Error "Git not found. Please install Git."
    exit 1
}

# Check if cargo is available
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo not found. Please install Rust."
    exit 1
}

# Check if we're on the main branch
$currentBranch = git branch --show-current
if ($currentBranch -ne "main") {
    Write-Error "Not on main branch. Current branch: $currentBranch"
    Write-Info "Please switch to main branch before releasing."
    exit 1
}

# Check if working directory is clean
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Error "Working directory is not clean. Please commit or stash changes."
    git status
    exit 1
}

Write-Success "Prerequisites check passed"

# Step 2: Update version in Cargo.toml
Write-Step "Updating version in Cargo.toml..."

if (-not $DryRun) {
    $cargoContent = Get-Content "Cargo.toml"
    $cargoContent = $cargoContent -replace 'version = "\d+\.\d+\.\d+"', "version = `"$Version`""
    $cargoContent | Set-Content "Cargo.toml"
}

Write-Success "Version updated to $Version"

# Step 3: Run tests
Write-Step "Running tests..."

if (-not $DryRun) {
    $testResult = cargo test --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed. Please fix issues before releasing."
        exit 1
    }
}

Write-Success "All tests passed"

# Step 4: Build release
Write-Step "Building release..."

if (-not $DryRun) {
    $buildResult = cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Release build failed."
        exit 1
    }
}

Write-Success "Release build completed"

# Step 5: Build VSCode extension
Write-Step "Building VSCode extension..."

if (-not $DryRun) {
    Push-Location "vscode-extension"
    try {
        npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Error "npm install failed"
            exit 1
        }
        
        npm run compile
        if ($LASTEXITCODE -ne 0) {
            Write-Error "TypeScript compilation failed"
            exit 1
        }
        
        # Package extension
        $vsceCheck = Get-Command vsce -ErrorAction SilentlyContinue
        if (-not $vsceCheck) {
            Write-Info "Installing vsce..."
            npm install -g vsce
        }
        
        vsce package --out "rez-lsp-extension-$Version.vsix"
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Extension packaging failed"
            exit 1
        }
    }
    finally {
        Pop-Location
    }
}

Write-Success "VSCode extension built"

# Step 6: Update CHANGELOG
Write-Step "Updating CHANGELOG..."

if (-not $DryRun) {
    $changelogContent = Get-Content "CHANGELOG.md"
    $newChangelog = @()
    $foundUnreleased = $false
    
    foreach ($line in $changelogContent) {
        if ($line -match "## \[Unreleased\]") {
            $newChangelog += $line
            $newChangelog += ""
            $newChangelog += "## [$Version] - $(Get-Date -Format 'yyyy-MM-dd')"
            $foundUnreleased = $true
        } elseif ($line -match "^\[Unreleased\]:") {
            $newChangelog += "[Unreleased]: https://github.com/loonghao/rez-lsp-server/compare/v$Version...HEAD"
            $newChangelog += "[$Version]: https://github.com/loonghao/rez-lsp-server/releases/tag/v$Version"
        } else {
            $newChangelog += $line
        }
    }
    
    $newChangelog | Set-Content "CHANGELOG.md"
}

Write-Success "CHANGELOG updated"

# Step 7: Commit changes
Write-Step "Committing release changes..."

if (-not $DryRun) {
    git add Cargo.toml CHANGELOG.md "vscode-extension/rez-lsp-extension-$Version.vsix"
    git commit -m "Release v$Version

- Update version to $Version
- Update CHANGELOG
- Build VSCode extension package"
}

Write-Success "Changes committed"

# Step 8: Create tag
Write-Step "Creating release tag..."

if (-not $DryRun) {
    git tag -a "v$Version" -m "Release v$Version"
}

Write-Success "Tag v$Version created"

# Step 9: Summary
Write-Host ""
Write-Host "🎉 Release v$Version prepared successfully!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Review the changes:"
Write-Host "   git show v$Version"
Write-Host ""
Write-Host "2. Push the release:"
Write-Host "   git push origin main"
Write-Host "   git push origin v$Version"
Write-Host ""
Write-Host "3. Create GitHub release:"
Write-Host "   - Go to https://github.com/loonghao/rez-lsp-server/releases"
Write-Host "   - Click 'Create a new release'"
Write-Host "   - Select tag v$Version"
Write-Host "   - Add release notes from CHANGELOG.md"
Write-Host "   - Upload vscode-extension/rez-lsp-extension-$Version.vsix"
Write-Host ""
Write-Host "4. Publish to crates.io (optional):"
Write-Host "   cargo publish"
Write-Host ""

if ($DryRun) {
    Write-Warning "This was a dry run. No changes were made."
    Write-Info "Run without -DryRun to perform the actual release."
}
