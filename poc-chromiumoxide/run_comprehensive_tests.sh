#!/bin/bash

# Comprehensive Test Runner for RainbowBrowserAI Chromiumoxide Edition
# This is the main entry point for all testing activities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$PROJECT_DIR/tests"
REPORTS_DIR="$TEST_DIR/reports"
LOG_FILE="$REPORTS_DIR/comprehensive_test_$(date +%Y%m%d_%H%M%S).log"

# Test categories
declare -A TEST_SUITES=(
    ["api"]="API Endpoint Tests"
    ["ui"]="User Interface Tests"  
    ["browser"]="Browser Automation Tests"
    ["tools"]="Tools Functionality Tests"
    ["perception"]="Perception Module Tests"
    ["performance"]="Performance and Load Tests"
    ["e2e"]="End-to-End Workflow Tests"
    ["integration"]="Integration Tests"
)

# Results tracking
TOTAL_SUITES=0
PASSED_SUITES=0
FAILED_SUITES=0
DETAILED_RESULTS=()

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

print_header() {
    local title="$1"
    echo "" | tee -a "$LOG_FILE"
    echo "==========================================" | tee -a "$LOG_FILE"
    echo -e "${PURPLE}$title${NC}" | tee -a "$LOG_FILE"
    echo "==========================================" | tee -a "$LOG_FILE"
}

print_section() {
    local section="$1"
    echo "" | tee -a "$LOG_FILE"
    echo -e "${CYAN}--- $section ---${NC}" | tee -a "$LOG_FILE"
}

# Server management
start_test_server() {
    log_info "Starting test server..."
    
    # Kill any existing servers
    pkill -f "serve --port 3002" 2>/dev/null || true
    sleep 2
    
    # Start server in background
    cd "$PROJECT_DIR"
    RAINBOW_MOCK_MODE=true cargo run --release -- serve --port 3002 --headless > "$REPORTS_DIR/server.log" 2>&1 &
    SERVER_PID=$!
    
    # Wait for server to be ready
    log_info "Waiting for server to start (PID: $SERVER_PID)..."
    for i in {1..30}; do
        if curl -s http://localhost:3002/api/health > /dev/null 2>&1; then
            log_success "Test server is ready"
            return 0
        fi
        sleep 1
    done
    
    log_error "Server failed to start within 30 seconds"
    return 1
}

stop_test_server() {
    if [ -n "$SERVER_PID" ]; then
        log_info "Stopping test server (PID: $SERVER_PID)..."
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Test execution functions
run_test_suite() {
    local suite_name="$1"
    local suite_description="$2"
    local script_path="$3"
    
    print_section "Running $suite_description"
    
    local start_time=$(date +%s)
    
    if [ -f "$script_path" ] && [ -x "$script_path" ]; then
        log_info "Executing: $script_path"
        
        if "$script_path" > "$REPORTS_DIR/${suite_name}_test.log" 2>&1; then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "$suite_description completed successfully (${duration}s)"
            DETAILED_RESULTS+=("✅ $suite_description - PASSED (${duration}s)")
            ((PASSED_SUITES++))
            return 0
        else
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_error "$suite_description failed (${duration}s)"
            DETAILED_RESULTS+=("❌ $suite_description - FAILED (${duration}s)")
            ((FAILED_SUITES++))
            return 1
        fi
    else
        log_error "Test script not found or not executable: $script_path"
        DETAILED_RESULTS+=("❌ $suite_description - SCRIPT NOT FOUND")
        ((FAILED_SUITES++))
        return 1
    fi
}

# Pre-flight checks
pre_flight_checks() {
    print_section "Pre-flight Checks"
    
    # Check required tools
    local required_tools=("curl" "jq" "bc")
    for tool in "${required_tools[@]}"; do
        if command -v "$tool" &> /dev/null; then
            log_success "$tool is available"
        else
            log_error "$tool is required but not installed"
            return 1
        fi
    done
    
    # Check project structure
    if [ -d "$TEST_DIR" ]; then
        log_success "Test directory structure exists"
    else
        log_error "Test directory not found: $TEST_DIR"
        return 1
    fi
    
    # Check if we can build the project
    log_info "Verifying project can be built..."
    cd "$PROJECT_DIR"
    if RAINBOW_MOCK_MODE=true timeout 60s cargo check --message-format short > "$REPORTS_DIR/build_check.log" 2>&1; then
        log_success "Project builds successfully"
    else
        log_error "Project build failed - see $REPORTS_DIR/build_check.log"
        return 1
    fi
    
    return 0
}

# Main execution
main() {
    local test_filter="${1:-all}"
    
    # Create reports directory
    mkdir -p "$REPORTS_DIR"
    
    # Initialize log
    echo "Comprehensive Test Run - $(date)" > "$LOG_FILE"
    echo "Filter: $test_filter" >> "$LOG_FILE"
    echo "Project: $PROJECT_DIR" >> "$LOG_FILE"
    echo "=========================================" >> "$LOG_FILE"
    
    print_header "RainbowBrowserAI Comprehensive Test Suite"
    log_info "Starting comprehensive test run with filter: $test_filter"
    log_info "Project Directory: $PROJECT_DIR"
    log_info "Test Directory: $TEST_DIR"
    log_info "Reports Directory: $REPORTS_DIR"
    log_info "Log File: $LOG_FILE"
    
    # Cleanup function
    cleanup() {
        log_info "Cleaning up..."
        stop_test_server
        log_info "Cleanup complete"
    }
    trap cleanup EXIT
    
    # Pre-flight checks
    if ! pre_flight_checks; then
        log_error "Pre-flight checks failed"
        exit 1
    fi
    
    # Start test server
    if ! start_test_server; then
        log_error "Failed to start test server"
        exit 1
    fi
    
    # Run test suites based on filter
    print_header "Test Suite Execution"
    
    if [ "$test_filter" = "all" ] || [ "$test_filter" = "quick" ]; then
        # Run all test suites or quick subset
        local suites_to_run
        if [ "$test_filter" = "quick" ]; then
            suites_to_run=("api" "ui" "tools")
            log_info "Running quick test suite (API, UI, Tools)"
        else
            suites_to_run=("api" "ui" "browser" "tools" "perception" "integration" "performance" "e2e")
            log_info "Running complete test suite"
        fi
        
        for suite in "${suites_to_run[@]}"; do
            if [[ -n "${TEST_SUITES[$suite]}" ]]; then
                local script_path
                case "$suite" in
                    "integration")
                        # Run both integration test scripts
                        run_test_suite "api_integration" "API Integration Tests" "$TEST_DIR/integration/test_api_integration.sh"
                        ((TOTAL_SUITES++))
                        run_test_suite "browser_integration" "Browser Integration Tests" "$TEST_DIR/integration/test_browser_integration.sh"
                        ((TOTAL_SUITES++))
                        continue
                        ;;
                    *)
                        script_path="$TEST_DIR/$suite/test_${suite}_suite.sh"
                        if [ "$suite" = "api" ]; then
                            script_path="$TEST_DIR/$suite/test_all_endpoints.sh"
                        elif [ "$suite" = "ui" ]; then
                            script_path="$TEST_DIR/$suite/test_interface.sh"
                        elif [ "$suite" = "tools" ]; then
                            script_path="$TEST_DIR/$suite/test_all_tools.sh"
                        elif [ "$suite" = "perception" ]; then
                            script_path="$TEST_DIR/$suite/test_perception_suite.sh"
                        fi
                        ;;
                esac
                
                run_test_suite "$suite" "${TEST_SUITES[$suite]}" "$script_path"
                ((TOTAL_SUITES++))
            fi
        done
    else
        # Run specific test suite
        if [[ -n "${TEST_SUITES[$test_filter]}" ]]; then
            local script_path="$TEST_DIR/$test_filter/test_${test_filter}_suite.sh"
            case "$test_filter" in
                "api") script_path="$TEST_DIR/$test_filter/test_all_endpoints.sh" ;;
                "ui") script_path="$TEST_DIR/$test_filter/test_interface.sh" ;;
                "tools") script_path="$TEST_DIR/$test_filter/test_all_tools.sh" ;;
                "perception") script_path="$TEST_DIR/$test_filter/test_perception_suite.sh" ;;
                "integration")
                    run_test_suite "api_integration" "API Integration Tests" "$TEST_DIR/integration/test_api_integration.sh"
                    ((TOTAL_SUITES++))
                    run_test_suite "browser_integration" "Browser Integration Tests" "$TEST_DIR/integration/test_browser_integration.sh"
                    ((TOTAL_SUITES++))
                    ;;
                *) script_path="$TEST_DIR/$test_filter/test_${test_filter}_suite.sh" ;;
            esac
            
            if [ "$test_filter" != "integration" ]; then
                run_test_suite "$test_filter" "${TEST_SUITES[$test_filter]}" "$script_path"
                ((TOTAL_SUITES++))
            fi
        else
            log_error "Unknown test filter: $test_filter"
            log_info "Available filters: all, quick, ${!TEST_SUITES[*]}"
            exit 1
        fi
    fi
    
    # Generate comprehensive report
    print_header "Test Results Summary"
    
    local success_rate=0
    if [ $TOTAL_SUITES -gt 0 ]; then
        success_rate=$(echo "scale=2; $PASSED_SUITES * 100 / $TOTAL_SUITES" | bc -l)
    fi
    
    echo "" | tee -a "$LOG_FILE"
    echo "Total Test Suites: $TOTAL_SUITES" | tee -a "$LOG_FILE"
    echo -e "${GREEN}Passed: $PASSED_SUITES${NC}" | tee -a "$LOG_FILE"
    echo -e "${RED}Failed: $FAILED_SUITES${NC}" | tee -a "$LOG_FILE"
    echo "Success Rate: ${success_rate}%" | tee -a "$LOG_FILE"
    
    echo "" | tee -a "$LOG_FILE"
    echo "Detailed Results:" | tee -a "$LOG_FILE"
    for result in "${DETAILED_RESULTS[@]}"; do
        echo "  $result" | tee -a "$LOG_FILE"
    done
    
    # Report file locations
    echo "" | tee -a "$LOG_FILE"
    echo "Reports and Logs:" | tee -a "$LOG_FILE"
    echo "  Main Log: $LOG_FILE" | tee -a "$LOG_FILE"
    echo "  Reports Directory: $REPORTS_DIR" | tee -a "$LOG_FILE"
    find "$REPORTS_DIR" -name "*.log" -newer "$LOG_FILE" | while read -r logfile; do
        echo "  $(basename "$logfile"): $logfile" | tee -a "$LOG_FILE"
    done
    
    # Final result
    if [ $FAILED_SUITES -eq 0 ]; then
        echo "" | tee -a "$LOG_FILE"
        log_success "🎉 All test suites passed! System is ready for deployment."
        exit 0
    else
        echo "" | tee -a "$LOG_FILE"
        log_error "❌ $FAILED_SUITES test suite(s) failed. Please review the detailed logs."
        log_info "💡 Use 'run_comprehensive_tests.sh quick' for faster feedback during development"
        exit 1
    fi
}

# Show usage if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "RainbowBrowserAI Comprehensive Test Runner"
    echo ""
    echo "Usage: $0 [test_filter]"
    echo ""
    echo "Test Filters:"
    echo "  all          - Run all test suites (default)"
    echo "  quick        - Run essential tests (API, UI, Tools)"
    for key in "${!TEST_SUITES[@]}"; do
        echo "  $key          - ${TEST_SUITES[$key]}"
    done
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 quick             # Run essential tests only"
    echo "  $0 perception        # Run perception tests only"
    echo "  $0 performance       # Run performance tests only"
    exit 0
fi

# Run main function
main "$@"