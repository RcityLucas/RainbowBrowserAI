#!/bin/bash

# Test script for Plugin API endpoints
echo "🔌 Testing Plugin API Endpoints"

API_BASE="http://localhost:3000"

# Start the API server in background
echo "🚀 Starting API server..."
RUST_LOG=info cargo run --bin rainbow-poc api &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 5

# Function to test API endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo ""
    echo "🧪 Testing: $description"
    echo "📡 $method $API_BASE$endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "HTTP_STATUS:%{http_code}" "$API_BASE$endpoint")
    else
        response=$(curl -s -w "HTTP_STATUS:%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_BASE$endpoint")
    fi
    
    body=$(echo "$response" | sed 's/HTTP_STATUS:.*//')
    status=$(echo "$response" | sed 's/.*HTTP_STATUS://')
    
    echo "📊 Status: $status"
    echo "📄 Response: $body" | jq . 2>/dev/null || echo "📄 Response: $body"
    
    if [ "$status" -eq 200 ]; then
        echo "✅ Test passed"
    else
        echo "❌ Test failed"
    fi
}

# Test health endpoint first
test_endpoint "GET" "/health" "" "Health check"

# Test plugin discovery
test_endpoint "POST" "/plugins" '{"action": "discover"}' "Plugin discovery"

# Test plugin listing
test_endpoint "POST" "/plugins" '{"action": "list"}' "List plugins"

# Test plugin metrics
test_endpoint "GET" "/plugins/metrics" "" "Plugin metrics"

# Test invalid action
test_endpoint "POST" "/plugins" '{"action": "invalid"}' "Invalid plugin action"

# Test load plugin (should fail if no plugins discovered)
test_endpoint "POST" "/plugins" '{"action": "load", "plugin_id": "hello-world"}' "Load plugin"

# Test unload plugin (should fail if not loaded)
test_endpoint "POST" "/plugins" '{"action": "unload", "plugin_id": "hello-world"}' "Unload plugin"

# Test configure plugin (should fail if not loaded)
test_endpoint "POST" "/plugins" '{"action": "configure", "plugin_id": "hello-world", "config": {"setting1": "value1"}}' "Configure plugin"

echo ""
echo "🔌 Plugin API test completed!"

# Stop the server
echo "🛑 Stopping API server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo "✅ All tests completed!"