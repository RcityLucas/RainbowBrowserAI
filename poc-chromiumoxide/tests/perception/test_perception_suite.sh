#!/bin/bash

# Perception Module Tests
# Tests AI perception capabilities, natural language processing, and intelligent automation

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

test_perception_endpoint() {
    local endpoint="$1"
    local data="$2"
    local test_name="$3"
    local expected_field="$4"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$SERVER_URL$endpoint")
    status_code=$(echo "$response" | jq -r '.success // false')
    
    if [ "$status_code" = "true" ] && echo "$response" | jq -e ".$expected_field" > /dev/null 2>&1; then
        test_passed "$test_name"
        return 0
    else
        test_failed "$test_name"
        echo "  Response: $response"
        return 1
    fi
}

echo "========================================="
echo "     Perception Module Tests"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Setup - Navigate to test page
echo "--- Test Setup ---"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

if [ $? -eq 0 ]; then
    test_passed "Navigation to test page (example.com)"
else
    test_failed "Navigation to test page"
fi

# Page Analysis Tests
echo ""
echo "--- Page Analysis Tests ---"

# Basic page analysis
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{}' "$SERVER_URL/api/perception/analyze")
if echo "$response" | jq -e '.data.url' > /dev/null 2>&1; then
    test_passed "Page analysis returns URL"
else
    test_failed "Page analysis missing URL"
fi

if echo "$response" | jq -e '.data.title' > /dev/null 2>&1; then
    test_passed "Page analysis returns title"
else
    test_failed "Page analysis missing title"
fi

if echo "$response" | jq -e '.data.page_type' > /dev/null 2>&1; then
    test_passed "Page analysis returns page type"
else
    test_failed "Page analysis missing page type"
fi

if echo "$response" | jq -e '.data.timestamp' > /dev/null 2>&1; then
    test_passed "Page analysis includes timestamp"
else
    test_failed "Page analysis missing timestamp"
fi

# Element Detection Tests
echo ""
echo "--- Element Detection Tests ---"

# Test finding common elements
test_cases=(
    '{"description":"link"}:element found by description (link)'
    '{"description":"heading"}:element found by description (heading)'
    '{"description":"paragraph"}:element found by description (paragraph)'
    '{"description":"body"}:element found by description (body)'
)

for test_case in "${test_cases[@]}"; do
    IFS=':' read -r data description <<< "$test_case"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$SERVER_URL/api/perception/find")
    if echo "$response" | jq -e '.data.selector' > /dev/null 2>&1; then
        test_passed "$description"
    else
        test_failed "$description"
    fi
done

# Test element detection with confidence scoring
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"description":"link"}' "$SERVER_URL/api/perception/find")
if echo "$response" | jq -e '.data.confidence' > /dev/null 2>&1; then
    confidence=$(echo "$response" | jq -r '.data.confidence')
    if (( $(echo "$confidence >= 0.0" | bc -l) )) && (( $(echo "$confidence <= 1.0" | bc -l) )); then
        test_passed "Element detection includes valid confidence score ($confidence)"
    else
        test_failed "Element detection confidence score out of range ($confidence)"
    fi
else
    test_failed "Element detection missing confidence score"
fi

# Intelligent Command Tests  
echo ""
echo "--- Intelligent Command Tests ---"

# Test basic intelligent commands
intelligent_commands=(
    '{"command":{"action":"click","description":"first link","parameters":{}}}:intelligent click command'
    '{"command":{"action":"extract","description":"page title","parameters":{}}}:intelligent extraction command'
    '{"command":{"action":"analyze","description":"page content","parameters":{}}}:intelligent analysis command'
)

for cmd in "${intelligent_commands[@]}"; do
    IFS=':' read -r data description <<< "$cmd"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$SERVER_URL/api/perception/command")
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$response" | jq -r '.success')
        if [ "$success" = "true" ]; then
            test_passed "$description"
        else
            test_failed "$description (success: $success)"
        fi
    else
        test_failed "$description (invalid response format)"
    fi
done

# Form Analysis Tests
echo ""
echo "--- Form Analysis Tests ---"

# Test form analysis on current page
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"form_selector":null}' "$SERVER_URL/api/perception/forms/analyze")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "true" ] || [ "$success" = "false" ]; then
        test_passed "Form analysis endpoint responding"
        
        # Check response structure
        if echo "$response" | jq -e '.data.form_type' > /dev/null 2>&1; then
            test_passed "Form analysis includes form type"
        else
            test_failed "Form analysis missing form type"
        fi
        
        if echo "$response" | jq -e '.data.fields' > /dev/null 2>&1; then
            test_passed "Form analysis includes fields array"
        else
            test_failed "Form analysis missing fields array"
        fi
    else
        test_failed "Form analysis invalid success value"
    fi
else
    test_failed "Form analysis invalid response format"
fi

# Navigate to a page with forms for better testing
echo ""
echo "--- Form Analysis with Form Page ---"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://httpbin.org/forms/post"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

if [ $? -eq 0 ]; then
    test_passed "Navigation to form test page"
    
    # Test form analysis on page with actual forms
    response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"form_selector":"form"}' "$SERVER_URL/api/perception/forms/analyze")
    if echo "$response" | jq -e '.data.fields[] | select(.field_type != null)' > /dev/null 2>&1; then
        test_passed "Form analysis detects form fields"
    else
        test_failed "Form analysis missing field detection"
    fi
else
    test_failed "Navigation to form test page"
fi

# Natural Language Processing Tests
echo ""
echo "--- Natural Language Processing Tests ---"

# Test various natural language descriptions
nl_tests=(
    "button:button element recognition"
    "search box:search input recognition"
    "submit button:submit button recognition"
    "first link:positional element recognition"
    "main heading:semantic element recognition"
)

for nl_test in "${nl_tests[@]}"; do
    IFS=':' read -r description test_name <<< "$nl_test"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"description\":\"$description\"}" \
        "$SERVER_URL/api/perception/find")
        
    if echo "$response" | jq -e '.data.selector' > /dev/null 2>&1; then
        test_passed "$test_name"
    else
        test_failed "$test_name"
    fi
done

# Error Handling Tests
echo ""
echo "--- Error Handling Tests ---"

# Test with invalid/non-existent elements
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"description":"non-existent-super-unique-element"}' \
    "$SERVER_URL/api/perception/find")
    
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        test_passed "Perception handles non-existent elements gracefully"
    else
        test_failed "Perception should fail gracefully for non-existent elements"
    fi
else
    test_failed "Perception error handling response format"
fi

# Performance Tests
echo ""
echo "--- Performance Tests ---"

# Test response times
start_time=$(date +%s.%N)
curl -s -X POST -H "Content-Type: application/json" -d '{"description":"link"}' "$SERVER_URL/api/perception/find" > /dev/null
end_time=$(date +%s.%N)
response_time=$(echo "$end_time - $start_time" | bc -l)

if (( $(echo "$response_time < 5.0" | bc -l) )); then
    test_passed "Perception response time acceptable (${response_time}s)"
else
    test_failed "Perception response time too slow (${response_time}s)"
fi

# Concurrent Requests Test
echo ""
echo "--- Concurrent Processing Tests ---"
temp_files=()
for i in {1..3}; do
    temp_file="/tmp/perception_test_$i.json"
    temp_files+=("$temp_file")
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"description\":\"test element $i\"}" \
        "$SERVER_URL/api/perception/find" > "$temp_file" &
done

wait

# Check all responses
all_good=true
for temp_file in "${temp_files[@]}"; do
    if ! jq -e '.success' "$temp_file" > /dev/null 2>&1; then
        all_good=false
        break
    fi
    rm -f "$temp_file"
done

if [ "$all_good" = true ]; then
    test_passed "Perception handles concurrent requests"
else
    test_failed "Perception concurrent request handling"
fi

# Integration Tests
echo ""
echo "--- Integration Tests ---"

# Test perception + tools integration
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}' \
    "$SERVER_URL/api/tools/execute" > /dev/null

response=$(curl -s -X POST -H "Content-Type: application/json" -d '{}' "$SERVER_URL/api/perception/analyze")
if echo "$response" | jq -e '.data.url' > /dev/null 2>&1; then
    url=$(echo "$response" | jq -r '.data.url')
    if [[ "$url" == *"example.com"* ]]; then
        test_passed "Perception integrates with navigation tools"
    else
        test_failed "Perception navigation integration (got URL: $url)"
    fi
else
    test_failed "Perception navigation integration (no URL returned)"
fi

# Summary
echo ""
echo "========================================="
echo "        PERCEPTION TEST SUMMARY"
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All perception tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED perception test(s) failed"
    exit 1
fi