#!/bin/bash

# Development watch script for RainbowBrowserAI
# Automatically rebuilds and restarts the application when source code changes

set -e

echo "🔍 Starting development watch mode..."

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch
fi

# Kill any existing processes on port 3001
echo "🧹 Cleaning up existing processes..."
pkill -f "rainbow-poc-chromiumoxide" || true
lsof -ti:3001 | xargs kill -9 2>/dev/null || true

# Change to poc-chromiumoxide directory
cd poc-chromiumoxide

echo "👀 Watching for changes in src/..."
echo "🌐 Server will be available at http://localhost:3001"
echo "💡 Press Ctrl+C to stop"
echo ""

# Watch for changes and restart
cargo watch \
    --watch src \
    --watch static \
    --watch Cargo.toml \
    --clear \
    --exec "run --bin rainbow-poc-chromiumoxide -- serve --port 3001" \
    --env RUST_LOG=debug \
    --env RAINBOW_MOCK_MODE=true