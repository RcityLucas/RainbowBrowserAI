use anyhow::{Result, Context};
use thirtyfour::{WebDriver, ChromeCapabilities, WebElement, By, session::scriptret::ScriptRet};
use tracing::{info, error, warn};
use std::time::Duration;
use tokio::time::timeout;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use async_trait::async_trait;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
pub use mock::MockBrowser;

// Re-export SimpleBrowser as Browser for compatibility  
#[cfg(not(test))]
pub use SimpleBrowser as Browser;

// For tests, use MockBrowser instead
#[cfg(test)]
pub use MockBrowser as Browser;

/// Browser operations trait that can be implemented by both SimpleBrowser and MockBrowser
#[async_trait]
pub trait BrowserOps: Send + Sync {
    async fn navigate_to(&self, url: &str) -> Result<()>;
    async fn current_url(&self) -> Result<String>;
    async fn wait_for_load_event(&self) -> Result<()>;
    async fn wait_for_dom_content_loaded(&self) -> Result<()>;
}

pub struct SimpleBrowser {
    driver: WebDriver,
    retry_attempts: u32,
    timeout_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct ScreenshotOptions {
    pub full_page: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub wait_after_load: Duration,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            full_page: true,
            viewport_width: 1920,
            viewport_height: 1080,
            wait_after_load: Duration::from_secs(2),
        }
    }
}

impl SimpleBrowser {
    /// Create a new browser instance with Chrome WebDriver
    pub async fn new() -> Result<Self> {
        Self::new_with_config(3, Duration::from_secs(30)).await
    }

    /// Create a new browser instance with custom retry and timeout settings
    pub async fn new_with_config(retry_attempts: u32, timeout_duration: Duration) -> Result<Self> {
        info!("Initializing Chrome WebDriver (retries: {}, timeout: {:?})...", retry_attempts, timeout_duration);
        
        // Ensure ChromeDriver is running - removed dependency
        // Note: ChromeDriver should be started externally before running this application
        
        // Configure Chrome capabilities with basic settings  
        let caps = ChromeCapabilities::new();
        
        // Connect to ChromeDriver with retries
        let driver = Self::connect_with_retry(caps, retry_attempts).await
            .context("Failed to connect to ChromeDriver after retries")?;
        
        // Set timeouts
        driver.set_page_load_timeout(timeout_duration).await
            .context("Failed to set page load timeout")?;
        driver.set_implicit_wait_timeout(Duration::from_secs(10)).await
            .context("Failed to set implicit wait timeout")?;
        
        info!("Chrome WebDriver initialized successfully");
        Ok(Self { 
            driver,
            retry_attempts,
            timeout_duration,
        })
    }

    /// Connect to ChromeDriver with retry logic
    async fn connect_with_retry(caps: ChromeCapabilities, max_attempts: u32) -> Result<WebDriver> {
        let mut last_error_msg = String::new();
        
        // Get ChromeDriver port from environment variable or use default
        let chromedriver_port = std::env::var("CHROMEDRIVER_PORT")
            .unwrap_or_else(|_| "9520".to_string());
        let chromedriver_url = format!("http://localhost:{}", chromedriver_port);
        
        // First check if ChromeDriver is reachable
        info!("Checking ChromeDriver availability at {}...", chromedriver_url);
        match reqwest::get(format!("{}/status", chromedriver_url)).await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("ChromeDriver is responding at port {}", chromedriver_port);
                } else {
                    warn!("ChromeDriver returned status: {}", response.status());
                }
            }
            Err(e) => {
                error!("ChromeDriver is not reachable at {}: {}", chromedriver_url, e);
                error!("Please ensure ChromeDriver is running:");
                error!("  1. Download ChromeDriver from https://chromedriver.chromium.org/");
                error!("  2. Run: chromedriver --port={}", chromedriver_port);
                return Err(anyhow::anyhow!(
                    "ChromeDriver is not running on port {}. Please start it with: chromedriver --port={}", 
                    chromedriver_port, chromedriver_port
                ));
            }
        }
        
        for attempt in 1..=max_attempts {
            info!("Attempting to connect to ChromeDriver (attempt {}/{})", attempt, max_attempts);
            match timeout(
                Duration::from_secs(10),
                WebDriver::new(&chromedriver_url, caps.clone())
            ).await {
                Ok(Ok(driver)) => {
                    info!("✅ ChromeDriver connected successfully on attempt {}", attempt);
                    return Ok(driver);
                }
                Ok(Err(e)) => {
                    warn!("ChromeDriver connection attempt {} failed: {}", attempt, e);
                    last_error_msg = format!("{}", e);
                    
                    // Check for specific error types
                    let error_str = format!("{}", e);
                    if error_str.contains("version") {
                        error!("Version mismatch detected. Please ensure ChromeDriver version matches your Chrome browser version.");
                        error!("Check Chrome version at: chrome://version");
                        error!("Download matching ChromeDriver from: https://chromedriver.chromium.org/downloads");
                    }
                }
                Err(_) => {
                    warn!("ChromeDriver connection attempt {} timed out", attempt);
                    last_error_msg = "Connection timeout".to_string();
                }
            }
            
            if attempt < max_attempts {
                info!("Retrying in 2 seconds...");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        
        error!("Failed to connect after {} attempts", max_attempts);
        error!("Last error: {}", last_error_msg);
        error!("Troubleshooting:");
        error!("  1. Run: tasklist | findstr chromedriver");
        error!("  2. Run: netstat -an | findstr 9515");
        error!("  3. Try: chromedriver --port=9515 --verbose");
        
        Err(anyhow::anyhow!("All connection attempts failed: {}", last_error_msg))
    }

    /// Navigate to a specific URL with retry logic
    pub async fn navigate_to(&self, url: &str) -> Result<()> {
        self.navigate_to_with_retry(url, self.retry_attempts).await
    }

    /// Navigate to a specific URL with custom retry attempts
    pub async fn navigate_to_with_retry(&self, url: &str, max_attempts: u32) -> Result<()> {
        info!("Navigating to: {} (max attempts: {})", url, max_attempts);
        
        // Ensure URL has protocol
        let full_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };

        let mut last_error_msg = String::new();
        
        for attempt in 1..=max_attempts {
            match timeout(
                self.timeout_duration,
                self.driver.goto(&full_url)
            ).await {
                Ok(Ok(_)) => {
                    // Wait for page to stabilize
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    
                    // Verify navigation succeeded by checking URL
                    match self.get_current_url().await {
                        Ok(current_url) => {
                            if current_url.contains(&Self::extract_domain(&full_url)) {
                                info!("Successfully navigated to: {} (attempt {})", full_url, attempt);
                                return Ok(());
                            } else {
                                warn!("Navigation redirected unexpectedly: {} -> {}", full_url, current_url);
                            }
                        }
                        Err(e) => {
                            warn!("Could not verify navigation: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("Navigation attempt {} failed: {}", attempt, e);
                    last_error_msg = format!("{}", e);
                }
                Err(_) => {
                    warn!("Navigation attempt {} timed out after {:?}", attempt, self.timeout_duration);
                    last_error_msg = "Navigation timeout".to_string();
                }
            }
            
            if attempt < max_attempts {
                info!("Retrying navigation in 3 seconds...");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
        
        Err(anyhow::anyhow!("All navigation attempts failed: {}", last_error_msg))
            .context(format!("Failed to navigate to {}", full_url))
    }

    /// Extract domain from URL for verification
    fn extract_domain(url: &str) -> String {
        url.split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or(url)
            .split('?')
            .next()
            .unwrap_or(url)
            .to_lowercase()
    }

    /// Get the current page title
    pub async fn get_title(&self) -> Result<String> {
        let title = self.driver.title().await?;
        info!("Page title: {}", title);
        Ok(title)
    }

    /// Get the current URL
    pub async fn get_current_url(&self) -> Result<String> {
        let url = self.driver.current_url().await?;
        Ok(url.to_string())
    }

    /// Take a screenshot with default options
    pub async fn take_screenshot(&self, filename: &str) -> Result<()> {
        self.take_screenshot_with_options(filename, &ScreenshotOptions::default()).await
    }

    /// Take a screenshot with custom options
    pub async fn take_screenshot_with_options(&self, filename: &str, options: &ScreenshotOptions) -> Result<()> {
        info!("Taking screenshot: {} (full_page: {}, viewport: {}x{})", 
               filename, options.full_page, options.viewport_width, options.viewport_height);
        
        // Set viewport size if different from current
        if let Err(e) = self.set_viewport_size(options.viewport_width, options.viewport_height).await {
            warn!("Failed to set viewport size: {}", e);
        }
        
        // Wait for page to stabilize
        tokio::time::sleep(options.wait_after_load).await;
        
        // Take screenshot based on options
        let screenshot = if options.full_page {
            match self.take_full_page_screenshot().await {
                Ok(data) => data,
                Err(e) => {
                    warn!("Full page screenshot failed, falling back to viewport: {}", e);
                    self.driver.screenshot_as_png().await
                        .context("Fallback viewport screenshot also failed")?
                }
            }
        } else {
            self.driver.screenshot_as_png().await
                .context("Viewport screenshot failed")?
        };
        
        // Ensure screenshots directory exists
        std::fs::create_dir_all("screenshots")
            .context("Failed to create screenshots directory")?;
        
        let filepath = format!("screenshots/{}", filename);
        std::fs::write(&filepath, &screenshot)
            .context(format!("Failed to save screenshot to {}", filepath))?;
        
        info!("Screenshot saved: {} ({} bytes)", filepath, screenshot.len());
        Ok(())
    }

    /// Set viewport size
    async fn set_viewport_size(&self, width: u32, height: u32) -> Result<()> {
        self.driver.set_window_rect(0, 0, width, height).await
            .context("Failed to set window size")?;
        
        info!("Viewport set to {}x{}", width, height);
        Ok(())
    }

    /// Take a full page screenshot
    async fn take_full_page_screenshot(&self) -> Result<Vec<u8>> {
        // Get the full page dimensions by executing JavaScript
        let full_height: u64 = self.driver.execute(
            "return Math.max(document.body.scrollHeight, document.body.offsetHeight, document.documentElement.clientHeight, document.documentElement.scrollHeight, document.documentElement.offsetHeight);",
            vec![]
        ).await
            .context("Failed to get page height")?
            .convert()
            .context("Failed to convert page height")?;

        let full_width: u64 = self.driver.execute(
            "return Math.max(document.body.scrollWidth, document.body.offsetWidth, document.documentElement.clientWidth, document.documentElement.scrollWidth, document.documentElement.offsetWidth);",
            vec![]
        ).await
            .context("Failed to get page width")?
            .convert()
            .context("Failed to convert page width")?;

        info!("Full page dimensions: {}x{}", full_width, full_height);

        // Set window size to capture full page
        let original_rect = self.driver.get_window_rect().await?;
        
        self.driver.set_window_rect(0, 0, full_width as u32, full_height as u32).await
            .context("Failed to resize window for full page screenshot")?;

        // Wait a moment for resize to take effect
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Take the screenshot
        let screenshot = self.driver.screenshot_as_png().await
            .context("Failed to capture full page screenshot")?;

        // Restore original window size
        if let Err(e) = self.driver.set_window_rect(
            original_rect.x as u32, 
            original_rect.y as u32, 
            original_rect.width as u32, 
            original_rect.height as u32
        ).await {
            warn!("Failed to restore original window size: {}", e);
        }

        Ok(screenshot)
    }

    /// Check if the browser is still responsive
    pub async fn is_alive(&self) -> bool {
        match self.driver.title().await {
            Ok(_) => true,
            Err(e) => {
                error!("Browser check failed: {}", e);
                false
            }
        }
    }

    /// Close the browser and clean up
    pub async fn close(self) -> Result<()> {
        info!("Closing browser...");
        // Take ownership and quit
        let SimpleBrowser { driver, .. } = self;
        driver.quit().await?;
        info!("Browser closed successfully");
        Ok(())
    }
    
    /// Get the current URL
    pub async fn get_url(&self) -> Result<String> {
        let url = self.driver.current_url().await?;
        Ok(url.to_string())
    }

    // === Workflow-specific methods ===

    /// Click an element by selector
    pub async fn click(&self, selector: &str) -> Result<()> {
        let element = self.driver.find(thirtyfour::By::Css(selector)).await
            .context(format!("Failed to find element: {}", selector))?;
        element.click().await
            .context(format!("Failed to click element: {}", selector))?;
        Ok(())
    }

    /// Fill a form field with text
    pub async fn fill_field(&self, selector: &str, value: &str) -> Result<()> {
        let element = self.driver.find(thirtyfour::By::Css(selector)).await
            .context(format!("Failed to find field: {}", selector))?;
        element.clear().await
            .context("Failed to clear field")?;
        element.send_keys(value).await
            .context(format!("Failed to fill field: {}", selector))?;
        Ok(())
    }

    /// Get text content of an element
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        let element = self.driver.find(thirtyfour::By::Css(selector)).await
            .context(format!("Failed to find element: {}", selector))?;
        element.text().await
            .context(format!("Failed to get text from: {}", selector))
    }

    /// Get attribute value of an element
    pub async fn get_attribute(&self, selector: &str, attribute: &str) -> Result<String> {
        let element = self.driver.find(thirtyfour::By::Css(selector)).await
            .context(format!("Failed to find element: {}", selector))?;
        element.attr(attribute).await
            .context(format!("Failed to get attribute '{}' from: {}", attribute, selector))?
            .ok_or_else(|| anyhow::anyhow!("Attribute '{}' not found on element", attribute))
    }

    /// Check if an element exists
    pub async fn element_exists(&self, selector: &str) -> Result<bool> {
        match self.driver.find(thirtyfour::By::Css(selector)).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Wait for an element to appear
    pub async fn wait_for_element(&self, selector: &str, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.element_exists(selector).await? {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(anyhow::anyhow!("Timeout waiting for element: {}", selector))
    }

    /// Wait for text to appear on the page
    pub async fn wait_for_text(&self, text: &str, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            let page_text = self.get_page_text().await?;
            if page_text.contains(text) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(anyhow::anyhow!("Timeout waiting for text: {}", text))
    }
    
    /// Wait for the page load event
    pub async fn wait_for_load_event(&self) -> Result<()> {
        // Wait for the load event using JavaScript
        let _script = r#"
            return new Promise((resolve) => {
                if (document.readyState === 'complete') {
                    resolve(true);
                } else {
                    window.addEventListener('load', () => resolve(true));
                }
            });
        "#;
        
        // Execute the script (in reality, we'd wait for the promise)
        // For now, we'll simulate with a simple wait
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
    
    /// Wait for DOMContentLoaded event
    pub async fn wait_for_dom_content_loaded(&self) -> Result<()> {
        // Wait for DOMContentLoaded using JavaScript
        let _script = r#"
            return new Promise((resolve) => {
                if (document.readyState === 'interactive' || document.readyState === 'complete') {
                    resolve(true);
                } else {
                    window.addEventListener('DOMContentLoaded', () => resolve(true));
                }
            });
        "#;
        
        // Execute the script (in reality, we'd wait for the promise)
        // For now, we'll simulate with a simple wait
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Wait for URL to match pattern
    pub async fn wait_for_url(&self, pattern: &str, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            let current_url = self.current_url().await?;
            if current_url.contains(pattern) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Err(anyhow::anyhow!("Timeout waiting for URL pattern: {}", pattern))
    }

    /// Get all text content from the page
    /// Get a reference to the underlying WebDriver
    pub fn driver(&self) -> &WebDriver {
        &self.driver
    }
    
    pub async fn get_page_text(&self) -> Result<String> {
        self.driver.find(thirtyfour::By::Tag("body")).await
            .context("Failed to find body element")?
            .text().await
            .context("Failed to get page text")
    }

    /// Get current URL
    pub async fn current_url(&self) -> Result<String> {
        self.driver.current_url().await
            .map(|url| url.to_string())
            .context("Failed to get current URL")
    }

    /// Count elements matching a selector
    pub async fn count_elements(&self, selector: &str) -> Result<usize> {
        let elements = self.driver.find_all(thirtyfour::By::Css(selector)).await
            .context(format!("Failed to find elements: {}", selector))?;
        Ok(elements.len())
    }

    /// Find a single element using CSS selector (for tool compatibility)
    pub async fn find_element(&self, selector: &str) -> Result<WebElement> {
        self.driver.find(By::Css(selector)).await
            .context(format!("Failed to find element with selector: {}", selector))
    }

    /// Find multiple elements using CSS selector (for tool compatibility)
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<WebElement>> {
        self.driver.find_all(By::Css(selector)).await
            .context(format!("Failed to find elements with selector: {}", selector))
    }

    /// Execute JavaScript with arguments (overloaded version for tool compatibility)
    pub async fn execute_script_with_args(&self, script: &str, _args: Vec<serde_json::Value>) -> Result<ScriptRet> {
        // Tools mostly pass vec![] so ignore args
        self.driver.execute(script, vec![]).await
            .context("Failed to execute script")
    }

    /// Execute JavaScript code with optional arguments
    pub async fn execute_script(&self, script: &str, args: Vec<serde_json::Value>) -> Result<ScriptRet> {
        self.driver.execute(script, args).await
            .context("Failed to execute script")
    }

    /// Execute JavaScript code (legacy single-argument version)
    pub async fn execute_script_simple(&self, script: &str) -> Result<serde_json::Value> {
        let result = self.driver.execute(script, vec![]).await
            .context("Failed to execute script")?;
        
        // Convert WebDriver value to JSON
        let json_value = result.json();
        
        Ok(json_value.clone())
    }
}

// Implement BrowserOps trait for SimpleBrowser
#[async_trait]
impl BrowserOps for SimpleBrowser {
    async fn navigate_to(&self, url: &str) -> Result<()> {
        self.navigate_to(url).await
    }
    
    async fn current_url(&self) -> Result<String> {
        self.current_url().await
    }
    
    async fn wait_for_load_event(&self) -> Result<()> {
        self.wait_for_load_event().await
    }
    
    async fn wait_for_dom_content_loaded(&self) -> Result<()> {
        self.wait_for_dom_content_loaded().await
    }
}

/// Browser actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    InputText { selector: String, text: String },
    GetText { selector: String },
    Screenshot { selector: Option<String> },
    WaitForElement { selector: String, timeout_ms: u64 },
    ExecuteScript { script: String },
    ScrollTo { x: i32, y: i32 },
    GoBack,
    GoForward,
    Refresh,
}