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
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build headless config: {}", e))?
        } else {
            BrowserConfig::builder()
                .with_head()
                .arg("--window-size=1200,800")
                .arg("--window-position=0,0")
                .arg("--force-device-scale-factor=1")
                .arg("--high-dpi-support=1")
                .arg("--force-color-profile=srgb")
                .arg("--disable-infobars")
                .arg("--disable-background-timer-throttling")
                .arg("--disable-renderer-backgrounding")
                .arg("--disable-features=TranslateUI,VizDisplayCompositor")
                .arg("--disable-ipc-flooding-protection")
                .arg("--no-first-run")
                .arg("--no-default-browser-check")
                .arg("--disable-default-apps")
                .arg("--disable-popup-blocking")
                .arg("--disable-translate")
                .arg("--disable-component-update")
                .arg("--allow-running-insecure-content")
                .arg("--disable-dev-shm-usage")
                .arg("--disable-web-security")
                .arg("--no-sandbox")
                .arg("--disable-extensions")
                .arg("--disable-plugins")
                .arg("--disable-background-networking")
                .arg("--disable-blink-features=AutomationControlled")
                .arg("--enable-use-zoom-for-dsf")
                .arg("--app-shell-host-window-size=1200x800")
                .arg("--force-app-mode")
                .arg("--disable-session-crashed-bubble")
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
        
        // Wait a bit for any returning browsers from previous operations
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Try to reuse an existing browser, checking if it's still connected
        {
            let mut browsers = self.browsers.write().await;
            while let Some(browser) = browsers.pop() {
                // Check if the browser is still connected
                if browser.is_connected().await {
                    info!("Reusing existing browser from pool");
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
        
        // Create a new browser with retry logic
        info!("Creating new browser for pool");
        let mut retries = 5;  // Increased retries for better reliability
        let mut last_error = None;
        
        while retries > 0 {
            // Wait between retries to avoid resource conflicts
            if retries < 5 {
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
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to create browser after {} retries", 5)))
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
        
        // Return browser to pool immediately 
        // We need to ensure the browser is returned for reuse
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            rt.spawn(async move {
                // Don't navigate to about:blank - keep the current page
                // This allows tools to maintain state between operations
                let mut browsers = pool.write().await;
                browsers.push(browser);
            });
        } else {
            // Fallback - try to return browser using a new runtime
            let _ = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    let mut browsers = pool.write().await;
                    browsers.push(browser);
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