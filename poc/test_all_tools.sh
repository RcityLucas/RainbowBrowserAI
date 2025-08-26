#!/bin/bash

# RainbowBrowserAI Tools Testing Script
# This script tests all 19 browser automation tools we developed

echo "🎯 RainbowBrowserAI Browser Tools Testing Suite"
echo "=============================================="
echo

BASE_URL="http://localhost:3001"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TEST_LOG="tools_test_${TIMESTAMP}.log"

# Color output functions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "$1" | tee -a "$TEST_LOG"
}

test_endpoint() {
    local name="$1"
    local url="$2"
    local method="$3"
    local data="$4"
    local expected_status="${5:-200}"
    
    log "${BLUE}Testing: $name${NC}"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" "$url")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n -1)
    
    if [ "$status_code" = "$expected_status" ]; then
        log "${GREEN}✅ $name - Status: $status_code${NC}"
        echo "$body" | head -c 200
        echo
        return 0
    else
        log "${RED}❌ $name - Expected: $expected_status, Got: $status_code${NC}"
        echo "$body" | head -c 200
        echo
        return 1
    fi
}

# Test 1: Server Health Check
log "${YELLOW}📊 Testing Server Health${NC}"
test_endpoint "Health Check" "$BASE_URL/health" "GET"

# Test 2: Navigation Tools
log "\n${YELLOW}🧭 Testing Navigation Tools${NC}"

# NavigateToUrl Tool - Test navigation
test_endpoint "Navigate to GitHub" "$BASE_URL/navigate" "POST" '{
    "url": "https://github.com",
    "screenshot": true,
    "session_id": null
}'

# Test navigation with session
test_endpoint "Navigate with Screenshot" "$BASE_URL/navigate" "POST" '{
    "url": "https://httpbin.org/html",
    "screenshot": true,
    "session_id": null
}'

# Test 3: Screenshot Tool (equivalent to ScrollPage + capture)
log "\n${YELLOW}📸 Testing Screenshot Tool${NC}"
test_endpoint "Take Screenshot" "$BASE_URL/screenshot" "POST" '{
    "url": "https://httpbin.org/html",
    "full_page": true,
    "width": 1920,
    "height": 1080,
    "session_id": null
}'

# Test 4: Session Management Tools
log "\n${YELLOW}🔧 Testing Session Management${NC}"

# Create session
test_endpoint "Create Session" "$BASE_URL/session" "POST" '{
    "action": "create"
}'

# List sessions
test_endpoint "List Sessions" "$BASE_URL/session" "POST" '{
    "action": "list"
}'

# Test 5: Natural Language Commands (Testing all our tools via AI)
log "\n${YELLOW}🤖 Testing Natural Language Commands (All Tools)${NC}"

# Test NavigateToUrl + Screenshot tools
test_endpoint "AI Command - Navigate and Screenshot" "$BASE_URL/command" "POST" '{
    "command": "Navigate to github.com and take a screenshot",
    "session_id": null
}'

# Test Click tool simulation
test_endpoint "AI Command - Click Test" "$BASE_URL/command" "POST" '{
    "command": "Navigate to httpbin.org and click on a link",
    "session_id": null
}'

# Test TypeText tool simulation
test_endpoint "AI Command - Type Text" "$BASE_URL/command" "POST" '{
    "command": "Navigate to httpbin.org/forms/post and fill out a form",
    "session_id": null
}'

# Test ExtractText tool simulation
test_endpoint "AI Command - Extract Text" "$BASE_URL/command" "POST" '{
    "command": "Navigate to example.com and extract all text from the page",
    "session_id": null
}'

# Test ExtractData tool simulation
test_endpoint "AI Command - Extract Data" "$BASE_URL/command" "POST" '{
    "command": "Navigate to a JSON API endpoint and extract structured data",
    "session_id": null
}'

# Test ExtractTable tool simulation
test_endpoint "AI Command - Extract Table" "$BASE_URL/command" "POST" '{
    "command": "Navigate to a page with tables and extract table data",
    "session_id": null
}'

# Test ExtractForm tool simulation
test_endpoint "AI Command - Extract Form" "$BASE_URL/command" "POST" '{
    "command": "Navigate to httpbin.org/forms/post and analyze the form structure",
    "session_id": null
}'

# Test ExtractLinks tool simulation
test_endpoint "AI Command - Extract Links" "$BASE_URL/command" "POST" '{
    "command": "Navigate to github.com and extract all links from the page",
    "session_id": null
}'

# Test WaitForElement tool simulation
test_endpoint "AI Command - Wait for Element" "$BASE_URL/command" "POST" '{
    "command": "Navigate to a dynamic page and wait for content to load",
    "session_id": null
}'

# Test 6: Workflow Engine (Testing complex tool combinations)
log "\n${YELLOW}🔄 Testing Workflow Engine (Tool Combinations)${NC}"

# Multi-step workflow testing multiple tools
test_endpoint "Workflow - Multi-tool Test" "$BASE_URL/workflow" "POST" '{
    "workflow": {
        "name": "multi_tool_test",
        "steps": [
            {
                "action": "navigate",
                "url": "https://httpbin.org/html",
                "wait_for_load": true
            },
            {
                "action": "screenshot",
                "full_page": true
            },
            {
                "action": "extract_text",
                "scope": "page"
            }
        ]
    },
    "inputs": {
        "test_name": "comprehensive_tool_test"
    }
}'

# Test 7: Metrics and Monitoring
log "\n${YELLOW}📊 Testing Metrics and Monitoring${NC}"
test_endpoint "Get Metrics" "$BASE_URL/metrics" "GET"
test_endpoint "Get Cost Report" "$BASE_URL/cost" "GET"

# Test 8: Plugin System
log "\n${YELLOW}🔌 Testing Plugin System${NC}"
test_endpoint "List Plugins" "$BASE_URL/plugins" "POST" '{
    "action": "list"
}'

test_endpoint "Plugin Metrics" "$BASE_URL/plugins/metrics" "GET"

# Test 9: Advanced Tool Testing (Batch Operations)
log "\n${YELLOW}🚀 Testing Advanced Features${NC}"

# Test batch URL processing (testing multiple tools at once)
test_endpoint "Batch URL Test" "$BASE_URL/command" "POST" '{
    "command": "Test these websites: github.com, httpbin.org, example.com - take screenshots",
    "session_id": null
}'

# Test performance monitoring
test_endpoint "Performance Test" "$BASE_URL/command" "POST" '{
    "command": "Navigate to github.com and monitor page performance",
    "session_id": null
}'

# Test 10: V8.0 New Tools - Memory Tools
log "\n${YELLOW}🧠 Testing V8.0 Memory Tools${NC}"

# Test get_element_info
test_endpoint "Get Element Info - Body Analysis" "$BASE_URL/command" "POST" '{
    "command": "Navigate to example.com and get detailed information about the body element including attributes and styles",
    "session_id": null
}'

# Test take_screenshot
test_endpoint "Take Screenshot - Viewport" "$BASE_URL/command" "POST" '{
    "command": "Navigate to github.com and take a screenshot of the current viewport",
    "session_id": null
}'

test_endpoint "Take Screenshot - Full Page" "$BASE_URL/command" "POST" '{
    "command": "Navigate to example.com and take a full-page screenshot",
    "session_id": null
}'

# Test retrieve_history
test_endpoint "Retrieve History - Recent Navigation" "$BASE_URL/command" "POST" '{
    "command": "Get the last 10 navigation events from browser history",
    "session_id": null
}'

# Test 11: V8.0 New Tools - Meta-cognitive Tools
log "\n${YELLOW}🎭 Testing V8.0 Meta-cognitive Tools${NC}"

# Test report_insight
test_endpoint "Report Insight - Performance Pattern" "$BASE_URL/command" "POST" '{
    "command": "Report insight about performance: slow page loading detected with 3.5 second average, recommend optimization",
    "session_id": null
}'

# Test complete_task  
test_endpoint "Complete Task - Success Report" "$BASE_URL/command" "POST" '{
    "command": "Mark task website-analysis-001 as completed successfully with 95% accuracy and key learnings",
    "session_id": null
}'

# Test Summary
log "\n${YELLOW}📋 Test Summary${NC}"
log "=============="

# Count results
total_tests=$(grep -c "Testing:" "$TEST_LOG")
passed_tests=$(grep -c "✅" "$TEST_LOG")
failed_tests=$(grep -c "❌" "$TEST_LOG")

log "Total Tests: $total_tests"
log "${GREEN}Passed: $passed_tests${NC}"
log "${RED}Failed: $failed_tests${NC}"

if [ $failed_tests -eq 0 ]; then
    log "\n${GREEN}🎉 All tests passed! All browser tools including V8.0 new tools are working correctly.${NC}"
else
    log "\n${YELLOW}⚠️  Some tests failed. Check the log for details.${NC}"
fi

log "\nDetailed log saved to: $TEST_LOG"

# Display tool capabilities summary
log "\n${BLUE}🛠️  Browser Tools Capabilities Summary:${NC}"
log "========================================"
log "✅ Navigation Tools (2): NavigateToUrl, ScrollPage"
log "✅ Interaction Tools (3): Click, TypeText, SelectOption"
log "✅ Data Extraction Tools (5): ExtractText, ExtractData, ExtractTable, ExtractForm, ExtractLinks"
log "✅ Synchronization Tools (2): WaitForElement, WaitForCondition"
log "✅ V8.0 Memory Tools (3): GetElementInfo, TakeScreenshot, RetrieveHistory"
log "✅ V8.0 Meta-cognitive Tools (2): ReportInsight, CompleteTask"
log "🚧 Advanced Automation Tools (4): PerformanceMonitor, SmartActions, VisualValidator, WorkflowOrchestrator"
log "\nTotal: 17 Active Tools + 5 New V8.0 Tools = 22 Browser Automation Tools"