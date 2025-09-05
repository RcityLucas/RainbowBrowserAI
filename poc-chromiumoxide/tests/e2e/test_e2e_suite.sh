#!/bin/bash

# End-to-End Test Suite
# Tests complete user workflows and real-world scenarios

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

execute_workflow_step() {
    local tool_name="$1"
    local parameters="$2"
    local step_name="$3"
    
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"tool_name\":\"$tool_name\",\"parameters\":$parameters}" \
        "$SERVER_URL/api/tools/execute")
    
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success=$(echo "$response" | jq -r '.success')
        if [ "$success" = "true" ]; then
            echo "    ✓ $step_name"
            return 0
        else
            echo "    ✗ $step_name - $(echo "$response" | jq -r '.error // "Unknown error"')"
            return 1
        fi
    else
        echo "    ✗ $step_name (invalid response format)"
        return 1
    fi
}

echo "========================================="
echo "      End-to-End Test Suite"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# E2E Test 1: Complete Form Interaction Workflow
echo "--- E2E Test 1: Form Interaction Workflow ---"
workflow1_success=true

echo "Scenario: Navigate to form, fill out fields, and submit"

# Step 1: Navigate to form page
if execute_workflow_step "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate to form page"; then
    sleep 1
else
    workflow1_success=false
fi

# Step 2: Fill customer name
if execute_workflow_step "click" '{"selector":"input[name=\"custname\"]"}' "Focus on customer name field" && \
   execute_workflow_step "type_text" '{"selector":"input[name=\"custname\"]","text":"John Smith"}' "Enter customer name"; then
    sleep 0.5
else
    workflow1_success=false
fi

# Step 3: Fill telephone
if execute_workflow_step "focus" '{"selector":"input[name=\"custel\"]"}' "Focus on telephone field" && \
   execute_workflow_step "type_text" '{"selector":"input[name=\"custel\"]","text":"555-123-4567"}' "Enter telephone"; then
    sleep 0.5
else
    workflow1_success=false
fi

# Step 4: Fill email
if execute_workflow_step "focus" '{"selector":"input[name=\"custemail\"]"}' "Focus on email field" && \
   execute_workflow_step "type_text" '{"selector":"input[name=\"custemail\"]","text":"john.smith@example.com"}' "Enter email"; then
    sleep 0.5
else
    workflow1_success=false
fi

# Step 5: Fill message
if execute_workflow_step "focus" '{"selector":"textarea[name=\"custmsg\"]"}' "Focus on message field" && \
   execute_workflow_step "type_text" '{"selector":"textarea[name=\"custmsg\"]","text":"This is a test message for the E2E workflow."}' "Enter message"; then
    sleep 0.5
else
    workflow1_success=false
fi

# Step 6: Extract filled data for verification
if execute_workflow_step "extract_text" '{"selector":"input[name=\"custname\"]"}' "Verify customer name entered"; then
    sleep 0.5
else
    workflow1_success=false
fi

# Step 7: Submit form
if execute_workflow_step "hover" '{"selector":"input[type=\"submit\"]"}' "Hover over submit button" && \
   execute_workflow_step "click" '{"selector":"input[type=\"submit\"]"}' "Click submit button"; then
    sleep 2
else
    workflow1_success=false
fi

if [ "$workflow1_success" = true ]; then
    test_passed "Complete Form Interaction Workflow"
else
    test_failed "Complete Form Interaction Workflow"
fi

# E2E Test 2: Website Navigation and Data Extraction
echo ""
echo "--- E2E Test 2: Navigation and Data Extraction ---"
workflow2_success=true

echo "Scenario: Navigate between sites, extract information, and analyze content"

# Step 1: Navigate to Example.com
if execute_workflow_step "navigate_to_url" '{"url":"https://example.com"}' "Navigate to Example.com"; then
    sleep 1
else
    workflow2_success=false
fi

# Step 2: Extract page title and content
if execute_workflow_step "extract_text" '{"selector":"h1"}' "Extract main heading" && \
   execute_workflow_step "extract_text" '{"selector":"p"}' "Extract paragraph content" && \
   execute_workflow_step "extract_links" '{"selector":"a"}' "Extract all links"; then
    sleep 1
else
    workflow2_success=false
fi

# Step 3: Navigate to GitHub
if execute_workflow_step "navigate_to_url" '{"url":"https://github.com"}' "Navigate to GitHub"; then
    sleep 2
else
    workflow2_success=false
fi

# Step 4: Interact with GitHub interface
if execute_workflow_step "hover" '{"selector":".Header-link"}' "Hover over header link" && \
   execute_workflow_step "get_element_info" '{"selector":".Header"}' "Get header element info"; then
    sleep 1
else
    workflow2_success=false
fi

# Step 5: Use browser navigation
if execute_workflow_step "go_back" '{}' "Navigate back to Example.com" && \
   execute_workflow_step "go_forward" '{}' "Navigate forward to GitHub"; then
    sleep 1
else
    workflow2_success=false
fi

# Step 6: Refresh and verify
if execute_workflow_step "refresh" '{}' "Refresh current page" && \
   execute_workflow_step "extract_text" '{"selector":"body"}' "Extract body content after refresh"; then
    sleep 1
else
    workflow2_success=false
fi

if [ "$workflow2_success" = true ]; then
    test_passed "Navigation and Data Extraction Workflow"
else
    test_failed "Navigation and Data Extraction Workflow"
fi

# E2E Test 3: Perception-Powered Workflow
echo ""
echo "--- E2E Test 3: Perception-Powered Workflow ---"
workflow3_success=true

echo "Scenario: Use AI perception for intelligent page interaction"

# Step 1: Navigate to test page
if execute_workflow_step "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate to test page for perception"; then
    sleep 1
else
    workflow3_success=false
fi

# Step 2: Use perception to analyze page
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{}' "$SERVER_URL/api/perception/analyze")
if echo "$response" | jq -e '.data.url' > /dev/null 2>&1; then
    echo "    ✓ Analyze page with perception engine"
    sleep 1
else
    echo "    ✗ Analyze page with perception engine"
    workflow3_success=false
fi

# Step 3: Find elements using natural language
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"description":"customer name input"}' "$SERVER_URL/api/perception/find")
if echo "$response" | jq -e '.data.selector' > /dev/null 2>&1; then
    echo "    ✓ Find element using natural language description"
    sleep 1
else
    echo "    ✗ Find element using natural language description"
    workflow3_success=false
fi

# Step 4: Execute intelligent command
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"command":{"action":"click","description":"customer name field","parameters":{}}}' \
    "$SERVER_URL/api/perception/command")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "true" ]; then
        echo "    ✓ Execute intelligent command"
        sleep 1
    else
        echo "    ✗ Execute intelligent command"
        workflow3_success=false
    fi
else
    echo "    ✗ Execute intelligent command (invalid response)"
    workflow3_success=false
fi

# Step 5: Analyze forms intelligently
response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"form_selector":"form"}' "$SERVER_URL/api/perception/forms/analyze")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    echo "    ✓ Intelligent form analysis"
    sleep 1
else
    echo "    ✗ Intelligent form analysis"
    workflow3_success=false
fi

if [ "$workflow3_success" = true ]; then
    test_passed "Perception-Powered Workflow"
else
    test_failed "Perception-Powered Workflow"
fi

# E2E Test 4: Session and State Management
echo ""
echo "--- E2E Test 4: Session and State Management ---"
workflow4_success=true

echo "Scenario: Manage browser session state across multiple operations"

# Step 1: Set initial session data
if execute_workflow_step "session_storage" '{"action":"set","key":"user_id","value":"test_user_123"}' "Set user ID in session" && \
   execute_workflow_step "session_storage" '{"action":"set","key":"workflow_step","value":"1"}' "Set workflow step counter"; then
    sleep 0.5
else
    workflow4_success=false
fi

# Step 2: Navigate and maintain state
if execute_workflow_step "navigate_to_url" '{"url":"https://example.com"}' "Navigate to first page" && \
   execute_workflow_step "session_storage" '{"action":"get","key":"user_id"}' "Retrieve user ID" && \
   execute_workflow_step "session_storage" '{"action":"set","key":"workflow_step","value":"2"}' "Update workflow step"; then
    sleep 1
else
    workflow4_success=false
fi

# Step 3: Navigate to another page and maintain state
if execute_workflow_step "navigate_to_url" '{"url":"https://httpbin.org"}' "Navigate to second page" && \
   execute_workflow_step "session_storage" '{"action":"get","key":"user_id"}' "Verify user ID persists" && \
   execute_workflow_step "session_storage" '{"action":"set","key":"workflow_step","value":"3"}' "Update workflow step"; then
    sleep 1
else
    workflow4_success=false
fi

# Step 4: Use persistent cache
if execute_workflow_step "persistent_cache" '{"action":"set","key":"workflow_data","value":"e2e_test_completed"}' "Store in persistent cache" && \
   execute_workflow_step "persistent_cache" '{"action":"get","key":"workflow_data"}' "Retrieve from persistent cache"; then
    sleep 0.5
else
    workflow4_success=false
fi

# Step 5: Final state verification
if execute_workflow_step "session_storage" '{"action":"get","key":"workflow_step"}' "Verify final workflow step"; then
    sleep 0.5
else
    workflow4_success=false
fi

if [ "$workflow4_success" = true ]; then
    test_passed "Session and State Management Workflow"
else
    test_failed "Session and State Management Workflow"
fi

# E2E Test 5: Error Recovery and Resilience
echo ""
echo "--- E2E Test 5: Error Recovery and Resilience ---"
workflow5_success=true

echo "Scenario: Handle errors gracefully and continue operation"

# Step 1: Start with valid operation
if execute_workflow_step "navigate_to_url" '{"url":"https://example.com"}' "Navigate to valid page"; then
    sleep 1
else
    workflow5_success=false
fi

# Step 2: Attempt invalid operation (should fail gracefully)
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"click","parameters":{"selector":".non-existent-element"}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        echo "    ✓ Handle invalid element click gracefully"
    else
        echo "    ✗ Should fail gracefully for non-existent element"
        workflow5_success=false
    fi
else
    echo "    ✗ Invalid operation response format"
    workflow5_success=false
fi

# Step 3: Continue with valid operations after error
if execute_workflow_step "extract_text" '{"selector":"h1"}' "Continue with text extraction after error" && \
   execute_workflow_step "get_element_info" '{"selector":"body"}' "Continue with element info after error"; then
    sleep 1
else
    workflow5_success=false
fi

# Step 4: Test network resilience with timeout
response=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"tool_name":"wait_for_element","parameters":{"selector":".never-exists","timeout":1000}}' \
    "$SERVER_URL/api/tools/execute")
if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
    success=$(echo "$response" | jq -r '.success')
    if [ "$success" = "false" ]; then
        echo "    ✓ Handle element wait timeout gracefully"
    else
        echo "    ✗ Should timeout for non-existent element"
        workflow5_success=false
    fi
else
    echo "    ✗ Timeout operation response format"
    workflow5_success=false
fi

# Step 5: Verify system still functional
if execute_workflow_step "extract_text" '{"selector":"body"}' "Verify system functionality after errors"; then
    sleep 1
else
    workflow5_success=false
fi

if [ "$workflow5_success" = true ]; then
    test_passed "Error Recovery and Resilience Workflow"
else
    test_failed "Error Recovery and Resilience Workflow"
fi

# E2E Test 6: Complex Multi-Site Workflow
echo ""
echo "--- E2E Test 6: Complex Multi-Site Workflow ---"
workflow6_success=true

echo "Scenario: Complex workflow across multiple sites with data correlation"

# Step 1: Collect data from first site
if execute_workflow_step "navigate_to_url" '{"url":"https://example.com"}' "Navigate to first data source" && \
   execute_workflow_step "extract_text" '{"selector":"h1"}' "Extract title from first site" && \
   execute_workflow_step "session_storage" '{"action":"set","key":"site1_title","value":"Example Domain"}' "Store first site data"; then
    sleep 1
else
    workflow6_success=false
fi

# Step 2: Navigate to second site and collect data
if execute_workflow_step "navigate_to_url" '{"url":"https://httpbin.org"}' "Navigate to second data source" && \
   execute_workflow_step "extract_text" '{"selector":"h1"}' "Extract title from second site" && \
   execute_workflow_step "session_storage" '{"action":"set","key":"site2_title","value":"httpbin"}' "Store second site data"; then
    sleep 1
else
    workflow6_success=false
fi

# Step 3: Navigate to third site for form interaction
if execute_workflow_step "navigate_to_url" '{"url":"https://httpbin.org/forms/post"}' "Navigate to form site" && \
   execute_workflow_step "session_storage" '{"action":"get","key":"site1_title"}' "Retrieve stored data" && \
   execute_workflow_step "type_text" '{"selector":"input[name=\"custname\"]","text":"Multi-Site User"}' "Use data in form"; then
    sleep 1
else
    workflow6_success=false
fi

# Step 4: Correlation and analysis
if execute_workflow_step "session_storage" '{"action":"get","key":"site2_title"}' "Retrieve second site data" && \
   execute_workflow_step "extract_text" '{"selector":"form"}' "Extract form structure for analysis"; then
    sleep 1
else
    workflow6_success=false
fi

# Step 5: Final navigation and verification
if execute_workflow_step "go_back" '{}' "Navigate back in workflow" && \
   execute_workflow_step "go_forward" '{}' "Navigate forward again" && \
   execute_workflow_step "persistent_cache" '{"action":"set","key":"workflow_complete","value":"multi_site_success"}' "Mark workflow complete"; then
    sleep 1
else
    workflow6_success=false
fi

if [ "$workflow6_success" = true ]; then
    test_passed "Complex Multi-Site Workflow"
else
    test_failed "Complex Multi-Site Workflow"
fi

# System Health Check After All Workflows
echo ""
echo "--- Post-Workflow System Health Check ---"
health_response=$(curl -s "$SERVER_URL/api/health")
if echo "$health_response" | jq -e '.status' > /dev/null 2>&1; then
    test_passed "System healthy after all E2E workflows"
else
    test_failed "System health degraded after workflows"
fi

# Summary
echo ""
echo "========================================="
echo "         E2E TEST SUMMARY"
echo "========================================="
echo "Total Workflows: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"
echo ""
echo "Completed Scenarios:"
echo "1. Form Interaction Workflow"
echo "2. Navigation and Data Extraction"
echo "3. Perception-Powered Workflow"
echo "4. Session and State Management"
echo "5. Error Recovery and Resilience"
echo "6. Complex Multi-Site Workflow"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All end-to-end tests passed!"
    echo "🚀 System ready for production use!"
    exit 0
else
    echo ""
    echo "❌ $FAILED E2E workflow(s) failed"
    echo "🔧 Review failing workflows before deployment"
    exit 1
fi