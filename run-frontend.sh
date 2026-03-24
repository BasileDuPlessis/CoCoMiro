#!/bin/bash

# Mori Frontend Runner
# Builds and serves the frontend with proper dependency resolution

set -e  # Exit on any error

echo "🚀 Starting Mori Frontend"
echo "========================"

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if backend is running (optional but recommended)
if ! lsof -i:3000 > /dev/null 2>&1; then
    echo "⚠️  Warning: Backend server not detected on port 3000"
    echo "   Consider running: cd backend && cargo run"
    echo ""
fi

echo "🔨 Building frontend package (workspace dependencies)..."
cargo build --package hello-world-frontend
echo "✅ Build successful"

echo "🌐 Starting frontend dev server on http://localhost:8080"
echo "   Press Ctrl+C to stop"
echo ""

cd frontend
trunk serve --open