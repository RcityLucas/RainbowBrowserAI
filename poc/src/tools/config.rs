//! Configuration for tools module

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Global configuration for all tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Default timeout for operations (milliseconds)
    pub default_timeout_ms: u64,
    
    /// Default poll interval for waiting operations (milliseconds)
    pub default_poll_interval_ms: u64,
    
    /// Enable caching of tool results
    pub enable_cache: bool,
    
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    
    /// Enable retry on failure
    pub enable_retry: bool,
    
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Retry backoff multiplier
    pub retry_backoff_multiplier: f64,
    
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    
    /// Enable detailed logging
    pub enable_logging: bool,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    
    /// Browser-specific configuration
    pub browser: BrowserConfig,
    
    /// Screenshot configuration
    pub screenshot: ScreenshotConfig,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            default_poll_interval_ms: 100,
            enable_cache: true,
            cache_ttl_seconds: 300,
            enable_retry: true,
            max_retries: 3,
            retry_backoff_multiplier: 2.0,
            enable_metrics: true,
            enable_logging: true,
            rate_limit: RateLimitConfig::default(),
            browser: BrowserConfig::default(),
            screenshot: ScreenshotConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    
    /// Maximum requests per second
    pub max_requests_per_second: u32,
    
    /// Burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests_per_second: 10,
            burst_size: 20,
        }
    }
}

/// Browser-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Browser viewport size
    pub viewport_width: u32,
    pub viewport_height: u32,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// Enable JavaScript
    pub enable_javascript: bool,
    
    /// Enable cookies
    pub enable_cookies: bool,
    
    /// Headless mode
    pub headless: bool,
    
    /// Device pixel ratio
    pub device_pixel_ratio: f64,
    
    /// Default navigation timeout
    pub navigation_timeout_ms: u64,
    
    /// Enable request interception
    pub enable_request_interception: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: None,
            enable_javascript: true,
            enable_cookies: true,
            headless: true,
            device_pixel_ratio: 1.0,
            navigation_timeout_ms: 30000,
            enable_request_interception: false,
        }
    }
}

/// Screenshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    /// Default screenshot format
    pub default_format: String,
    
    /// JPEG quality (0-100)
    pub jpeg_quality: u8,
    
    /// Enable screenshot optimization
    pub optimize: bool,
    
    /// Maximum screenshot size in bytes
    pub max_size_bytes: usize,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            default_format: "png".to_string(),
            jpeg_quality: 85,
            optimize: true,
            max_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl ToolsConfig {
    /// Get default timeout as Duration
    pub fn default_timeout(&self) -> Duration {
        Duration::from_millis(self.default_timeout_ms)
    }
    
    /// Get default poll interval as Duration
    pub fn default_poll_interval(&self) -> Duration {
        Duration::from_millis(self.default_poll_interval_ms)
    }
    
    /// Get navigation timeout as Duration
    pub fn navigation_timeout(&self) -> Duration {
        Duration::from_millis(self.browser.navigation_timeout_ms)
    }
}