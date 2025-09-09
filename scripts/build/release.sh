#!/bin/bash

# Release build script for RainbowBrowserAI
# Creates optimized production build

set -e

echo "🚀 Building RainbowBrowserAI for production..."

# Change to project directory
cd poc-chromiumoxide

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Run pre-build checks
echo "📋 Running pre-build checks..."

# Format check
echo "  🎨 Checking code formatting..."
cargo fmt -- --check

# Clippy check
echo "  🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Tests
echo "  🧪 Running tests..."
RAINBOW_MOCK_MODE=true cargo test --all-features

# Build optimized release
echo "🔨 Building optimized release..."
cargo build --release --bin rainbow-poc-chromiumoxide

# Verify build
echo "✅ Verifying build..."
if [ -f "target/release/rainbow-poc-chromiumoxide" ] || [ -f "target/release/rainbow-poc-chromiumoxide.exe" ]; then
    echo "✅ Build successful!"
    
    # Get binary info
    ls -lah target/release/rainbow-poc-chromiumoxide* 2>/dev/null || true
    
    # Test the binary
    echo "🧪 Testing binary..."
    timeout 5 ./target/release/rainbow-poc-chromiumoxide* --help || true
    
else
    echo "❌ Build failed - binary not found"
    exit 1
fi

echo ""
echo "🎉 Release build complete!"
echo "📁 Binary location: $(pwd)/target/release/rainbow-poc-chromiumoxide*"
echo "🏗️  Ready for production deployment"