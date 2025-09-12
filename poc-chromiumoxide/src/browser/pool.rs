use anyhow::{Result, Context, anyhow};
use chromiumoxide::BrowserConfig;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use tracing::{info, warn, error};
use super::core::Browser;

/// Browser pool for managing multiple browser instances
pub struct BrowserPool {
    browsers: Arc<RwLock<Vec<Arc<Browser>>>>,
    semaphore: Arc<Semaphore>,
    max_browsers: usize,
    config: BrowserConfig,
}

impl BrowserPool {
    /// Create a new browser pool with default (headed) configuration
    pub fn new(max_browsers: usize) -> Result<Self> {
        Self::new_with_headless(max_browsers, false)
    }

    /// Create a new browser pool with specified headless mode
    pub fn new_with_headless(max_browsers: usize, headless: bool) -> Result<Self> {
        let config = if headless {
            BrowserConfig::builder()
                .arg("--headless")
                .arg("--disable-gpu")
                .arg("--disable-web-security")
                .arg("--disable-features=VizDisplayCompositor")
                .arg("--disable-dev-shm-usage")
                .arg("--no-sandbox")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build headless config: {}", e))?
        } else {
            // For headed mode, don't use .with_head() as it seems to cause issues
            // Just create a browser without --headless flag, which makes it visible by default
            BrowserConfig::builder()
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
                .map_err(|e| anyhow::anyhow!("Failed to build headed config: {}", e))?
        };
        
        Ok(Self {
            browsers: Arc::new(RwLock::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_browsers)),
            max_browsers,
            config,
        })
    }

    /// Create a pool with custom browser configuration
    pub fn with_config(max_browsers: usize, config: BrowserConfig) -> Self {
        Self {
            browsers: Arc::new(RwLock::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max_browsers)),
            max_browsers,
            config,
        }
    }

    /// Acquire a browser from the pool
    pub async fn acquire(&self) -> Result<BrowserGuard> {
        let permit = self.semaphore.clone().acquire_owned().await
            .context("Failed to acquire semaphore permit")?;
        
        // Try to reuse an existing browser first with multiple attempts
        for attempt in 0..5 {
            if attempt > 0 {
                // Wait longer between attempts to allow browsers to be returned to pool
                let wait_time = std::cmp::min(500 * attempt, 2000); // 500ms, 1s, 1.5s, 2s, 2s
                info!("Waiting {}ms for browser to become available (attempt {}/5)", wait_time, attempt + 1);
                tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
            }
            
            // Try to reuse an existing browser, checking if it's still connected
            {
                let mut browsers = self.browsers.write().await;
                while let Some(browser) = browsers.pop() {
                    // Check if the browser is still connected
                    if browser.is_connected().await {
                        info!("Reusing existing browser from pool (attempt {})", attempt + 1);
                        return Ok(BrowserGuard {
                            browser,
                            pool: self.browsers.clone(),
                            _permit: permit,
                        });
                    } else {
                        warn!("Discarding disconnected browser from pool");
                        // Browser is dead, discard it and try the next one
                    }
                }
            }
        }
        
        // Only create a new browser if we have less than max_browsers total
        let current_pool_size = self.size().await;
        if current_pool_size >= self.max_browsers {
            return Err(anyhow!("Maximum browser instances ({}) reached. Pool exhausted.", self.max_browsers));
        }
        
        // Create a new browser with retry logic
        warn!("No browsers available in pool after 5 attempts. Creating new browser (pool size: {}/{})", 
              current_pool_size, self.max_browsers);
        let mut retries = 3;  // Reduced retries since we already waited
        let mut last_error = None;
        
        while retries > 0 {
            // Wait between retries to avoid resource conflicts
            if retries < 3 {
                let wait_time = if retries == 1 { 5 } else { 2 };
                info!("Waiting {} seconds before retry...", wait_time);
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
            }
            
            match Browser::new_with_config(self.config.clone()).await {
                Ok(browser) => {
                    info!("Successfully created new browser for pool");
                    // Test the new browser is actually connected
                    let browser_arc = Arc::new(browser);
                    if browser_arc.is_connected().await {
                        return Ok(BrowserGuard {
                            browser: browser_arc,
                            pool: self.browsers.clone(),
                            _permit: permit,
                        });
                    } else {
                        warn!("Newly created browser is not connected, retrying...");
                        retries -= 1;
                        continue;
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Failed to create browser (retries left: {}): {}", retries - 1, error_msg);
                    last_error = Some(e);
                    retries -= 1;
                    
                    // If it's a port/resource conflict, wait longer
                    if error_msg.contains("Address already in use") || 
                       error_msg.contains("port") ||
                       error_msg.contains("Cannot connect to") ||
                       error_msg.contains("WebSocket") {
                        warn!("Resource conflict detected, waiting longer...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to create browser after {} retries", 3)))
    }

    /// Get the current pool size
    pub async fn size(&self) -> usize {
        self.browsers.read().await.len()
    }

    /// Clear all browsers from the pool
    pub async fn clear(&self) {
        let mut browsers = self.browsers.write().await;
        browsers.clear();
        info!("Browser pool cleared");
    }
    
    /// Clean up disconnected browsers from the pool
    pub async fn cleanup_disconnected(&self) -> usize {
        let mut browsers = self.browsers.write().await;
        let initial_count = browsers.len();
        
        // Keep only connected browsers
        let mut connected_browsers = Vec::new();
        while let Some(browser) = browsers.pop() {
            if browser.is_connected().await {
                connected_browsers.push(browser);
            }
        }
        
        let removed_count = initial_count - connected_browsers.len();
        if removed_count > 0 {
            info!("Cleaned up {} disconnected browsers from pool", removed_count);
        }
        
        // Put connected browsers back
        *browsers = connected_browsers;
        removed_count
    }

    /// Preload browsers into the pool
    pub async fn preload(&self, count: usize) -> Result<()> {
        let count = count.min(self.max_browsers);
        info!("Preloading {} browsers into pool", count);
        
        let mut browsers = Vec::new();
        for i in 0..count {
            match Browser::new_with_config(self.config.clone()).await {
                Ok(browser) => {
                    browsers.push(Arc::new(browser));
                    info!("Preloaded browser {}/{}", i + 1, count);
                }
                Err(e) => {
                    error!("Failed to preload browser {}: {}", i + 1, e);
                }
            }
        }
        
        let mut pool = self.browsers.write().await;
        pool.extend(browsers);
        
        Ok(())
    }
}

/// Guard for automatically returning browsers to the pool
pub struct BrowserGuard {
    browser: Arc<Browser>,
    pool: Arc<RwLock<Vec<Arc<Browser>>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl BrowserGuard {
    /// Get a reference to the browser
    pub fn browser(&self) -> &Browser {
        &self.browser
    }
    
    /// Get a cloned Arc<Browser>
    pub fn browser_arc(&self) -> Arc<Browser> {
        Arc::clone(&self.browser)
    }
}

impl Drop for BrowserGuard {
    fn drop(&mut self) {
        let browser = self.browser.clone();
        let pool = self.pool.clone();
        
        // Return browser to pool immediately with better error handling
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                // Check if browser is still connected before returning to pool
                if browser.is_connected().await {
                    // Don't navigate to about:blank - keep the current page
                    // This allows tools to maintain state between operations
                    let mut browsers = pool.write().await;
                    browsers.push(browser);
                    info!("Browser returned to pool successfully");
                } else {
                    warn!("Not returning disconnected browser to pool");
                }
            });
        } else {
            // Fallback - try to return browser using a blocking approach
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    warn!("Failed to create runtime for browser return: {}", e);
                    return;
                }
            };
            
            let _ = std::thread::spawn(move || {
                rt.block_on(async move {
                    // Check if browser is still connected before returning to pool
                    if browser.is_connected().await {
                        let mut browsers = pool.write().await;
                        browsers.push(browser);
                        info!("Browser returned to pool successfully (fallback)");
                    } else {
                        warn!("Not returning disconnected browser to pool (fallback)");
                    }
                });
            });
        }
    }
}

impl std::ops::Deref for BrowserGuard {
    type Target = Browser;
    
    fn deref(&self) -> &Self::Target {
        &self.browser
    }
}