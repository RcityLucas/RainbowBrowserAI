#!/bin/bash

# Comprehensive Tools Test Suite  
# Tests all available tools with various scenarios and edge cases

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

execute_tool() {
    local tool_name="$1"
    local parameters="$2"
    local test_name="$3"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"$tool_name\",\"parameters\":$parameters}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$response" | jq -r '.success')
        if [ "$success" = "true" ]; then
            test_passed "$test_name"
            return 0
        else
            test_failed "$test_name - $(echo "$response" | jq -r '.error // "Unknown error"')"
            return 1
        fi
    else
        test_failed "$test_name (invalid response format)"
        return 1
    fi
}

echo "========================================="
echo "     Comprehensive Tools Tests"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Get available tools first
echo "--- Tool Discovery Tests ---"
tools_response=$(curl -s "$SERVER_URL/api/tools")
if echo "$tools_response" | jq -e '.tools' > /dev/null 2>&1; then
    tool_count=$(echo "$tools_response" | jq '.tools | length')
    test_passed "Tools API endpoint accessible ($tool_count tools available)"
else
    test_failed "Tools API endpoint inaccessible"
fi

# Navigation Tools Tests
echo ""
echo "--- Navigation Tools Tests ---"

# Setup test environment
execute_tool "navigate_to_url" '{"url":"https://example.com"}' "Navigate to example.com"
execute_tool "navigate_to_url" '{"url":"https://httpbin.org"}' "Navigate to httpbin.org"
execute_tool "go_back" '{}' "Go back navigation"
execute_tool "go_forward" '{}' "Go forward navigation"
execute_tool "refresh" '{}' "Page refresh"

# Test navigation with complex URLs
execute_tool "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate to form page"
execute_tool "navigate_to_url" '{"url":"https://github.com"}' "Navigate to GitHub"

# Interaction Tools Tests
echo ""
echo "--- Interaction Tools Tests ---"

# Setup for interaction tests
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Click tests
execute_tool "click" '{"selector":"input[name=\"custname\"]"}' "Click on customer name input"
execute_tool "click" '{"selector":"input[type=\"submit\"]"}' "Click on submit button"

# Type text tests
execute_tool "type_text" '{"selector":"input[name=\"custname\"]","text":"John Doe"}' "Type in customer name field"
execute_tool "type_text" '{"selector":"input[name=\"custel\"]","text":"555-1234"}' "Type in telephone field"
execute_tool "type_text" '{"selector":"input[name=\"custemail\"]","text":"john@example.com"}' "Type in email field"

# Hover tests
execute_tool "hover" '{"selector":"input[type=\"submit\"]"}' "Hover over submit button"
execute_tool "hover" '{"selector":"input[name=\"custname\"]"}' "Hover over name input"

# Focus tests
execute_tool "focus" '{"selector":"input[name=\"custname\"]"}' "Focus on customer name"
execute_tool "focus" '{"selector":"textarea[name=\"custmsg\"]"}' "Focus on message textarea"

# Select option tests - navigate to page with select elements
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Data Extraction Tools Tests
echo ""
echo "--- Data Extraction Tools Tests ---"

# Setup for extraction tests
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Extract text tests
execute_tool "extract_text" '{"selector":"h1"}' "Extract h1 text"
execute_tool "extract_text" '{"selector":"p"}' "Extract paragraph text"
execute_tool "extract_text" '{"selector":"body"}' "Extract body text"

# Extract links tests
execute_tool "extract_links" '{"selector":"a"}' "Extract all links"
execute_tool "extract_links" '{"selector":"a[href*=\"iana\"]"}' "Extract specific links"

# Extract data tests
execute_tool "extract_data" '{"selector":"*","attributes":["href","src","alt"]}' "Extract element attributes"

# Get element info tests
execute_tool "get_element_info" '{"selector":"h1"}' "Get h1 element info"
execute_tool "get_element_info" '{"selector":"body"}' "Get body element info"

# Utility & Wait Tools Tests
echo ""
echo "--- Utility & Wait Tools Tests ---"

# Wait for element tests
execute_tool "wait_for_element" '{"selector":"body","timeout":5000}' "Wait for body element"
execute_tool "wait_for_element" '{"selector":"h1","timeout":3000}' "Wait for h1 element"

# Wait for element that doesn't exist (should fail gracefully)
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"wait_for_element","parameters":{"selector":".non-existent","timeout":2000}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Wait timeout handling (non-existent element)"
    else
        test_failed "Wait should timeout for non-existent element"
    fi
else
    test_failed "Wait timeout response format"
fi

# Session storage tests
execute_tool "session_storage" '{"action":"set","key":"test_data","value":"sample_value"}' "Session storage set"
execute_tool "session_storage" '{"action":"get","key":"test_data"}' "Session storage get"
execute_tool "session_storage" '{"action":"clear"}' "Session storage clear"

# Persistent cache tests
execute_tool "persistent_cache" '{"action":"get"}' "Persistent cache get"
execute_tool "persistent_cache" '{"action":"set","key":"cache_test","value":"cached_data"}' "Persistent cache set"

# Advanced Interaction Tests
echo ""
echo "--- Advanced Interaction Tests ---"

# Navigate to GitHub for more complex interactions
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://github.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Search functionality tests
execute_tool "click" '{"selector":"input[name=\"q\"]"}' "Click GitHub search box"
execute_tool "type_text" '{"selector":"input[name=\"q\"]","text":"rust browser automation"}' "Type in GitHub search"

# Test complex selectors
execute_tool "hover" '{"selector":".Header-link"}' "Hover over header link"
execute_tool "get_element_info" '{"selector":".Header"}' "Get header element info"

# Error Handling Tests
echo ""
echo "--- Error Handling Tests ---"

# Test with invalid selectors
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

# Test with missing parameters
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"type_text","parameters":{"selector":"input"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Missing parameter error handling"
    else
        test_failed "Missing parameter should fail"
    fi
else
    test_failed "Missing parameter response format"
fi

# Test with non-existent tool
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"non_existent_tool","parameters":{}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Non-existent tool error handling"
    else
        test_failed "Non-existent tool should fail"
    fi
else
    test_failed "Non-existent tool response format"
fi

# Performance Tests
echo ""
echo "--- Tool Performance Tests ---"

# Test rapid successive tool calls
start_time=$(date +%s.%N)
for i in {1..5}; do
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"get_element_info","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > /dev/null
done
end_time=$(date +%s.%N)
total_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$total_time < 15.0" | bc -l) )); then
    test_passed "Rapid tool execution performance (${total_time}s for 5 calls)"
else
    test_failed "Tool execution too slow (${total_time}s for 5 calls)"
fi

# Concurrent tool execution test
echo ""
echo "--- Concurrent Execution Tests ---"
temp_files=()
for i in {1..3}; do
    temp_file="/tmp/tool_test_$i.json"
    temp_files+=("$temp_file")
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > "$temp_file" &
done

wait

# Check all concurrent responses
all_good=true
for temp_file in "${temp_files[@]}"; do
    if ! jq -e '.success' "$temp_file" > /dev/null 2>&1; then
        all_good=false
        break
    fi
    rm -f "$temp_file"
done

if [ "$all_good" = true ]; then
    test_passed "Concurrent tool execution handling"
else
    test_failed "Concurrent tool execution handling"
fi

# Tool Chain Tests
echo ""
echo "--- Tool Chain Tests ---"

# Test complex workflow
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

execute_tool "click" '{"selector":"input[name=\"custname\"]"}' "Chain step 1: Click name field"
execute_tool "type_text" '{"selector":"input[name=\"custname\"]","text":"Test User"}' "Chain step 2: Type name"
execute_tool "focus" '{"selector":"input[name=\"custel\"]"}' "Chain step 3: Focus phone field"
execute_tool "type_text" '{"selector":"input[name=\"custel\"]","text":"555-0123"}' "Chain step 4: Type phone"
execute_tool "extract_text" '{"selector":"input[name=\"custname\"]"}' "Chain step 5: Extract entered name"

# Summary
echo ""
echo "========================================="
echo "          TOOLS TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All tools tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED tools test(s) failed"
    exit 1
fi