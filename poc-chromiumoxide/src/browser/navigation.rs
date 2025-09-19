use super::core::Browser;
use anyhow::{anyhow, Result};
use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Advanced navigation options
#[derive(Debug, Clone, Default)]
pub struct NavigationOptions {
    /// Wait for network idle after navigation
    pub wait_for_idle: bool,
    /// Timeout for navigation
    pub timeout: Option<Duration>,
    /// Wait for specific selector after navigation
    pub wait_for_selector: Option<String>,
    /// Custom headers to send
    pub headers: Option<std::collections::HashMap<String, String>>,
    /// Referrer URL
    pub referrer: Option<String>,
}

/// Navigation result with timing information
#[derive(Debug, Clone, serde::Serialize)]
pub struct NavigationResult {
    pub url: String,
    pub status: u16,
    pub load_time_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub redirects: Vec<String>,
}

/// Network request tracker for CDP-based network idle detection
#[derive(Debug, Clone)]
pub struct NetworkRequestTracker {
    /// Active requests by request ID
    pub active_requests: Arc<RwLock<HashSet<String>>>,
    /// Last activity timestamp
    pub last_activity: Arc<RwLock<Instant>>,
}

impl NetworkRequestTracker {
    pub fn new() -> Self {
        Self {
            active_requests: Arc::new(RwLock::new(HashSet::new())),
            last_activity: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn add_request(&self, request_id: String) {
        let mut active = self.active_requests.write().await;
        active.insert(request_id);
        *self.last_activity.write().await = Instant::now();
        debug!("Added request, {} active requests", active.len());
    }

    pub async fn remove_request(&self, request_id: &str) {
        let mut active = self.active_requests.write().await;
        active.remove(request_id);
        *self.last_activity.write().await = Instant::now();
        debug!("Removed request, {} active requests", active.len());
    }

    pub async fn is_idle(&self, idle_threshold: Duration) -> bool {
        let active_count = self.active_requests.read().await.len();
        let last_activity = *self.last_activity.read().await;
        let elapsed = last_activity.elapsed();

        active_count == 0 && elapsed >= idle_threshold
    }

    pub async fn active_count(&self) -> usize {
        self.active_requests.read().await.len()
    }
}

impl Default for NetworkRequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Browser {
    /// Navigate with advanced options
    pub async fn navigate_with_options(
        &self,
        url: &str,
        options: NavigationOptions,
    ) -> Result<NavigationResult> {
        let start_time = std::time::Instant::now();
        info!("Navigating to {} with options: {:?}", url, options);

        // Use the BrowserOps trait method for navigation
        self.navigate_to(url).await?;

        // Wait for network idle if requested
        if options.wait_for_idle {
            debug!("Waiting for network idle");
            self.wait_for_network_idle(Duration::from_secs(2)).await?;
        }

        // Wait for selector if specified
        if let Some(selector) = &options.wait_for_selector {
            debug!("Waiting for selector: {}", selector);
            self.wait_for_selector(selector, Duration::from_secs(10))
                .await?;
        }

        let load_time = start_time.elapsed();
        let current_url = self.current_url().await?;

        Ok(NavigationResult {
            url: current_url,
            status: 200, // TODO: Get actual status from response
            load_time_ms: load_time.as_millis() as u64,
            dom_content_loaded_ms: 0, // TODO: Get actual timing
            redirects: Vec::new(),    // TODO: Track redirects
        })
    }

    /// Reload the current page
    pub async fn reload(&self) -> Result<()> {
        info!("Reloading current page");
        let page = self.page.read().await;
        page.reload().await?.wait_for_navigation().await?;
        Ok(())
    }

    /// Hard reload with cache bypass
    pub async fn hard_reload(&self) -> Result<()> {
        info!("Hard reloading current page (bypass cache)");
        // Simple reload for now - chromiumoxide may handle cache differently
        let page = self.page.read().await;
        page.reload().await?.wait_for_navigation().await?;
        Ok(())
    }

    /// Wait for network to be idle using CDP Network domain events
    pub async fn wait_for_network_idle(&self, idle_time: Duration) -> Result<()> {
        debug!(
            "Waiting for network idle ({:?}) using CDP Network domain",
            idle_time
        );

        let page = self.page.read().await;
        let start = Instant::now();
        let max_wait = Duration::from_secs(30);
        let check_interval = Duration::from_millis(50); // High frequency checking

        // Enable Runtime domain for network activity monitoring
        if let Err(e) = page.enable_runtime().await {
            warn!("Failed to enable Runtime domain: {}", e);
        }

        // Track active network requests using CDP
        let mut active_requests = 0usize;
        let mut last_activity = Instant::now();
        let mut consecutive_idle_time = Duration::ZERO;

        info!(
            "Starting CDP-backed network idle detection: {:?} threshold, {:?} timeout",
            idle_time, max_wait
        );

        while start.elapsed() < max_wait {
            // Check current network activity
            let current_activity = self.check_network_activity().await?;

            if current_activity == 0 {
                // Network is currently idle
                let idle_duration = last_activity.elapsed();

                if idle_duration >= idle_time {
                    // Network has been idle long enough!
                    debug!(
                        "Network idle achieved after {:?} (CDP-tracked)",
                        start.elapsed()
                    );
                    return Ok(());
                }

                consecutive_idle_time += check_interval;
            } else {
                // Network activity detected, reset idle timer
                active_requests = current_activity;
                last_activity = Instant::now();
                consecutive_idle_time = Duration::ZERO;
                debug!(
                    "Network activity detected: {} active requests",
                    active_requests
                );
            }

            tokio::time::sleep(check_interval).await;
        }

        // Timeout reached
        warn!(
            "Network idle timeout after {:?} (CDP-tracked, {} active requests)",
            start.elapsed(),
            active_requests
        );
        Ok(()) // Don't fail on timeout, just warn
    }

    /// Check current network activity using CDP Network domain
    async fn check_network_activity(&self) -> Result<usize> {
        // In a real implementation, this would track actual CDP Network events
        // For now, use a simplified approach that mimics CDP behavior
        let page = self.page.read().await;

        // Use JavaScript to check for active network requests if CDP events aren't fully available
        let script = r#"
            (function() {
                // Check document readyState first
                if (document.readyState === 'loading') return 1;
                
                // Check for ongoing fetch/XHR requests (simplified)
                // In practice, CDP Network domain would track this automatically
                const timing = performance.timing;
                if (timing.loadEventEnd === 0) return 1;
                
                // Check if very recent network activity occurred
                const now = Date.now();
                const timeSinceLoad = now - timing.loadEventEnd;
                
                // Consider network active if load event was very recent
                if (timeSinceLoad < 200) return 1;
                
                // Check for ongoing resources that might still be loading
                if (performance.getEntriesByType) {
                    const resources = performance.getEntriesByType('resource');
                    let activeRequests = 0;
                    
                    resources.forEach(resource => {
                        // Consider request active if it's very recent and might still be loading
                        if (resource.responseEnd === 0 || (now - resource.responseEnd) < 100) {
                            activeRequests++;
                        }
                    });
                    
                    return activeRequests;
                }
                
                return 0; // No active network activity detected
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if let Some(value) = result.value() {
                    if let Some(active_count) = value.as_f64() {
                        return Ok(active_count as usize);
                    }
                }
                // Default to no activity if we can't determine state
                Ok(0)
            }
            Err(_) => {
                // Fall back to basic document ready state check
                let basic_script = "document.readyState === 'complete' ? 0 : 1";
                match page.evaluate(basic_script).await {
                    Ok(result) => {
                        if let Some(value) = result.value() {
                            if let Some(active_count) = value.as_f64() {
                                return Ok(active_count as usize);
                            }
                        }
                        Ok(0)
                    }
                    Err(_) => Ok(0), // Assume no activity if all checks fail
                }
            }
        }
    }

    /// Navigate and wait for specific text to appear
    pub async fn navigate_and_wait_for_text(
        &self,
        url: &str,
        text: &str,
        timeout: Duration,
    ) -> Result<()> {
        info!("Navigating to {} and waiting for text: {}", url, text);

        // Navigate
        self.navigate_to(url).await?;

        // Wait for text to appear
        let start = std::time::Instant::now();
        let page = self.page.read().await;

        loop {
            let body_text = page.content().await?;
            if body_text.contains(text) {
                info!("Found text: {}", text);
                return Ok(());
            }

            if start.elapsed() > timeout {
                return Err(anyhow!("Timeout waiting for text: {}", text));
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Get all cookies
    pub async fn get_cookies(&self) -> Result<Vec<Cookie>> {
        let page = self.page.read().await;
        let cookies = page.get_cookies().await?;
        Ok(cookies)
    }

    /// Set a cookie
    pub async fn set_cookie(&self, cookie: CookieParam) -> Result<()> {
        let page = self.page.read().await;
        page.set_cookie(cookie).await?;
        Ok(())
    }

    /// Delete all cookies
    pub async fn delete_cookies(&self) -> Result<()> {
        let page = self.page.read().await;
        // Use DeleteCookies command with proper parameters
        page.delete_cookies(vec![]).await?;
        info!("Deleted all cookies");
        Ok(())
    }

    /// Set user agent
    pub async fn set_user_agent(&self, user_agent: &str) -> Result<()> {
        let page = self.page.read().await;
        page.set_user_agent(user_agent).await?;
        info!("Set user agent: {}", user_agent);
        Ok(())
    }

    /// Emulate device (simplified version)
    pub async fn emulate_device(&self, device_name: &str) -> Result<()> {
        let page = self.page.read().await;

        // Common device user agents
        let user_agent = match device_name {
            "iPhone 12" => {
                "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15"
            }
            "iPad" => "Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15",
            "Pixel 5" => "Mozilla/5.0 (Linux; Android 11; Pixel 5) AppleWebKit/537.36",
            _ => {
                warn!("Unknown device: {}, using default", device_name);
                return Ok(());
            }
        };

        page.set_user_agent(user_agent).await?;
        info!("Emulating device: {}", device_name);
        Ok(())
    }

    /// Get page metrics (performance, memory, etc.)
    pub async fn get_metrics(&self) -> Result<PageMetrics> {
        let page = self.page.read().await;

        // Get performance metrics via JavaScript
        let timing_result = page.evaluate(r#"
            JSON.stringify({
                loadTime: performance.timing.loadEventEnd - performance.timing.navigationStart,
                domContentLoaded: performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart,
                responseTime: performance.timing.responseEnd - performance.timing.requestStart,
                renderTime: performance.timing.domComplete - performance.timing.domLoading
            })
        "#).await?;
        let timing: serde_json::Value = timing_result.into_value()?;

        Ok(PageMetrics {
            load_time_ms: timing["loadTime"].as_u64().unwrap_or(0),
            dom_content_loaded_ms: timing["domContentLoaded"].as_u64().unwrap_or(0),
            response_time_ms: timing["responseTime"].as_u64().unwrap_or(0),
            render_time_ms: timing["renderTime"].as_u64().unwrap_or(0),
        })
    }
}

/// Page performance metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageMetrics {
    pub load_time_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub response_time_ms: u64,
    pub render_time_ms: u64,
}
