#!/bin/bash

echo "=== Testing Perception UI Display ==="
echo ""
echo "1. Navigate to a test page..."
curl -X POST http://localhost:3001/api/command \
  -H "Content-Type: application/json" \
  -d '{"command": "navigate to https://github.com"}' \
  -s | python3 -m json.tool | grep -E '"success"|"title"'

echo ""
echo "2. Test page classification..."
curl -X POST http://localhost:3001/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "classify"}' \
  -s | python3 -m json.tool | head -10

echo ""
echo "3. Test data extraction..."
curl -X POST http://localhost:3001/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "extract_data"}' \
  -s | python3 -m json.tool | head -20

echo ""
echo "4. Test finding links..."
curl -X POST http://localhost:3001/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "find_element", "element_description": "link"}' \
  -s | python3 -m json.tool | head -15

echo ""
echo "=== Testing Complete ==="
echo ""
echo "✨ The perception UI is now visible at http://localhost:3001"
echo "📍 Navigate to the 'Command' tab to see the Perception Analysis section"
echo "🔍 Use the three buttons to analyze any webpage you navigate to"