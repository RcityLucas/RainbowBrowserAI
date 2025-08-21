// Common utilities shared across all tool modules
//
// This module provides reusable components and utilities that are used
// by multiple tools to avoid code duplication and ensure consistency.

// TODO: Implement utility modules as needed
// pub mod wait_strategies;
// pub mod element_finder;
// pub mod screenshot_utils;
// pub mod history_manager;
// pub mod metrics_collector;

// Re-export commonly used utilities (commented out until implementation)
// pub use wait_strategies::{WaitStrategy, PollingWaiter};
// pub use element_finder::{ElementFinder, SelectorStrategy};
// pub use screenshot_utils::{ScreenshotProcessor, ImageProcessor};
// pub use history_manager::{HistoryManager, HistoryEntry};
// pub use metrics_collector::{MetricsCollector, OperationMetrics};

use std::time::Duration;
use thirtyfour::WebDriver;
use anyhow::Result;

/// Common error types for tools
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("WebDriver error: {0}")]
    WebDriverError(#[from] thirtyfour::error::WebDriverError),
    
    #[error("Element not found: {selector}")]
    ElementNotFound { selector: String },
    
    #[error("Timeout waiting for condition after {timeout:?}")]
    TimeoutError { timeout: Duration },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    #[error("Operation failed: {message}")]
    OperationFailed { message: String },
    
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },
}

/// Base trait for all browser automation tools
#[async_trait::async_trait]
pub trait BrowserTool: Send + Sync {
    type Input: Send + Sync + std::fmt::Debug;
    type Output: Send + Sync + std::fmt::Debug;
    
    /// Execute the tool with given input
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, ToolError>;
    
    /// Get the tool name
    fn name(&self) -> &str;
    
    /// Get the tool description
    fn description(&self) -> &str;
    
    /// Validate input parameters
    fn validate_input(&self, input: &Self::Input) -> Result<(), ToolError> {
        Ok(())
    }
    
    /// Estimate execution time
    fn estimate_duration(&self, _input: &Self::Input) -> Duration {
        Duration::from_millis(100)
    }
    
    /// Estimate resource cost
    fn estimate_cost(&self, _input: &Self::Input) -> f32 {
        0.001
    }
}

/// Helper function to create a WebDriver instance
pub async fn create_webdriver(port: u16) -> Result<WebDriver, ToolError> {
    use thirtyfour::ChromeCapabilities;
    
    let caps = ChromeCapabilities::new();
    let driver = WebDriver::new(&format!("http://localhost:{}", port), caps).await?;
    Ok(driver)
}

/// Standard timeout values used across tools
pub struct StandardTimeouts;

impl StandardTimeouts {
    pub const SHORT: Duration = Duration::from_secs(5);
    pub const MEDIUM: Duration = Duration::from_secs(15);
    pub const LONG: Duration = Duration::from_secs(30);
    pub const VERY_LONG: Duration = Duration::from_secs(60);
}

/// Standard polling intervals
pub struct StandardIntervals;

impl StandardIntervals {
    pub const FAST: Duration = Duration::from_millis(50);
    pub const NORMAL: Duration = Duration::from_millis(100);
    pub const SLOW: Duration = Duration::from_millis(250);
    pub const VERY_SLOW: Duration = Duration::from_millis(500);
}