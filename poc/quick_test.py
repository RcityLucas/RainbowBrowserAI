#!/usr/bin/env python3
import requests
import json
import subprocess
import time
import sys
import os

def test_batch_functionality():
    print("🧪 Testing RainbowBrowserAI Mock Mode Batch Fix")
    print("=" * 50)
    
    # Start the server
    print("📡 Starting server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--bin", "rainbow-poc"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd="/mnt/d/github/RainbowBrowserAI/poc"
    )
    
    # Wait for server to start
    print("⏳ Waiting for server startup...")
    time.sleep(15)
    
    try:
        # Test the batch command
        print("🚀 Testing batch command...")
        url = "http://localhost:3000/command"
        data = {
            "command": "test google,github with screenshots"
        }
        
        response = requests.post(url, json=data, timeout=30)
        
        print(f"📊 Response Status: {response.status_code}")
        print("📄 Response Content:")
        print("-" * 30)
        
        try:
            result = response.json()
            print(json.dumps(result, indent=2))
            
            # Check if it's the old mock response
            if "message" in result and "not executed in mock mode" in result.get("message", ""):
                print("\n❌ ISSUE: Still showing mock message - fix didn't work!")
                return False
            elif "results" in result and "total_tests" in result:
                print("\n✅ SUCCESS: Batch testing is now executing properly!")
                return True
            else:
                print("\n❓ UNKNOWN: Unexpected response format")
                return False
                
        except json.JSONDecodeError:
            print("Raw response:", response.text)
            return False
            
    except requests.RequestException as e:
        print(f"❌ Request failed: {e}")
        return False
    finally:
        # Cleanup
        print("\n🧹 Cleaning up...")
        server_process.terminate()
        time.sleep(2)
        if server_process.poll() is None:
            server_process.kill()

if __name__ == "__main__":
    success = test_batch_functionality()
    sys.exit(0 if success else 1)