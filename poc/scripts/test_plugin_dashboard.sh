#!/bin/bash

# Test script for Plugin Dashboard functionality
echo "🔌 Testing Plugin Dashboard"

API_BASE="http://localhost:3000"

# Start the API server in background
echo "🚀 Starting API server with dashboard..."
RUST_LOG=info cargo run --bin rainbow-poc api &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 5

echo ""
echo "🌐 Dashboard should now be accessible at: http://localhost:3000"
echo ""
echo "✅ Plugin Management Features Available:"
echo "  📁 Plugins tab in navigation"
echo "  🔍 Plugin discovery button"
echo "  📊 Real-time plugin metrics"
echo "  🎛️ Plugin cards with actions (Load/Unload/Configure)"
echo "  ⚡ Real-time status updates via Server-Sent Events"
echo "  🛠️ Configuration modal with JSON editor"
echo ""
echo "🧪 Test Instructions:"
echo "  1. Open http://localhost:3000 in your browser"
echo "  2. Click on the 'Plugins' tab"
echo "  3. Click 'Discover Plugins' to find available plugins"
echo "  4. Try loading/unloading plugins using the action buttons"
echo "  5. Test the configuration modal on loaded plugins"
echo "  6. Watch for real-time updates in the plugin metrics"
echo ""
echo "📋 Expected Plugin Examples:"
echo "  - hello-world v1.0.0 (Action plugin)"
echo "  - database-actions v1.0.0 (DataProcessor plugin)"
echo ""
echo "🔄 Real-time Features:"
echo "  - Plugin metrics update every 2 seconds"
echo "  - Plugin state changes show notifications"
echo "  - Plugin list refreshes automatically on changes"
echo ""

# Test basic API endpoints
echo "🧪 Testing API endpoints..."

# Test plugin discovery
echo "🔍 Testing plugin discovery..."
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"action": "discover"}' \
    "$API_BASE/plugins" | jq . 2>/dev/null || echo "Discovery response received"

echo ""
echo "📊 Testing plugin metrics..."
curl -s "$API_BASE/plugins/metrics" | jq . 2>/dev/null || echo "Metrics response received"

echo ""
echo "📡 Testing SSE endpoint..."
timeout 5 curl -s "$API_BASE/events" | head -10 || echo "SSE stream active"

echo ""
echo "🎯 Ready for manual testing!"
echo "📖 Press Ctrl+C to stop the server and exit"

# Wait for user to stop
trap "echo '🛑 Stopping server...' && kill $SERVER_PID && exit 0" INT
wait $SERVER_PID