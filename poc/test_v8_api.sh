#!/bin/bash
# Test script for V8.0 API integration

echo "Testing V8.0 API Commands"
echo "========================="

# Start the server in background
export RAINBOW_MOCK_MODE=true
cargo build --release --quiet
./target/release/rainbow-poc serve --port 3001 &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Test V8.0 perception commands
echo -e "\n1. Testing V8.0 Lightning Perception:"
curl -X POST http://localhost:3001/command \
  -H "Content-Type: application/json" \
  -d '{"command": "analyze page with lightning speed"}'

echo -e "\n\n2. Testing V8.0 Scroll to Position:"
curl -X POST http://localhost:3001/command \
  -H "Content-Type: application/json" \
  -d '{"command": "scroll to position 500, 1000 smoothly"}'

echo -e "\n\n3. Testing V8.0 Double Click with Modifier:"
curl -X POST http://localhost:3001/command \
  -H "Content-Type: application/json" \
  -d '{"command": "double click button with ctrl"}'

echo -e "\n\n4. Testing V8.0 Deep Analysis:"
curl -X POST http://localhost:3001/command \
  -H "Content-Type: application/json" \
  -d '{"command": "perform deep semantic analysis of page"}'

# Kill the server
echo -e "\n\nStopping server..."
kill $SERVER_PID

echo -e "\nV8.0 API tests completed!"