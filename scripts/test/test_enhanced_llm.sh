#!/bin/bash

# Test script for enhanced LLM functionality

cd /mnt/d/github/RainbowBrowserAI/poc

echo "Testing enhanced LLM service with complex commands..."
echo ""
echo "Setting mock mode..."
export RAINBOW_MOCK_MODE=true

echo ""
echo "Test 1: Travel plan request"
echo "Command: 'give me a travel plan'"
timeout 10s cargo run -- ask "give me a travel plan" 2>/dev/null

echo ""
echo "Test 2: Search request"
echo "Command: 'search for best restaurants in Tokyo'"
timeout 10s cargo run -- ask "search for best restaurants in Tokyo" 2>/dev/null

echo ""
echo "Test 3: Basic navigation (should still work)"
echo "Command: 'go to stackoverflow and take screenshot'"
timeout 10s cargo run -- ask "go to stackoverflow and take screenshot" 2>/dev/null

echo ""
echo "Done!"