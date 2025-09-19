#!/bin/bash

# Test script for perception of tool operation interfaces

echo "====================================="
echo "Tool Operation Interface Perception Test"
echo "====================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Start the server
print_info "Starting server..."
cargo build --release
cargo run --release &
SERVER_PID=$!
sleep 5

# Test URL with various tool interfaces
TEST_URL="https://www.example.com"

echo ""
echo "Test 1: Navigate and perceive with tool registry context"
echo "---------------------------------------------------------"
curl -X POST http://localhost:3000/api/perception/navigate-and-perceive \
  -H "Content-Type: application/json" \
  -d '{
    "url": "'$TEST_URL'",
    "mode": "standard"
  }' | jq '.'

echo ""
echo "Test 2: Analyze page for tool-operable elements"
echo "------------------------------------------------"
curl -X POST http://localhost:3000/api/perception/analyze-page \
  -H "Content-Type: application/json" \
  -d '{
    "url": "'$TEST_URL'"
  }' | jq '.'

echo ""
echo "Test 3: Smart element search for interactive components"
echo "-------------------------------------------------------"
curl -X POST http://localhost:3000/api/perception/smart-search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "button",
    "max_results": 5
  }' | jq '.'

echo ""
echo "Test 4: Test tool list availability"
echo "------------------------------------"
curl http://localhost:3000/api/tools/list | jq '.'

echo ""
echo "Test 5: Execute tool with perception context"
echo "--------------------------------------------"
curl -X POST http://localhost:3000/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{
    "name": "extract_links",
    "input": {}
  }' | jq '.'

echo ""
echo "Test 6: Test perception of form elements (tool-operable)"
echo "---------------------------------------------------------"
curl -X POST http://localhost:3000/api/perception/analyze-form \
  -H "Content-Type: application/json" \
  -d '{}' | jq '.'

echo ""
echo "Test 7: Test intelligent action recognition"
echo "-------------------------------------------"
curl -X POST http://localhost:3000/api/perception/intelligent-command \
  -H "Content-Type: application/json" \
  -d '{
    "command": {
      "action": "click",
      "target": "submit button"
    }
  }' | jq '.'

# Clean up
echo ""
print_info "Cleaning up..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

print_success "Test completed!"