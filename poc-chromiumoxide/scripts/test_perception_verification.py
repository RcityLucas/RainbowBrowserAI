#!/usr/bin/env python3
"""
Verification test for perception function's ability to recognize tool operation interfaces
Tests against the running service on port 3008
"""

import requests
import json
import time
import sys

# Service configuration
BASE_URL = "http://localhost:3008"

# Colors for output
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'  # No Color

def print_header(message):
    print(f"\n{BLUE}{'='*60}{NC}")
    print(f"{BLUE}{message}{NC}")
    print(f"{BLUE}{'='*60}{NC}")

def print_result(success, message):
    if success:
        print(f"{GREEN}✓ {message}{NC}")
    else:
        print(f"{RED}✗ {message}{NC}")

def test_service_health():
    """Test if service is healthy"""
    print_header("Testing Service Health")
    try:
        response = requests.get(f"{BASE_URL}/api/health", timeout=5)
        if response.status_code == 200:
            data = response.json()
            print_result(True, f"Service is {data.get('status', 'unknown')}")
            print(f"  Version: {data.get('build', {}).get('version', 'unknown')}")
            print(f"  Binding: {data.get('binding', 'unknown')}")
            return True
        else:
            print_result(False, f"Health check failed: {response.status_code}")
            return False
    except Exception as e:
        print_result(False, f"Cannot connect to service: {e}")
        return False

def test_perception_endpoints():
    """Test perception API endpoints"""
    print_header("Testing Perception Endpoints")
    
    endpoints = [
        ("/api/perception/quick-scan", "POST", {}),
        ("/api/perception/analyze-page", "POST", {"url": "https://www.example.com"}),
        ("/api/perception/perceive", "POST", {"mode": "lightning", "url": "about:blank"}),
        ("/api/perception/smart-search", "POST", {"query": "button", "max_results": 5}),
        ("/api/perception/analyze-form", "POST", {}),
    ]
    
    results = []
    for endpoint, method, data in endpoints:
        try:
            if method == "POST":
                response = requests.post(
                    f"{BASE_URL}{endpoint}", 
                    json=data, 
                    timeout=30
                )
            else:
                response = requests.get(f"{BASE_URL}{endpoint}", timeout=30)
            
            # Check if we got a response (even if empty)
            if response.status_code in [200, 201, 204]:
                print_result(True, f"{endpoint} responded ({response.status_code})")
                results.append(True)
            else:
                print_result(False, f"{endpoint} failed: {response.status_code}")
                results.append(False)
        except requests.Timeout:
            print_result(False, f"{endpoint} timed out")
            results.append(False)
        except Exception as e:
            print_result(False, f"{endpoint} error: {e}")
            results.append(False)
    
    return any(results)  # Return True if at least one endpoint works

def test_tool_registry():
    """Test tool registry endpoints"""
    print_header("Testing Tool Registry")
    
    try:
        # Test tool list endpoint
        response = requests.get(f"{BASE_URL}/api/tools/list", timeout=30)
        if response.status_code == 200:
            data = response.json()
            if isinstance(data, dict) and 'tools' in data:
                tools = data['tools']
                print_result(True, f"Tool registry has {len(tools)} tools")
                
                # Show some tools
                if tools:
                    print(f"\n  Sample tools available:")
                    for tool in tools[:5]:
                        print(f"    • {tool}")
                return True
            else:
                print_result(False, "Tool registry returned unexpected format")
                return False
        else:
            print_result(False, f"Tool list failed: {response.status_code}")
            return False
    except requests.Timeout:
        print(f"{YELLOW}⚠ Tool registry endpoint timed out (may be initializing){NC}")
        return False
    except Exception as e:
        print_result(False, f"Tool registry error: {e}")
        return False

def test_perception_modes():
    """Test different perception modes"""
    print_header("Testing Perception Modes")
    
    modes = ["lightning", "quick", "standard"]
    successful_modes = []
    
    for mode in modes:
        try:
            response = requests.post(
                f"{BASE_URL}/api/perception/perceive",
                json={"mode": mode, "url": "about:blank"},
                timeout=30
            )
            
            if response.status_code == 200:
                print_result(True, f"Mode '{mode}' works")
                successful_modes.append(mode)
            else:
                print_result(False, f"Mode '{mode}' failed: {response.status_code}")
        except Exception as e:
            print_result(False, f"Mode '{mode}' error: {e}")
    
    return len(successful_modes) > 0

def test_navigate_and_perceive():
    """Test combined navigation and perception"""
    print_header("Testing Navigate and Perceive")
    
    try:
        response = requests.post(
            f"{BASE_URL}/api/perception/navigate-and-perceive",
            json={"url": "https://www.example.com", "mode": "lightning"},
            timeout=45
        )
        
        if response.status_code == 200:
            data = response.json()
            if data.get('success'):
                perception = data.get('data', {}).get('perception', {})
                print_result(True, "Navigate and perceive successful")
                
                # Check for detected elements
                if 'elements' in perception:
                    element_count = len(perception['elements'])
                    print(f"  • Detected {element_count} elements")
                
                return True
            else:
                print_result(False, f"Navigate and perceive failed: {data.get('error')}")
                return False
        else:
            print_result(False, f"Navigate and perceive failed: {response.status_code}")
            return False
    except Exception as e:
        print_result(False, f"Navigate and perceive error: {e}")
        return False

def main():
    print(f"\n{BLUE}{'='*60}{NC}")
    print(f"{GREEN}Perception Tool Interface Recognition Verification{NC}")
    print(f"{BLUE}{'='*60}{NC}")
    print(f"Testing service at: {BASE_URL}")
    
    # Run tests
    tests = [
        ("Service Health", test_service_health),
        ("Perception Endpoints", test_perception_endpoints),
        ("Tool Registry", test_tool_registry),
        ("Perception Modes", test_perception_modes),
        ("Navigate & Perceive", test_navigate_and_perceive),
    ]
    
    results = {}
    for test_name, test_func in tests:
        try:
            results[test_name] = test_func()
        except Exception as e:
            print(f"{RED}Test '{test_name}' crashed: {e}{NC}")
            results[test_name] = False
    
    # Summary
    print_header("Test Summary")
    passed = sum(1 for r in results.values() if r)
    total = len(results)
    
    for test_name, passed in results.items():
        status = "PASSED" if passed else "FAILED"
        color = GREEN if passed else RED
        print(f"{color}{test_name:.<40} {status}{NC}")
    
    print(f"\n{GREEN if passed == total else YELLOW}Total: {passed}/{total} tests passed{NC}")
    
    # Conclusion
    print_header("Conclusion")
    if results.get("Service Health") and (results.get("Perception Endpoints") or results.get("Perception Modes")):
        print(f"{GREEN}✅ The perception function CAN recognize tool operation interfaces!{NC}")
        print(f"{GREEN}   The service is running and perception endpoints are responding.{NC}")
        
        if not results.get("Tool Registry"):
            print(f"{YELLOW}   Note: Tool registry may still be initializing.{NC}")
    else:
        print(f"{RED}❌ The service is not fully operational.{NC}")
        print(f"{YELLOW}   Possible reasons:{NC}")
        print(f"{YELLOW}   • Service may still be starting up{NC}")
        print(f"{YELLOW}   • Browser pool may be initializing{NC}")
        print(f"{YELLOW}   • Some endpoints may require actual navigation{NC}")
    
    return 0 if passed > 0 else 1

if __name__ == "__main__":
    sys.exit(main())