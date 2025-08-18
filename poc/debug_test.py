#!/usr/bin/env python3
import requests
import json
import time
import os

# Set environment variable to ensure mock mode
os.environ['RAINBOW_MOCK_MODE'] = 'true'

def test_current_server():
    print("🔍 Testing current server response (assuming it's running)")
    print("=" * 50)
    
    try:
        # Test the batch command
        print("🚀 Testing batch command...")
        url = "http://localhost:3000/command"
        data = {
            "command": "test google,github with screenshots"
        }
        
        response = requests.post(url, json=data, timeout=10)
        
        print(f"📊 Response Status: {response.status_code}")
        
        if response.status_code == 200:
            try:
                result = response.json()
                print("📄 Response Content:")
                print("-" * 30)
                print(json.dumps(result, indent=2))
                
                # Check the result structure
                if "result" in result and isinstance(result["result"], dict):
                    inner_result = result["result"]
                    
                    if "message" in inner_result and "not executed in mock mode" in inner_result.get("message", ""):
                        print("\n❌ ISSUE: Still showing old mock message!")
                        print("The fix may not have been applied or the server needs restart.")
                        return False
                    elif "results" in inner_result or "total_tests" in inner_result:
                        print("\n✅ SUCCESS: Batch testing appears to be working!")
                        return True
                    elif inner_result.get("action") == "test" and "results" not in inner_result:
                        print("\n❓ PARTIAL: Test action recognized but format unclear")
                        return False
                        
                print("\n❓ UNKNOWN: Unexpected response format")
                return False
                    
            except json.JSONDecodeError:
                print("❌ Invalid JSON response")
                print("Raw response:", response.text)
                return False
        else:
            print(f"❌ HTTP Error: {response.status_code}")
            print("Response:", response.text)
            return False
            
    except requests.RequestException as e:
        print(f"❌ Request failed: {e}")
        print("Make sure the server is running on localhost:3000")
        return False

if __name__ == "__main__":
    success = test_current_server()
    
    if not success:
        print("\n💡 TROUBLESHOOTING:")
        print("1. Make sure server is running: cargo run --bin rainbow-poc")
        print("2. Set environment: RAINBOW_MOCK_MODE=true")
        print("3. Restart server if it was already running")
        print("4. Check logs for any errors")