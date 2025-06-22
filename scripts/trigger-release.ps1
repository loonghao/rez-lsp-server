# Trigger GitHub Actions Release
# This script triggers the automated release process via GitHub Actions

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [switch]$PreRelease,
    [switch]$Help
)

if ($Help) {
    Write-Host "GitHub Actions Release Trigger" -ForegroundColor Blue
    Write-Host ""
    Write-Host "This script triggers the automated release process via GitHub Actions."
    Write-Host ""
    Write-Host "Usage: .\scripts\trigger-release.ps1 -Version <version> [options]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Parameters:" -ForegroundColor Cyan
    Write-Host "  -Version <version>    Version to release (e.g., '0.1.0')"
    Write-Host "  -PreRelease          Mark as pre-release"
    Write-Host "  -Help                Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\scripts\trigger-release.ps1 -Version '0.1.0'"
    Write-Host "  .\scripts\trigger-release.ps1 -Version '0.2.0-beta.1' -PreRelease"
    Write-Host ""
    Write-Host "Prerequisites:" -ForegroundColor Magenta
    Write-Host "  - GitHub CLI (gh) installed and authenticated"
    Write-Host "  - Repository secrets configured:"
    Write-Host "    * VSCODE_MARKETPLACE_TOKEN"
    Write-Host "    * OPEN_VSX_TOKEN"
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

# Validate version format
if (-not ($Version -match '^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[0-9]+)?)?$')) {
    Write-Error "Invalid version format. Expected format: x.y.z or x.y.z-prerelease (e.g., 0.1.0 or 0.1.0-beta.1)"
    exit 1
}

Write-Host "🚀 GitHub Actions Release Trigger" -ForegroundColor Blue
Write-Host "=================================" -ForegroundColor Blue
Write-Host ""

Write-Info "Release version: $Version"
if ($PreRelease) {
    Write-Info "Release type: Pre-release"
} else {
    Write-Info "Release type: Stable"
}
Write-Host ""

# Step 1: Check prerequisites
Write-Step "Checking prerequisites..."

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Cargo.toml not found. Please run this script from the project root."
    exit 1
}

# Check if GitHub CLI is available
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Error "GitHub CLI (gh) not found. Please install it from https://cli.github.com/"
    exit 1
}

# Check if authenticated with GitHub
$authStatus = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Not authenticated with GitHub CLI. Please run 'gh auth login'"
    exit 1
}

Write-Success "Prerequisites check passed"

# Step 2: Check repository status
Write-Step "Checking repository status..."

# Check if we're on the main branch
$currentBranch = git branch --show-current
if ($currentBranch -ne "main") {
    Write-Warning "Not on main branch. Current branch: $currentBranch"
    Write-Info "Releases are typically created from the main branch."
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        Write-Info "Release cancelled."
        exit 0
    }
}

# Check if working directory is clean
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Warning "Working directory is not clean."
    git status
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        Write-Info "Release cancelled. Please commit or stash changes."
        exit 0
    }
}

Write-Success "Repository status check passed"

# Step 3: Check if tag already exists
Write-Step "Checking if tag v$Version already exists..."

$tagExists = git tag -l "v$Version"
if ($tagExists) {
    Write-Error "Tag v$Version already exists. Please use a different version."
    exit 1
}

Write-Success "Tag v$Version is available"

# Step 4: Trigger GitHub Actions workflow
Write-Step "Triggering GitHub Actions release workflow..."

try {
    if ($PreRelease) {
        $result = gh workflow run release.yml --field version=$Version --field prerelease=true
    } else {
        $result = gh workflow run release.yml --field version=$Version --field prerelease=false
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Release workflow triggered successfully"
    } else {
        Write-Error "Failed to trigger release workflow"
        exit 1
    }
} catch {
    Write-Error "Error triggering workflow: $_"
    exit 1
}

# Step 5: Provide monitoring information
Write-Host ""
Write-Host "🎉 Release workflow triggered successfully!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host ""

Write-Host "Monitor the release process:" -ForegroundColor Yellow
Write-Host "1. GitHub Actions:"
Write-Host "   gh workflow view release.yml --web"
Write-Host "   or visit: https://github.com/loonghao/rez-lsp-server/actions"
Write-Host ""
Write-Host "2. The workflow will automatically:"
Write-Host "   ✅ Validate the version format"
Write-Host "   ✅ Run comprehensive tests"
Write-Host "   ✅ Build binaries for all platforms"
Write-Host "   ✅ Package VSCode extension"
Write-Host "   ✅ Publish to VSCode Marketplace"
Write-Host "   ✅ Publish to Open VSX Registry"
Write-Host "   ✅ Create GitHub release with assets"
Write-Host ""
Write-Host "3. Expected timeline:"
Write-Host "   - Tests and validation: ~5-10 minutes"
Write-Host "   - Binary builds: ~10-15 minutes"
Write-Host "   - Extension publishing: ~2-5 minutes"
Write-Host "   - Total: ~20-30 minutes"
Write-Host ""
Write-Host "4. After completion, verify:"
Write-Host "   - GitHub Release: https://github.com/loonghao/rez-lsp-server/releases"
Write-Host "   - VSCode Marketplace: https://marketplace.visualstudio.com/publishers/loonghao"
Write-Host "   - Open VSX Registry: https://open-vsx.org/user/loonghao"
Write-Host ""

Write-Host "🤖 Automated Process Benefits:" -ForegroundColor Cyan
Write-Host "- ✅ No manual binary compilation needed"
Write-Host "- ✅ No manual extension packaging needed"
Write-Host "- ✅ No manual marketplace uploads needed"
Write-Host "- ✅ Consistent, reproducible releases"
Write-Host "- ✅ Full audit trail and logs"
Write-Host "- ✅ Automatic rollback on failures"
Write-Host ""

Write-Info "You can now close this terminal and monitor the release via GitHub Actions."

# Optional: Open the workflow in browser
$openBrowser = Read-Host "Open GitHub Actions in browser? (Y/n)"
if ($openBrowser -ne "n" -and $openBrowser -ne "N") {
    try {
        gh workflow view release.yml --web
    } catch {
        Write-Warning "Could not open browser automatically. Please visit:"
        Write-Host "https://github.com/loonghao/rez-lsp-server/actions" -ForegroundColor Blue
    }
}
