#!/bin/bash

# Test script for RainbowBrowserAI Web Dashboard

echo "🌈 Testing RainbowBrowserAI Web Dashboard..."
echo "=========================================="

# Start the API server in the background
echo "Starting API server..."
cargo run --release -- serve &
SERVER_PID=$!

# Wait for server to start
echo "Waiting for server to be ready..."
sleep 3

# Check if server is running
if ! curl -s http://localhost:3000/health > /dev/null; then
    echo "❌ Server failed to start"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

echo "✅ Server is running on http://localhost:3000"
echo ""
echo "📊 Dashboard available at: http://localhost:3000/"
echo ""
echo "Available endpoints:"
echo "  - Dashboard: http://localhost:3000/"
echo "  - Health: http://localhost:3000/health"
echo "  - Metrics: http://localhost:3000/metrics"
echo "  - Cost: http://localhost:3000/cost"
echo ""
echo "Press Ctrl+C to stop the server..."

# Wait for user to stop
trap "echo 'Stopping server...'; kill $SERVER_PID 2>/dev/null; exit 0" INT TERM
wait $SERVER_PID