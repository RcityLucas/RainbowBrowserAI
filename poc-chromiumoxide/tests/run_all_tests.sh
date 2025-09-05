#!/bin/bash

# Master Test Runner for RainbowBrowserAI Chromiumoxide Edition
# Usage: ./tests/run_all_tests.sh [test_type]
# test_type: all, unit, integration, e2e, performance, api, ui, perception, browser, tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$TEST_DIR")"
REPORTS_DIR="$TEST_DIR/reports"
SERVER_PORT=3002
SERVER_PID=""

# Create reports directory
mkdir -p "$REPORTS_DIR"

# Cleanup function
cleanup() {
    if [ -n "$SERVER_PID" ]; then
        echo -e "${YELLOW}Stopping test server (PID: $SERVER_PID)...${NC}"
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Set trap for cleanup
trap cleanup EXIT

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Start server for testing
start_server() {
    log_info "Starting test server on port $SERVER_PORT..."
    cd "$PROJECT_DIR"
    
    # Kill any existing server on the port
    pkill -f "serve --port $SERVER_PORT" 2>/dev/null || true
    sleep 2
    
    # Start server in background
    RAINBOW_MOCK_MODE=true cargo run --release -- serve --port $SERVER_PORT --headless > "$REPORTS_DIR/server.log" 2>&1 &
    SERVER_PID=$!
    
    # Wait for server to be ready
    log_info "Waiting for server to start..."
    for i in {1..30}; do
        if curl -s http://localhost:$SERVER_PORT/api/health > /dev/null 2>&1; then
            log_success "Server is ready"
            return 0
        fi
        sleep 1
    done
    
    log_error "Server failed to start within 30 seconds"
    return 1
}

# Test functions
run_unit_tests() {
    log_info "Running unit tests..."
    cd "$PROJECT_DIR"
    
    # Rust unit tests
    log_info "Running Rust unit tests..."
    cargo test --lib > "$REPORTS_DIR/unit_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Rust unit tests passed"
    else
        log_error "Rust unit tests failed"
        return 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    
    # API integration tests
    "$TEST_DIR/integration/test_api_integration.sh" > "$REPORTS_DIR/api_integration.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "API integration tests passed"
    else
        log_error "API integration tests failed"
        return 1
    fi
    
    # Browser integration tests  
    "$TEST_DIR/integration/test_browser_integration.sh" > "$REPORTS_DIR/browser_integration.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Browser integration tests passed"
    else
        log_error "Browser integration tests failed"
        return 1
    fi
}

run_api_tests() {
    log_info "Running API tests..."
    "$TEST_DIR/api/test_all_endpoints.sh" > "$REPORTS_DIR/api_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "API tests passed"
    else
        log_error "API tests failed"
        return 1
    fi
}

run_ui_tests() {
    log_info "Running UI tests..."
    "$TEST_DIR/ui/test_interface.sh" > "$REPORTS_DIR/ui_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "UI tests passed"
    else
        log_error "UI tests failed"
        return 1
    fi
}

run_perception_tests() {
    log_info "Running perception tests..."
    "$TEST_DIR/perception/test_perception_suite.sh" > "$REPORTS_DIR/perception_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Perception tests passed"
    else
        log_error "Perception tests failed"
        return 1
    fi
}

run_browser_tests() {
    log_info "Running browser automation tests..."
    "$TEST_DIR/browser/test_browser_suite.sh" > "$REPORTS_DIR/browser_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Browser tests passed"
    else
        log_error "Browser tests failed"
        return 1
    fi
}

run_tools_tests() {
    log_info "Running tools tests..."
    "$TEST_DIR/tools/test_all_tools.sh" > "$REPORTS_DIR/tools_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Tools tests passed"
    else
        log_error "Tools tests failed"
        return 1
    fi
}

run_performance_tests() {
    log_info "Running performance tests..."
    "$TEST_DIR/performance/test_performance_suite.sh" > "$REPORTS_DIR/performance_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Performance tests passed"
    else
        log_error "Performance tests failed"
        return 1
    fi
}

run_e2e_tests() {
    log_info "Running end-to-end tests..."
    "$TEST_DIR/e2e/test_e2e_suite.sh" > "$REPORTS_DIR/e2e_tests.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "End-to-end tests passed"
    else
        log_error "End-to-end tests failed"
        return 1
    fi
}

# Generate test report
generate_report() {
    log_info "Generating test report..."
    
    REPORT_FILE="$REPORTS_DIR/test_summary_$(date +%Y%m%d_%H%M%S).json"
    
    cat > "$REPORT_FILE" << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "test_run": "$TEST_TYPE",
    "environment": {
        "os": "$(uname -s)",
        "arch": "$(uname -m)",
        "server_port": $SERVER_PORT
    },
    "results": {
        "total_tests": $TOTAL_TESTS,
        "passed": $PASSED_TESTS,
        "failed": $FAILED_TESTS,
        "success_rate": "$(echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l)%"
    },
    "files": {
        "logs_directory": "$REPORTS_DIR",
        "server_log": "server.log"
    }
}
EOF

    log_success "Test report generated: $REPORT_FILE"
}

# Main execution
main() {
    TEST_TYPE="${1:-all}"
    TOTAL_TESTS=0
    PASSED_TESTS=0
    FAILED_TESTS=0
    
    echo "========================================="
    echo "  RainbowBrowserAI Test Suite Runner"
    echo "========================================="
    echo "Test Type: $TEST_TYPE"
    echo "Timestamp: $(date)"
    echo "Reports Dir: $REPORTS_DIR"
    echo "========================================="
    echo ""
    
    # Start server
    if ! start_server; then
        log_error "Failed to start server, aborting tests"
        exit 1
    fi
    
    # Run tests based on type
    case "$TEST_TYPE" in
        "all")
            run_unit_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_api_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_ui_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_browser_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_tools_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_perception_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_integration_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_performance_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            run_e2e_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "unit")
            run_unit_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "integration")
            run_integration_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "api")
            run_api_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "ui")
            run_ui_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "perception")
            run_perception_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "browser")
            run_browser_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "tools")
            run_tools_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "performance")
            run_performance_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        "e2e")
            run_e2e_tests && ((PASSED_TESTS++)) || ((FAILED_TESTS++)); ((TOTAL_TESTS++))
            ;;
        *)
            log_error "Unknown test type: $TEST_TYPE"
            echo "Available types: all, unit, integration, api, ui, perception, browser, tools, performance, e2e"
            exit 1
            ;;
    esac
    
    # Generate report
    generate_report
    
    # Summary
    echo ""
    echo "========================================="
    echo "           TEST SUMMARY"
    echo "========================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo "Passed: $PASSED_TESTS"
    echo "Failed: $FAILED_TESTS"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_success "All tests passed! 🎉"
        exit 0
    else
        log_error "$FAILED_TESTS test(s) failed"
        exit 1
    fi
}

# Run main function
main "$@"