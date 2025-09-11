use anyhow::{anyhow, Result, Context};
use chromiumoxide::{Browser as ChromeBrowser, BrowserConfig, Page, Element};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use std::time::Duration;
use tracing::{info, error, warn};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Browser operations trait for abstraction
#[async_trait]
pub trait BrowserOps: Send + Sync {
    async fn navigate_to(&self, url: &str) -> Result<()>;
    async fn current_url(&self) -> Result<String>;
    async fn wait_for_load(&self) -> Result<()>;
    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>>;
    async fn execute_script(&self, script: &str) -> Result<serde_json::Value>;
    async fn find_element(&self, selector: &str) -> Result<ElementInfo>;
    async fn find_elements(&self, selector: &str) -> Result<Vec<ElementInfo>>;
    async fn click(&self, selector: &str) -> Result<()>;
    async fn type_text(&self, selector: &str, text: &str) -> Result<()>;
    async fn get_text(&self, selector: &str) -> Result<String>;
    async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()>;
    async fn scroll_to(&self, x: i32, y: i32) -> Result<()>;
    async fn close(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub full_page: bool,
    pub format: String, // "png" or "jpeg"
    pub quality: Option<u8>, // For JPEG
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub wait_after_load: Duration,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            full_page: true,
            format: "png".to_string(),
            quality: None,
            viewport_width: 1920,
            viewport_height: 1080,
            wait_after_load: Duration::from_secs(2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub tag_name: String,
    pub text: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub rect: Option<ElementRect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Main browser struct using chromiumoxide
pub struct Browser {
    pub(crate) browser: Arc<ChromeBrowser>,
    pub(crate) page: Arc<RwLock<Page>>,
}

impl Browser {
    /// Create a new browser instance
    pub async fn new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-renderer-backgrounding")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-blink-features=AutomationControlled")
            .build()
            .unwrap();
        Self::new_with_config(config).await
    }

    /// Create a browser with custom configuration
    pub async fn new_with_config(config: BrowserConfig) -> Result<Self> {
        info!("Launching Chrome browser with chromiumoxide...");
        
        // Note: BrowserConfig builder pattern should be used to add arguments
        // Arguments should be added when creating the config, not here
        
        let (browser, mut handler) = ChromeBrowser::launch(config).await
            .context("Failed to launch Chrome browser")?;
        
        // Spawn handler in background with proper error handling
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    // Only log non-critical errors, don't break on minor issues
                    match e.to_string().as_str() {
                        s if s.contains("ResetWithoutClosingHandshake") => {
                            warn!("WebSocket connection reset (non-fatal): {}", e);
                        },
                        s if s.contains("Connection reset") => {
                            warn!("Connection reset (non-fatal): {}", e);
                        },
                        _ => {
                            error!("Browser handler error: {:?}", e);
                            // Only break on truly critical errors
                            if e.to_string().contains("Browser closed") || 
                               e.to_string().contains("Process exited") {
                                break;
                            }
                        }
                    }
                } else {
                    // Handle successful events if needed
                }
            }
            warn!("Browser handler task terminated");
        });
        
        // Create a new page
        let page = browser.new_page("about:blank").await
            .context("Failed to create new page")?;
        
        info!("Browser initialized successfully");
        
        Ok(Self {
            browser: Arc::new(browser),
            page: Arc::new(RwLock::new(page)),
        })
    }

    /// Create browser in headless mode
    pub async fn new_headless() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--headless")
            .arg("--disable-gpu")
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .arg("--disable-dev-shm-usage")
            .build()
            .unwrap();
        Self::new_with_config(config).await
    }

    /// Create browser in new headless mode (Chrome 109+)
    /// The new headless mode provides better compatibility and performance
    pub async fn new_headless_new() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--headless=new")  // Use the new headless mode API
            .arg("--disable-gpu")
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-renderer-backgrounding")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-extensions")
            .arg("--disable-component-extensions-with-background-pages")
            .arg("--remote-debugging-port=0") // Use dynamic port
            .build()
            .unwrap();
        Self::new_with_config(config).await
    }

    /// Create browser in old headless mode (Chrome 108 and earlier)
    /// Use this for compatibility with older Chrome versions
    pub async fn new_headless_old() -> Result<Self> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--headless=old")  // Explicitly use old headless mode
            .arg("--disable-gpu")
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .arg("--disable-dev-shm-usage")
            .build()
            .unwrap();
        Self::new_with_config(config).await
    }

    /// Create browser with visible window (headed mode)
    pub async fn new_headed() -> Result<Self> {
        let config = BrowserConfig::builder()
            .with_head()
            .no_sandbox()
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .build()
            .unwrap();
        Self::new_with_config(config).await
    }

    /// Get the underlying browser instance
    pub fn browser(&self) -> Arc<ChromeBrowser> {
        self.browser.clone()
    }

    /// Get the current page
    pub async fn page(&self) -> Page {
        self.page.read().await.clone()
    }

    /// Create a new page/tab
    pub async fn new_page(&self) -> Result<Page> {
        let page = self.browser.new_page("about:blank").await?;
        Ok(page)
    }

    /// Helper to find element with retry
    async fn find_element_with_retry(&self, selector: &str, max_retries: u32) -> Result<Element> {
        let page = self.page.read().await;
        let mut retries = 0;
        
        loop {
            match page.find_element(selector).await {
                Ok(element) => return Ok(element),
                Err(_e) if retries < max_retries => {
                    retries += 1;
                    warn!("Element not found (attempt {}/{}): {}", retries, max_retries, selector);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => return Err(anyhow!("Element not found after {} retries: {}", max_retries, e)),
            }
        }
    }
    
    // Public methods that delegate to BrowserOps trait
    pub async fn navigate_to(&self, url: &str) -> Result<()> {
        <Self as BrowserOps>::navigate_to(self, url).await
    }
    
    pub async fn current_url(&self) -> Result<String> {
        <Self as BrowserOps>::current_url(self).await
    }
    
    pub async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>> {
        <Self as BrowserOps>::screenshot(self, options).await
    }
    
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        <Self as BrowserOps>::execute_script(self, script).await
    }
    
    pub async fn find_element(&self, selector: &str) -> Result<ElementInfo> {
        <Self as BrowserOps>::find_element(self, selector).await
    }
    
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        <Self as BrowserOps>::get_text(self, selector).await
    }
    
    pub async fn close(&self) -> Result<()> {
        <Self as BrowserOps>::close(self).await
    }
    
    pub async fn click(&self, selector: &str) -> Result<()> {
        <Self as BrowserOps>::click(self, selector).await
    }
    
    pub async fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        <Self as BrowserOps>::type_text(self, selector, text).await
    }
    
    pub async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()> {
        <Self as BrowserOps>::wait_for_selector(self, selector, timeout).await
    }
    
    pub async fn scroll_to(&self, x: i32, y: i32) -> Result<()> {
        <Self as BrowserOps>::scroll_to(self, x, y).await
    }
    
    /// Check if the browser connection is still alive
    pub async fn is_connected(&self) -> bool {
        // Try to get the current page and check if we can get its URL
        let page = self.page().await;
        // Try a simple operation to verify the connection is really alive
        page.url().await.is_ok()
    }

    /// Set zoom level for the page
    pub async fn set_zoom_level(&self, zoom_factor: f64) -> Result<()> {
        let page = self.page().await;
        // Use JavaScript to set CSS zoom
        let script = format!("document.body.style.zoom = '{}';", zoom_factor);
        page.evaluate(script).await?;
        Ok(())
    }

    /// Force browser to render properly (screenshot trigger)
    pub async fn trigger_proper_rendering(&self) -> Result<()> {
        let _screenshot = self.screenshot(crate::browser::ScreenshotOptions::default()).await?;
        // Taking a screenshot forces the browser to properly render and reflow content
        info!("Screenshot taken to trigger proper rendering");
        Ok(())
    }

    /// Complete window and content fix (combines screenshot trigger + scaling)
    pub async fn fix_window_completely(&self) -> Result<()> {
        // First trigger proper rendering with screenshot
        self.trigger_proper_rendering().await?;
        
        // Then apply content scaling
        self.fix_content_scaling().await?;
        
        Ok(())
    }

    /// Force content to fit window properly
    pub async fn fix_content_scaling(&self) -> Result<()> {
        let page = self.page().await;
        
        // Comprehensive script to scale content to fit the actual window size
        let script = r#"
        (function() {
            const windowWidth = window.innerWidth;
            const windowHeight = window.innerHeight;
            
            // Remove any existing viewport and scaling
            const existingViewport = document.querySelector('meta[name="viewport"]');
            if (existingViewport) {
                existingViewport.remove();
            }
            
            const existingStyle = document.querySelector('#claude-scaling-fix');
            if (existingStyle) {
                existingStyle.remove();
            }
            
            // Calculate scaling to fit content in window
            // Target content width of 1200px to fit in actual window width
            const targetContentWidth = 1200;
            const scaleX = Math.min(windowWidth / targetContentWidth, 1.0);
            const scaleY = Math.min(windowHeight / 600, 1.0); // Assume minimum content height of 600
            const scale = Math.min(scaleX, scaleY);
            
            // Add responsive viewport meta tag
            const viewport = document.createElement('meta');
            viewport.name = 'viewport';
            viewport.content = `width=${targetContentWidth}, initial-scale=${scale}, minimum-scale=${scale}, maximum-scale=${scale}, user-scalable=no`;
            document.head.appendChild(viewport);
            
            // Create comprehensive scaling CSS
            const style = document.createElement('style');
            style.id = 'claude-scaling-fix';
            style.textContent = `
                /* Reset and force full window usage */
                html, body {
                    margin: 0 !important;
                    padding: 0 !important;
                    width: 100vw !important;
                    height: 100vh !important;
                    max-width: 100vw !important;
                    max-height: 100vh !important;
                    overflow-x: auto !important;
                    overflow-y: auto !important;
                    box-sizing: border-box !important;
                }
                
                /* Scale the entire body content */
                body {
                    transform: scale(${scale}) !important;
                    transform-origin: 0 0 !important;
                    width: ${100/scale}% !important;
                    min-width: ${targetContentWidth}px !important;
                }
                
                /* Ensure child elements don't break the scaling */
                body > * {
                    max-width: 100% !important;
                    box-sizing: border-box !important;
                }
                
                /* GitHub specific responsive fixes */
                .js-header-wrapper, .Header, .application-main, .container-xl {
                    max-width: none !important;
                    width: 100% !important;
                }
                
                /* Handle any absolute positioned elements */
                [style*="position: fixed"], [style*="position: absolute"] {
                    transform: scale(${1/scale}) !important;
                }
            `;
            document.head.appendChild(style);
            
            // Force layout recalculation
            document.body.offsetHeight; // Force reflow
            window.dispatchEvent(new Event('resize'));
            
            // Also apply CSS zoom as fallback
            document.body.style.zoom = scale.toString();
            
            return {
                windowWidth: windowWidth,
                windowHeight: windowHeight,
                scale: scale,
                targetContentWidth: targetContentWidth,
                documentWidth: document.documentElement.scrollWidth,
                documentHeight: document.documentElement.scrollHeight,
                bodyWidth: document.body.scrollWidth,
                bodyHeight: document.body.scrollHeight
            };
        })();
        "#;
        
        let result = page.evaluate(script).await?;
        info!("Adaptive content scaling fix applied: {:?}", result);
        Ok(())
    }
    
}

#[async_trait]
impl BrowserOps for Browser {
    async fn navigate_to(&self, url: &str) -> Result<()> {
        info!("Navigating to: {}", url);
        
        // Validate and fix URL format
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            if url.starts_with("//") {
                format!("https:{}", url)
            } else {
                format!("https://{}", url)
            }
        } else {
            url.to_string()
        };
        
        let page = self.page.read().await;
        
        // Navigate with timeout and retry logic
        let mut retries = 3;
        let mut last_error = None;
        
        while retries > 0 {
            match tokio::time::timeout(
                Duration::from_secs(30),
                page.goto(&url)
            ).await {
                Ok(Ok(nav)) => {
                    // Wait for navigation to complete
                    match tokio::time::timeout(
                        Duration::from_secs(15),
                        nav.wait_for_navigation()
                    ).await {
                        Ok(Ok(_)) => {
                            // Give the page a moment to stabilize
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            return Ok(());
                        },
                        Ok(Err(e)) => {
                            warn!("Navigation wait failed (retries left: {}): {}", retries - 1, e);
                            last_error = Some(anyhow!("Navigation wait failed: {}", e));
                        },
                        Err(_) => {
                            warn!("Navigation wait timeout (retries left: {})", retries - 1);
                            last_error = Some(anyhow!("Navigation wait timeout"));
                        }
                    }
                },
                Ok(Err(e)) => {
                    warn!("Navigation failed (retries left: {}): {}", retries - 1, e);
                    last_error = Some(anyhow!("Navigation failed: {}", e));
                },
                Err(_) => {
                    warn!("Navigation timeout (retries left: {})", retries - 1);
                    last_error = Some(anyhow!("Navigation timeout"));
                }
            }
            
            retries -= 1;
            if retries > 0 {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Navigation failed after retries")))
    }

    async fn current_url(&self) -> Result<String> {
        let page = self.page.read().await;
        let url = page.url().await?;
        Ok(url.unwrap_or_else(|| "about:blank".to_string()))
    }

    async fn wait_for_load(&self) -> Result<()> {
        let page = self.page.read().await;
        // Wait for network to be idle
        page.wait_for_navigation().await?;
        Ok(())
    }

    async fn screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>> {
        let page = self.page.read().await;
        
        // Wait if specified
        tokio::time::sleep(options.wait_after_load).await;
        
        let format = match options.format.as_str() {
            "jpeg" | "jpg" => CaptureScreenshotFormat::Jpeg,
            _ => CaptureScreenshotFormat::Png,
        };
        
        let screenshot = page.screenshot(
            ScreenshotParams::builder()
                .full_page(options.full_page)
                .format(format)
                .build()
        ).await?;
        
        Ok(screenshot)
    }

    async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        let page = self.page.read().await;
        let result = page.evaluate(script).await?;
        // Convert EvaluationResult to serde_json::Value
        let value = result.into_value()?;
        Ok(value)
    }

    async fn find_element(&self, selector: &str) -> Result<ElementInfo> {
        let page = self.page.read().await;
        
        // Use JavaScript to get element information
        let script = format!(r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return null;
                const rect = el.getBoundingClientRect();
                return {{
                    tag_name: el.tagName.toLowerCase(),
                    text: el.innerText || el.textContent || '',
                    rect: {{
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height
                    }}
                }};
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        if value.is_null() {
            return Err(anyhow!("Element not found: {}", selector));
        }
        
        let tag_name = value["tag_name"].as_str().unwrap_or("").to_string();
        let text = value["text"].as_str().unwrap_or("").to_string();
        let rect = if let Some(rect_obj) = value["rect"].as_object() {
            Some(ElementRect {
                x: rect_obj["x"].as_f64().unwrap_or(0.0),
                y: rect_obj["y"].as_f64().unwrap_or(0.0),
                width: rect_obj["width"].as_f64().unwrap_or(0.0),
                height: rect_obj["height"].as_f64().unwrap_or(0.0),
            })
        } else {
            None
        };
        
        Ok(ElementInfo {
            tag_name,
            text,
            attributes: std::collections::HashMap::new(),
            rect,
        })
    }

    async fn find_elements(&self, selector: &str) -> Result<Vec<ElementInfo>> {
        let page = self.page.read().await;
        
        // Use JavaScript to get all elements information
        let script = format!(r#"
            (function() {{
                const elements = document.querySelectorAll('{}');
                const results = [];
                elements.forEach(el => {{
                    const rect = el.getBoundingClientRect();
                    results.push({{
                        tag_name: el.tagName.toLowerCase(),
                        text: el.innerText || el.textContent || '',
                        rect: {{
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height
                        }}
                    }});
                }});
                return results;
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        let mut element_infos = Vec::new();
        if let Some(array) = value.as_array() {
            for item in array {
                let tag_name = item["tag_name"].as_str().unwrap_or("").to_string();
                let text = item["text"].as_str().unwrap_or("").to_string();
                let rect = if let Some(rect_obj) = item["rect"].as_object() {
                    Some(ElementRect {
                        x: rect_obj["x"].as_f64().unwrap_or(0.0),
                        y: rect_obj["y"].as_f64().unwrap_or(0.0),
                        width: rect_obj["width"].as_f64().unwrap_or(0.0),
                        height: rect_obj["height"].as_f64().unwrap_or(0.0),
                    })
                } else {
                    None
                };
                
                element_infos.push(ElementInfo {
                    tag_name,
                    text,
                    attributes: std::collections::HashMap::new(),
                    rect,
                });
            }
        }
        
        Ok(element_infos)
    }

    async fn click(&self, selector: &str) -> Result<()> {
        info!("Clicking element: {}", selector);
        let element = self.find_element_with_retry(selector, 3).await?;
        element.click().await
            .context(format!("Failed to click element: {}", selector))?;
        Ok(())
    }

    async fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        info!("Typing text into: {}", selector);
        let element = self.find_element_with_retry(selector, 3).await?;
        element.click().await?; // Focus the element
        element.type_str(text).await
            .context(format!("Failed to type text into: {}", selector))?;
        Ok(())
    }

    async fn get_text(&self, selector: &str) -> Result<String> {
        let element = self.find_element_with_retry(selector, 3).await?;
        let text = element.inner_text().await?
            .unwrap_or_default();
        Ok(text)
    }

    async fn wait_for_selector(&self, selector: &str, timeout: Duration) -> Result<()> {
        let page = self.page.read().await;
        let start = std::time::Instant::now();
        
        loop {
            if page.find_element(selector).await.is_ok() {
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                return Err(anyhow!("Timeout waiting for selector: {}", selector));
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn scroll_to(&self, x: i32, y: i32) -> Result<()> {
        let page = self.page.read().await;
        let script = format!("window.scrollTo({}, {})", x, y);
        page.evaluate(script).await?;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        // Browser will be closed when dropped
        Ok(())
    }
}

// Additional browser methods for the new tools
impl Browser {
    /// Refresh the current page
    pub async fn refresh(&self) -> Result<()> {
        let page = self.page.read().await;
        page.reload().await?
            .wait_for_navigation().await?;
        Ok(())
    }
    
    /// Navigate back in history
    pub async fn go_back(&self) -> Result<()> {
        let page = self.page.read().await;
        let script = "window.history.back()";
        page.evaluate(script).await?;
        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for navigation
        Ok(())
    }
    
    /// Navigate forward in history
    pub async fn go_forward(&self) -> Result<()> {
        let page = self.page.read().await;
        let script = "window.history.forward()";
        page.evaluate(script).await?;
        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for navigation
        Ok(())
    }
    
    /// Hover over an element
    pub async fn hover(&self, selector: &str) -> Result<()> {
        let page = self.page.read().await;
        // Use JavaScript to simulate hover
        let script = format!(r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                const event = new MouseEvent('mouseover', {{
                    view: window,
                    bubbles: true,
                    cancelable: true
                }});
                el.dispatchEvent(event);
                return true;
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let success: bool = result.into_value()?;
        if !success {
            return Err(anyhow!("Failed to hover: element not found"));
        }
        Ok(())
    }
    
    /// Focus on an element
    pub async fn focus(&self, selector: &str) -> Result<()> {
        let element = self.find_element_with_retry(selector, 3).await?;
        element.focus().await?;
        Ok(())
    }
    
    /// Select an option from a dropdown
    pub async fn select_option(&self, selector: &str, value: &str) -> Result<()> {
        let page = self.page.read().await;
        let script = format!(r#"
            (function() {{
                const select = document.querySelector('{}');
                if (!select) return false;
                select.value = '{}';
                select.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }})()
        "#, selector, value);
        
        let result = page.evaluate(script.as_str()).await?;
        let success: bool = result.into_value()?;
        if !success {
            return Err(anyhow!("Failed to select option: element not found"));
        }
        Ok(())
    }
    
    /// Extract all links from the page
    pub async fn extract_links(&self, selector: &str) -> Result<Vec<String>> {
        let page = self.page.read().await;
        let script = format!(r#"
            (function() {{
                const links = document.querySelectorAll('{}');
                const hrefs = [];
                links.forEach(link => {{
                    if (link.href) hrefs.push(link.href);
                }});
                return hrefs;
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        let links = value.as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();
        
        Ok(links)
    }
    
    /// Extract attributes from elements
    pub async fn extract_attributes(&self, selector: &str, attributes: &[String]) -> Result<Vec<serde_json::Value>> {
        let page = self.page.read().await;
        let attrs_json = serde_json::to_string(attributes)?;
        let script = format!(r#"
            (function() {{
                const elements = document.querySelectorAll('{}');
                const attributes = {};
                const results = [];
                elements.forEach(el => {{
                    const item = {{}};
                    attributes.forEach(attr => {{
                        item[attr] = el.getAttribute(attr);
                    }});
                    results.push(item);
                }});
                return results;
            }})()
        "#, selector, attrs_json);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        let data = value.as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        
        Ok(data)
    }
    
    /// Extract table data
    pub async fn extract_table(&self, selector: &str) -> Result<Vec<Vec<String>>> {
        let page = self.page.read().await;
        let script = format!(r#"
            (function() {{
                const table = document.querySelector('{}');
                if (!table) return [];
                const rows = table.querySelectorAll('tr');
                const data = [];
                rows.forEach(row => {{
                    const cells = row.querySelectorAll('td, th');
                    const rowData = [];
                    cells.forEach(cell => {{
                        rowData.push(cell.innerText || cell.textContent || '');
                    }});
                    if (rowData.length > 0) data.push(rowData);
                }});
                return data;
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        let table = value.as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        row.as_array().map(|cells| {
                            cells.iter()
                                .filter_map(|cell| cell.as_str().map(String::from))
                                .collect()
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(table)
    }
    
    /// Extract form data
    pub async fn extract_form(&self, selector: &str) -> Result<serde_json::Value> {
        let page = self.page.read().await;
        let script = format!(r#"
            (function() {{
                const form = document.querySelector('{}');
                if (!form) return null;
                const formData = {{}};
                const inputs = form.querySelectorAll('input, select, textarea');
                inputs.forEach(input => {{
                    if (input.name) {{
                        if (input.type === 'checkbox') {{
                            formData[input.name] = input.checked;
                        }} else if (input.type === 'radio') {{
                            if (input.checked) formData[input.name] = input.value;
                        }} else {{
                            formData[input.name] = input.value;
                        }}
                    }}
                }});
                return formData;
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value = result.into_value()?;
        Ok(value)
    }
    
    /// Wait for a JavaScript condition to be true
    pub async fn wait_for_condition(&self, condition: &str, timeout: Duration) -> Result<()> {
        let page = self.page.read().await;
        let start = std::time::Instant::now();
        
        loop {
            let script = format!("(function() {{ return {}; }})()", condition);
            let result = page.evaluate(script.as_str()).await?;
            let value: bool = result.into_value().unwrap_or(false);
            
            if value {
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                return Err(anyhow!("Timeout waiting for condition: {}", condition));
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Get element information
    pub async fn get_element_info(&self, selector: &str) -> Result<serde_json::Value> {
        let page = self.page.read().await;
        let script = format!(r#"
            (function() {{
                const el = document.querySelector('{}');
                if (!el) return null;
                const rect = el.getBoundingClientRect();
                const style = window.getComputedStyle(el);
                return {{
                    tagName: el.tagName.toLowerCase(),
                    id: el.id,
                    className: el.className,
                    innerText: el.innerText || '',
                    innerHTML: el.innerHTML,
                    value: el.value || '',
                    href: el.href || '',
                    src: el.src || '',
                    alt: el.alt || '',
                    title: el.title || '',
                    isVisible: rect.width > 0 && rect.height > 0,
                    isEnabled: !el.disabled,
                    rect: {{
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height
                    }},
                    style: {{
                        display: style.display,
                        visibility: style.visibility,
                        opacity: style.opacity
                    }}
                }};
            }})()
        "#, selector);
        
        let result = page.evaluate(script.as_str()).await?;
        let value: serde_json::Value = result.into_value()?;
        
        if value.is_null() {
            return Err(anyhow!("Element not found: {}", selector));
        }
        
        Ok(value)
    }
}

// Export public types
pub use self::{
    BrowserOps as BrowserOperations,
};