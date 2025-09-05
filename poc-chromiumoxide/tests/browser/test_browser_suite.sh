#!/bin/bash

# Browser Automation Test Suite
# Tests core browser functionality, navigation, and automation features

set -e

# Configuration
SERVER_URL="http://localhost:3002"
PASSED=0
FAILED=0

# Test utilities
test_passed() {
    echo "✅ $1"
    ((PASSED++))
}

test_failed() {
    echo "❌ $1"
    ((FAILED++))
}

test_navigation() {
    local url="$1"
    local test_name="$2"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"navigate_to_url\",\"parameters\":{\"url\":\"$url\"}}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$response" | jq -r '.success')
        if [ "$success" = "true" ]; then
            test_passed "$test_name"
            return 0
        else
            test_failed "$test_name (navigation failed)"
            return 1
        fi
    else
        test_failed "$test_name (invalid response)"
        return 1
    fi
}

echo "========================================="
echo "     Browser Automation Tests"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Navigation Tests
echo "--- Navigation Tests ---"
test_navigation "https://example.com" "Navigate to example.com"
test_navigation "https://httpbin.org" "Navigate to httpbin.org"
test_navigation "https://github.com" "Navigate to github.com"

# Test invalid URLs
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"invalid-url"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Invalid URL handling"
    else
        test_failed "Invalid URL should fail gracefully"
    fi
else
    test_failed "Invalid URL response format"
fi

# Browser Controls Tests
echo ""
echo "--- Browser Controls Tests ---"

# Test refresh
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"refresh","parameters":{}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Page refresh functionality"
else
    test_failed "Page refresh functionality"
fi

# Test go back
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"go_back","parameters":{}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Browser back navigation"
else
    test_failed "Browser back navigation"
fi

# Test go forward
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"go_forward","parameters":{}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Browser forward navigation"
else
    test_failed "Browser forward navigation"
fi

# Element Interaction Tests
echo ""
echo "--- Element Interaction Tests ---"

# Navigate to test page first
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Test click functionality
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"click","parameters":{"selector":"input[type=\"text\"]"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Element click functionality"
else
    test_failed "Element click functionality"
fi

# Test typing functionality
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"type_text","parameters":{"selector":"input[name=\"custname\"]","text":"Test User"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Text input functionality"
else
    test_failed "Text input functionality"
fi

# Test hover functionality
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"hover","parameters":{"selector":"input[type=\"submit\"]"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Element hover functionality"
else
    test_failed "Element hover functionality"
fi

# Test focus functionality
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"focus","parameters":{"selector":"input[name=\"custel\"]"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Element focus functionality"
else
    test_failed "Element focus functionality"
fi

# Data Extraction Tests
echo ""
echo "--- Data Extraction Tests ---"

# Navigate to content-rich page
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Test text extraction
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"extract_text","parameters":{"selector":"h1"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.data.text' > /dev/null 2>&1; then
    text=$(echo "$response" | jq -r '.data.text')
    if [ -n "$text" ] && [ "$text" != "null" ]; then
        test_passed "Text extraction functionality"
    else
        test_failed "Text extraction returned empty"
    fi
else
    test_failed "Text extraction functionality"
fi

# Test link extraction
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"extract_links","parameters":{"selector":"a"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.data.links' > /dev/null 2>&1; then
    test_passed "Link extraction functionality"
else
    test_failed "Link extraction functionality"
fi

# Test element info extraction
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"get_element_info","parameters":{"selector":"body"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.data' > /dev/null 2>&1; then
    test_passed "Element info extraction"
else
    test_failed "Element info extraction"
fi

# Wait and Timing Tests
echo ""
echo "--- Wait and Timing Tests ---"

# Test wait for element
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"wait_for_element","parameters":{"selector":"body","timeout":5000}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "true" ]; then
        test_passed "Wait for element (existing element)"
    else
        test_failed "Wait for element should succeed for body"
    fi
else
    test_failed "Wait for element response format"
fi

# Test wait timeout
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"wait_for_element","parameters":{"selector":".non-existent-element","timeout":2000}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Wait for element timeout handling"
    else
        test_failed "Wait should timeout for non-existent element"
    fi
else
    test_failed "Wait timeout response format"
fi

# Session Management Tests
echo ""
echo "--- Session Management Tests ---"

# Test session storage
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"session_storage","parameters":{"action":"set","key":"test_key","value":"test_value"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Session storage set operation"
    
    # Test session storage get
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"session_storage","parameters":{"action":"get","key":"test_key"}}' \
        "$SERVER_URL/api/tools/execute")
    if echo "$response" | jq -e '.data.value' > /dev/null 2>&1; then
        value=$(echo "$response" | jq -r '.data.value')
        if [ "$value" = "test_value" ]; then
            test_passed "Session storage get operation"
        else
            test_failed "Session storage value mismatch"
        fi
    else
        test_failed "Session storage get operation"
    fi
else
    test_failed "Session storage set operation"
fi

# Test persistent cache
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"persistent_cache","parameters":{"action":"get"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    test_passed "Persistent cache access"
else
    test_failed "Persistent cache access"
fi

# Error Handling Tests
echo ""
echo "--- Error Handling Tests ---"

# Test with invalid selector
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"click","parameters":{"selector":"invalid>>selector"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Invalid selector error handling"
    else
        test_failed "Invalid selector should fail"
    fi
else
    test_failed "Invalid selector response format"
fi

# Performance Tests
echo ""
echo "--- Performance Tests ---"

# Test navigation speed
start_time=$(date +%s.%N)
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null
end_time=$(date +%s.%N)
navigation_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$navigation_time < 10.0" | bc -l) )); then
    test_passed "Navigation performance acceptable (${navigation_time}s)"
else
    test_failed "Navigation too slow (${navigation_time}s)"
fi

# Summary
echo ""
echo "========================================="
echo "        BROWSER TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All browser tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED browser test(s) failed"
    exit 1
fi