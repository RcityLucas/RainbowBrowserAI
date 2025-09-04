use anyhow::{Result, Context, anyhow};
use chromiumoxide::BrowserConfig;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use tracing::{info, warn, error};
use super::core::{Browser, BrowserOps};

/// Browser pool for managing multiple browser instances
pub struct BrowserPool {
    browsers: Arc<RwLock<Vec<Arc<Browser>>>>,
    semaphore: Arc<Semaphore>,
    max_browsers: usize,
    config: BrowserConfig,
}

impl BrowserPool {
    /// Create a new browser pool
    pub fn new(max_browsers: usize) -> Result<Self> {
        let config = BrowserConfig::builder()
            .with_head()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build config: {}", e))?;
        
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
        
        // Try to reuse an existing browser
        {
            let mut browsers = self.browsers.write().await;
            if let Some(browser) = browsers.pop() {
                info!("Reusing existing browser from pool");
                return Ok(BrowserGuard {
                    browser,
                    pool: self.browsers.clone(),
                    _permit: permit,
                });
            }
        }
        
        // Create a new browser with retry logic
        info!("Creating new browser for pool");
        let mut retries = 3;
        let mut last_error = None;
        
        while retries > 0 {
            // Wait between retries to avoid resource conflicts
            if retries < 3 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            
            match Browser::new_with_config(self.config.clone()).await {
                Ok(browser) => {
                    info!("Successfully created new browser for pool");
                    return Ok(BrowserGuard {
                        browser: Arc::new(browser),
                        pool: self.browsers.clone(),
                        _permit: permit,
                    });
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Failed to create browser (retries left: {}): {}", retries - 1, error_msg);
                    last_error = Some(e);
                    retries -= 1;
                    
                    // If it's a port/resource conflict, wait longer
                    if error_msg.contains("Address already in use") || 
                       error_msg.contains("port") ||
                       error_msg.contains("Cannot connect to") {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to create browser after retries")))
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
}

impl Drop for BrowserGuard {
    fn drop(&mut self) {
        let browser = self.browser.clone();
        let pool = self.pool.clone();
        
        // Return browser to pool in background
        tokio::spawn(async move {
            // Don't navigate to about:blank - keep the current page
            // This allows tools to maintain state between operations
            let mut browsers = pool.write().await;
            browsers.push(browser);
        });
    }
}

impl std::ops::Deref for BrowserGuard {
    type Target = Browser;
    
    fn deref(&self) -> &Self::Target {
        &self.browser
    }
}