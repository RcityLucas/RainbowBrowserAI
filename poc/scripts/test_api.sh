#!/bin/bash

# Test script for RainbowBrowserAI REST API

API_URL="http://localhost:3000"

echo "🌈 Testing RainbowBrowserAI REST API..."
echo "========================================="

# Test 1: Health check
echo -e "\n📍 Test 1: Health Check"
curl -s "$API_URL/health" | jq .

# Test 2: Metrics
echo -e "\n📍 Test 2: Metrics"
curl -s "$API_URL/metrics" | jq .

# Test 3: Cost report
echo -e "\n📍 Test 3: Cost Report"
curl -s "$API_URL/cost" | jq .

# Test 4: Create a session
echo -e "\n📍 Test 4: Create Session"
SESSION_RESPONSE=$(curl -s -X POST "$API_URL/session" \
  -H "Content-Type: application/json" \
  -d '{"action": "create"}')
echo "$SESSION_RESPONSE" | jq .
SESSION_ID=$(echo "$SESSION_RESPONSE" | jq -r '.session_id')
echo "Session ID: $SESSION_ID"

# Test 5: Navigate to a URL
echo -e "\n📍 Test 5: Navigate to URL"
curl -s -X POST "$API_URL/navigate" \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"https://example.com\", \"screenshot\": true, \"session_id\": \"$SESSION_ID\"}" | jq .

# Test 6: Take a screenshot
echo -e "\n📍 Test 6: Take Screenshot"
curl -s -X POST "$API_URL/screenshot" \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"https://github.com\", \"full_page\": true}" | jq .

# Test 7: List sessions
echo -e "\n📍 Test 7: List Sessions"
curl -s -X POST "$API_URL/session" \
  -H "Content-Type: application/json" \
  -d '{"action": "list"}' | jq .

# Test 8: Natural language command
echo -e "\n📍 Test 8: Natural Language Command"
curl -s -X POST "$API_URL/command" \
  -H "Content-Type: application/json" \
  -d '{"command": "navigate to google and take a screenshot"}' | jq .

# Test 9: Execute a simple workflow
echo -e "\n📍 Test 9: Execute Workflow"
curl -s -X POST "$API_URL/workflow" \
  -H "Content-Type: application/json" \
  -d '{
    "workflow": {
      "name": "test-workflow",
      "description": "Test workflow",
      "steps": [
        {
          "name": "navigate",
          "action": {
            "type": "navigate",
            "url": "https://example.com"
          }
        }
      ]
    }
  }' | jq .

# Test 10: Destroy session
echo -e "\n📍 Test 10: Destroy Session"
curl -s -X POST "$API_URL/session" \
  -H "Content-Type: application/json" \
  -d "{\"action\": \"destroy\", \"session_id\": \"$SESSION_ID\"}" | jq .

echo -e "\n✅ API tests complete!"