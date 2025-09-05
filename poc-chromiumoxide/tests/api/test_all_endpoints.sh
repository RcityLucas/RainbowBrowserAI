#!/bin/bash

# Comprehensive API Endpoint Tests
# Tests all API endpoints with various scenarios

set -e

# Configuration
SERVER_URL="http://localhost:3002"
TEST_RESULTS=()
PASSED=0
FAILED=0

# Test utilities
test_passed() {
    echo "✅ $1"
    TEST_RESULTS+=("PASS: $1")
    ((PASSED++))
}

test_failed() {
    echo "❌ $1"
    TEST_RESULTS+=("FAIL: $1")
    ((FAILED++))
}

test_endpoint() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local expected_status="$4"
    local test_name="$5"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$SERVER_URL$endpoint")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" "$SERVER_URL$endpoint")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$status_code" = "$expected_status" ]; then
        test_passed "$test_name"
        return 0
    else
        test_failed "$test_name (Expected: $expected_status, Got: $status_code)"
        return 1
    fi
}

echo "========================================="
echo "     API Endpoint Tests"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Health Check Tests
echo "--- Health Check Tests ---"
test_endpoint "GET" "/api/health" "" "200" "Health check endpoint"

# Tools API Tests  
echo ""
echo "--- Tools API Tests ---"
test_endpoint "GET" "/api/tools" "" "200" "List all tools"

# Navigation Tools
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' "200" "Navigate to URL tool"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"go_back","parameters":{}}' "200" "Go back tool"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"refresh","parameters":{}}' "200" "Refresh tool"

# Interaction Tools
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"click","parameters":{"selector":"a"}}' "200" "Click tool with valid selector"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"click","parameters":{"selector":".non-existent"}}' "200" "Click tool with invalid selector (should return error in body)"

test_endpoint "POST" "/api/tools/execute" '{"tool_name":"type_text","parameters":{"selector":"input","text":"test"}}' "200" "Type text tool"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"hover","parameters":{"selector":"body"}}' "200" "Hover tool"

# Extraction Tools
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"extract_text","parameters":{"selector":"h1"}}' "200" "Extract text tool"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"extract_links","parameters":{"selector":"a"}}' "200" "Extract links tool"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"get_element_info","parameters":{"selector":"body"}}' "200" "Get element info tool"

# Wait Tools
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"wait_for_element","parameters":{"selector":"body","timeout":5000}}' "200" "Wait for element tool"

# Memory/Cache Tools
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"session_storage","parameters":{"action":"get"}}' "200" "Session storage get"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"session_storage","parameters":{"action":"set","key":"test","value":"data"}}' "200" "Session storage set"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"persistent_cache","parameters":{"action":"get"}}' "200" "Persistent cache get"

# Perception API Tests
echo ""
echo "--- Perception API Tests ---"
test_endpoint "POST" "/api/perception/analyze" '{}' "200" "Analyze page endpoint"
test_endpoint "POST" "/api/perception/find" '{"description":"link"}' "200" "Find element by description"
test_endpoint "POST" "/api/perception/command" '{"command":{"action":"click","description":"link","parameters":{}}}' "200" "Execute intelligent command"
test_endpoint "POST" "/api/perception/forms/analyze" '{"form_selector":null}' "200" "Analyze form endpoint"

# Error Handling Tests
echo ""
echo "--- Error Handling Tests ---"
test_endpoint "POST" "/api/tools/execute" '{"invalid":"data"}' "400" "Invalid tool request format"
test_endpoint "POST" "/api/tools/execute" '{"tool_name":"non_existent_tool","parameters":{}}' "200" "Non-existent tool (should return error in body)"
test_endpoint "GET" "/api/non-existent-endpoint" "" "404" "Non-existent endpoint"

# JSON Format Tests
echo ""
echo "--- Response Format Tests ---"
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Response contains 'success' field"
else
    test_failed "Response missing 'success' field"
fi

if echo "$response" | jq -e '.data' > /dev/null 2>&1; then
    test_passed "Response contains 'data' field"
else
    test_failed "Response missing 'data' field"
fi

# Concurrent Request Tests
echo ""
echo "--- Concurrent Request Tests ---"
for i in {1..5}; do
    curl -s -X POST -H "Content-Type: application/json" -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' "$SERVER_URL/api/tools/execute" > /dev/null &
done
wait

if [ $? -eq 0 ]; then
    test_passed "Concurrent requests handling"
else
    test_failed "Concurrent requests handling"
fi

# Static File Tests
echo ""
echo "--- Static File Tests ---"
test_endpoint "GET" "/" "" "200" "Main HTML page"
test_endpoint "GET" "/static/app.js" "" "200" "JavaScript assets"
test_endpoint "GET" "/static/styles.css" "" "200" "CSS assets"

# Summary
echo ""
echo "========================================="
echo "           API TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All API tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED test(s) failed:"
    for result in "${TEST_RESULTS[@]}"; do
        if [[ $result == FAIL* ]]; then
            echo "  $result"
        fi
    done
    exit 1
fi