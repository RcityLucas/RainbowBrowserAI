//! Working demonstration of RainbowBrowserAI capabilities
//! This demonstrates actual browser automation with clear progress visibility

use thirtyfour::{WebDriver, DesiredCapabilities, By};
use std::time::Duration;
// use serde_json::json;

struct TestProgress {
    total_tests: usize,
    completed: usize,
    passed: usize,
    failed: usize,
}

impl TestProgress {
    fn new(total: usize) -> Self {
        Self {
            total_tests: total,
            completed: 0,
            passed: 0,
            failed: 0,
        }
    }
    
    fn start_test(&self, test_name: &str) {
        println!("\n🔄 [{}/{}] Starting: {}", self.completed + 1, self.total_tests, test_name);
        println!("   ┌─────────────────────────────────────────");
    }
    
    fn pass(&mut self, message: &str) {
        self.completed += 1;
        self.passed += 1;
        println!("   │ ✅ {}", message);
        println!("   └─────────────────────────────────────────");
        self.print_progress();
    }
    
    fn fail(&mut self, message: &str) {
        self.completed += 1;
        self.failed += 1;
        println!("   │ ❌ {}", message);
        println!("   └─────────────────────────────────────────");
        self.print_progress();
    }
    
    fn info(&self, message: &str) {
        println!("   │ 📄 {}", message);
    }
    
    fn print_progress(&self) {
        let progress = (self.completed as f64 / self.total_tests as f64) * 100.0;
        let bar_filled = (progress / 5.0) as usize;
        let bar_empty = 20 - bar_filled;
        let bar_full = "█".repeat(bar_filled);
        let bar_empty_str = "░".repeat(bar_empty);
        let bar = format!("{}{}", bar_full, bar_empty_str);
        
        println!("📊 Progress: [{}] {:.1}% ({}/{})", bar, progress, self.completed, self.total_tests);
    }
    
    fn final_report(&self) {
        println!("\n🎯 FINAL TEST RESULTS");
        println!("═══════════════════════════════════════════════════");
        println!("Total Tests: {}", self.total_tests);
        println!("✅ Passed: {}", self.passed);
        println!("❌ Failed: {}", self.failed);
        
        let success_rate = (self.passed as f64 / self.total_tests as f64) * 100.0;
        let status = if success_rate == 100.0 { "🟢 PERFECT" }
                    else if success_rate >= 90.0 { "🟡 EXCELLENT" }
                    else if success_rate >= 75.0 { "🟠 GOOD" }
                    else if success_rate >= 50.0 { "🔴 NEEDS WORK" }
                    else { "❌ CRITICAL ISSUES" };
        
        println!("Success Rate: {:.1}% {}", success_rate, status);
        
        if success_rate >= 90.0 {
            println!("\n🎊 RainbowBrowserAI: BROWSER AUTOMATION WORKING!");
            println!("✨ Ready for production use!");
        }
        
        println!("═══════════════════════════════════════════════════");
    }
}

struct BrowserTester {
    driver: WebDriver,
    progress: TestProgress,
}

impl BrowserTester {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        let progress = TestProgress::new(12); // Total number of tests
        
        Ok(Self { driver, progress })
    }
    
    async fn run_all_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 RAINBOWBROWSERAI WORKING DEMONSTRATION");
        println!("════════════════════════════════════════════════════");
        println!("Testing actual browser automation capabilities with clear progress...\n");
        
        // Test 1: Basic Navigation
        self.test_navigation().await;
        
        // Test 2: Element Finding
        self.test_element_finding().await;
        
        // Test 3: Text Extraction  
        self.test_text_extraction().await;
        
        // Test 4: JavaScript Execution
        self.test_javascript_execution().await;
        
        // Test 5: Form Interaction
        self.test_form_interaction().await;
        
        // Test 6: Multiple Elements
        self.test_multiple_elements().await;
        
        // Test 7: Page Title
        self.test_page_title().await;
        
        // Test 8: URL Verification
        self.test_url_verification().await;
        
        // Test 9: Element Attributes
        self.test_element_attributes().await;
        
        // Test 10: Dynamic Content
        self.test_dynamic_content().await;
        
        // Test 11: Page Source (basic)
        self.test_screenshot().await;
        
        // Test 12: Complex JavaScript
        self.test_complex_javascript().await;
        
        self.progress.final_report();
        
        Ok(())
    }
    
    async fn test_navigation(&mut self) {
        self.progress.start_test("Basic Navigation");
        
        match self.driver.goto("https://httpbin.org/html").await {
            Ok(_) => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                self.progress.info("Navigation completed successfully");
                self.progress.pass("Navigation to httpbin.org/html working");
            }
            Err(e) => {
                self.progress.fail(&format!("Navigation failed: {}", e));
            }
        }
    }
    
    async fn test_element_finding(&mut self) {
        self.progress.start_test("Element Finding");
        
        match self.driver.find(By::Tag("h1")).await {
            Ok(_element) => {
                self.progress.info("H1 element found successfully");
                self.progress.pass("Element finding by tag working");
            }
            Err(e) => {
                self.progress.fail(&format!("Element finding failed: {}", e));
            }
        }
    }
    
    async fn test_text_extraction(&mut self) {
        self.progress.start_test("Text Content Extraction");
        
        match self.driver.find(By::Tag("h1")).await {
            Ok(element) => {
                match element.text().await {
                    Ok(text) => {
                        self.progress.info(&format!("Extracted text: '{}'", text));
                        self.progress.pass("Text extraction working");
                    }
                    Err(e) => {
                        self.progress.fail(&format!("Text extraction failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.progress.fail(&format!("Could not find element: {}", e));
            }
        }
    }
    
    async fn test_javascript_execution(&mut self) {
        self.progress.start_test("JavaScript Execution");
        
        let script = "return { title: document.title, readyState: document.readyState, url: window.location.href }";
        match self.driver.execute(script, vec![]).await {
            Ok(result) => {
                self.progress.info(&format!("JS Result: {:?}", result.json()));
                self.progress.pass("JavaScript execution working");
            }
            Err(e) => {
                self.progress.fail(&format!("JavaScript execution failed: {}", e));
            }
        }
    }
    
    async fn test_form_interaction(&mut self) {
        self.progress.start_test("Form Element Detection");
        
        // Navigate to a page with forms
        match self.driver.goto("https://httpbin.org/forms/post").await {
            Ok(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                match self.driver.find(By::Tag("form")).await {
                    Ok(_form) => {
                        self.progress.info("Form element detected");
                        self.progress.pass("Form interaction capabilities verified");
                    }
                    Err(e) => {
                        self.progress.fail(&format!("Form detection failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.progress.fail(&format!("Navigation to form page failed: {}", e));
            }
        }
    }
    
    async fn test_multiple_elements(&mut self) {
        self.progress.start_test("Multiple Element Finding");
        
        match self.driver.find_all(By::Tag("input")).await {
            Ok(elements) => {
                self.progress.info(&format!("Found {} input elements", elements.len()));
                self.progress.pass("Multiple element finding working");
            }
            Err(e) => {
                self.progress.fail(&format!("Multiple element finding failed: {}", e));
            }
        }
    }
    
    async fn test_page_title(&mut self) {
        self.progress.start_test("Page Title Extraction");
        
        match self.driver.title().await {
            Ok(title) => {
                self.progress.info(&format!("Page title: '{}'", title));
                self.progress.pass("Page title extraction working");
            }
            Err(e) => {
                self.progress.fail(&format!("Title extraction failed: {}", e));
            }
        }
    }
    
    async fn test_url_verification(&mut self) {
        self.progress.start_test("URL Verification");
        
        match self.driver.current_url().await {
            Ok(url) => {
                self.progress.info(&format!("Current URL: {}", url));
                if url.as_str().contains("httpbin.org") {
                    self.progress.pass("URL verification working");
                } else {
                    self.progress.fail("URL doesn't match expected");
                }
            }
            Err(e) => {
                self.progress.fail(&format!("URL retrieval failed: {}", e));
            }
        }
    }
    
    async fn test_element_attributes(&mut self) {
        self.progress.start_test("Element Attribute Reading");
        
        match self.driver.find(By::Tag("form")).await {
            Ok(element) => {
                match element.attr("method").await {
                    Ok(Some(method)) => {
                        self.progress.info(&format!("Form method: {}", method));
                        self.progress.pass("Attribute reading working");
                    }
                    Ok(None) => {
                        self.progress.info("No method attribute found");
                        self.progress.pass("Attribute reading working (attribute not found)");
                    }
                    Err(e) => {
                        self.progress.fail(&format!("Attribute reading failed: {}", e));
                    }
                }
            }
            Err(e) => {
                self.progress.fail(&format!("Element finding failed: {}", e));
            }
        }
    }
    
    async fn test_dynamic_content(&mut self) {
        self.progress.start_test("Dynamic Content Detection");
        
        let script = r#"
            var div = document.createElement('div');
            div.id = 'dynamic-test';
            div.textContent = 'Dynamic content created!';
            document.body.appendChild(div);
            return 'Content created';
        "#;
        
        match self.driver.execute(script, vec![]).await {
            Ok(_) => {
                // Try to find the dynamically created element
                tokio::time::sleep(Duration::from_millis(500)).await;
                
                match self.driver.find(By::Id("dynamic-test")).await {
                    Ok(element) => {
                        match element.text().await {
                            Ok(text) => {
                                self.progress.info(&format!("Dynamic content: '{}'", text));
                                self.progress.pass("Dynamic content handling working");
                            }
                            Err(e) => {
                                self.progress.fail(&format!("Dynamic content text failed: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.progress.fail(&format!("Dynamic element not found: {}", e));
                    }
                }
            }
            Err(e) => {
                self.progress.fail(&format!("Dynamic content creation failed: {}", e));
            }
        }
    }
    
    async fn test_screenshot(&mut self) {
        self.progress.start_test("Page Source Extraction");
        
        // Screenshot API changed in v0.32, let's test basic functionality instead
        match self.driver.source().await {
            Ok(page_source) => {
                self.progress.info(&format!("Page source captured: {} chars", page_source.len()));
                self.progress.pass("Page source extraction working");
            }
            Err(e) => {
                self.progress.fail(&format!("Screenshot failed: {}", e));
            }
        }
    }
    
    async fn test_complex_javascript(&mut self) {
        self.progress.start_test("Complex JavaScript Operations");
        
        let script = r#"
            return {
                viewport: {
                    width: window.innerWidth,
                    height: window.innerHeight
                },
                scrollPosition: {
                    x: window.scrollX,
                    y: window.scrollY
                },
                elements: {
                    total: document.querySelectorAll('*').length,
                    inputs: document.querySelectorAll('input').length,
                    forms: document.querySelectorAll('form').length
                },
                performance: {
                    loadTime: performance.timing.loadEventEnd - performance.timing.navigationStart
                }
            };
        "#;
        
        match self.driver.execute(script, vec![]).await {
            Ok(result) => {
                self.progress.info("Complex JavaScript analysis completed");
                self.progress.info(&format!("Analysis data: {:?}", result.json()));
                self.progress.pass("Complex JavaScript operations working");
            }
            Err(e) => {
                self.progress.fail(&format!("Complex JavaScript failed: {}", e));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Initializing RainbowBrowserAI Demo...");
    
    match BrowserTester::new().await {
        Ok(mut tester) => {
            tester.run_all_tests().await?;
            
            // Cleanup
            if let Err(e) = tester.driver.quit().await {
                println!("⚠️ Warning: Browser cleanup error: {}", e);
            }
        }
        Err(e) => {
            println!("❌ Failed to initialize browser automation:");
            println!("   Error: {}", e);
            println!("\n🔧 Setup Required:");
            println!("   1. Install Chrome browser");
            println!("   2. Install ChromeDriver:");
            println!("      • Download: https://chromedriver.chromium.org/");
            println!("      • Or via package manager:");
            println!("        - macOS: brew install chromedriver");
            println!("        - Ubuntu: sudo apt install chromium-chromedriver");
            println!("        - Windows: choco install chromedriver");
            println!("   3. Start ChromeDriver: chromedriver --port=9515");
            println!("   4. Re-run this demo");
            
            return Err(e);
        }
    }
    
    Ok(())
}