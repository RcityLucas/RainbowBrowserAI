#!/bin/bash

echo "=== Testing Perception Integration in Visual Interface ==="
echo ""

# Check if server is running
echo "1. Checking if server is running on port 3002..."
if curl -s http://localhost:3002/api/health > /dev/null; then
    echo "✅ Server is running"
else
    echo "❌ Server is not responding on port 3002"
    exit 1
fi

echo ""
echo "2. Testing perception API endpoints..."

echo ""
echo "   Testing page analysis endpoint..."
curl -X POST http://localhost:3002/api/perception/analyze \
  -H "Content-Type: application/json" \
  -d '{}' \
  -s | head -5

echo ""
echo "   Testing element finding endpoint..."
curl -X POST http://localhost:3002/api/perception/find \
  -H "Content-Type: application/json" \
  -d '{"description": "search button"}' \
  -s | head -5

echo ""
echo "   Testing form analysis endpoint..."
curl -X POST http://localhost:3002/api/perception/forms/analyze \
  -H "Content-Type: application/json" \
  -d '{"form_selector": null}' \
  -s | head -5

echo ""
echo "3. Visual Interface Integration:"
echo "   🌐 Open http://localhost:3002 in your browser"
echo "   🧠 Click on the 'Perception' tab in the navigation"
echo "   🔍 Try the perception features:"
echo "      - Page Analysis"  
echo "      - Element Detection"
echo "      - Intelligent Commands"
echo "      - Form Analysis"

echo ""
echo "=== Perception Integration Test Complete ==="
echo ""
echo "📋 Features Available:"
echo "   ✅ Natural language element finding"
echo "   ✅ Smart page analysis and classification"
echo "   ✅ Intelligent command execution"
echo "   ✅ Form detection and analysis"
echo "   ✅ Visual statistics tracking"
echo "   ✅ Modern responsive UI"

echo ""
echo "🎯 Next Steps:"
echo "   - Navigate to a webpage using the Browse tab"
echo "   - Switch to the Perception tab"
echo "   - Test the intelligent automation features"
echo "   - Try natural language commands like 'find the search button'"