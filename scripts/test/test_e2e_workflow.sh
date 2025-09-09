#\!/bin/bash

echo "=== End-to-End Workflow Test ==="
echo "Testing complete browser automation workflow..."
echo

# Test navigation and screenshot
echo "1. Testing navigation to example.com..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "navigate",
    "parameters": {
      "url": "https://example.com"
    }
  }' | jq '.'

sleep 2

echo -e "\n2. Testing screenshot capture..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "take_screenshot",
    "parameters": {}
  }' | jq '.'

echo -e "\n3. Testing element waiting..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "wait_for_element",
    "parameters": {
      "selector": "h1"
    }
  }' | jq '.'

echo -e "\n4. Testing text extraction..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "extract_text",
    "parameters": {
      "selector": "h1"
    }
  }' | jq '.'

echo -e "\n5. Testing link extraction..."
curl -X POST http://localhost:3001/execute \
  -H "Content-Type: application/json" \
  -d '{
    "command": "extract_links",
    "parameters": {}
  }' | jq '.'

echo -e "\n=== Workflow Test Complete ==="
