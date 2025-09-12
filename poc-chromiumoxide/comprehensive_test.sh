#!/bin/bash

# Comprehensive test script for RainbowBrowserAI poc-chromiumoxide
echo "=== RainbowBrowserAI Comprehensive Test Suite ==="
echo

BASE_URL="http://localhost:3002"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0

# Function to test an endpoint
test_endpoint() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_field="$5"
    
    echo -n "Testing $name... "
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s "$BASE_URL$endpoint")
    else
        response=$(curl -s -X "$method" "$BASE_URL$endpoint" -H "Content-Type: application/json" -d "$data")
    fi
    
    if [ -n "$expected_field" ]; then
        if echo "$response" | grep -q "$expected_field"; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((PASSED++))
        else
            echo -e "${RED}✗ FAILED${NC}"
            echo "  Response: $response"
            ((FAILED++))
        fi
    else
        if [ -n "$response" ]; then
            echo -e "${GREEN}✓ PASSED${NC}"
            ((PASSED++))
        else
            echo -e "${RED}✗ FAILED${NC}"
            ((FAILED++))
        fi
    fi
}

echo "1. Testing Basic Endpoints"
echo "=========================="
test_endpoint "Health Check" "GET" "/api/health" "" "healthy"

echo
echo "2. Testing Navigation Tools"
echo "==========================="
test_endpoint "Navigate to URL" "POST" "/api/tools/execute" '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' "success"
test_endpoint "Refresh Page" "POST" "/api/tools/execute" '{"tool_name":"refresh_page","parameters":{}}' "success"
test_endpoint "Scroll" "POST" "/api/tools/execute" '{"tool_name":"scroll_page","parameters":{"x":0,"y":100}}' "success"

echo
echo "3. Testing Interaction Tools"
echo "============================"
test_endpoint "Click Element" "POST" "/api/tools/execute" '{"tool_name":"click","parameters":{"selector":"body"}}' "success"
test_endpoint "Type Text" "POST" "/api/tools/execute" '{"tool_name":"type_text","parameters":{"selector":"body","text":"test"}}' "success"

echo
echo "4. Testing Data Extraction Tools"
echo "================================"
test_endpoint "Extract Text" "POST" "/api/tools/execute" '{"tool_name":"extract_text","parameters":{"selector":"body"}}' "success"
test_endpoint "Extract Links" "POST" "/api/tools/execute" '{"tool_name":"extract_links","parameters":{"selector":"a"}}' "success"
test_endpoint "Extract Data" "POST" "/api/tools/execute" '{"tool_name":"extract_data","parameters":{"selector":"body","attributes":["id","class"]}}' "success"

echo
echo "5. Testing Memory Tools"
echo "======================"
test_endpoint "Screenshot" "POST" "/api/tools/execute" '{"tool_name":"screenshot","parameters":{}}' "data_base64"
test_endpoint "Session Memory" "POST" "/api/tools/execute" '{"tool_name":"session_memory","parameters":{"action":"Store","key":"test","value":"data"}}' "success"
test_endpoint "Get Element Info" "POST" "/api/tools/execute" '{"tool_name":"get_element_info","parameters":{"selector":"body"}}' "success"

echo
echo "6. Testing Perception API"
echo "========================"
test_endpoint "Analyze Page (Quick)" "POST" "/api/perception/analyze" '{"url":"https://example.com","mode":"quick"}' "layered_result"
test_endpoint "Analyze Page (Deep)" "POST" "/api/perception/analyze" '{"url":"https://example.com","mode":"deep"}' "layered_result"
test_endpoint "Smart Search" "POST" "/api/perception/smart_search" '{"query":"button","session_id":"test"}' "success"
test_endpoint "Find Element" "POST" "/api/perception/find_element" '{"description":"heading","session_id":"test"}' "success"

echo
echo "7. Testing Synchronization Tools"
echo "================================"
test_endpoint "Wait for Element" "POST" "/api/tools/execute" '{"tool_name":"wait_for_element","parameters":{"selector":"body","timeout_ms":1000}}' "success"

echo
echo "======================================="
echo "Test Results Summary:"
echo "======================================="
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please review the output above.${NC}"
    exit 1
fi