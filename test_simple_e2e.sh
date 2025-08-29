#\!/bin/bash

echo "=== Simple E2E Workflow Test ==="
echo

echo "1. Testing navigation..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "navigate", "parameters": {"url": "https://example.com"}}'
echo -e "\n"

sleep 2

echo "2. Testing screenshot..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "take_screenshot", "parameters": {}}'
echo -e "\n"

echo "3. Testing wait for element..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "wait_for_element", "parameters": {"selector": "h1"}}'
echo -e "\n"

echo "4. Testing text extraction..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "extract_text", "parameters": {"selector": "h1"}}'
echo -e "\n"

echo "5. Testing click action..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "click", "parameters": {"selector": "a[href=\"https://www.iana.org/domains/example\"]"}}'
echo -e "\n"

echo "=== Test Complete ==="
