#!/bin/bash

# API Testing Script for RainbowBrowserAI
# Tests all major API endpoints

set -e

BASE_URL="${BASE_URL:-http://localhost:3001}"
TIMEOUT=30

echo "🧪 Testing RainbowBrowserAI API endpoints..."
echo "🌐 Base URL: $BASE_URL"
echo ""

# Function to make HTTP requests with timeout
http_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=$4
    
    echo -n "Testing $method $endpoint... "
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "%{http_code}" -m $TIMEOUT \
            -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$BASE_URL$endpoint" 2>/dev/null || echo "000")
    else
        response=$(curl -s -w "%{http_code}" -m $TIMEOUT \
            -X "$method" \
            "$BASE_URL$endpoint" 2>/dev/null || echo "000")
    fi
    
    status_code="${response: -3}"
    body="${response%???}"
    
    if [ "$status_code" = "$expected_status" ]; then
        echo "✅ $status_code"
        if [ -n "$body" ] && [ "$body" != "null" ]; then
            echo "    Response: $(echo "$body" | head -c 100)..."
        fi
    else
        echo "❌ Expected $expected_status, got $status_code"
        if [ -n "$body" ]; then
            echo "    Response: $body"
        fi
        return 1
    fi
}

# Wait for server to be ready
echo "⏳ Waiting for server to be ready..."
for i in {1..30}; do
    if curl -s -m 5 "$BASE_URL/api/health" >/dev/null 2>&1; then
        echo "✅ Server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Server not ready after 30 attempts"
        exit 1
    fi
    sleep 1
done

echo ""
echo "🔍 Running API tests..."
echo ""

# Test health endpoint
http_request "GET" "/api/health" "" "200"

# Test session creation
echo -n "Creating session for further tests... "
session_response=$(curl -s -m $TIMEOUT -X POST "$BASE_URL/api/session/create" 2>/dev/null)
session_id=$(echo "$session_response" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)

if [ -n "$session_id" ]; then
    echo "✅ Session created: $session_id"
else
    echo "❌ Failed to create session"
    echo "Response: $session_response"
    exit 1
fi

# Test navigation
http_request "POST" "/api/navigate" \
    "{\"url\":\"https://example.com\",\"session_id\":\"$session_id\"}" "200"

# Test perception modes
perception_modes=("lightning" "quick" "standard" "deep" "adaptive")
for mode in "${perception_modes[@]}"; do
    http_request "POST" "/api/perceive-mode" \
        "{\"mode\":\"$mode\",\"session_id\":\"$session_id\"}" "200"
done

# Test element interaction endpoints
http_request "POST" "/api/smart-element-search" \
    "{\"query\":\"button\",\"max_results\":5,\"session_id\":\"$session_id\"}" "200"

http_request "POST" "/api/click" \
    "{\"selector\":\"button\",\"session_id\":\"$session_id\"}" "200"

http_request "POST" "/api/type" \
    "{\"selector\":\"input\",\"text\":\"test\",\"session_id\":\"$session_id\"}" "200"

http_request "POST" "/api/screenshot" \
    "{\"session_id\":\"$session_id\"}" "200"

# Test form analysis
http_request "POST" "/api/analyze-forms" \
    "{\"session_id\":\"$session_id\"}" "200"

# Test session list
http_request "GET" "/api/sessions" "" "200"

# Test session deletion
http_request "DELETE" "/api/session/$session_id" "" "200"

# Test error handling - invalid session
http_request "POST" "/api/navigate" \
    "{\"url\":\"https://example.com\",\"session_id\":\"invalid-session\"}" "404"

echo ""
echo "🎉 All API tests completed!"
echo ""

# Optional: Load testing
if command -v ab >/dev/null 2>&1; then
    echo "🔥 Running basic load test..."
    ab -n 100 -c 10 -q "$BASE_URL/api/health" | grep -E "(Requests per second|Time per request)"
    echo ""
fi

echo "✅ API testing complete!"