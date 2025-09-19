#!/bin/bash

# Script to safely start the refactoring process

echo "====================================="
echo "RainbowBrowserAI Refactoring Helper"
echo "====================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Step 1: Check git status
echo -e "${BLUE}Step 1: Checking git status...${NC}"
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "${YELLOW}⚠ You have uncommitted changes. Please commit or stash them first.${NC}"
    echo -e "Run: git add . && git commit -m 'Save work before refactoring'"
    exit 1
fi
echo -e "${GREEN}✓ Working directory clean${NC}"

# Step 2: Create backup branch
echo -e "\n${BLUE}Step 2: Creating backup branch...${NC}"
git checkout -b backup-$(date +%Y%m%d-%H%M%S) 2>/dev/null || {
    echo -e "${YELLOW}Backup branch may already exist, continuing...${NC}"
}
git checkout -b refactoring-work 2>/dev/null || {
    echo -e "${YELLOW}Already on refactoring branch${NC}"
}
echo -e "${GREEN}✓ Backup created${NC}"

# Step 3: Analyze current state
echo -e "\n${BLUE}Step 3: Analyzing project...${NC}"

# Count warnings
WARNING_COUNT=$(cargo build 2>&1 | grep -c warning || echo 0)
echo -e "Current warnings: ${YELLOW}$WARNING_COUNT${NC}"

# Count lines of code
echo -e "\nLines of code by module:"
for dir in src/*/; do
    if [ -d "$dir" ]; then
        module=$(basename "$dir")
        lines=$(find "$dir" -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
        echo -e "  $module: $lines lines"
    fi
done

# Find duplicate implementations
echo -e "\n${BLUE}Step 4: Finding duplicates...${NC}"

echo -e "\nTool Registry implementations:"
grep -r "struct.*ToolRegistry" src/ --include="*.rs" | grep -v "^Binary file"

echo -e "\nPerception Engine implementations:"
grep -r "struct.*PerceptionEngine" src/ --include="*.rs" | grep -v "^Binary file"

echo -e "\nBrowser management:"
grep -r "struct.*Browser.*Pool\|Browser.*Manager\|Browser.*Context" src/ --include="*.rs" | grep -v "^Binary file"

# Step 5: Quick fixes
echo -e "\n${BLUE}Step 5: Quick fixes available...${NC}"
echo -e "${YELLOW}Run these commands for quick wins:${NC}"
echo ""
echo "# Auto-fix imports and simple issues:"
echo "cargo fix --allow-dirty"
echo ""
echo "# Apply clippy suggestions:"
echo "cargo clippy --fix --allow-dirty"
echo ""
echo "# Format code:"
echo "cargo fmt"
echo ""

# Step 6: Generate dead code report
echo -e "${BLUE}Step 6: Generating dead code report...${NC}"
cargo build 2>&1 | grep -E "warning:.*never|warning:.*unused" > dead_code_report.txt || true
if [ -s dead_code_report.txt ]; then
    echo -e "${GREEN}✓ Dead code report saved to dead_code_report.txt${NC}"
    echo -e "  Found $(wc -l < dead_code_report.txt) dead code warnings"
else
    echo -e "${YELLOW}No dead code warnings found${NC}"
fi

# Step 7: Recommendations
echo -e "\n${BLUE}Step 7: Recommended next steps:${NC}"
echo -e "${GREEN}1.${NC} Start with removing dead code:"
echo "   - Review dead_code_report.txt"
echo "   - Delete unused structs and functions"
echo ""
echo -e "${GREEN}2.${NC} Remove the coordination module:"
echo "   - Extract any unique features first"
echo "   - Move session management to src/session/"
echo "   - Delete src/coordination/"
echo ""
echo -e "${GREEN}3.${NC} Consolidate tool registries:"
echo "   - Keep src/tools/registry.rs"
echo "   - Remove duplicates"
echo ""
echo -e "${GREEN}4.${NC} Simplify API:"
echo "   - Consolidate perception endpoints"
echo "   - Remove duplicate handlers"
echo ""

echo -e "\n${YELLOW}Safety tip: Test after each major change!${NC}"
echo -e "Run: ${BLUE}cargo test && ./start.sh --headless${NC}"
echo ""
echo -e "${GREEN}Refactoring preparation complete!${NC}"