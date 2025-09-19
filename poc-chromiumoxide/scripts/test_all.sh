#!/bin/bash

# Comprehensive test script for RainbowBrowserAI Chromiumoxide Edition
# This script runs all tests and generates a report

set -e

echo "========================================="
echo "RainbowBrowserAI Chromiumoxide Test Suite"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test categories
declare -a test_categories=(
    "navigation"
    "interaction"
    "extraction"
    "synchronization"
    "memory"
    "api_integration"
    "performance"
    "error_handling"
)

# Run unit tests
echo -e "${YELLOW}Running Unit Tests...${NC}"
cargo test --lib --quiet 2>&1 | tee test_results_unit.log
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
else
    echo -e "${RED}✗ Unit tests failed${NC}"
fi
echo ""

# Run integration tests
echo -e "${YELLOW}Running Integration Tests...${NC}"
for category in "${test_categories[@]}"; do
    echo -n "  Testing $category... "
    if cargo test --test tools_integration_test ${category}_tests --quiet 2>&1 > test_results_${category}.log; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        echo "    See test_results_${category}.log for details"
    fi
done
echo ""

# Run doc tests
echo -e "${YELLOW}Running Documentation Tests...${NC}"
cargo test --doc --quiet 2>&1 | tee test_results_doc.log
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Doc tests passed${NC}"
else
    echo -e "${RED}✗ Doc tests failed${NC}"
fi
echo ""

# Check code coverage (if tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}Generating Code Coverage Report...${NC}"
    cargo tarpaulin --out Html --output-dir coverage 2>&1 | tee test_coverage.log
    echo -e "${GREEN}✓ Coverage report generated in coverage/index.html${NC}"
else
    echo -e "${YELLOW}Skipping coverage (cargo-tarpaulin not installed)${NC}"
fi
echo ""

# Run benchmarks (if available)
echo -e "${YELLOW}Running Benchmarks...${NC}"
if cargo bench --no-run 2>/dev/null; then
    cargo bench 2>&1 | tee test_benchmarks.log
    echo -e "${GREEN}✓ Benchmarks completed${NC}"
else
    echo -e "${YELLOW}No benchmarks configured${NC}"
fi
echo ""

# API endpoint tests
echo -e "${YELLOW}Testing API Endpoints...${NC}"
if curl -s http://localhost:3002/api/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ API is running${NC}"
    
    # Test each tool endpoint
    declare -a tools=(
        "navigate_to_url"
        "scroll"
        "refresh"
        "go_back"
        "go_forward"
        "click"
        "type_text"
        "hover"
        "focus"
        "select_option"
        "extract_text"
        "extract_links"
        "extract_data"
        "extract_table"
        "extract_form"
        "wait_for_element"
        "wait_for_condition"
        "screenshot"
        "session_memory"
        "get_element_info"
        "history_tracker"
        "persistent_cache"
    )
    
    echo "  Testing individual tools:"
    for tool in "${tools[@]}"; do
        echo -n "    $tool... "
        # Create appropriate test parameters based on tool
        case $tool in
            "navigate_to_url")
                params='{"url":"https://example.com"}'
                ;;
            "scroll")
                params='{"x":0,"y":100}'
                ;;
            "click"|"hover"|"focus"|"extract_text")
                params='{"selector":"body"}'
                ;;
            "type_text")
                params='{"selector":"input","text":"test"}'
                ;;
            "select_option")
                params='{"selector":"select","value":"option1"}'
                ;;
            "extract_links"|"extract_table"|"extract_form")
                params='{"selector":"a"}'
                ;;
            "extract_data")
                params='{"selector":"a","attributes":["href"]}'
                ;;
            "wait_for_element")
                params='{"selector":"body","timeout":1000}'
                ;;
            "wait_for_condition")
                params='{"condition":"true","timeout":1000}'
                ;;
            "screenshot")
                params='{"full_page":false}'
                ;;
            "session_memory"|"history_tracker"|"persistent_cache")
                params='{"action":"get"}'
                ;;
            "get_element_info")
                params='{"selector":"body"}'
                ;;
            *)
                params='{}'
                ;;
        esac
        
        response=$(curl -s -X POST http://localhost:3002/api/tools/execute \
            -H "Content-Type: application/json" \
            -d "{\"tool_name\":\"$tool\",\"parameters\":$params}" 2>/dev/null)
        
        if echo "$response" | grep -q '"success":true'; then
            echo -e "${GREEN}✓${NC}"
        else
            echo -e "${RED}✗${NC}"
            echo "      Response: $response" >> test_api_errors.log
        fi
    done
else
    echo -e "${RED}✗ API is not running on port 3002${NC}"
    echo "  Start the server with: cargo run --release -- serve --port 3002"
fi
echo ""

# Generate summary report
echo "========================================="
echo "Test Summary Report"
echo "========================================="
echo ""

# Count test results
total_tests=0
passed_tests=0
failed_tests=0

for log_file in test_results_*.log; do
    if [ -f "$log_file" ]; then
        if grep -q "test result: ok" "$log_file" 2>/dev/null; then
            passed_tests=$((passed_tests + 1))
        elif grep -q "FAILED" "$log_file" 2>/dev/null || grep -q "error" "$log_file" 2>/dev/null; then
            failed_tests=$((failed_tests + 1))
        fi
        total_tests=$((total_tests + 1))
    fi
done

echo "Total test categories: $total_tests"
echo -e "Passed: ${GREEN}$passed_tests${NC}"
echo -e "Failed: ${RED}$failed_tests${NC}"
echo ""

if [ $failed_tests -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed successfully!${NC}"
else
    echo -e "${RED}⚠️ Some tests failed. Check the log files for details.${NC}"
fi

echo ""
echo "Log files generated:"
ls -la test_*.log 2>/dev/null || echo "No log files generated"
echo ""
echo "========================================="
echo "Test run completed at $(date)"
echo "========================================="