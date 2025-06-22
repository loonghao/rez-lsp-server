#!/bin/bash
# Code quality check script for Rez LSP Server

set -e

echo "🔍 Running code quality checks..."

# Check formatting
echo "📝 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi
echo "✅ Code formatting is correct"

# Run clippy
echo "🔧 Running clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues"
    exit 1
fi
echo "✅ Clippy checks passed"

# Run tests
echo "🧪 Running tests..."
if ! cargo test --lib; then
    echo "❌ Tests failed"
    exit 1
fi
echo "✅ All tests passed"

# Build project
echo "🏗️ Building project..."
if ! cargo build --release; then
    echo "❌ Build failed"
    exit 1
fi
echo "✅ Build successful"

echo "🎉 All quality checks passed!"
