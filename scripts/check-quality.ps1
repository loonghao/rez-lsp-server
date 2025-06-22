#!/usr/bin/env pwsh
# Code quality check script for Rez LSP Server

Write-Host "🔍 Running code quality checks..." -ForegroundColor Blue

# Check formatting
Write-Host "📝 Checking code formatting..." -ForegroundColor Yellow
$formatResult = cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Code formatting check failed. Run 'cargo fmt --all' to fix." -ForegroundColor Red
    exit 1
}
Write-Host "✅ Code formatting is correct" -ForegroundColor Green

# Run clippy
Write-Host "🔧 Running clippy..." -ForegroundColor Yellow
$clippyResult = cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy found issues" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Clippy checks passed" -ForegroundColor Green

# Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Yellow
$testResult = cargo test --lib
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tests failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ All tests passed" -ForegroundColor Green

# Build project
Write-Host "🏗️ Building project..." -ForegroundColor Yellow
$buildResult = cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Build successful" -ForegroundColor Green

Write-Host "🎉 All quality checks passed!" -ForegroundColor Green
