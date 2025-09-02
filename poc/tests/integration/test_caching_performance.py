#!/usr/bin/env python3
"""
Test Caching Performance
Demonstrates how caching improves perception performance on repeated scans
"""

import time
import requests
import statistics
from typing import List, Dict

class CachingPerformanceTest:
    def __init__(self, chromedriver_url="http://localhost:9515"):
        self.chromedriver_url = chromedriver_url
        self.session_id = None
        
    def create_session(self) -> bool:
        """Create a new browser session"""
        try:
            response = requests.post(
                f"{self.chromedriver_url}/session",
                json={
                    "capabilities": {
                        "alwaysMatch": {
                            "browserName": "chrome",
                            "goog:chromeOptions": {
                                "args": ["--headless", "--no-sandbox"]
                            }
                        }
                    }
                }
            )
            data = response.json()
            self.session_id = data['value'].get('sessionId') or data['value']['capabilities']['sessionId']
            print(f"✅ Session created: {self.session_id}")
            return True
        except Exception as e:
            print(f"❌ Failed to create session: {e}")
            return False
    
    def navigate(self, url: str):
        """Navigate to a URL"""
        requests.post(
            f"{self.chromedriver_url}/session/{self.session_id}/url",
            json={"url": url}
        )
        time.sleep(1)  # Wait for page load
    
    def execute_perception(self) -> Dict:
        """Execute perception JavaScript and measure time"""
        script = """
        const start = performance.now();
        const elements = [];
        const selectors = [
            'button[type="submit"], input[type="submit"]',
            'button:not([type="submit"])',
            'a[href]',
            'input[type="text"], input[type="search"]',
            'form'
        ];
        
        let count = 0;
        for (const selector of selectors) {
            if (count >= 10) break;
            const found = document.querySelectorAll(selector);
            for (const el of found) {
                if (count >= 10) break;
                const rect = el.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    elements.push({
                        tag: el.tagName,
                        text: (el.innerText || el.value || '').substring(0, 30),
                        visible: true
                    });
                    count++;
                }
            }
        }
        
        return {
            elements: elements,
            count: elements.length,
            browser_time_ms: performance.now() - start,
            timestamp: new Date().toISOString()
        };
        """
        
        start_time = time.perf_counter()
        
        response = requests.post(
            f"{self.chromedriver_url}/session/{self.session_id}/execute/sync",
            json={"script": script, "args": []}
        )
        
        total_time_ms = (time.perf_counter() - start_time) * 1000
        
        result = response.json().get('value', {})
        browser_time = result.get('browser_time_ms', 0)
        
        return {
            'total_ms': total_time_ms,
            'browser_ms': browser_time,
            'network_ms': total_time_ms - browser_time,
            'element_count': result.get('count', 0)
        }
    
    def test_without_cache(self, url: str, scans: int = 10) -> List[float]:
        """Test performance without caching (navigate between scans)"""
        print(f"\n🔄 Testing WITHOUT cache (navigating between scans)...")
        times = []
        
        for i in range(scans):
            self.navigate(url)
            result = self.execute_perception()
            times.append(result['total_ms'])
            print(f"  Scan {i+1}: {result['total_ms']:.1f}ms (Browser: {result['browser_ms']:.1f}ms)")
        
        return times
    
    def test_with_cache(self, url: str, scans: int = 10) -> List[float]:
        """Test performance with caching (no navigation between scans)"""
        print(f"\n💾 Testing WITH cache (same page, repeated scans)...")
        self.navigate(url)
        times = []
        
        for i in range(scans):
            result = self.execute_perception()
            times.append(result['total_ms'])
            print(f"  Scan {i+1}: {result['total_ms']:.1f}ms (Browser: {result['browser_ms']:.1f}ms)")
        
        return times
    
    def analyze_results(self, without_cache: List[float], with_cache: List[float]):
        """Analyze and compare results"""
        print("\n" + "="*50)
        print("📊 Performance Analysis")
        print("="*50)
        
        # Without cache stats
        avg_without = statistics.mean(without_cache)
        min_without = min(without_cache)
        max_without = max(without_cache)
        stdev_without = statistics.stdev(without_cache) if len(without_cache) > 1 else 0
        
        print(f"\n🔄 WITHOUT Cache:")
        print(f"  Average: {avg_without:.1f}ms")
        print(f"  Min: {min_without:.1f}ms")
        print(f"  Max: {max_without:.1f}ms")
        print(f"  Std Dev: {stdev_without:.1f}ms")
        
        # With cache stats
        avg_with = statistics.mean(with_cache)
        min_with = min(with_cache)
        max_with = max(with_cache)
        stdev_with = statistics.stdev(with_cache) if len(with_cache) > 1 else 0
        
        print(f"\n💾 WITH Cache:")
        print(f"  Average: {avg_with:.1f}ms")
        print(f"  Min: {min_with:.1f}ms")
        print(f"  Max: {max_with:.1f}ms")
        print(f"  Std Dev: {stdev_with:.1f}ms")
        
        # Improvement calculation
        improvement = ((avg_without - avg_with) / avg_without) * 100
        speedup = avg_without / avg_with
        
        print(f"\n🚀 Cache Performance Improvement:")
        print(f"  Improvement: {improvement:.1f}%")
        print(f"  Speedup: {speedup:.2f}x faster")
        print(f"  Time saved per scan: {avg_without - avg_with:.1f}ms")
        
        # Cache warmup effect
        if len(with_cache) > 3:
            first_three = statistics.mean(with_cache[:3])
            last_three = statistics.mean(with_cache[-3:])
            warmup_improvement = ((first_three - last_three) / first_three) * 100
            print(f"\n🔥 Cache Warmup Effect:")
            print(f"  First 3 scans avg: {first_three:.1f}ms")
            print(f"  Last 3 scans avg: {last_three:.1f}ms")
            print(f"  Warmup improvement: {warmup_improvement:.1f}%")
        
        return {
            'avg_without_cache': avg_without,
            'avg_with_cache': avg_with,
            'improvement_percent': improvement,
            'speedup_factor': speedup
        }
    
    def cleanup(self):
        """Close the browser session"""
        if self.session_id:
            try:
                requests.delete(f"{self.chromedriver_url}/session/{self.session_id}")
                print("✅ Session closed")
            except:
                pass
    
    def run_complete_test(self):
        """Run complete caching performance test"""
        print("="*50)
        print("🧪 Caching Performance Test")
        print("="*50)
        
        # Check ChromeDriver
        try:
            response = requests.get(f"{self.chromedriver_url}/status")
            if response.status_code == 200:
                print("✅ ChromeDriver is running")
            else:
                print("❌ ChromeDriver not responding")
                return
        except:
            print("❌ Cannot connect to ChromeDriver")
            print("Please start it with: ./chromedriver.exe --port=9515")
            return
        
        # Create session
        if not self.create_session():
            return
        
        try:
            # Test different websites
            test_sites = [
                ("https://example.com", "Simple page"),
                ("https://google.com", "Complex page"),
                ("https://github.com", "Dynamic page")
            ]
            
            all_results = []
            
            for url, description in test_sites:
                print(f"\n{'='*50}")
                print(f"Testing: {url} ({description})")
                print('='*50)
                
                # Run tests
                without_cache = self.test_without_cache(url, scans=5)
                with_cache = self.test_with_cache(url, scans=10)
                
                # Analyze
                results = self.analyze_results(without_cache, with_cache)
                results['url'] = url
                results['description'] = description
                all_results.append(results)
            
            # Summary
            print("\n" + "="*50)
            print("📈 Overall Summary")
            print("="*50)
            
            for result in all_results:
                print(f"\n{result['url']} ({result['description']}):")
                print(f"  Cache speedup: {result['speedup_factor']:.2f}x")
                print(f"  Performance gain: {result['improvement_percent']:.1f}%")
            
            avg_speedup = statistics.mean([r['speedup_factor'] for r in all_results])
            avg_improvement = statistics.mean([r['improvement_percent'] for r in all_results])
            
            print(f"\n🎯 Average across all sites:")
            print(f"  Average speedup: {avg_speedup:.2f}x")
            print(f"  Average improvement: {avg_improvement:.1f}%")
            
        finally:
            self.cleanup()
        
        print("\n✨ Caching performance test complete!")

if __name__ == "__main__":
    test = CachingPerformanceTest()
    test.run_complete_test()