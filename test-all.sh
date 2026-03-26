#!/bin/bash

# CoCoMiro Test Runner
# Ensures all tests pass before commits

set -e  # Exit on any error

echo "🚀 Running CoCoMiro Test Suite"
echo "=========================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ] || [ ! -d "backend" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

echo "📋 Checking code formatting..."
cargo fmt --all -- --check
echo "✅ Code formatting OK"

echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy checks passed"

echo "🔨 Building workspace..."
cargo build --workspace
echo "✅ Build successful"

echo "🧪 Running E2E tests..."
cd frontend
npm test
cd ..
echo "✅ E2E tests passed"

echo ""
echo "🎉 All tests passed! Ready to commit."
echo ""
echo "Next steps:"
echo "  git add ."
echo "  git commit -m 'feat: your changes'"
echo "  git push"