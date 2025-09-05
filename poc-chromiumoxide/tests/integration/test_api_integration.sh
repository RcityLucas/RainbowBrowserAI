#!/bin/bash

# API Integration Test Suite
# Tests integration between API endpoints, browser automation, and perception systems

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

test_api_integration() {
    local endpoint="$1"
    local method="$2"
    local data="$3"
    local test_name="$4"
    local expected_field="$5"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s "$SERVER_URL$endpoint")
    else
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$data" "$SERVER_URL$endpoint")
    fi
    
    if echo "$response" | jq -e "$expected_field" > /dev/null 2>&1; then
        test_passed "$test_name"
        return 0
    else
        test_failed "$test_name"
        echo "  Response: $response"
        return 1
    fi
}

echo "========================================="
echo "     API Integration Test Suite"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Basic API Connectivity Tests
echo "--- API Connectivity Tests ---"
test_api_integration "/api/health" "GET" "" "Health API connectivity" ".status"
test_api_integration "/api/tools" "GET" "" "Tools API connectivity" ".tools"

# Tools and Browser Integration
echo ""
echo "--- Tools-Browser Integration Tests ---"

# Test navigation integration
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "true" ]; then
        test_passed "Navigation tool integration"
        
        # Follow up with extraction to verify browser state
        extract_response=$(curl -s -X POST -H "Content-Type: application/json" \
            -d '{"tool_name":"extract_text","parameters":{"selector":"h1"}}' \
            "$SERVER_URL/api/tools/execute")
        
        if echo "$extract_response" | jq -e '.data.text' > /dev/null 2>&1; then
            test_passed "Post-navigation extraction integration"
        else
            test_failed "Post-navigation extraction integration"
        fi
    else
        test_failed "Navigation tool integration"
    fi
else
    test_failed "Navigation tool integration (invalid response)"
fi

# Test interaction tool chain integration
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

# Chain: Click -> Type -> Extract
click_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"click","parameters":{"selector":"input[name=\"custname\"]"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$click_response" | jq -e '.success' > /dev/null 2>&1; then
    type_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"type_text","parameters":{"selector":"input[name=\"custname\"]","text":"Integration Test"}}' \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$type_response" | jq -e '.success' > /dev/null 2>&1; then
        verify_response=$(curl -s -X POST -H "Content-Type: application/json" \
            -d '{"tool_name":"extract_text","parameters":{"selector":"input[name=\"custname\"]"}}' \
            "$SERVER_URL/api/tools/execute")
        
        if echo "$verify_response" | jq -e '.data.text' > /dev/null 2>&1; then
            extracted_text=$(echo "$verify_response" | jq -r '.data.text')
            if [[ "$extracted_text" == *"Integration Test"* ]]; then
                test_passed "Tool chain integration (Click → Type → Extract)"
            else
                test_failed "Tool chain integration - text mismatch"
            fi
        else
            test_failed "Tool chain integration - extraction failed"
        fi
    else
        test_failed "Tool chain integration - typing failed"
    fi
else
    test_failed "Tool chain integration - click failed"
fi

# Perception-API Integration Tests
echo ""
echo "--- Perception-API Integration Tests ---"

# Test perception with browser state
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

perception_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{}' \
    "$SERVER_URL/api/perception/analyze")

if echo "$perception_response" | jq -e '.data.url' > /dev/null 2>&1; then
    perception_url=$(echo "$perception_response" | jq -r '.data.url')
    if [[ "$perception_url" == *"example.com"* ]]; then
        test_passed "Perception-browser state integration"
    else
        test_failed "Perception-browser state mismatch"
    fi
else
    test_failed "Perception-browser integration"
fi

# Test perception element finding with browser
find_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"description":"heading"}' \
    "$SERVER_URL/api/perception/find")

if echo "$find_response" | jq -e '.data.selector' > /dev/null 2>&1; then
    selector=$(echo "$find_response" | jq -r '.data.selector')
    
    # Use the found selector with tools API
    tool_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"extract_text\",\"parameters\":{\"selector\":\"$selector\"}}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$tool_response" | jq -e '.data.text' > /dev/null 2>&1; then
        test_passed "Perception-to-tools selector integration"
    else
        test_failed "Perception-to-tools selector integration"
    fi
else
    test_failed "Perception element finding integration"
fi

# Cross-API Data Flow Tests
echo ""
echo "--- Cross-API Data Flow Tests ---"

# Test session data flow between APIs
session_set_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"session_storage","parameters":{"action":"set","key":"integration_test","value":"api_flow_test"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$session_set_response" | jq -e '.success' > /dev/null 2>&1; then
    session_get_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"session_storage","parameters":{"action":"get","key":"integration_test"}}' \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$session_get_response" | jq -e '.data.value' > /dev/null 2>&1; then
        stored_value=$(echo "$session_get_response" | jq -r '.data.value')
        if [ "$stored_value" = "api_flow_test" ]; then
            test_passed "Session storage API data flow"
        else
            test_failed "Session storage data mismatch"
        fi
    else
        test_failed "Session storage retrieval"
    fi
else
    test_failed "Session storage integration"
fi

# Error Propagation Integration Tests
echo ""
echo "--- Error Propagation Integration Tests ---"

# Test error handling across API layers
invalid_nav_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"invalid-url"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$invalid_nav_response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$invalid_nav_response" | jq -r '.success')
    if [ "$success" = "false" ] && echo "$invalid_nav_response" | jq -e '.error' > /dev/null 2>&1; then
        test_passed "Error propagation through API layers"
    else
        test_failed "Error propagation not working"
    fi
else
    test_failed "Error response format"
fi

# Test cascade error handling
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

invalid_extract_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"extract_text","parameters":{"selector":"invalid>>selector"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$invalid_extract_response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$invalid_extract_response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        # Test that system recovers
        recovery_response=$(curl -s -X POST -H "Content-Type: application/json" \
            -d '{"tool_name":"extract_text","parameters":{"selector":"h1"}}' \
            "$SERVER_URL/api/tools/execute")
        
        if echo "$recovery_response" | jq -e '.success' > /dev/null 2>&1; then
            recovery_success=$(echo "$recovery_response" | jq -r '.success')
            if [ "$recovery_success" = "true" ]; then
                test_passed "System recovery after API errors"
            else
                test_failed "System recovery after errors"
            fi
        else
            test_failed "System recovery response format"
        fi
    else
        test_failed "Invalid selector should fail"
    fi
else
    test_failed "Invalid extraction response format"
fi

# Concurrent API Integration Tests
echo ""
echo "--- Concurrent API Integration Tests ---"

# Test concurrent API calls
temp_files=()
for i in {1..3}; do
    temp_file="/tmp/api_integration_$i.json"
    temp_files+=("$temp_file")
    
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > "$temp_file" &
done

wait

# Check all concurrent responses
all_successful=true
for temp_file in "${temp_files[@]}"; do
    if ! echo "$(cat "$temp_file")" | jq -e '.success' > /dev/null 2>&1; then
        all_successful=false
        break
    fi
    rm -f "$temp_file"
done

if [ "$all_successful" = true ]; then
    test_passed "Concurrent API request integration"
else
    test_failed "Concurrent API request integration"
fi

# API Response Format Consistency Tests
echo ""
echo "--- API Response Format Consistency Tests ---"

# Test that all APIs return consistent response format
apis_to_test=(
    "POST:/api/tools/execute:{\"tool_name\":\"extract_text\",\"parameters\":{\"selector\":\"body\"}}"
    "POST:/api/perception/analyze:{}"
    "POST:/api/perception/find:{\"description\":\"link\"}"
    "GET:/api/health:"
    "GET:/api/tools:"
)

for api_test in "${apis_to_test[@]}"; do
    IFS=':' read -r method endpoint data <<< "$api_test"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s "$SERVER_URL$endpoint")
    else
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$data" "$SERVER_URL$endpoint")
    fi
    
    # Check for consistent response structure
    if echo "$response" | jq -e 'type == "object"' > /dev/null 2>&1; then
        if [[ "$endpoint" == *"/health"* ]] || [[ "$endpoint" == *"/tools"* ]]; then
            # Health and tools have different structure
            test_passed "Response format consistency for $endpoint"
        else
            # Other APIs should have success field
            if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
                test_passed "Response format consistency for $endpoint"
            else
                test_failed "Response format consistency for $endpoint (missing success field)"
            fi
        fi
    else
        test_failed "Response format consistency for $endpoint (not JSON object)"
    fi
done

# API Version and Compatibility Tests
echo ""
echo "--- API Compatibility Tests ---"

# Test API endpoint availability
required_endpoints=(
    "/api/health"
    "/api/tools"
    "/api/tools/execute"
    "/api/perception/analyze"
    "/api/perception/find"
    "/api/perception/command"
    "/api/perception/forms/analyze"
)

for endpoint in "${required_endpoints[@]}"; do
    if [[ "$endpoint" == *"/execute"* ]] || [[ "$endpoint" == *"/analyze"* ]] || [[ "$endpoint" == *"/find"* ]] || [[ "$endpoint" == *"/command"* ]]; then
        # POST endpoints
        status_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Content-Type: application/json" -d '{}' "$SERVER_URL$endpoint")
    else
        # GET endpoints
        status_code=$(curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL$endpoint")
    fi
    
    if [ "$status_code" != "404" ]; then
        test_passed "API endpoint availability: $endpoint"
    else
        test_failed "API endpoint not found: $endpoint"
    fi
done

# Summary
echo ""
echo "========================================="
echo "    API INTEGRATION TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All API integration tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED API integration test(s) failed"
    exit 1
fi