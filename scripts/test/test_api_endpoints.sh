#\!/bin/bash

echo "=== Testing RainbowBrowserAI API Endpoints ==="
echo

echo "1. Testing navigate endpoint..."
curl -s -X POST http://localhost:3001/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}' | python3 -m json.tool 2>/dev/null || echo "Navigate response received"

sleep 2

echo -e "\n2. Testing screenshot endpoint..."
curl -s -X POST http://localhost:3001/screenshot \
  -H "Content-Type: application/json" \
  -d '{}' | head -c 100

echo -e "\n\n3. Testing health check..."
curl -s http://localhost:3001/health | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3001/health

echo -e "\n\n4. Testing metrics endpoint..."
curl -s http://localhost:3001/metrics | python3 -m json.tool 2>/dev/null || curl -s http://localhost:3001/metrics

echo -e "\n\n=== API Test Complete ==="
