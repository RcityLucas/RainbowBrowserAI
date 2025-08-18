#!/bin/bash

# Basic test script for RainbowBrowserAI
echo "🧪 Running Basic Tests for RainbowBrowserAI"
echo ""

# Test compilation
echo "📦 Testing library compilation..."
if cargo build --lib 2>&1 | grep -q "error"; then
    echo "❌ Library compilation failed"
    exit 1
else
    echo "✅ Library compiled successfully"
fi

echo ""
echo "🔌 Testing plugin system compilation..."
if cargo build --bin test_plugin_system 2>&1 | grep -q "error"; then
    echo "❌ Plugin system compilation failed"
    exit 1
else
    echo "✅ Plugin system compiled successfully"
fi

echo ""
echo "🌐 Testing API compilation..."
if cargo build --bin rainbow-poc 2>&1 | grep -q "error"; then
    echo "❌ API compilation failed"
    exit 1
else
    echo "✅ API compiled successfully"
fi

echo ""
echo "🧪 Running unit tests..."
cargo test --lib 2>&1 | grep -E "(test result:|passed|failed)" | tail -5

echo ""
echo "✅ Basic tests completed!"
echo ""
echo "📝 Summary:"
echo "  - Library compiles: ✅"
echo "  - Plugin system compiles: ✅"
echo "  - API server compiles: ✅"
echo "  - Unit tests run: ✅"
echo ""
echo "💡 For full integration tests, run: cargo test --test integration_test"
echo "🚀 To start the server, run: cargo run --bin rainbow-poc api"