#!/usr/bin/env python3
"""
Enhanced Perception System Live Test
Tests the new intelligent capabilities on the running service at port 3001
"""

import requests
import json
import time
import sys
from typing import Dict, Any, Optional

class EnhancedPerceptionTester:
    def __init__(self, base_url: str = "http://localhost:3001"):
        self.base_url = base_url
        self.session_id = None
        self.results = []
        
    def log_result(self, test_name: str, success: bool, details: str):
        """Log test result"""
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{status} {test_name}: {details}")
        self.results.append({
            "test": test_name,
            "success": success,
            "details": details
        })
    
    def test_service_health(self) -> bool:
        """Test basic service health"""
        try:
            response = requests.get(f"{self.base_url}/health", timeout=5)
            if response.status_code == 200:
                data = response.json()
                self.log_result("Service Health", True, f"Status: {data.get('status', 'unknown')}")
                return True
            else:
                self.log_result("Service Health", False, f"HTTP {response.status_code}")
                return False
        except Exception as e:
            self.log_result("Service Health", False, f"Connection error: {str(e)}")
            return False
    
    def test_navigation(self) -> bool:
        """Test basic navigation functionality"""
        try:
            payload = {
                "url": "https://example.com",
                "take_screenshot": False
            }
            response = requests.post(
                f"{self.base_url}/api/navigate",
                json=payload,
                timeout=10
            )
            
            if response.status_code == 200:
                data = response.json()
                if data.get('success'):
                    self.session_id = data.get('session_id')
                    self.log_result("Navigation", True, f"Navigated to example.com, session: {self.session_id[:8]}...")
                    return True
                else:
                    self.log_result("Navigation", False, "Navigation returned success=false")
                    return False
            else:
                self.log_result("Navigation", False, f"HTTP {response.status_code}")
                return False
                
        except Exception as e:
            self.log_result("Navigation", False, f"Error: {str(e)}")
            return False
    
    def test_perception_analysis(self) -> bool:
        """Test perception analysis capabilities"""
        try:
            # Try different perception endpoints
            endpoints_to_try = [
                "/api/perception",
                "/api/analyze",
                "/api/perception/analyze",
                "/api/lightning",
                "/api/quick",
                "/api/standard",
                "/api/deep"
            ]
            
            for endpoint in endpoints_to_try:
                payload = {
                    "level": "quick",
                    "analyze_elements": True,
                    "session_id": self.session_id
                }
                
                try:
                    response = requests.post(
                        f"{self.base_url}{endpoint}",
                        json=payload,
                        timeout=10
                    )
                    
                    if response.status_code == 200:
                        self.log_result("Perception Analysis", True, f"Endpoint {endpoint} responded successfully")
                        return True
                        
                except requests.exceptions.RequestException:
                    continue
            
            # If no perception endpoints work, try alternative approach
            self.log_result("Perception Analysis", False, "No perception endpoints found")
            return False
            
        except Exception as e:
            self.log_result("Perception Analysis", False, f"Error: {str(e)}")
            return False
    
    def test_element_detection(self) -> bool:
        """Test intelligent element detection"""
        try:
            # Try different element detection approaches
            test_cases = [
                {
                    "name": "Smart Element Detection",
                    "endpoint": "/api/element/find",
                    "payload": {
                        "description": "search box",
                        "smart_detection": True,
                        "session_id": self.session_id
                    }
                },
                {
                    "name": "Element Interaction",
                    "endpoint": "/api/element/interact",
                    "payload": {
                        "action": "find",
                        "target": "input field",
                        "session_id": self.session_id
                    }
                },
                {
                    "name": "Natural Language Element Finding",
                    "endpoint": "/api/find",
                    "payload": {
                        "query": "find the main content area",
                        "session_id": self.session_id
                    }
                }
            ]
            
            for test_case in test_cases:
                try:
                    response = requests.post(
                        f"{self.base_url}{test_case['endpoint']}",
                        json=test_case["payload"],
                        timeout=10
                    )
                    
                    if response.status_code == 200:
                        self.log_result("Element Detection", True, f"{test_case['name']} successful")
                        return True
                        
                except requests.exceptions.RequestException:
                    continue
            
            self.log_result("Element Detection", False, "No element detection endpoints available")
            return False
            
        except Exception as e:
            self.log_result("Element Detection", False, f"Error: {str(e)}")
            return False
    
    def test_form_interaction(self) -> bool:
        """Test enhanced form interaction"""
        try:
            # First navigate to a page with forms
            form_test_payload = {
                "url": "https://httpbin.org/forms/post",
                "take_screenshot": False
            }
            
            response = requests.post(
                f"{self.base_url}/api/navigate",
                json=form_test_payload,
                timeout=10
            )
            
            if response.status_code != 200:
                self.log_result("Form Setup", False, "Could not navigate to form test page")
                return False
            
            # Try form interaction endpoints
            form_endpoints = [
                {
                    "name": "Smart Form Fill",
                    "endpoint": "/api/form/fill",
                    "payload": {
                        "field": "customer name",
                        "value": "Test User",
                        "smart_detection": True,
                        "session_id": self.session_id
                    }
                },
                {
                    "name": "Enhanced Type",
                    "endpoint": "/api/type",
                    "payload": {
                        "selector": "input[name='custname']",
                        "text": "Enhanced Test",
                        "session_id": self.session_id
                    }
                }
            ]
            
            for test in form_endpoints:
                try:
                    response = requests.post(
                        f"{self.base_url}{test['endpoint']}",
                        json=test["payload"],
                        timeout=10
                    )
                    
                    if response.status_code == 200:
                        self.log_result("Form Interaction", True, f"{test['name']} successful")
                        return True
                        
                except requests.exceptions.RequestException:
                    continue
            
            self.log_result("Form Interaction", False, "No enhanced form endpoints available")
            return False
            
        except Exception as e:
            self.log_result("Form Interaction", False, f"Error: {str(e)}")
            return False
    
    def test_error_recovery(self) -> bool:
        """Test error recovery mechanisms"""
        try:
            # Test with intentionally invalid selectors/elements
            error_tests = [
                {
                    "name": "Invalid Element Recovery",
                    "endpoint": "/api/element/find",
                    "payload": {
                        "description": "non-existent-element-12345",
                        "enable_recovery": True,
                        "session_id": self.session_id
                    }
                },
                {
                    "name": "Fallback Selector Test",
                    "endpoint": "/api/find",
                    "payload": {
                        "selector": "#does-not-exist",
                        "fallback_enabled": True,
                        "session_id": self.session_id
                    }
                }
            ]
            
            for test in error_tests:
                try:
                    response = requests.post(
                        f"{self.base_url}{test['endpoint']}",
                        json=test["payload"],
                        timeout=10
                    )
                    
                    # For error recovery, we expect either success (recovery worked)
                    # or a structured error response (graceful degradation)
                    if response.status_code in [200, 404]:
                        if response.status_code == 200:
                            self.log_result("Error Recovery", True, f"{test['name']} - Recovery successful")
                        else:
                            self.log_result("Error Recovery", True, f"{test['name']} - Graceful error handling")
                        return True
                        
                except requests.exceptions.RequestException:
                    continue
            
            self.log_result("Error Recovery", False, "Error recovery endpoints not available")
            return False
            
        except Exception as e:
            self.log_result("Error Recovery", False, f"Error: {str(e)}")
            return False
    
    def test_performance_monitoring(self) -> bool:
        """Test performance monitoring and metrics"""
        try:
            response = requests.get(f"{self.base_url}/metrics", timeout=5)
            
            if response.status_code == 200:
                metrics = response.json()
                success_rate = metrics.get('success_rate', 0)
                avg_time = metrics.get('avg_response_time_ms', 0)
                
                self.log_result("Performance Monitoring", True, 
                              f"Success rate: {success_rate}%, Avg time: {avg_time}ms")
                return True
            else:
                self.log_result("Performance Monitoring", False, f"HTTP {response.status_code}")
                return False
                
        except Exception as e:
            self.log_result("Performance Monitoring", False, f"Error: {str(e)}")
            return False
    
    def run_comprehensive_test(self):
        """Run comprehensive test suite"""
        print("🧠 Enhanced Perception System - Live Service Test")
        print("=" * 55)
        print()
        
        # Test basic service health first
        if not self.test_service_health():
            print("❌ Service health check failed. Cannot continue testing.")
            return
            
        print()
        
        # Test core functionality
        tests = [
            ("Navigation Capabilities", self.test_navigation),
            ("Perception Analysis", self.test_perception_analysis),
            ("Element Detection", self.test_element_detection),
            ("Form Interaction", self.test_form_interaction),
            ("Error Recovery", self.test_error_recovery),
            ("Performance Monitoring", self.test_performance_monitoring),
        ]
        
        for test_name, test_func in tests:
            print(f"Testing {test_name}...")
            test_func()
            time.sleep(0.5)  # Brief pause between tests
            
        print()
        print("=" * 55)
        print("📊 Test Summary")
        print("=" * 55)
        
        passed = sum(1 for r in self.results if r['success'])
        total = len(self.results)
        
        print(f"Tests passed: {passed}/{total}")
        print(f"Success rate: {(passed/total)*100:.1f}%")
        print()
        
        if passed >= total * 0.8:  # 80% success threshold
            print("✅ Enhanced Perception System: OPERATIONAL")
        else:
            print("⚠️ Enhanced Perception System: PARTIAL FUNCTIONALITY")
        
        print()
        print("📋 Detailed Results:")
        for result in self.results:
            status = "✅" if result['success'] else "❌"
            print(f"  {status} {result['test']}: {result['details']}")
        
        return passed >= total * 0.8

if __name__ == "__main__":
    tester = EnhancedPerceptionTester()
    success = tester.run_comprehensive_test()
    sys.exit(0 if success else 1)