#!/bin/bash

# RainbowBrowserAI Cleanup Script
# Removes temporary files and build artifacts

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}       🧹 RainbowBrowserAI Cleanup Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Function to safely remove files
safe_remove() {
    local file=$1
    if [ -f "$file" ]; then
        rm -f "$file"
        echo -e "${GREEN}  ✓ Removed: $file${NC}"
    fi
}

# Function to check file size
check_size() {
    local path=$1
    if [ -e "$path" ]; then
        local size=$(du -sh "$path" 2>/dev/null | cut -f1)
        echo "$size"
    else
        echo "0"
    fi
}

# Track space freed
INITIAL_SIZE=$(du -sh . 2>/dev/null | cut -f1)

echo -e "${YELLOW}🔍 Analyzing temporary files...${NC}"
echo ""

# 1. Remove test executables and debug symbols
echo -e "${BLUE}Cleaning test executables...${NC}"
safe_remove "perception_validation.exe"
safe_remove "perception_validation.pdb"
safe_remove "standalone_perception.exe"
safe_remove "standalone_perception.pdb"

# 2. Clean tmp files
echo -e "\n${BLUE}Cleaning temporary files...${NC}"
safe_remove "/tmp/chromedriver.log"
safe_remove "/tmp/last_response.json"
for file in /tmp/concurrent_*.txt; do
    safe_remove "$file"
done

# 3. Clean local logs
echo -e "\n${BLUE}Cleaning log files...${NC}"
safe_remove "chromedriver.log"
safe_remove "geckodriver.log"
safe_remove "perception_performance.log"
safe_remove "perception_test_results.json"
safe_remove "perception_test_report.json"

# 4. Optional: Clean Python cache
echo -e "\n${BLUE}Cleaning Python cache...${NC}"
find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null
find . -name "*.pyc" -delete 2>/dev/null
find . -name "*.pyo" -delete 2>/dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}  ✓ Python cache cleaned${NC}"
fi

# 5. Check for old ChromeDriver versions
echo -e "\n${BLUE}Checking ChromeDriver versions...${NC}"
if [ -f "chromedriver_v120.exe" ] && [ -f "chromedriver.exe" ]; then
    echo -e "${YELLOW}  ⚠ Found multiple ChromeDriver versions:${NC}"
    echo -e "    - chromedriver.exe ($(check_size chromedriver.exe))"
    echo -e "    - chromedriver_v120.exe ($(check_size chromedriver_v120.exe))"
    echo -e "${YELLOW}  Consider keeping only the latest version${NC}"
fi

# 6. Report on build artifacts
echo -e "\n${BLUE}Build artifacts analysis...${NC}"
if [ -d "target" ]; then
    TARGET_SIZE=$(check_size "target")
    echo -e "${YELLOW}  📦 target/ directory: $TARGET_SIZE${NC}"
    echo -e "  To clean build artifacts, run: ${GREEN}cargo clean${NC}"
    
    # Check age of build
    if [ -f "target/release/rainbow-poc" ]; then
        LAST_BUILD=$(stat -c %y "target/release/rainbow-poc" 2>/dev/null | cut -d' ' -f1)
        echo -e "  Last build: $LAST_BUILD"
    fi
fi

# 7. List remaining test files
echo -e "\n${BLUE}Test files summary...${NC}"
TEST_FILES=$(ls -la test_*.{py,sh} 2>/dev/null | wc -l)
if [ $TEST_FILES -gt 0 ]; then
    echo -e "${YELLOW}  Found $TEST_FILES test files:${NC}"
    ls -la test_*.{py,sh} 2>/dev/null | while read line; do
        echo "    $line"
    done
    echo -e "${YELLOW}  Keep these for regression testing or remove if not needed${NC}"
fi

# 8. Reports and documentation
echo -e "\n${BLUE}Documentation files...${NC}"
REPORT_COUNT=$(ls -la *_REPORT.md 2>/dev/null | wc -l)
if [ $REPORT_COUNT -gt 0 ]; then
    echo -e "${GREEN}  Found $REPORT_COUNT report files (recommended to keep)${NC}"
fi

# Calculate space freed
FINAL_SIZE=$(du -sh . 2>/dev/null | cut -f1)
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Cleanup Complete!${NC}"
echo -e "  Initial size: $INITIAL_SIZE"
echo -e "  Final size: $FINAL_SIZE"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

# Offer additional cleanup options
echo ""
echo -e "${YELLOW}Additional cleanup options:${NC}"
echo -e "  1. Run '${GREEN}cargo clean${NC}' to remove all build artifacts (~700MB)"
echo -e "  2. Run '${GREEN}rm -f test_*.{py,sh}${NC}' to remove test scripts"
echo -e "  3. Run '${GREEN}rm -f *_REPORT.md${NC}' to remove reports (not recommended)"
echo ""
echo -e "${BLUE}For a deep clean, run: ${GREEN}./cleanup.sh --deep${NC}"

# Handle deep clean option
if [ "$1" == "--deep" ]; then
    echo ""
    echo -e "${RED}⚠️  Deep clean requested!${NC}"
    read -p "This will remove build artifacts and test files. Continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Running deep clean...${NC}"
        cargo clean 2>/dev/null && echo -e "${GREEN}  ✓ Build artifacts removed${NC}"
        rm -f test_*.{py,sh} 2>/dev/null && echo -e "${GREEN}  ✓ Test scripts removed${NC}"
        echo -e "${GREEN}Deep clean complete!${NC}"
    else
        echo -e "${YELLOW}Deep clean cancelled${NC}"
    fi
fi