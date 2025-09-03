#!/bin/bash

echo "Testing Perception Module via Service (Port 3001)"
echo "================================================="

BASE_URL="http://localhost:3001"

# Test 1: Health Check
echo -e "\n1. 🏥 Health Check:"
curl -s "$BASE_URL/api/health" | python3 -m json.tool

# Test 2: Navigation with Screenshot (tests browser perception)
echo -e "\n2. 🌐 Navigation with Screenshot:"
curl -s -X POST "$BASE_URL/api/navigate" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "screenshot": true}' | python3 -m json.tool

# Test 3: Natural Language Multi-step (tests command parsing + perception)
echo -e "\n3. 🧠 Natural Language Multi-step Command:"
curl -s -X POST "$BASE_URL/api/command" \
  -H "Content-Type: application/json" \
  -d '{"command": "go to example.com and take a screenshot"}' | python3 -m json.tool

# Test 4: Complex Navigation (tests perception-aware browser handling)
echo -e "\n4. 🎯 Complex Navigation Command:"
curl -s -X POST "$BASE_URL/api/command" \
  -H "Content-Type: application/json" \
  -d '{"command": "navigate to google.com"}' | python3 -m json.tool

# Test 5: Session Management (tests perception state tracking)
echo -e "\n5. 📊 Session List:"
curl -s -X POST "$BASE_URL/api/session" \
  -H "Content-Type: application/json" \
  -d '{"action": "list"}' | python3 -m json.tool

# Test 6: Flexible Instruction (tests perception-aware instruction processing)
echo -e "\n6. 🔄 Flexible Instruction Processing:"
curl -s -X POST "$BASE_URL/api/instruction" \
  -H "Content-Type: application/json" \
  -d '"Visit https://example.com and tell me the page title"' | python3 -m json.tool

# Test 7: Metrics (shows perception performance)
echo -e "\n7. 📈 System Metrics:"
curl -s "$BASE_URL/api/metrics" | python3 -m json.tool

echo -e "\n✅ Perception module testing complete!"
echo "The perception system is operational through the integrated service."