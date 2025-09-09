#\!/bin/bash

echo "=== Testing Advanced Browser Interactions ==="
echo

# Test navigation to a form page
echo "1. Navigating to w3schools form example..."
curl -s -X POST http://localhost:3001/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.w3schools.com/html/html_forms.asp"}' | python3 -m json.tool 2>/dev/null

sleep 2

# Test clicking a button
echo -e "\n2. Testing click action on Try it Yourself button..."
curl -s -X POST http://localhost:3001/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.example.com"}' | python3 -m json.tool 2>/dev/null

sleep 1

# Navigate to a page with dropdowns
echo -e "\n3. Testing form interactions..."
curl -s -X POST http://localhost:3001/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select"}' | python3 -m json.tool 2>/dev/null

echo -e "\n=== Advanced Interaction Test Complete ==="

# Final metrics check
echo -e "\n4. Final metrics check..."
curl -s http://localhost:3001/metrics | python3 -m json.tool
