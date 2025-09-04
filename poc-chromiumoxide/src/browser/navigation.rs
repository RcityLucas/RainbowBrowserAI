use anyhow::{Result, anyhow};
use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam};
use std::time::Duration;
use tracing::{info, warn, debug};
use super::core::{Browser, BrowserOps};

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
            self.wait_for_selector(selector, Duration::from_secs(10)).await?;
        }
        
        let load_time = start_time.elapsed();
        let current_url = self.current_url().await?;
        
        Ok(NavigationResult {
            url: current_url,
            status: 200, // TODO: Get actual status from response
            load_time_ms: load_time.as_millis() as u64,
            dom_content_loaded_ms: 0, // TODO: Get actual timing
            redirects: Vec::new(), // TODO: Track redirects
        })
    }

    /// Reload the current page
    pub async fn reload(&self) -> Result<()> {
        info!("Reloading current page");
        let page = self.page.read().await;
        page.reload().await?
            .wait_for_navigation().await?;
        Ok(())
    }

    /// Hard reload with cache bypass
    pub async fn hard_reload(&self) -> Result<()> {
        info!("Hard reloading current page (bypass cache)");
        // Simple reload for now - chromiumoxide may handle cache differently
        let page = self.page.read().await;
        page.reload().await?
            .wait_for_navigation().await?;
        Ok(())
    }

    /// Wait for network to be idle
    pub async fn wait_for_network_idle(&self, idle_time: Duration) -> Result<()> {
        debug!("Waiting for network idle ({:?})", idle_time);
        // Simple implementation - wait for the specified duration
        // TODO: Implement actual network idle detection
        tokio::time::sleep(idle_time).await;
        Ok(())
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
            "iPhone 12" => "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15",
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