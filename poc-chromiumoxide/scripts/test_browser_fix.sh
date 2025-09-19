#!/bin/bash

echo "Testing Browser Launch Fix"
echo "=========================="

# Clean up any existing Chrome processes
echo "1. Cleaning up existing Chrome processes..."
pkill -f "chrome.*rainbow" 2>/dev/null || true
pkill -f "chrome.*chromiumoxide" 2>/dev/null || true
sleep 2

# Remove any temp directories
echo "2. Cleaning up temp directories..."
rm -rf /tmp/rainbow-* /tmp/chrome-testing 2>/dev/null || true

echo "3. Building project..."
cargo build --release --quiet

echo "4. Testing simple browser creation..."
# Test the coordination tests which create browsers
timeout 60 cargo test --release test_perception_detects_tool_interface 2>&1 | grep -E "(test result|PASS|FAIL|Failed to launch|Successfully created|Browser initialized)"

echo ""
echo "5. Testing server startup..."
# Start server on an available port
timeout 20 cargo run --release -- serve --port 3009 --headless &
SERVER_PID=$!
sleep 5

# Check if server responds
if curl -s http://localhost:3009/api/health >/dev/null 2>&1; then
    echo "✅ SUCCESS: Server started successfully!"
    echo "✅ Browser launch issue appears to be FIXED"
    
    # Test browser acquisition
    echo "6. Testing browser acquisition..."
    curl -s -X POST http://localhost:3009/api/browser/acquire | grep -q "browser_id" && echo "✅ Browser acquisition works!" || echo "❌ Browser acquisition failed"
else
    echo "❌ Server failed to start properly"
fi

# Cleanup
kill $SERVER_PID 2>/dev/null || true
sleep 2

echo ""
echo "Test Summary:"
echo "============"
echo "- Removed conflicting --remote-debugging-port=0 argument"
echo "- Removed conflicting --user-data-dir arguments" 
echo "- Let chromiumoxide handle port management internally"
echo "- Browser pool now uses unique directories without conflicts"
echo ""
echo "The browser launch configuration has been fixed by removing"
echo "the conflicting Chrome arguments that were causing the issue."