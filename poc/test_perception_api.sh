#!/bin/bash

# Test script for perception API endpoints

echo "Testing perception API endpoints..."

# Start the service if not running
if ! pgrep -f "rainbow-poc serve" > /dev/null; then
    echo "Starting service on port 3002..."
    RAINBOW_MOCK_MODE=true ./target/debug/rainbow-poc serve --port 3002 &
    SERVICE_PID=$!
    sleep 5
else
    echo "Service already running"
fi

# Test health endpoint
echo -e "\n1. Testing health endpoint:"
curl -s http://localhost:3002/api/health | python3 -m json.tool

# Test perception classify with simple perception
echo -e "\n2. Testing perception classify (simple):"
curl -s -X POST http://localhost:3002/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "classify", "url": "https://example.com", "use_simple": true}' \
  | python3 -m json.tool 2>/dev/null || echo "Endpoint not available"

# Test perception classify with MVP perception
echo -e "\n3. Testing perception classify (MVP):"
curl -s -X POST http://localhost:3002/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "classify", "url": "https://example.com", "use_simple": false}' \
  | python3 -m json.tool 2>/dev/null || echo "Endpoint not available"

# Test perception find element
echo -e "\n4. Testing perception find element:"
curl -s -X POST http://localhost:3002/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "find_element", "url": "https://example.com", "element_description": "title", "use_simple": false}' \
  | python3 -m json.tool 2>/dev/null || echo "Endpoint not available"

# Test perception data extraction
echo -e "\n5. Testing perception extract data:"
curl -s -X POST http://localhost:3002/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "extract_data", "url": "https://example.com", "use_simple": false}' \
  | python3 -m json.tool 2>/dev/null || echo "Endpoint not available"

# Test perception test endpoint
echo -e "\n6. Testing perception test endpoint:"
curl -s http://localhost:3002/api/perception/test \
  | python3 -m json.tool 2>/dev/null || echo "Endpoint not available"

echo -e "\nPerception API test complete!"

# Kill the service if we started it
if [ ! -z "$SERVICE_PID" ]; then
    echo "Stopping service..."
    kill $SERVICE_PID 2>/dev/null
fi