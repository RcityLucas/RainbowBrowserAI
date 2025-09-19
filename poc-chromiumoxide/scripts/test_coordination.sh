#!/bin/bash

# Test script for coordination architecture
# This script tests the new perception-to-tools coordination

set -e

echo "================================================================"
echo "Testing RainbowBrowserAI Coordination Architecture"
echo "================================================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run a test
run_test() {
    local test_name=$1
    echo -e "${YELLOW}Running: $test_name${NC}"
    
    if cargo test --test coordination_integration_test $test_name -- --nocapture; then
        echo -e "${GREEN}✓ $test_name passed${NC}\n"
        return 0
    else
        echo -e "${RED}✗ $test_name failed${NC}\n"
        return 1
    fi
}

# Build the project first
echo "Building project..."
if cargo build --lib; then
    echo -e "${GREEN}✓ Build successful${NC}\n"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Run individual tests
echo "Running coordination tests..."
echo "================================"

FAILED_TESTS=()

# Test 1: Perception detects tool interface
if ! run_test "test_perception_detects_tool_interface"; then
    FAILED_TESTS+=("test_perception_detects_tool_interface")
fi

# Test 2: Shared instance coordination
if ! run_test "test_shared_instance_coordination"; then
    FAILED_TESTS+=("test_shared_instance_coordination")
fi

# Test 3: Navigation cache invalidation
if ! run_test "test_navigation_cache_invalidation"; then
    FAILED_TESTS+=("test_navigation_cache_invalidation")
fi

# Test 4: Error handling with fallback
if ! run_test "test_error_handling_with_fallback"; then
    FAILED_TESTS+=("test_error_handling_with_fallback")
fi

# Test 5: Module coordinator
if ! run_test "test_module_coordinator"; then
    FAILED_TESTS+=("test_module_coordinator")
fi

# Summary
echo ""
echo "================================================================"
echo "Test Summary"
echo "================================================================"

if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All coordination tests passed!${NC}"
    echo ""
    echo "The coordination architecture successfully:"
    echo "  • Shares browser instances between modules"
    echo "  • Enables perception to detect tool-compatible interfaces"
    echo "  • Coordinates cache invalidation on navigation"
    echo "  • Provides unified error handling with fallback"
    echo "  • Manages multiple sessions efficiently"
else
    echo -e "${RED}✗ ${#FAILED_TESTS[@]} test(s) failed:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    echo ""
    echo "Please check the test output above for details."
    exit 1
fi

echo ""
echo "You can also run the full test suite with:"
echo "  cargo test coordination"
echo ""
echo "Or run a specific test with:"
echo "  cargo test --test coordination_integration_test test_perception_detects_tool_interface -- --nocapture"