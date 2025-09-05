#!/bin/bash

# User Interface Tests
# Tests the web interface functionality, responsiveness, and user experience

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

check_html_content() {
    local url="$1"
    local expected_content="$2" 
    local test_name="$3"
    
    response=$(curl -s "$url")
    if echo "$response" | grep -q "$expected_content"; then
        test_passed "$test_name"
        return 0
    else
        test_failed "$test_name"
        return 1
    fi
}

echo "========================================="
echo "     User Interface Tests"
echo "========================================="
echo "Server: $SERVER_URL"
echo "Timestamp: $(date)"
echo ""

# Basic HTML Structure Tests
echo "--- HTML Structure Tests ---"
check_html_content "$SERVER_URL/" "RainbowBrowserAI Dashboard" "Main page title"
check_html_content "$SERVER_URL/" "Chromiumoxide Edition" "Edition branding"
check_html_content "$SERVER_URL/" "nav-item.*data-tab" "Navigation structure"

# Navigation Tests
echo ""
echo "--- Navigation Tests ---"
check_html_content "$SERVER_URL/" 'data-tab="command"' "Tools Test tab"
check_html_content "$SERVER_URL/" 'data-tab="browse"' "Browse tab"  
check_html_content "$SERVER_URL/" 'data-tab="workflow"' "Workflow tab"
check_html_content "$SERVER_URL/" 'data-tab="sessions"' "Sessions tab"
check_html_content "$SERVER_URL/" 'data-tab="perception"' "Perception tab"
check_html_content "$SERVER_URL/" 'data-tab="settings"' "Settings tab"

# Tool Categories Tests
echo ""
echo "--- Tool Categories Tests ---"
check_html_content "$SERVER_URL/" "Navigation Tools" "Navigation tools section"
check_html_content "$SERVER_URL/" "Interaction Tools.*5" "Interaction tools section with count"
check_html_content "$SERVER_URL/" "Data Extraction Tools" "Extraction tools section" 
check_html_content "$SERVER_URL/" "Utility & Wait Tools" "Utility tools section"

# Interaction Tools Tests
echo ""
echo "--- Interaction Tools Tests ---"
check_html_content "$SERVER_URL/" "executeClickTool" "Click tool function"
check_html_content "$SERVER_URL/" "executeTypeTool" "Type tool function"
check_html_content "$SERVER_URL/" "executeHoverTool" "Hover tool function"  
check_html_content "$SERVER_URL/" "executeFocusTool" "Focus tool function"
check_html_content "$SERVER_URL/" "executeSelectOptionTool" "Select option tool function"

# Perception Interface Tests
echo ""
echo "--- Perception Interface Tests ---"
check_html_content "$SERVER_URL/" "perception-tab" "Perception tab content"
check_html_content "$SERVER_URL/" "Page Analysis" "Perception page analysis section"
check_html_content "$SERVER_URL/" "Smart Element Detection" "Perception element detection"
check_html_content "$SERVER_URL/" "Intelligent Commands" "Perception intelligent commands"
check_html_content "$SERVER_URL/" "Smart Form Analysis" "Perception form analysis"
check_html_content "$SERVER_URL/" "Perception Statistics" "Perception statistics section"

# Help System Tests
echo ""
echo "--- Help System Tests ---"
check_html_content "$SERVER_URL/" "Common Element Selectors Guide" "Help guide presence"
check_html_content "$SERVER_URL/" "helper-text" "Helper text for inputs"
check_html_content "$SERVER_URL/" "GitHub.com" "Site-specific help"
check_html_content "$SERVER_URL/" "CSS Syntax" "CSS syntax help"

# JavaScript Assets Tests  
echo ""
echo "--- JavaScript Assets Tests ---"
js_response=$(curl -s "$SERVER_URL/static/app.js")

if echo "$js_response" | grep -q "executeClickTool"; then
    test_passed "JavaScript contains tool functions"
else
    test_failed "JavaScript missing tool functions"
fi

if echo "$js_response" | grep -q "addEventListener.*click"; then
    test_passed "JavaScript has event listeners"
else
    test_failed "JavaScript missing event listeners"
fi

if echo "$js_response" | grep -q "analyzePage.*function"; then
    test_passed "JavaScript contains perception functions"
else
    test_failed "JavaScript missing perception functions"  
fi

if echo "$js_response" | grep -q "switchTab"; then
    test_passed "JavaScript has tab switching"
else
    test_failed "JavaScript missing tab switching"
fi

# CSS Assets Tests
echo ""
echo "--- CSS Assets Tests ---"
css_response=$(curl -s "$SERVER_URL/static/styles.css")

if echo "$css_response" | grep -q "perception-panel"; then
    test_passed "CSS contains perception styles"
else
    test_failed "CSS missing perception styles"
fi

if echo "$css_response" | grep -q "helper-text"; then
    test_passed "CSS contains helper text styles"
else
    test_failed "CSS missing helper text styles"
fi

if echo "$css_response" | grep -q "quick-help"; then
    test_passed "CSS contains help system styles"
else
    test_failed "CSS missing help system styles"
fi

# Form Elements Tests
echo ""
echo "--- Form Elements Tests ---"
main_html=$(curl -s "$SERVER_URL/")

# Count input fields
input_count=$(echo "$main_html" | grep -c '<input.*type=')
if [ "$input_count" -ge 10 ]; then
    test_passed "Sufficient input fields present ($input_count found)"
else
    test_failed "Insufficient input fields ($input_count found, expected >= 10)"
fi

# Count buttons
button_count=$(echo "$main_html" | grep -c '<button.*onclick=')
if [ "$button_count" -ge 15 ]; then
    test_passed "Sufficient interactive buttons ($button_count found)"
else
    test_failed "Insufficient buttons ($button_count found, expected >= 15)"
fi

# Accessibility Tests
echo ""
echo "--- Accessibility Tests ---"
check_html_content "$SERVER_URL/" 'aria-label\|role=' "ARIA attributes present"
check_html_content "$SERVER_URL/" '<label.*for=' "Form labels present"

# Check for semantic HTML
if echo "$main_html" | grep -q '<main\|<nav\|<section\|<article\|<header'; then
    test_passed "Semantic HTML elements used"
else
    test_failed "Missing semantic HTML elements"
fi

# Responsive Design Tests
echo ""  
echo "--- Responsive Design Tests ---"
if echo "$css_response" | grep -q '@media.*max-width'; then
    test_passed "CSS media queries for responsive design"
else
    test_failed "Missing responsive CSS media queries"
fi

# Icon Integration Tests
echo ""
echo "--- Icon Integration Tests ---"
check_html_content "$SERVER_URL/" "font-awesome" "Font Awesome integration"
check_html_content "$SERVER_URL/" "fas fa-" "Font Awesome icons usage"

# Performance Tests
echo ""
echo "--- Performance Tests ---"
main_page_size=$(curl -s "$SERVER_URL/" | wc -c)
if [ "$main_page_size" -lt 100000 ]; then  # 100KB
    test_passed "Main page size reasonable (${main_page_size} bytes)"
else
    test_failed "Main page too large (${main_page_size} bytes)"
fi

js_size=$(curl -s "$SERVER_URL/static/app.js" | wc -c)
if [ "$js_size" -lt 50000 ]; then  # 50KB
    test_passed "JavaScript size reasonable (${js_size} bytes)"
else  
    test_failed "JavaScript file too large (${js_size} bytes)"
fi

# Configuration Tests
echo ""
echo "--- Configuration Tests ---"
check_html_content "$SERVER_URL/" 'value.*localhost:3002' "Default API endpoint configuration"

# Summary
echo ""
echo "========================================="
echo "           UI TEST SUMMARY" 
echo "========================================="
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Success Rate: $(echo "scale=2; $PASSED * 100 / ($PASSED + $FAILED)" | bc -l)%"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "🎉 All UI tests passed!"
    exit 0
else
    echo ""
    echo "❌ $FAILED UI test(s) failed"
    exit 1
fi