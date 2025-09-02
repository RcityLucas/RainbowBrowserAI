#!/usr/bin/env python3
"""
Perception Performance Benchmark Tool
Tests Lightning (<50ms) and Quick (<200ms) perception layers
"""

import json
import time
import requests
import statistics
from typing import Dict, List, Tuple
from datetime import datetime

class PerceptionBenchmark:
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
                                "args": ["--headless", "--no-sandbox", "--disable-dev-shm-usage"]
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
    
    def navigate(self, url: str) -> bool:
        """Navigate to a URL"""
        try:
            response = requests.post(
                f"{self.chromedriver_url}/session/{self.session_id}/url",
                json={"url": url}
            )
            time.sleep(1)  # Wait for page load
            return response.status_code == 200
        except:
            return False
    
    def execute_script(self, script: str) -> Tuple[Dict, float]:
        """Execute JavaScript and measure time"""
        start_time = time.perf_counter()
        
        response = requests.post(
            f"{self.chromedriver_url}/session/{self.session_id}/execute/sync",
            json={"script": script, "args": []}
        )
        
        total_time_ms = (time.perf_counter() - start_time) * 1000
        
        try:
            result = response.json()
            return result.get('value', {}), total_time_ms
        except:
            return {}, total_time_ms
    
    def lightning_perception_script(self) -> str:
        """Lightning perception JavaScript (<50ms target)"""
        return """
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
    
    def quick_perception_script(self) -> str:
        """Quick perception JavaScript (<200ms target)"""
        return """
        const start = performance.now();
        
        // Analyze interactive elements
        const interactions = [];
        const interactiveEls = document.querySelectorAll(
            'button, a[href], input, select, textarea, [onclick], [role="button"]'
        );
        
        for (let i = 0; i < Math.min(30, interactiveEls.length); i++) {
            const el = interactiveEls[i];
            const rect = el.getBoundingClientRect();
            if (rect.width > 0 && rect.height > 0) {
                interactions.push({
                    type: el.tagName,
                    interactive: true,
                    text: (el.innerText || el.value || '').substring(0, 20),
                    x: rect.x,
                    y: rect.y
                });
            }
        }
        
        // Analyze layout
        const layout = {
            hasHeader: !!document.querySelector('header, .header, #header'),
            hasNav: !!document.querySelector('nav, .navigation, .nav'),
            hasSidebar: !!document.querySelector('aside, .sidebar, #sidebar'),
            hasFooter: !!document.querySelector('footer, .footer, #footer'),
            hasForm: !!document.querySelector('form'),
            mainContent: !!document.querySelector('main, [role="main"], #main, .main')
        };
        
        // Count navigation links
        const navLinks = document.querySelectorAll('nav a, .nav a, .navigation a');
        
        // Analyze forms
        const forms = [];
        document.querySelectorAll('form').forEach((form, idx) => {
            if (idx < 3) {
                forms.push({
                    fields: form.querySelectorAll('input, select, textarea').length,
                    hasSubmit: !!form.querySelector('[type="submit"]')
                });
            }
        });
        
        return {
            interactions_count: interactions.length,
            interactions_sample: interactions.slice(0, 5),
            layout: layout,
            nav_links: navLinks.length,
            forms: forms,
            browser_time_ms: performance.now() - start,
            timestamp: new Date().toISOString()
        };
        """
    
    def benchmark_site(self, url: str, runs: int = 3) -> Dict:
        """Benchmark perception on a specific site"""
        print(f"\n🌐 Testing: {url}")
        print("─" * 40)
        
        if not self.navigate(url):
            return {"error": "Failed to navigate"}
        
        results = {
            "url": url,
            "lightning": [],
            "quick": []
        }
        
        # Run Lightning perception tests
        print("⚡ Lightning Perception (<50ms target):")
        for i in range(runs):
            result, total_time = self.execute_script(self.lightning_perception_script())
            browser_time = result.get('browser_time_ms', 0)
            element_count = result.get('count', 0)
            
            results["lightning"].append({
                "total_ms": round(total_time, 2),
                "browser_ms": round(browser_time, 2),
                "elements": element_count
            })
            
            status = "✅" if browser_time < 50 else "⚠️"
            print(f"  Run {i+1}: Total={total_time:.1f}ms, Browser={browser_time:.1f}ms, Elements={element_count} {status}")
        
        # Run Quick perception tests
        print("\n🔍 Quick Perception (<200ms target):")
        for i in range(runs):
            result, total_time = self.execute_script(self.quick_perception_script())
            browser_time = result.get('browser_time_ms', 0)
            interactions = result.get('interactions_count', 0)
            nav_links = result.get('nav_links', 0)
            forms_count = len(result.get('forms', []))
            
            results["quick"].append({
                "total_ms": round(total_time, 2),
                "browser_ms": round(browser_time, 2),
                "interactions": interactions,
                "nav_links": nav_links,
                "forms": forms_count
            })
            
            status = "✅" if browser_time < 200 else "⚠️"
            print(f"  Run {i+1}: Total={total_time:.1f}ms, Browser={browser_time:.1f}ms, Interactive={interactions}, Nav={nav_links} {status}")
        
        # Calculate statistics
        lightning_browser_times = [r['browser_ms'] for r in results['lightning']]
        quick_browser_times = [r['browser_ms'] for r in results['quick']]
        
        results["stats"] = {
            "lightning": {
                "avg_browser_ms": round(statistics.mean(lightning_browser_times), 2),
                "min_browser_ms": round(min(lightning_browser_times), 2),
                "max_browser_ms": round(max(lightning_browser_times), 2),
                "meets_target": statistics.mean(lightning_browser_times) < 50
            },
            "quick": {
                "avg_browser_ms": round(statistics.mean(quick_browser_times), 2),
                "min_browser_ms": round(min(quick_browser_times), 2),
                "max_browser_ms": round(max(quick_browser_times), 2),
                "meets_target": statistics.mean(quick_browser_times) < 200
            }
        }
        
        return results
    
    def test_caching_effect(self, url: str, runs: int = 5) -> List[float]:
        """Test the effect of caching on repeated scans"""
        print(f"\n📊 Testing cache effect ({runs} runs):")
        self.navigate(url)
        
        times = []
        for i in range(runs):
            _, total_time = self.execute_script(self.lightning_perception_script())
            times.append(total_time)
            print(f"  Run {i+1}: {total_time:.1f}ms")
        
        if len(times) > 1:
            improvement = ((times[0] - times[-1]) / times[0]) * 100
            print(f"  Cache improvement: {improvement:.1f}%")
        
        return times
    
    def cleanup(self):
        """Close the browser session"""
        if self.session_id:
            try:
                requests.delete(f"{self.chromedriver_url}/session/{self.session_id}")
                print("✅ Session closed")
            except:
                pass
    
    def run_full_benchmark(self):
        """Run complete benchmark suite"""
        print("=" * 50)
        print("🚀 Perception Performance Benchmark")
        print("=" * 50)
        
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
            print("Please start it with: ./chromedriver_v120.exe --port=9515")
            return
        
        # Create session
        if not self.create_session():
            return
        
        # Test sites
        test_sites = [
            "https://example.com",
            "https://google.com",
            "https://github.com",
            "https://wikipedia.org"
        ]
        
        all_results = []
        
        for site in test_sites:
            results = self.benchmark_site(site, runs=3)
            all_results.append(results)
        
        # Test caching on example.com
        self.test_caching_effect("https://example.com", runs=5)
        
        # Print summary
        print("\n" + "=" * 50)
        print("📈 Performance Summary")
        print("=" * 50)
        
        for result in all_results:
            if "error" not in result:
                url = result['url'].replace('https://', '')
                lightning_stats = result['stats']['lightning']
                quick_stats = result['stats']['quick']
                
                print(f"\n{url}:")
                print(f"  Lightning: {lightning_stats['avg_browser_ms']}ms avg " +
                      ("✅" if lightning_stats['meets_target'] else "❌"))
                print(f"  Quick:     {quick_stats['avg_browser_ms']}ms avg " +
                      ("✅" if quick_stats['meets_target'] else "❌"))
        
        # Save results to file
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"perception_benchmark_{timestamp}.json"
        with open(filename, 'w') as f:
            json.dump(all_results, f, indent=2)
        print(f"\n📁 Results saved to {filename}")
        
        # Cleanup
        self.cleanup()
        
        print("\n✨ Benchmark complete!")

if __name__ == "__main__":
    benchmark = PerceptionBenchmark()
    benchmark.run_full_benchmark()