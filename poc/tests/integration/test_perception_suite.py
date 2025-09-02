#!/usr/bin/env python3
"""
Complete Perception Test Suite
Tests all perception layers and features through the API
"""

import requests
import json
import time
import sys
from typing import Dict, List
from datetime import datetime

class PerceptionTestSuite:
    def __init__(self, api_url="http://localhost:3001"):
        self.api_url = api_url
        self.test_results = []
        self.chromedriver_url = "http://localhost:9515"
        
    def check_services(self) -> bool:
        """Check if required services are running"""
        print("🔍 Checking services...")
        
        # Check API
        try:
            response = requests.get(f"{self.api_url}/health")
            if response.status_code == 200:
                print("  ✅ API server is running")
            else:
                print(f"  ❌ API server returned status {response.status_code}")
                return False
        except Exception as e:
            print(f"  ❌ Cannot connect to API: {e}")
            return False
        
        # Check ChromeDriver
        try:
            response = requests.get(f"{self.chromedriver_url}/status")
            if response.status_code == 200:
                print("  ✅ ChromeDriver is running")
            else:
                print(f"  ❌ ChromeDriver returned status {response.status_code}")
                return False
        except Exception as e:
            print(f"  ❌ Cannot connect to ChromeDriver: {e}")
            return False
        
        return True
    
    def test_browser_connection(self) -> Dict:
        """Test browser connection endpoint"""
        print("\n📡 Testing Browser Connection...")
        
        try:
            response = requests.get(f"{self.api_url}/test/browser-connection")
            data = response.json()
            
            if response.status_code == 200 and data.get("status") == "connected":
                print("  ✅ Browser connection successful")
                print(f"  Browser: {data.get('browser', 'Unknown')}")
                print(f"  Version: {data.get('version', 'Unknown')}")
                return {"test": "browser_connection", "status": "passed", "details": data}
            else:
                print(f"  ❌ Browser connection failed: {data}")
                return {"test": "browser_connection", "status": "failed", "details": data}
        except Exception as e:
            print(f"  ❌ Error: {e}")
            return {"test": "browser_connection", "status": "error", "error": str(e)}
    
    def test_lightning_perception(self, url: str = "https://example.com") -> Dict:
        """Test Lightning perception layer"""
        print(f"\n⚡ Testing Lightning Perception on {url}...")
        
        try:
            start_time = time.time()
            response = requests.post(
                f"{self.api_url}/perceive/lightning",
                json={"url": url}
            )
            elapsed_ms = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                data = response.json()
                print(f"  ✅ Lightning perception completed in {elapsed_ms:.1f}ms")
                print(f"  Key elements found: {len(data.get('key_elements', []))}")
                print(f"  Page ready: {data.get('page_status', {}).get('ready_state', 'unknown')}")
                
                # Check if meets performance target
                if elapsed_ms < 50:
                    print(f"  🎯 Meets <50ms target!")
                else:
                    print(f"  ⚠️  Exceeds 50ms target ({elapsed_ms:.1f}ms)")
                
                return {
                    "test": "lightning_perception",
                    "status": "passed",
                    "time_ms": elapsed_ms,
                    "meets_target": elapsed_ms < 50,
                    "elements_found": len(data.get('key_elements', [])),
                    "url": url
                }
            else:
                print(f"  ❌ Failed with status {response.status_code}")
                return {"test": "lightning_perception", "status": "failed", "error": response.text}
        except Exception as e:
            print(f"  ❌ Error: {e}")
            return {"test": "lightning_perception", "status": "error", "error": str(e)}
    
    def test_quick_perception(self, url: str = "https://example.com") -> Dict:
        """Test Quick perception layer"""
        print(f"\n🔍 Testing Quick Perception on {url}...")
        
        try:
            start_time = time.time()
            response = requests.post(
                f"{self.api_url}/perceive/quick",
                json={"url": url}
            )
            elapsed_ms = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                data = response.json()
                print(f"  ✅ Quick perception completed in {elapsed_ms:.1f}ms")
                print(f"  Interactive elements: {len(data.get('interaction_elements', []))}")
                print(f"  Layout type: {data.get('layout_structure', {}).get('layout_type', 'unknown')}")
                print(f"  Navigation items: {len(data.get('navigation_paths', []))}")
                print(f"  Forms found: {len(data.get('form_analysis', []))}")
                
                # Check if meets performance target
                if elapsed_ms < 200:
                    print(f"  🎯 Meets <200ms target!")
                else:
                    print(f"  ⚠️  Exceeds 200ms target ({elapsed_ms:.1f}ms)")
                
                return {
                    "test": "quick_perception",
                    "status": "passed",
                    "time_ms": elapsed_ms,
                    "meets_target": elapsed_ms < 200,
                    "interactions": len(data.get('interaction_elements', [])),
                    "url": url
                }
            else:
                print(f"  ❌ Failed with status {response.status_code}")
                return {"test": "quick_perception", "status": "failed", "error": response.text}
        except Exception as e:
            print(f"  ❌ Error: {e}")
            return {"test": "quick_perception", "status": "error", "error": str(e)}
    
    def test_caching_performance(self, url: str = "https://example.com") -> Dict:
        """Test caching effectiveness"""
        print(f"\n💾 Testing Caching Performance on {url}...")
        
        times = []
        
        try:
            # First scan (cache miss)
            start_time = time.time()
            response = requests.post(
                f"{self.api_url}/perceive/lightning",
                json={"url": url}
            )
            first_time_ms = (time.time() - start_time) * 1000
            times.append(first_time_ms)
            print(f"  First scan: {first_time_ms:.1f}ms (cache miss)")
            
            # Subsequent scans (cache hits)
            for i in range(4):
                start_time = time.time()
                response = requests.post(
                    f"{self.api_url}/perceive/lightning",
                    json={"url": url}
                )
                scan_time_ms = (time.time() - start_time) * 1000
                times.append(scan_time_ms)
                print(f"  Scan {i+2}: {scan_time_ms:.1f}ms")
            
            # Calculate improvement
            avg_cached = sum(times[1:]) / len(times[1:])
            improvement = ((first_time_ms - avg_cached) / first_time_ms) * 100
            speedup = first_time_ms / avg_cached
            
            print(f"\n  📊 Cache Performance:")
            print(f"  First scan: {first_time_ms:.1f}ms")
            print(f"  Avg cached: {avg_cached:.1f}ms")
            print(f"  Improvement: {improvement:.1f}%")
            print(f"  Speedup: {speedup:.2f}x")
            
            if improvement > 20:
                print(f"  ✅ Significant cache improvement!")
            else:
                print(f"  ⚠️  Limited cache improvement")
            
            return {
                "test": "caching_performance",
                "status": "passed",
                "first_scan_ms": first_time_ms,
                "avg_cached_ms": avg_cached,
                "improvement_percent": improvement,
                "speedup_factor": speedup
            }
        except Exception as e:
            print(f"  ❌ Error: {e}")
            return {"test": "caching_performance", "status": "error", "error": str(e)}
    
    def test_natural_language(self) -> Dict:
        """Test natural language element finding"""
        print("\n🗣️ Testing Natural Language Element Finding...")
        
        test_cases = [
            ("https://google.com", "search box", "Should find search input"),
            ("https://github.com", "sign in button", "Should find login button"),
            ("https://example.com", "more information link", "Should find the link"),
        ]
        
        results = []
        
        for url, description, expected in test_cases:
            print(f"\n  Testing: '{description}' on {url}")
            print(f"  Expected: {expected}")
            
            try:
                # First navigate to the page
                nav_response = requests.post(
                    f"{self.api_url}/navigate",
                    json={"url": url}
                )
                
                if nav_response.status_code != 200:
                    print(f"    ❌ Failed to navigate to {url}")
                    results.append({"description": description, "status": "failed", "error": "navigation"})
                    continue
                
                # Find element using natural language
                response = requests.post(
                    f"{self.api_url}/find/natural",
                    json={"description": description}
                )
                
                if response.status_code == 200:
                    data = response.json()
                    print(f"    ✅ Found: {data.get('selector', 'unknown')}")
                    print(f"    Type: {data.get('element_type', 'unknown')}")
                    print(f"    Confidence: {data.get('confidence', 0):.2f}")
                    results.append({
                        "description": description,
                        "status": "found",
                        "selector": data.get('selector'),
                        "confidence": data.get('confidence')
                    })
                else:
                    print(f"    ❌ Not found")
                    results.append({"description": description, "status": "not_found"})
            except Exception as e:
                print(f"    ❌ Error: {e}")
                results.append({"description": description, "status": "error", "error": str(e)})
        
        successful = sum(1 for r in results if r["status"] == "found")
        success_rate = (successful / len(results)) * 100 if results else 0
        
        print(f"\n  📊 Natural Language Results:")
        print(f"  Success rate: {success_rate:.1f}% ({successful}/{len(results)})")
        
        return {
            "test": "natural_language",
            "status": "passed" if success_rate > 50 else "failed",
            "success_rate": success_rate,
            "results": results
        }
    
    def test_performance_across_sites(self) -> Dict:
        """Test performance across different websites"""
        print("\n🌐 Testing Performance Across Different Sites...")
        
        test_sites = [
            ("https://example.com", "Simple static page"),
            ("https://google.com", "Complex search engine"),
            ("https://github.com", "Dynamic web application"),
            ("https://wikipedia.org", "Content-heavy site"),
        ]
        
        results = []
        
        for url, description in test_sites:
            print(f"\n  Testing {url} ({description})...")
            
            # Test Lightning
            lightning = self.test_lightning_perception(url)
            
            # Test Quick
            quick = self.test_quick_perception(url)
            
            results.append({
                "url": url,
                "description": description,
                "lightning_ms": lightning.get("time_ms", -1),
                "lightning_meets_target": lightning.get("meets_target", False),
                "quick_ms": quick.get("time_ms", -1),
                "quick_meets_target": quick.get("meets_target", False),
            })
        
        # Calculate averages
        avg_lightning = sum(r["lightning_ms"] for r in results) / len(results)
        avg_quick = sum(r["quick_ms"] for r in results) / len(results)
        lightning_success = sum(1 for r in results if r["lightning_meets_target"])
        quick_success = sum(1 for r in results if r["quick_meets_target"])
        
        print("\n  📊 Performance Summary:")
        print(f"  Lightning avg: {avg_lightning:.1f}ms")
        print(f"  Lightning success: {lightning_success}/{len(results)} sites meet target")
        print(f"  Quick avg: {avg_quick:.1f}ms")
        print(f"  Quick success: {quick_success}/{len(results)} sites meet target")
        
        return {
            "test": "performance_across_sites",
            "status": "passed",
            "avg_lightning_ms": avg_lightning,
            "avg_quick_ms": avg_quick,
            "lightning_success_rate": (lightning_success / len(results)) * 100,
            "quick_success_rate": (quick_success / len(results)) * 100,
            "sites": results
        }
    
    def run_full_suite(self):
        """Run the complete test suite"""
        print("="*60)
        print("🧪 PERCEPTION MODULE TEST SUITE")
        print("="*60)
        print(f"Timestamp: {datetime.now().isoformat()}")
        print(f"API URL: {self.api_url}")
        print(f"ChromeDriver URL: {self.chromedriver_url}")
        print("="*60)
        
        # Check services
        if not self.check_services():
            print("\n❌ Required services are not running!")
            print("Please ensure:")
            print("  1. API server is running (./start.sh)")
            print("  2. ChromeDriver is running (./chromedriver.exe --port=9515)")
            return
        
        # Run tests
        tests = [
            ("Browser Connection", self.test_browser_connection),
            ("Lightning Perception", lambda: self.test_lightning_perception()),
            ("Quick Perception", lambda: self.test_quick_perception()),
            ("Caching Performance", lambda: self.test_caching_performance()),
            ("Natural Language", self.test_natural_language),
            ("Cross-Site Performance", self.test_performance_across_sites),
        ]
        
        for test_name, test_func in tests:
            print(f"\n{'='*60}")
            print(f"Running: {test_name}")
            print("="*60)
            
            result = test_func()
            self.test_results.append(result)
            
            time.sleep(1)  # Brief pause between tests
        
        # Generate summary
        self.print_summary()
        
        # Save results
        self.save_results()
    
    def print_summary(self):
        """Print test summary"""
        print("\n" + "="*60)
        print("📈 TEST SUMMARY")
        print("="*60)
        
        passed = sum(1 for r in self.test_results if r.get("status") == "passed")
        failed = sum(1 for r in self.test_results if r.get("status") == "failed")
        errors = sum(1 for r in self.test_results if r.get("status") == "error")
        
        print(f"\nTotal Tests: {len(self.test_results)}")
        print(f"  ✅ Passed: {passed}")
        print(f"  ❌ Failed: {failed}")
        print(f"  ⚠️  Errors: {errors}")
        
        success_rate = (passed / len(self.test_results)) * 100 if self.test_results else 0
        print(f"\nSuccess Rate: {success_rate:.1f}%")
        
        # Performance highlights
        for result in self.test_results:
            if "time_ms" in result:
                print(f"\n{result['test']}:")
                print(f"  Time: {result['time_ms']:.1f}ms")
                if "meets_target" in result:
                    print(f"  Meets target: {'Yes' if result['meets_target'] else 'No'}")
        
        # Overall verdict
        print("\n" + "="*60)
        if success_rate >= 80:
            print("✨ PERCEPTION MODULE: EXCELLENT")
        elif success_rate >= 60:
            print("✅ PERCEPTION MODULE: GOOD")
        elif success_rate >= 40:
            print("⚠️  PERCEPTION MODULE: NEEDS IMPROVEMENT")
        else:
            print("❌ PERCEPTION MODULE: CRITICAL ISSUES")
        print("="*60)
    
    def save_results(self):
        """Save test results to file"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"perception_test_results_{timestamp}.json"
        
        with open(filename, 'w') as f:
            json.dump({
                "timestamp": datetime.now().isoformat(),
                "api_url": self.api_url,
                "results": self.test_results
            }, f, indent=2)
        
        print(f"\n📁 Results saved to {filename}")

def main():
    # Parse command line arguments
    api_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3001"
    
    # Run test suite
    suite = PerceptionTestSuite(api_url)
    suite.run_full_suite()

if __name__ == "__main__":
    main()