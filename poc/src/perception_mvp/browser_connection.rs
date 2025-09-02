// Real Browser Connection Module - Interfaces with ChromeDriver
// This module provides the actual browser automation capabilities

use anyhow::{Result, Context as AnyhowContext};
use thirtyfour::{
    prelude::*,
    DesiredCapabilities, 
    WebDriver,
};
use serde_json;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Browser connection configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub chromedriver_url: String,
    pub headless: bool,
    pub window_size: (u32, u32),
    pub page_load_timeout: Duration,
    pub implicit_wait: Duration,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        let chromedriver_port = std::env::var("CHROMEDRIVER_PORT")
            .unwrap_or_else(|_| "9515".to_string());
        
        Self {
            chromedriver_url: format!("http://localhost:{}", chromedriver_port),
            headless: true,
            window_size: (1920, 1080),
            page_load_timeout: Duration::from_secs(30),
            implicit_wait: Duration::from_millis(500),
        }
    }
}

/// Manages browser connections and lifecycle
pub struct BrowserConnection {
    driver: Arc<WebDriver>,
    config: BrowserConfig,
    is_connected: Arc<RwLock<bool>>,
}

impl BrowserConnection {
    /// Create a new browser connection
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("Initializing browser connection to {}", config.chromedriver_url);
        
        // Create Chrome capabilities
        let mut caps = DesiredCapabilities::chrome();
        
        // Set Chrome options as a JSON object
        let mut chrome_options = serde_json::json!({
            "args": vec![
                "--disable-gpu",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-blink-features=AutomationControlled",
                format!("--window-size={},{}", config.window_size.0, config.window_size.1),
            ],
            "excludeSwitches": vec!["enable-automation"],
            "useAutomationExtension": false
        });
        
        if config.headless {
            chrome_options["args"].as_array_mut().unwrap().push(serde_json::json!("--headless=new"));
        }
        
        // Set the chrome options
        caps.set_no_sandbox()?;
        if config.headless {
            caps.set_headless()?;
        }
        
        // Connect to ChromeDriver
        let driver = WebDriver::new(&config.chromedriver_url, caps)
            .await
            .context("Failed to connect to ChromeDriver")?;
        
        // Set timeouts
        driver.set_page_load_timeout(config.page_load_timeout).await?;
        driver.set_implicit_wait_timeout(config.implicit_wait).await?;
        
        info!("Successfully connected to ChromeDriver");
        
        Ok(Self {
            driver: Arc::new(driver),
            config,
            is_connected: Arc::new(RwLock::new(true)),
        })
    }
    
    /// Get the WebDriver instance
    pub fn driver(&self) -> Arc<WebDriver> {
        Arc::clone(&self.driver)
    }
    
    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        debug!("Navigating to: {}", url);
        self.driver.goto(url).await?;
        
        // Wait for page to be ready
        self.wait_for_ready_state().await?;
        
        Ok(())
    }
    
    /// Wait for document ready state
    pub async fn wait_for_ready_state(&self) -> Result<()> {
        let script = r#"
            return document.readyState === 'complete' || document.readyState === 'interactive';
        "#;
        
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);
        
        loop {
            let ready = self.driver
                .execute(script, vec![])
                .await?
                .as_bool()
                .unwrap_or(false);
            
            if ready {
                debug!("Page ready in {:?}", start.elapsed());
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                warn!("Page ready state timeout");
                return Ok(()); // Continue anyway
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Check if browser is still connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
    
    /// Get current URL
    pub async fn current_url(&self) -> Result<String> {
        Ok(self.driver.current_url().await?.to_string())
    }
    
    /// Get page title
    pub async fn title(&self) -> Result<String> {
        self.driver.title().await.context("Failed to get page title")
    }
    
    /// Take a screenshot
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        self.driver.screenshot_as_png().await.context("Failed to take screenshot")
    }
    
    /// Execute JavaScript
    pub async fn execute_script(&self, script: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value> {
        self.driver
            .execute(script, args)
            .await
            .context("Failed to execute script")
    }
    
    /// Find element by CSS selector
    pub async fn find_element(&self, selector: &str) -> Result<WebElement> {
        self.driver
            .find(By::Css(selector))
            .await
            .context(format!("Element not found: {}", selector))
    }
    
    /// Find multiple elements by CSS selector
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<WebElement>> {
        Ok(self.driver.find_all(By::Css(selector)).await?)
    }
    
    /// Get page source
    pub async fn page_source(&self) -> Result<String> {
        self.driver.source().await.context("Failed to get page source")
    }
    
    /// Close the browser
    pub async fn close(&self) -> Result<()> {
        info!("Closing browser connection");
        
        let mut connected = self.is_connected.write().await;
        if *connected {
            self.driver.quit().await?;
            *connected = false;
        }
        
        Ok(())
    }
    
    /// Create a new tab
    pub async fn new_tab(&self) -> Result<WindowHandle> {
        self.driver.new_window().await.context("Failed to create new tab")
    }
    
    /// Switch to a specific tab
    pub async fn switch_to_tab(&self, handle: WindowHandle) -> Result<()> {
        self.driver.switch_to_window(handle).await?;
        Ok(())
    }
    
    /// Get all window handles
    pub async fn get_windows(&self) -> Result<Vec<WindowHandle>> {
        self.driver.windows().await.context("Failed to get window handles")
    }
}

// Drop implementation removed - WebDriver doesn't support cloning for quit()
// Browser cleanup should be handled explicitly through close() method

/// Browser pool for managing multiple connections
pub struct BrowserPool {
    connections: Arc<RwLock<Vec<Arc<BrowserConnection>>>>,
    config: BrowserConfig,
    max_connections: usize,
}

impl BrowserPool {
    pub fn new(config: BrowserConfig, max_connections: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            config,
            max_connections,
        }
    }
    
    /// Get or create a browser connection
    pub async fn get_connection(&self) -> Result<Arc<BrowserConnection>> {
        let mut connections = self.connections.write().await;
        
        // Try to find an available connection
        for conn in connections.iter() {
            if conn.is_connected().await {
                return Ok(Arc::clone(conn));
            }
        }
        
        // Create a new connection if under limit
        if connections.len() < self.max_connections {
            let new_conn = Arc::new(BrowserConnection::new(self.config.clone()).await?);
            connections.push(Arc::clone(&new_conn));
            return Ok(new_conn);
        }
        
        // Otherwise, create a temporary connection
        Ok(Arc::new(BrowserConnection::new(self.config.clone()).await?))
    }
    
    /// Close all connections
    pub async fn close_all(&self) -> Result<()> {
        let connections = self.connections.write().await;
        for conn in connections.iter() {
            conn.close().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires ChromeDriver to be running
    async fn test_browser_connection() {
        let config = BrowserConfig::default();
        let browser = BrowserConnection::new(config).await.unwrap();
        
        // Test navigation
        browser.navigate("https://example.com").await.unwrap();
        
        // Test getting title
        let title = browser.title().await.unwrap();
        assert!(title.contains("Example"));
        
        // Test finding elements
        let body = browser.find_element("body").await.unwrap();
        assert!(body.is_enabled().await.unwrap());
        
        // Close browser
        browser.close().await.unwrap();
    }
}