#!/bin/bash

# Enhanced Perception System Test Script
# Tests the new intelligent element detection, error recovery, and form handling capabilities

set -e  # Exit on any error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_PORT=3003
CHROMEDRIVER_PORT=9517
BASE_URL="http://localhost:$TEST_PORT"
RAINBOW_LOG_LEVEL="info"

echo -e "${BLUE}🧠 Enhanced Perception System Test Suite${NC}"
echo "============================================="

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS") echo -e "${GREEN}✅ $message${NC}" ;;
        "ERROR") echo -e "${RED}❌ $message${NC}" ;;
        "WARNING") echo -e "${YELLOW}⚠️  $message${NC}" ;;
        "INFO") echo -e "${BLUE}ℹ️  $message${NC}" ;;
    esac
}

# Function to check if service is running
check_service() {
    local url=$1
    local service_name=$2
    
    if curl -s "$url" > /dev/null 2>&1; then
        print_status "SUCCESS" "$service_name is running"
        return 0
    else
        print_status "ERROR" "$service_name is not responding"
        return 1
    fi
}

# Function to test API endpoint with detailed output
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_field=$4
    local test_description=$5
    
    print_status "INFO" "Testing: $test_description"
    
    local response
    local http_code
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" "$BASE_URL$endpoint")
    else
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" "$BASE_URL$endpoint")
    fi
    
    http_code=$(echo "$response" | sed -E 's/.*HTTPSTATUS:([0-9]{3})$/\1/')
    body=$(echo "$response" | sed -E 's/HTTPSTATUS:[0-9]{3}$//')
    
    if [ "$http_code" = "200" ]; then
        if [ -n "$expected_field" ] && echo "$body" | grep -q "$expected_field"; then
            print_status "SUCCESS" "$test_description - Response contains expected data"
            return 0
        elif [ -z "$expected_field" ]; then
            print_status "SUCCESS" "$test_description - HTTP 200 OK"
            return 0
        else
            print_status "WARNING" "$test_description - Missing expected field: $expected_field"
            echo "Response: $body"
            return 1
        fi
    else
        print_status "ERROR" "$test_description - HTTP $http_code"
        echo "Response: $body"
        return 1
    fi
}

# Function to start the RainbowBrowserAI service
start_service() {
    print_status "INFO" "Starting RainbowBrowserAI Enhanced Perception Service..."
    
    # Set environment variables for enhanced perception testing
    export RAINBOW_MOCK_MODE=true
    export RAINBOW_V8_MODE=production
    export RAINBOW_PERCEPTION_ENHANCED=true
    export RAINBOW_SMART_DETECTION=true
    export RAINBOW_ERROR_RECOVERY=true
    export RAINBOW_FORM_INTELLIGENCE=true
    export RAINBOW_API_PORT=$TEST_PORT
    export CHROMEDRIVER_PORT=$CHROMEDRIVER_PORT
    export RUST_LOG=$RAINBOW_LOG_LEVEL,rainbow_poc=debug
    export RAINBOW_TEST_MODE=enhanced_perception
    
    # Start service in background
    cargo run --release -- serve --port $TEST_PORT &
    SERVICE_PID=$!
    
    # Wait for service to start
    local attempts=0
    while [ $attempts -lt 30 ]; do
        if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
            print_status "SUCCESS" "Service started successfully (PID: $SERVICE_PID)"
            return 0
        fi
        sleep 1
        attempts=$((attempts + 1))
    done
    
    print_status "ERROR" "Service failed to start within 30 seconds"
    return 1
}

# Function to stop the service
stop_service() {
    if [ -n "$SERVICE_PID" ]; then
        print_status "INFO" "Stopping service (PID: $SERVICE_PID)..."
        kill $SERVICE_PID 2>/dev/null || true
        wait $SERVICE_PID 2>/dev/null || true
        print_status "SUCCESS" "Service stopped"
    fi
}

# Trap to ensure cleanup on exit
trap stop_service EXIT

echo ""
print_status "INFO" "=== Phase 1: Service Startup and Health Check ==="

# Start the enhanced perception service
if ! start_service; then
    exit 1
fi

# Test basic health endpoint
test_endpoint "GET" "/health" "" "status" "Basic health check"

# Test enhanced perception health
test_endpoint "GET" "/api/perception/health" "" "" "Enhanced perception health check"

echo ""
print_status "INFO" "=== Phase 2: Intelligent Element Detection Tests ==="

# Test 1: Smart search box detection
print_status "INFO" "Testing intelligent search box detection..."
search_test='{
    "command": "find search box",
    "use_smart_detection": true,
    "enable_fallback": true
}'
test_endpoint "POST" "/api/perception/smart-detect" "$search_test" "element_found" "Smart search box detection"

# Test 2: Button detection with error recovery
print_status "INFO" "Testing button detection with error recovery..."
button_test='{
    "command": "find submit button",
    "use_error_recovery": true,
    "max_retries": 3
}'
test_endpoint "POST" "/api/perception/smart-detect" "$button_test" "element_found" "Smart button detection with recovery"

# Test 3: Form field detection
print_status "INFO" "Testing form field detection..."
field_test='{
    "command": "find email input field",
    "element_type": "input",
    "smart_detection": true
}'
test_endpoint "POST" "/api/perception/smart-detect" "$field_test" "element_found" "Smart form field detection"

echo ""
print_status "INFO" "=== Phase 3: Error Recovery System Tests ==="

# Test 4: Recovery from element not found
print_status "INFO" "Testing error recovery from element not found..."
recovery_test='{
    "command": "find non-existent-element-12345",
    "enable_error_recovery": true,
    "graceful_degradation": true
}'
test_endpoint "POST" "/api/perception/smart-detect" "$recovery_test" "recovery_attempted" "Error recovery test"

# Test 5: Fallback selector usage
print_status "INFO" "Testing fallback selector mechanism..."
fallback_test='{
    "command": "find login button",
    "primary_selector": "#non-existent",
    "enable_fallback": true
}'
test_endpoint "POST" "/api/perception/smart-detect" "$fallback_test" "fallback_used" "Fallback selector test"

echo ""
print_status "INFO" "=== Phase 4: Enhanced Form Handling Tests ==="

# Test 6: Intelligent form field filling
print_status "INFO" "Testing intelligent form field filling..."
form_fill_test='{
    "action": "fill_field",
    "field_description": "email address field",
    "value": "test@example.com",
    "smart_validation": true
}'
test_endpoint "POST" "/api/perception/form-interact" "$form_fill_test" "field_filled" "Smart form field filling"

# Test 7: Form submission with multiple strategies
print_status "INFO" "Testing form submission strategies..."
form_submit_test='{
    "action": "submit_form",
    "submit_description": "submit button",
    "use_multiple_strategies": true
}'
test_endpoint "POST" "/api/perception/form-interact" "$form_submit_test" "submit_attempted" "Smart form submission"

# Test 8: Complex form handling (select, checkbox, radio)
print_status "INFO" "Testing complex form element handling..."
complex_form_test='{
    "action": "fill_field",
    "field_description": "country dropdown",
    "value": "United States",
    "field_type": "select"
}'
test_endpoint "POST" "/api/perception/form-interact" "$complex_form_test" "field_updated" "Complex form element handling"

echo ""
print_status "INFO" "=== Phase 5: Advanced Perception Engine Tests ==="

# Test 9: Comprehensive page analysis
print_status "INFO" "Testing comprehensive page analysis..."
analysis_test='{
    "url": "https://example.com",
    "analysis_level": "comprehensive",
    "include_smart_detection": true,
    "include_form_analysis": true
}'
test_endpoint "POST" "/api/perception/analyze-page" "$analysis_test" "analysis_complete" "Comprehensive page analysis"

# Test 10: Adaptive layer selection
print_status "INFO" "Testing adaptive perception layer selection..."
adaptive_test='{
    "command": "analyze page for shopping workflow",
    "adaptive_layer_selection": true,
    "context": "e-commerce"
}'
test_endpoint "POST" "/api/perception/adaptive-analyze" "$adaptive_test" "layer_selected" "Adaptive layer selection"

echo ""
print_status "INFO" "=== Phase 6: Performance and Statistics Tests ==="

# Test 11: Performance metrics collection
print_status "INFO" "Testing performance metrics collection..."
test_endpoint "GET" "/api/perception/metrics" "" "total_requests" "Performance metrics collection"

# Test 12: Error recovery statistics
print_status "INFO" "Testing error recovery statistics..."
test_endpoint "GET" "/api/perception/recovery-stats" "" "successful_recoveries" "Error recovery statistics"

# Test 13: System health assessment
print_status "INFO" "Testing system health assessment..."
test_endpoint "GET" "/api/perception/system-health" "" "system_health" "System health assessment"

echo ""
print_status "INFO" "=== Phase 7: Integration and Real-World Scenario Tests ==="

# Test 14: End-to-end workflow simulation
print_status "INFO" "Testing end-to-end workflow simulation..."
e2e_test='{
    "workflow": "enhanced_perception_demo",
    "steps": [
        {"action": "navigate", "url": "https://example.com"},
        {"action": "smart_detect", "element": "search box"},
        {"action": "fill_field", "field": "search", "value": "test query"},
        {"action": "submit_form"}
    ],
    "use_enhanced_features": true
}'
test_endpoint "POST" "/api/workflows/execute" "$e2e_test" "workflow_completed" "End-to-end enhanced workflow"

# Test 15: Stress test - multiple concurrent requests
print_status "INFO" "Testing concurrent request handling..."
concurrent_pids=()
for i in {1..5}; do
    (test_endpoint "GET" "/api/perception/health" "" "status" "Concurrent request $i") &
    concurrent_pids+=($!)
done

# Wait for all concurrent tests to complete
for pid in "${concurrent_pids[@]}"; do
    wait $pid
done
print_status "SUCCESS" "Concurrent request handling completed"

echo ""
print_status "INFO" "=== Phase 8: Validation and Cleanup ==="

# Final validation - check that service is still healthy
test_endpoint "GET" "/health" "" "status" "Final health check"

# Generate test summary
echo ""
print_status "INFO" "=== Test Summary ==="
echo "Enhanced Perception System Test Suite completed successfully!"
echo "- ✅ Intelligent Element Detection: Tested"
echo "- ✅ Error Recovery System: Tested"
echo "- ✅ Enhanced Form Handling: Tested"
echo "- ✅ Advanced Perception Engine: Tested"
echo "- ✅ Performance Metrics: Tested"
echo "- ✅ Integration Scenarios: Tested"

print_status "SUCCESS" "All enhanced perception features are operational!"

echo ""
print_status "INFO" "Test logs and metrics are available in the service output."
print_status "INFO" "Enhanced perception system is ready for real-world usage!"

exit 0