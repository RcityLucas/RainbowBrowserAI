#!/usr/bin/env python3
"""
Test script for the new intelligent_action tool
"""

import json
import requests
import time
import sys

def test_intelligent_action():
    base_url = "http://127.0.0.1:3002"
    
    print("🚀 Testing Intelligent Action Tool")
    print("=" * 50)
    
    # Test 1: Navigate to a page
    print("\n📍 Test 1: Navigation Action")
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "navigate",
        "target": {},
        "url": "https://example.com",
        "retry_count": 2,
        "verify_result": True
    })
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Navigation: {result['success']}")
        print(f"   Execution time: {result['execution_time_ms']}ms")
        print(f"   Attempts: {result['attempts']}")
        if result.get('logs'):
            for log in result['logs'][-2:]:  # Show last 2 logs
                print(f"   📝 {log}")
    else:
        print(f"❌ Navigation failed: {response.status_code}")
        print(f"   Error: {response.text}")
    
    time.sleep(1)
    
    # Test 2: Click action with retry
    print("\n🖱️  Test 2: Click Action with Retry")
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "click",
        "target": {
            "selector": "a"
        },
        "retry_count": 3,
        "verify_result": True
    })
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Click: {result['success']}")
        print(f"   Execution time: {result['execution_time_ms']}ms")
        print(f"   Attempts: {result['attempts']}")
        if result.get('element_info'):
            elem = result['element_info']
            print(f"   🎯 Element: {elem['tag_name']} visible={elem['is_visible']}")
    else:
        print(f"❌ Click failed: {response.status_code}")
    
    time.sleep(1)
    
    # Test 3: Type action with advanced targeting
    print("\n⌨️  Test 3: Type Action")
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "type",
        "target": {
            "id": "search"  # Try to find by ID first
        },
        "text": "Hello from Intelligent Action!",
        "retry_count": 2,
        "verify_result": True
    })
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Type: {result['success']}")
        if not result['success'] and result.get('error'):
            print(f"   ⚠️  Error: {result['error']}")
    else:
        print(f"❌ Type failed: {response.status_code}")
    
    time.sleep(1)
    
    # Test 4: Screenshot action
    print("\n📸 Test 4: Screenshot Action")
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "screenshot",
        "target": {},
        "retry_count": 1
    })
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Screenshot: {result['success']}")
        print(f"   Verification: {result.get('verification_result', 'N/A')}")
    else:
        print(f"❌ Screenshot failed: {response.status_code}")
    
    # Test 5: Hover action with coordinate fallback
    print("\n🎯 Test 5: Hover Action")
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "hover",
        "target": {
            "selector": "h1"
        },
        "retry_count": 2
    })
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Hover: {result['success']}")
        if result.get('logs'):
            print(f"   📝 {result['logs'][-1]}")  # Show last log
    else:
        print(f"❌ Hover failed: {response.status_code}")
    
    # Test 6: Wait action
    print("\n⏳ Test 6: Wait Action")
    start_time = time.time()
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "wait",
        "target": {},
        "wait_condition": "1000"  # 1 second
    })
    
    actual_time = (time.time() - start_time) * 1000
    
    if response.status_code == 200:
        result = response.json()
        print(f"✅ Wait: {result['success']}")
        print(f"   Expected: 1000ms, Actual: {actual_time:.0f}ms")
    else:
        print(f"❌ Wait failed: {response.status_code}")
    
    print("\n" + "=" * 50)
    print("🎉 Intelligent Action Tool Test Complete!")
    
    return True

def test_invalid_inputs():
    base_url = "http://127.0.0.1:3002"
    
    print("\n🧪 Testing Input Validation")
    print("-" * 30)
    
    # Test invalid action type
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "invalid_action",
        "target": {"selector": "body"}
    })
    
    print(f"Invalid action type: {response.status_code == 400}")
    
    # Test missing text for type action
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "type",
        "target": {"selector": "input"}
    })
    
    print(f"Missing text validation: {response.status_code == 400}")
    
    # Test missing target
    response = requests.post(f"{base_url}/api/tools/intelligent_action", json={
        "action_type": "click",
        "target": {}
    })
    
    print(f"Missing target validation: {response.status_code == 400}")

if __name__ == "__main__":
    try:
        # Check if server is running
        response = requests.get("http://127.0.0.1:3002/health", timeout=5)
        if response.status_code != 200:
            print("❌ Server is not running on port 3002")
            sys.exit(1)
        
        test_intelligent_action()
        test_invalid_inputs()
        
    except requests.exceptions.ConnectionError:
        print("❌ Cannot connect to server at http://127.0.0.1:3002")
        print("   Make sure the server is running with: cargo run --release -- serve --port 3002")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Test failed with error: {e}")
        sys.exit(1)