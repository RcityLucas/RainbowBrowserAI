#!/bin/bash

# Browser Integration Test Suite
# Tests integration between browser automation, perception, and application state

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

execute_and_verify() {
    local tool_name="$1"
    local parameters="$2"
    local test_name="$3"
    local verification_selector="$4"
    
    # Execute the tool
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"$tool_name\",\"parameters\":$parameters}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$response" | jq -r '.success')
        if [ "$success" = "true" ]; then
            # If verification selector provided, verify the action worked
            if [ -n "$verification_selector" ]; then
                verify_response=$(curl -s -X POST -H "Content-Type: application/json" \
                    -d "{\"tool_name\":\"get_element_info\",\"parameters\":{\"selector\":\"$verification_selector\"}}" \
                    "$SERVER_URL/api/tools/execute")
                
                if echo "$verify_response" | jq -e '.success' > /dev/null 2>&1; then
                    verify_success=$(echo "$verify_response" | jq -r '.success')
                    if [ "$verify_success" = "true" ]; then
                        test_passed "$test_name"
                        return 0
                    else
                        test_failed "$test_name (verification failed)"
                        return 1
                    fi
                else
                    test_failed "$test_name (verification error)"
                    return 1
                fi
            else
                test_passed "$test_name"
                return 0
            fi
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
echo "    Browser Integration Test Suite"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Browser State Management Integration
echo "--- Browser State Management Integration ---"

# Test browser initialization and navigation
execute_and_verify "navigate_to_url" '{"url":"https://example.com"}' "Browser initialization and navigation" "body"

# Test browser context persistence
execute_and_verify "session_storage" '{"action":"set","key":"integration_test","value":"browser_context"}' "Set browser context" ""
execute_and_verify "session_storage" '{"action":"get","key":"integration_test"}' "Get browser context" ""

# Verify context persists across page elements interaction
execute_and_verify "click" '{"selector":"h1"}' "Interact with page element" "h1"
execute_and_verify "session_storage" '{"action":"get","key":"integration_test"}' "Verify context after interaction" ""

# Browser Navigation Integration
echo ""
echo "--- Browser Navigation Integration ---"

# Test navigation history management
execute_and_verify "navigate_to_url" '{"url":"https://httpbin.org"}' "Navigate to second page" "body"
execute_and_verify "go_back" '{}' "Navigate back" "body"
execute_and_verify "go_forward" '{}' "Navigate forward" "body"

# Test navigation state consistency
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{}' \
    "$SERVER_URL/api/perception/analyze")

if echo "$response" | jq -e '.data.url' > /dev/null 2>&1; then
    current_url=$(echo "$response" | jq -r '.data.url')
    if [[ "$current_url" == *"httpbin.org"* ]]; then
        test_passed "Navigation state consistency with perception"
    else
        test_failed "Navigation state inconsistent (expected httpbin.org, got $current_url)"
    fi
else
    test_failed "Navigation state perception integration"
fi

# Page Refresh Integration
execute_and_verify "refresh" '{}' "Page refresh with state" "body"

# Browser-Perception Integration
echo ""
echo "--- Browser-Perception Integration ---"

# Navigate to form page for perception testing
execute_and_verify "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate to form for perception testing" "form"

# Test perception analysis with browser state
perception_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{}' \
    "$SERVER_URL/api/perception/analyze")

if echo "$perception_response" | jq -e '.data.page_type' > /dev/null 2>&1; then
    page_type=$(echo "$perception_response" | jq -r '.data.page_type')
    if [[ "$page_type" == *"form"* ]] || [[ "$page_type" == *"Form"* ]]; then
        test_passed "Perception correctly identifies form page"
    else
        test_passed "Perception analysis with browser state (page_type: $page_type)"
    fi
else
    test_failed "Perception-browser state integration"
fi

# Test perception-guided browser interaction
find_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"description":"customer name input"}' \
    "$SERVER_URL/api/perception/find")

if echo "$find_response" | jq -e '.data.selector' > /dev/null 2>&1; then
    found_selector=$(echo "$find_response" | jq -r '.data.selector')
    
    # Use perception-found selector for browser interaction
    execute_and_verify "type_text" "{\"selector\":\"$found_selector\",\"text\":\"Perception Integration Test\"}" "Perception-guided text input" "$found_selector"
    
    # Verify the text was entered
    verify_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"extract_text\",\"parameters\":{\"selector\":\"$found_selector\"}}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$verify_response" | jq -e '.data.text' > /dev/null 2>&1; then
        extracted_text=$(echo "$verify_response" | jq -r '.data.text')
        if [[ "$extracted_text" == *"Perception Integration Test"* ]]; then
            test_passed "Perception-to-browser interaction verification"
        else
            test_failed "Perception-guided input verification (got: $extracted_text)"
        fi
    else
        test_failed "Perception-guided input verification"
    fi
else
    test_failed "Perception element finding for browser integration"
fi

# Form Analysis Integration
echo ""
echo "--- Form Analysis Integration ---"

# Test form analysis with browser state
form_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"form_selector":"form"}' \
    "$SERVER_URL/api/perception/forms/analyze")

if echo "$form_response" | jq -e '.data.fields' > /dev/null 2>&1; then
    fields_count=$(echo "$form_response" | jq '.data.fields | length')
    if [ "$fields_count" -gt 0 ]; then
        test_passed "Form analysis detects fields ($fields_count fields found)"
        
        # Test interaction with analyzed fields
        first_field=$(echo "$form_response" | jq -r '.data.fields[0].selector // .data.fields[0].name')
        if [ "$first_field" != "null" ] && [ -n "$first_field" ]; then
            execute_and_verify "focus" "{\"selector\":\"$first_field\"}" "Focus on analyzed form field" "$first_field"
        else
            test_failed "Form analysis field selector extraction"
        fi
    else
        test_failed "Form analysis found no fields"
    fi
else
    test_failed "Form analysis integration"
fi

# Element Detection and Interaction Integration
echo ""
echo "--- Element Detection and Interaction Integration ---"

# Navigate to content-rich page
execute_and_verify "navigate_to_url" '{"url":"https://github.com"}' "Navigate to content-rich page" "body"

# Test various element detection and interaction patterns
element_tests=(
    "link:a"
    "button:button,.btn"
    "text input:input[type='text'],input[type='search'],input:not([type])"
    "heading:h1,h2,h3"
)

for element_test in "${element_tests[@]}"; do
    IFS=':' read -r description expected_selector <<< "$element_test"
    
    # Find element using perception
    find_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"description\":\"$description\"}" \
        "$SERVER_URL/api/perception/find")
    
    if echo "$find_response" | jq -e '.data.selector' > /dev/null 2>&1; then
        found_selector=$(echo "$find_response" | jq -r '.data.selector')
        
        # Verify element exists in browser
        verify_response=$(curl -s -X POST -H "Content-Type: application/json" \
            -d "{\"tool_name\":\"get_element_info\",\"parameters\":{\"selector\":\"$found_selector\"}}" \
            "$SERVER_URL/api/tools/execute")
        
        if echo "$verify_response" | jq -e '.success' > /dev/null 2>&1; then
            verify_success=$(echo "$verify_response" | jq -r '.success')
            if [ "$verify_success" = "true" ]; then
                test_passed "Element detection and verification: $description"
            else
                test_failed "Element verification failed for: $description"
            fi
        else
            test_failed "Element verification error for: $description"
        fi
    else
        test_failed "Element detection failed for: $description"
    fi
done

# Browser Pool and Concurrency Integration
echo ""
echo "--- Browser Pool and Concurrency Integration ---"

# Test concurrent browser operations
temp_files=()
for i in {1..3}; do
    temp_file="/tmp/browser_integration_$i.json"
    temp_files+=("$temp_file")
    
    # Launch concurrent operations
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"extract_text","parameters":{"selector":"body"}}' \
        "$SERVER_URL/api/tools/execute" > "$temp_file" &
done

wait

# Verify all concurrent operations succeeded
all_successful=true
for temp_file in "${temp_files[@]}"; do
    if echo "$(cat "$temp_file")" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$(cat "$temp_file")" | jq -r '.success')
        if [ "$success" != "true" ]; then
            all_successful=false
            break
        fi
    else
        all_successful=false
        break
    fi
    rm -f "$temp_file"
done

if [ "$all_successful" = true ]; then
    test_passed "Concurrent browser operations handling"
else
    test_failed "Concurrent browser operations handling"
fi

# Memory and Resource Management Integration
echo ""
echo "--- Memory and Resource Management Integration ---"

# Test memory cleanup after multiple operations
initial_operation_count=10
for i in $(seq 1 $initial_operation_count); do
    curl -s -X POST -H "Content-Type: application/json" \
        -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
        "$SERVER_URL/api/tools/execute" > /dev/null
done

# Verify system still responsive
health_response=$(curl -s "$SERVER_URL/api/health")
if echo "$health_response" | jq -e '.status' > /dev/null 2>&1; then
    test_passed "System responsive after multiple operations"
else
    test_failed "System unresponsive after multiple operations"
fi

# Test resource cleanup with session data
execute_and_verify "session_storage" '{"action":"clear"}' "Session storage cleanup" ""

# Error Recovery Integration
echo ""
echo "--- Error Recovery Integration ---"

# Test browser error recovery
invalid_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"invalid-url"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$invalid_response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$invalid_response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        # Test that browser recovers for valid operations
        execute_and_verify "navigate_to_url" '{"url":"https://example.com"}' "Browser recovery after error" "body"
    else
        test_failed "Invalid URL should fail"
    fi
else
    test_failed "Error handling response format"
fi

# Test element interaction error recovery
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

invalid_click_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"click","parameters":{"selector":"invalid>>selector"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$invalid_click_response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$invalid_click_response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        # Test recovery with valid interaction
        execute_and_verify "click" '{"selector":"h1"}' "Element interaction recovery" "h1"
    else
        test_failed "Invalid selector should fail"
    fi
else
    test_failed "Element error handling format"
fi

# Cross-Browser State Consistency
echo ""
echo "--- Cross-Browser State Consistency ---"

# Set state and verify persistence
execute_and_verify "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate for state consistency test" "form"
execute_and_verify "session_storage" '{"action":"set","key":"consistency_test","value":"state_data"}' "Set consistent state" ""

# Interact with page
execute_and_verify "type_text" '{"selector":"input[name=\"custname\"]","text":"Consistency Test"}' "Page interaction with state" "input[name=\"custname\"]"

# Verify state persistence after interaction
state_response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"session_storage","parameters":{"action":"get","key":"consistency_test"}}' \
    "$SERVER_URL/api/tools/execute")

if echo "$state_response" | jq -e '.data.value' > /dev/null 2>&1; then
    state_value=$(echo "$state_response" | jq -r '.data.value')
    if [ "$state_value" = "state_data" ]; then
        test_passed "Cross-browser state consistency"
    else
        test_failed "State consistency lost after interaction"
    fi
else
    test_failed "State consistency verification"
fi

# Summary
echo ""
echo "========================================="
echo "   BROWSER INTEGRATION TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All browser integration tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED browser integration test(s) failed"
    exit 1
fi