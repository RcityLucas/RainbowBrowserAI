use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tracing::{info, warn, debug};
use crate::SimpleBrowser;

/// A pooled browser instance with metadata
struct PooledBrowser {
    browser: Option<SimpleBrowser>,
    created_at: Instant,
    last_used: Instant,
    usage_count: usize,
}

/// Browser connection pool for improved performance
pub struct BrowserPool {
    /// Maximum number of browsers in the pool
    max_size: usize,
    /// Maximum idle time before browser is closed
    idle_timeout: Duration,
    /// Maximum lifetime of a browser instance
    max_lifetime: Duration,
    /// Maximum uses before browser is recycled
    max_usage: usize,
    /// Pool of available browsers
    browsers: Arc<Mutex<VecDeque<PooledBrowser>>>,
    /// Semaphore to limit concurrent browser creation
    create_semaphore: Arc<Semaphore>,
    /// Statistics
    stats: Arc<Mutex<PoolStats>>,
}

#[derive(Debug, Default)]
pub struct PoolStats {
    pub total_created: usize,
    pub total_destroyed: usize,
    pub total_checkouts: usize,
    pub total_checkins: usize,
    pub current_size: usize,
    pub current_idle: usize,
}

impl BrowserPool {
    /// Create a new browser pool with default settings
    pub fn new() -> Self {
        Self::with_config(3, Duration::from_secs(300), Duration::from_secs(3600), 100)
    }

    /// Create a new browser pool with custom configuration
    pub fn with_config(
        max_size: usize,
        idle_timeout: Duration,
        max_lifetime: Duration,
        max_usage: usize,
    ) -> Self {
        info!(
            "Initializing browser pool (max_size: {}, idle_timeout: {:?})",
            max_size, idle_timeout
        );

        Self {
            max_size,
            idle_timeout,
            max_lifetime,
            max_usage,
            browsers: Arc::new(Mutex::new(VecDeque::new())),
            create_semaphore: Arc::new(Semaphore::new(max_size)),
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Acquire a browser from the pool
    pub async fn acquire(&self) -> Result<PooledBrowserHandle> {
        debug!("Acquiring browser from pool");
        
        // Try to get an existing browser
        let mut pool = self.browsers.lock().await;
        
        // Clean up expired browsers
        self.cleanup_expired(&mut pool).await;
        
        // Try to reuse an idle browser
        while let Some(mut pooled) = pool.pop_front() {
            let age = pooled.created_at.elapsed();
            let idle = pooled.last_used.elapsed();
            
            if age > self.max_lifetime || pooled.usage_count >= self.max_usage {
                // Browser is too old or overused, destroy it
                debug!("Browser expired (age: {:?}, usage: {})", age, pooled.usage_count);
                if let Some(browser) = pooled.browser.take() {
                    let _ = browser.close().await;
                }
                let mut stats = self.stats.lock().await;
                stats.total_destroyed += 1;
                stats.current_size -= 1;
                continue;
            }
            
            if idle > self.idle_timeout {
                // Browser has been idle too long, check if it's still alive
                if let Some(ref browser) = pooled.browser {
                    if !browser.is_alive().await {
                        debug!("Browser no longer responsive after idle timeout");
                        if let Some(browser) = pooled.browser.take() {
                            let _ = browser.close().await;
                        }
                        let mut stats = self.stats.lock().await;
                        stats.total_destroyed += 1;
                        stats.current_size -= 1;
                        continue;
                    }
                }
            }
            
            // Browser is good to reuse
            pooled.last_used = Instant::now();
            pooled.usage_count += 1;
            
            let mut stats = self.stats.lock().await;
            stats.total_checkouts += 1;
            stats.current_idle -= 1;
            
            info!("Reusing browser from pool (usage: {})", pooled.usage_count);
            
            return Ok(PooledBrowserHandle {
                pooled: Some(pooled),
                pool: Arc::clone(&self.browsers),
                stats: Arc::clone(&self.stats),
            });
        }
        
        drop(pool); // Release lock before creating new browser
        
        // Need to create a new browser
        let permit = self.create_semaphore.acquire().await?;
        
        info!("Creating new browser for pool");
        let browser = SimpleBrowser::new().await?;
        
        let pooled = PooledBrowser {
            browser: Some(browser),
            created_at: Instant::now(),
            last_used: Instant::now(),
            usage_count: 1,
        };
        
        let mut stats = self.stats.lock().await;
        stats.total_created += 1;
        stats.total_checkouts += 1;
        stats.current_size += 1;
        
        drop(permit); // Release semaphore
        
        Ok(PooledBrowserHandle {
            pooled: Some(pooled),
            pool: Arc::clone(&self.browsers),
            stats: Arc::clone(&self.stats),
        })
    }

    /// Clean up expired browsers from the pool
    async fn cleanup_expired(&self, pool: &mut VecDeque<PooledBrowser>) {
        let mut to_remove = Vec::new();
        
        for (i, pooled) in pool.iter().enumerate() {
            let age = pooled.created_at.elapsed();
            let idle = pooled.last_used.elapsed();
            
            if age > self.max_lifetime || idle > self.idle_timeout || pooled.usage_count >= self.max_usage {
                to_remove.push(i);
            }
        }
        
        // Remove in reverse order to maintain indices
        for i in to_remove.into_iter().rev() {
            if let Some(mut pooled) = pool.remove(i) {
                debug!("Removing expired browser from pool");
                if let Some(browser) = pooled.browser.take() {
                    let _ = browser.close().await;
                }
                let mut stats = self.stats.lock().await;
                stats.total_destroyed += 1;
                stats.current_size -= 1;
                stats.current_idle -= 1;
            }
        }
    }

    /// Get current pool statistics
    pub async fn stats(&self) -> PoolStats {
        let stats = self.stats.lock().await;
        let pool = self.browsers.lock().await;
        
        PoolStats {
            total_created: stats.total_created,
            total_destroyed: stats.total_destroyed,
            total_checkouts: stats.total_checkouts,
            total_checkins: stats.total_checkins,
            current_size: stats.current_size,
            current_idle: pool.len(),
        }
    }

    /// Clear all browsers from the pool
    pub async fn clear(&self) -> Result<()> {
        info!("Clearing browser pool");
        
        let mut pool = self.browsers.lock().await;
        while let Some(mut pooled) = pool.pop_front() {
            if let Some(browser) = pooled.browser.take() {
                browser.close().await?;
            }
        }
        
        let mut stats = self.stats.lock().await;
        stats.total_destroyed += stats.current_size;
        stats.current_size = 0;
        stats.current_idle = 0;
        
        Ok(())
    }
}

/// Handle to a pooled browser that returns to pool when dropped
pub struct PooledBrowserHandle {
    pooled: Option<PooledBrowser>,
    pool: Arc<Mutex<VecDeque<PooledBrowser>>>,
    stats: Arc<Mutex<PoolStats>>,
}

impl PooledBrowserHandle {
    /// Get a reference to the browser
    pub fn browser(&self) -> Option<&SimpleBrowser> {
        self.pooled.as_ref()?.browser.as_ref()
    }

    /// Get a mutable reference to the browser
    pub fn browser_mut(&mut self) -> Option<&mut SimpleBrowser> {
        self.pooled.as_mut()?.browser.as_mut()
    }

    /// Manually release the browser back to the pool
    pub async fn release(mut self) {
        if let Some(mut pooled) = self.pooled.take() {
            pooled.last_used = Instant::now();
            
            let mut pool = self.pool.lock().await;
            pool.push_back(pooled);
            
            let mut stats = self.stats.lock().await;
            stats.total_checkins += 1;
            stats.current_idle += 1;
            
            debug!("Browser returned to pool");
        }
    }

    /// Mark the browser as failed and don't return it to the pool
    pub async fn mark_failed(mut self) {
        if let Some(mut pooled) = self.pooled.take() {
            warn!("Browser marked as failed, destroying");
            
            if let Some(browser) = pooled.browser.take() {
                let _ = browser.close().await;
            }
            
            let mut stats = self.stats.lock().await;
            stats.total_destroyed += 1;
            stats.current_size -= 1;
        }
    }
}

impl Drop for PooledBrowserHandle {
    fn drop(&mut self) {
        if let Some(mut pooled) = self.pooled.take() {
            // Return to pool in a blocking manner
            // This is not ideal but necessary for Drop
            pooled.last_used = Instant::now();
            
            if let Ok(mut pool) = self.pool.try_lock() {
                pool.push_back(pooled);
                
                if let Ok(mut stats) = self.stats.try_lock() {
                    stats.total_checkins += 1;
                    stats.current_idle += 1;
                }
                
                debug!("Browser automatically returned to pool on drop");
            } else {
                warn!("Failed to return browser to pool on drop");
                // Browser will be lost but that's better than panicking
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_basic_operations() {
        let pool = BrowserPool::with_config(
            2,
            Duration::from_secs(60),
            Duration::from_secs(300),
            10
        );

        // Acquire and release
        {
            let handle = pool.acquire().await.unwrap();
            assert!(handle.browser().is_some());
        } // Should return to pool on drop

        let stats = pool.stats().await;
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.total_checkouts, 1);
        assert_eq!(stats.total_checkins, 1);
        assert_eq!(stats.current_idle, 1);
    }

    #[tokio::test]
    async fn test_pool_reuse() {
        let pool = BrowserPool::with_config(
            2,
            Duration::from_secs(60),
            Duration::from_secs(300),
            10
        );

        // First acquisition
        {
            let _handle = pool.acquire().await.unwrap();
        }

        // Second acquisition should reuse
        {
            let _handle = pool.acquire().await.unwrap();
        }

        let stats = pool.stats().await;
        assert_eq!(stats.total_created, 1); // Only one browser created
        assert_eq!(stats.total_checkouts, 2); // But checked out twice
    }
}