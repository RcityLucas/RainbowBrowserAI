#!/usr/bin/env python3
"""
Perception Integration Test
Tests the perception features through the actual API endpoints
"""

import requests
import json
import time
from datetime import datetime

class PerceptionIntegrationTest:
    def __init__(self, api_url="http://localhost:3001"):
        self.api_url = api_url
        self.session_id = None
        
    def test_api_health(self):
        """Test API health"""
        print("🔍 Testing API Health...")
        
        try:
            response = requests.get(f"{self.api_url}/api/health")
            if response.status_code == 200:
                data = response.json()
                print(f"  ✅ API is healthy")
                print(f"  Version: {data.get('version', 'Unknown')}")
                print(f"  Uptime: {data.get('uptime_seconds', 0)}s")
                print(f"  Active sessions: {data.get('active_sessions', 0)}")
                return True
            else:
                print(f"  ❌ API health check failed: {response.status_code}")
                return False
        except Exception as e:
            print(f"  ❌ Cannot connect to API: {e}")
            return False
    
    def test_navigation(self):
        """Test navigation to different sites"""
        print("\n🌐 Testing Navigation...")
        
        test_urls = [
            "https://example.com",
            "https://google.com",
            "https://github.com"
        ]
        
        results = []
        
        for url in test_urls:
            print(f"\n  Navigating to {url}...")
            
            try:
                start_time = time.time()
                response = requests.post(
                    f"{self.api_url}/api/navigate",
                    json={"url": url},
                    timeout=30
                )
                elapsed_ms = (time.time() - start_time) * 1000
                
                if response.status_code == 200:
                    data = response.json()
                    if data.get("success"):
                        print(f"    ✅ Success in {elapsed_ms:.1f}ms")
                        print(f"    Title: {data.get('title', 'N/A')}")
                        if data.get("session_id"):
                            self.session_id = data["session_id"]
                            print(f"    Session: {self.session_id}")
                        results.append({
                            "url": url,
                            "success": True,
                            "time_ms": elapsed_ms,
                            "title": data.get("title")
                        })
                    else:
                        print(f"    ❌ Navigation failed: {data}")
                        results.append({"url": url, "success": False, "error": str(data)})
                else:
                    print(f"    ❌ HTTP error: {response.status_code}")
                    results.append({"url": url, "success": False, "error": f"HTTP {response.status_code}"})
                    
            except Exception as e:
                print(f"    ❌ Exception: {e}")
                results.append({"url": url, "success": False, "error": str(e)})
        
        success_count = sum(1 for r in results if r["success"])
        print(f"\n  📊 Navigation Summary: {success_count}/{len(results)} successful")
        
        return results
    
    def test_screenshot(self):
        """Test screenshot functionality"""
        print("\n📸 Testing Screenshot...")
        
        try:
            # Navigate first
            response = requests.post(
                f"{self.api_url}/api/navigate",
                json={"url": "https://example.com"},
                timeout=30
            )
            
            if response.status_code != 200:
                print("  ❌ Failed to navigate for screenshot")
                return False
            
            # Take screenshot
            start_time = time.time()
            response = requests.post(
                f"{self.api_url}/api/screenshot",
                json={"filename": "test_screenshot.png"},
                timeout=30
            )
            elapsed_ms = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                data = response.json()
                if data.get("success"):
                    print(f"  ✅ Screenshot taken in {elapsed_ms:.1f}ms")
                    print(f"  Path: {data.get('screenshot_path', 'N/A')}")
                    return True
                else:
                    print(f"  ❌ Screenshot failed: {data}")
                    return False
            else:
                print(f"  ❌ HTTP error: {response.status_code}")
                return False
                
        except Exception as e:
            print(f"  ❌ Exception: {e}")
            return False
    
    def test_element_interaction(self):
        """Test element finding and interaction"""
        print("\n🖱️ Testing Element Interaction...")
        
        try:
            # Navigate to Google first
            response = requests.post(
                f"{self.api_url}/api/navigate",
                json={"url": "https://google.com"},
                timeout=30
            )
            
            if response.status_code != 200:
                print("  ❌ Failed to navigate to Google")
                return False
            
            # Try to find and interact with search box
            print("  Looking for search input...")
            response = requests.post(
                f"{self.api_url}/api/find-element",
                json={"selector": "input[name='q'], input[title='Search']"},
                timeout=15
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get("found"):
                    print(f"  ✅ Search element found")
                    
                    # Try to type in the search box
                    print("  Typing in search box...")
                    response = requests.post(
                        f"{self.api_url}/api/type-text",
                        json={
                            "selector": "input[name='q'], input[title='Search']",
                            "text": "RainbowBrowserAI test"
                        },
                        timeout=15
                    )
                    
                    if response.status_code == 200:
                        type_data = response.json()
                        if type_data.get("success"):
                            print("  ✅ Successfully typed text")
                            return True
                        else:
                            print(f"  ❌ Failed to type: {type_data}")
                            return False
                    else:
                        print(f"  ❌ Type request failed: {response.status_code}")
                        return False
                else:
                    print("  ❌ Search element not found")
                    return False
            else:
                print(f"  ❌ Element find failed: {response.status_code}")
                return False
                
        except Exception as e:
            print(f"  ❌ Exception: {e}")
            return False
    
    def test_page_analysis(self):
        """Test page analysis capabilities"""
        print("\n🔍 Testing Page Analysis...")
        
        try:
            # Navigate to a page with content
            response = requests.post(
                f"{self.api_url}/api/navigate",
                json={"url": "https://example.com"},
                timeout=30
            )
            
            if response.status_code != 200:
                print("  ❌ Failed to navigate")
                return False
            
            # Test page analysis
            start_time = time.time()
            response = requests.get(
                f"{self.api_url}/api/analyze-page",
                timeout=30
            )
            elapsed_ms = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                data = response.json()
                print(f"  ✅ Page analyzed in {elapsed_ms:.1f}ms")
                
                # Display analysis results
                if "elements" in data:
                    print(f"  Elements found: {len(data['elements'])}")
                
                if "links" in data:
                    print(f"  Links found: {len(data['links'])}")
                
                if "forms" in data:
                    print(f"  Forms found: {len(data['forms'])}")
                
                if "title" in data:
                    print(f"  Page title: {data['title']}")
                
                return True
            else:
                print(f"  ❌ Analysis failed: {response.status_code}")
                return False
                
        except Exception as e:
            print(f"  ❌ Exception: {e}")
            return False
    
    def test_performance_metrics(self):
        """Test performance tracking"""
        print("\n📈 Testing Performance Metrics...")
        
        try:
            response = requests.get(f"{self.api_url}/api/metrics", timeout=15)
            
            if response.status_code == 200:
                data = response.json()
                print("  ✅ Metrics retrieved")
                
                # Display key metrics
                if "operations" in data:
                    print(f"  Total operations: {data['operations']}")
                
                if "average_response_time_ms" in data:
                    print(f"  Avg response time: {data['average_response_time_ms']}ms")
                
                if "success_rate" in data:
                    print(f"  Success rate: {data['success_rate']}%")
                
                if "active_browsers" in data:
                    print(f"  Active browsers: {data['active_browsers']}")
                
                return True
            else:
                print(f"  ❌ Metrics request failed: {response.status_code}")
                return False
                
        except Exception as e:
            print(f"  ❌ Exception: {e}")
            return False
    
    def test_session_management(self):
        """Test session management"""
        print("\n🔗 Testing Session Management...")
        
        try:
            # List sessions
            response = requests.get(f"{self.api_url}/api/sessions", timeout=15)
            
            if response.status_code == 200:
                data = response.json()
                sessions = data.get("sessions", [])
                print(f"  ✅ Found {len(sessions)} active sessions")
                
                if sessions:
                    # Show details of first session
                    session = sessions[0]
                    print(f"  Session ID: {session.get('id', 'N/A')}")
                    print(f"  Created: {session.get('created', 'N/A')}")
                    print(f"  Last used: {session.get('last_used', 'N/A')}")
                
                return True
            else:
                print(f"  ❌ Session list failed: {response.status_code}")
                return False
                
        except Exception as e:
            print(f"  ❌ Exception: {e}")
            return False
    
    def run_integration_tests(self):
        """Run all integration tests"""
        print("="*60)
        print("🧪 PERCEPTION INTEGRATION TESTS")
        print("="*60)
        print(f"Timestamp: {datetime.now().isoformat()}")
        print(f"API URL: {self.api_url}")
        print("="*60)
        
        # Run tests
        tests = [
            ("API Health Check", self.test_api_health),
            ("Navigation", self.test_navigation),
            ("Screenshot", self.test_screenshot),
            ("Element Interaction", self.test_element_interaction),
            ("Page Analysis", self.test_page_analysis),
            ("Performance Metrics", self.test_performance_metrics),
            ("Session Management", self.test_session_management),
        ]
        
        results = []
        
        for test_name, test_func in tests:
            print(f"\n{'='*60}")
            print(f"Running: {test_name}")
            print("="*60)
            
            try:
                start_time = time.time()
                success = test_func()
                elapsed = (time.time() - start_time) * 1000
                
                results.append({
                    "test": test_name,
                    "success": success,
                    "time_ms": elapsed
                })
                
            except Exception as e:
                print(f"  ❌ Test exception: {e}")
                results.append({
                    "test": test_name,
                    "success": False,
                    "error": str(e)
                })
            
            time.sleep(1)  # Brief pause between tests
        
        # Print summary
        self.print_summary(results)
        
        return results
    
    def print_summary(self, results):
        """Print test summary"""
        print("\n" + "="*60)
        print("📊 INTEGRATION TEST SUMMARY")
        print("="*60)
        
        total_tests = len(results)
        successful = sum(1 for r in results if r["success"])
        failed = total_tests - successful
        
        print(f"\nTotal Tests: {total_tests}")
        print(f"  ✅ Successful: {successful}")
        print(f"  ❌ Failed: {failed}")
        
        success_rate = (successful / total_tests * 100) if total_tests > 0 else 0
        print(f"\nSuccess Rate: {success_rate:.1f}%")
        
        # Show timing for successful tests
        successful_results = [r for r in results if r["success"] and "time_ms" in r]
        if successful_results:
            avg_time = sum(r["time_ms"] for r in successful_results) / len(successful_results)
            print(f"Average Test Time: {avg_time:.1f}ms")
        
        # Overall verdict
        print("\n" + "="*60)
        if success_rate >= 80:
            print("✨ PERCEPTION INTEGRATION: EXCELLENT")
            print("🎯 The perception module is working well with the API!")
        elif success_rate >= 60:
            print("✅ PERCEPTION INTEGRATION: GOOD")
            print("🔧 Some improvements needed but core functionality works")
        elif success_rate >= 40:
            print("⚠️  PERCEPTION INTEGRATION: NEEDS WORK")
            print("🚧 Several issues found that need attention")
        else:
            print("❌ PERCEPTION INTEGRATION: CRITICAL ISSUES")
            print("🚨 Major problems preventing proper operation")
        print("="*60)

def main():
    test = PerceptionIntegrationTest()
    test.run_integration_tests()

if __name__ == "__main__":
    main()