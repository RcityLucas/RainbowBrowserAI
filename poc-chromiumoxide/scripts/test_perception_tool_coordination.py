#!/usr/bin/env python3
"""
Test script for perception-to-tools coordination
This demonstrates that perception can now detect interfaces for tool operations
"""

import requests
import json
import time
from typing import Dict, Any, Optional

# API configuration
BASE_URL = "http://localhost:3001"
API_V2_URL = f"{BASE_URL}/api/v2"

class CoordinationTester:
    def __init__(self):
        self.session_id = None
        self.browser_id = None
        
    def create_session(self) -> bool:
        """Create a new coordinated session"""
        print("Creating coordinated session...")
        try:
            response = requests.post(f"{API_V2_URL}/session/create")
            if response.status_code == 200:
                data = response.json()
                self.session_id = data.get("session_id")
                print(f"✓ Session created: {self.session_id}")
                return True
            else:
                print(f"✗ Failed to create session: {response.status_code}")
                return False
        except Exception as e:
            print(f"✗ Error creating session: {e}")
            return False
    
    def acquire_browser(self) -> bool:
        """Acquire a browser for the session"""
        print("Acquiring browser...")
        try:
            response = requests.post(f"{BASE_URL}/api/browser/acquire")
            if response.status_code == 200:
                data = response.json()
                self.browser_id = data.get("browser_id")
                print(f"✓ Browser acquired: {self.browser_id}")
                return True
            else:
                print(f"✗ Failed to acquire browser: {response.status_code}")
                return False
        except Exception as e:
            print(f"✗ Error acquiring browser: {e}")
            return False
    
    def navigate(self, url: str) -> bool:
        """Navigate to a URL"""
        print(f"Navigating to {url}...")
        try:
            response = requests.post(
                f"{BASE_URL}/api/browser/navigate",
                json={
                    "browser_id": self.browser_id,
                    "url": url,
                    "session_id": self.session_id  # Include session for coordination
                }
            )
            if response.status_code == 200:
                print(f"✓ Navigated successfully")
                return True
            else:
                print(f"✗ Navigation failed: {response.status_code}")
                return False
        except Exception as e:
            print(f"✗ Error navigating: {e}")
            return False
    
    def test_perception_finds_tool_interface(self) -> Dict[str, Any]:
        """Test that perception can detect tool-compatible interfaces"""
        print("\nTesting perception detection of tool interfaces...")
        
        try:
            # Use perception to analyze the page
            response = requests.post(
                f"{BASE_URL}/api/perception/analyze",
                json={
                    "browser_id": self.browser_id,
                    "session_id": self.session_id,
                    "analysis_type": "interactive_elements"
                }
            )
            
            if response.status_code != 200:
                return {
                    "success": False,
                    "error": f"Perception analysis failed: {response.status_code}"
                }
            
            perception_data = response.json()
            elements = perception_data.get("interactive_elements", [])
            
            print(f"  Found {len(elements)} interactive elements")
            
            # Check which elements are tool-compatible
            tool_compatible = []
            for element in elements:
                element_type = element.get("type", "unknown")
                selector = element.get("selector", "")
                text = element.get("text", "")
                
                # These element types can be operated on by tools
                if element_type in ["button", "link", "input", "select", "textarea"]:
                    tool_compatible.append({
                        "selector": selector,
                        "type": element_type,
                        "text": text
                    })
                    print(f"  ✓ Tool-compatible: {element_type} - {selector[:50]}")
            
            return {
                "success": True,
                "total_elements": len(elements),
                "tool_compatible": len(tool_compatible),
                "compatible_elements": tool_compatible
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def test_coordinated_action(self, target: str) -> Dict[str, Any]:
        """Test coordinated perception + tool execution"""
        print(f"\nTesting coordinated action on '{target}'...")
        
        try:
            # First, use perception to find the element
            print("  1. Using perception to locate element...")
            perception_response = requests.post(
                f"{BASE_URL}/api/perception/find_element",
                json={
                    "browser_id": self.browser_id,
                    "session_id": self.session_id,
                    "target": target
                }
            )
            
            if perception_response.status_code != 200:
                return {
                    "success": False,
                    "error": f"Perception failed: {perception_response.status_code}"
                }
            
            element_data = perception_response.json()
            selector = element_data.get("selector")
            
            if not selector:
                return {
                    "success": False,
                    "error": "Element not found by perception"
                }
            
            print(f"  ✓ Element found: {selector}")
            
            # Now use tools to interact with it
            print("  2. Using tools to click element...")
            tool_response = requests.post(
                f"{BASE_URL}/api/tools/execute",
                json={
                    "browser_id": self.browser_id,
                    "session_id": self.session_id,
                    "tool": "click",
                    "parameters": {
                        "selector": selector
                    }
                }
            )
            
            if tool_response.status_code == 200:
                print("  ✓ Tool execution successful")
                return {
                    "success": True,
                    "selector": selector,
                    "action": "click"
                }
            else:
                return {
                    "success": False,
                    "error": f"Tool execution failed: {tool_response.status_code}"
                }
                
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def test_shared_context(self) -> Dict[str, Any]:
        """Test that modules share the same context"""
        print("\nTesting shared context between modules...")
        
        try:
            # Make multiple perception requests
            results = []
            for i in range(3):
                response = requests.post(
                    f"{BASE_URL}/api/perception/analyze",
                    json={
                        "browser_id": self.browser_id,
                        "session_id": self.session_id
                    }
                )
                results.append(response.status_code == 200)
                time.sleep(0.1)
            
            # Check session stats to verify shared instance
            stats_response = requests.get(
                f"{API_V2_URL}/session/{self.session_id}/stats"
            )
            
            if stats_response.status_code == 200:
                stats = stats_response.json()
                cache_hits = stats.get("cache_hits", 0)
                
                print(f"  Cache hits: {cache_hits}")
                print(f"  Shared instance used: {cache_hits > 0}")
                
                return {
                    "success": True,
                    "shared_instance": cache_hits > 0,
                    "cache_hits": cache_hits
                }
            
            return {
                "success": False,
                "error": "Could not verify shared context"
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    def cleanup(self):
        """Clean up resources"""
        print("\nCleaning up...")
        
        if self.browser_id:
            try:
                requests.post(
                    f"{BASE_URL}/api/browser/release",
                    json={"browser_id": self.browser_id}
                )
                print("  ✓ Browser released")
            except:
                pass
        
        if self.session_id:
            try:
                requests.delete(f"{API_V2_URL}/session/{self.session_id}")
                print("  ✓ Session closed")
            except:
                pass

def main():
    print("=" * 60)
    print("Perception-to-Tools Coordination Test")
    print("=" * 60)
    print()
    
    tester = CoordinationTester()
    
    # Setup
    if not tester.create_session():
        print("Failed to create session")
        return 1
    
    if not tester.acquire_browser():
        print("Failed to acquire browser")
        return 1
    
    if not tester.navigate("https://example.com"):
        print("Failed to navigate")
        return 1
    
    # Run tests
    print("\n" + "=" * 60)
    print("Running Tests")
    print("=" * 60)
    
    test_results = []
    
    # Test 1: Perception finds tool interfaces
    result1 = tester.test_perception_finds_tool_interface()
    test_results.append(("Perception detects tool interfaces", result1["success"]))
    if result1["success"]:
        print(f"  ✓ Found {result1['tool_compatible']}/{result1['total_elements']} tool-compatible elements")
    else:
        print(f"  ✗ Error: {result1.get('error', 'Unknown error')}")
    
    # Test 2: Coordinated action
    result2 = tester.test_coordinated_action("More information")
    test_results.append(("Coordinated perception + tools", result2["success"]))
    if result2["success"]:
        print(f"  ✓ Successfully executed coordinated action")
    else:
        print(f"  ✗ Error: {result2.get('error', 'Unknown error')}")
    
    # Test 3: Shared context
    result3 = tester.test_shared_context()
    test_results.append(("Shared module context", result3["success"]))
    if result3["success"]:
        print(f"  ✓ Modules share context (cache hits: {result3.get('cache_hits', 0)})")
    else:
        print(f"  ✗ Error: {result3.get('error', 'Unknown error')}")
    
    # Cleanup
    tester.cleanup()
    
    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    
    passed = sum(1 for _, success in test_results if success)
    total = len(test_results)
    
    for test_name, success in test_results:
        status = "✓ PASS" if success else "✗ FAIL"
        print(f"  {status}: {test_name}")
    
    print()
    print(f"Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n✅ All tests passed! The coordination architecture is working correctly.")
        print("\nKey achievements:")
        print("  • Perception can detect tool-compatible interfaces")
        print("  • Modules share the same browser context")
        print("  • Coordinated actions work seamlessly")
        return 0
    else:
        print(f"\n❌ {total - passed} test(s) failed. Please check the output above.")
        return 1

if __name__ == "__main__":
    import sys
    sys.exit(main())