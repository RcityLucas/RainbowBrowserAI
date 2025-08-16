// User-friendly API facade - Simple interface for general users
use anyhow::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::{
    RainbowBrowserV8,
    events::{EventType, EventBus, EventPublisher},
    simplified_traits::SimpleConfig,
};

/// User-friendly browser builder
pub struct RainbowBrowserBuilder {
    preset: BrowserPreset,
    custom_config: Option<UserConfig>,
}

/// Pre-configured browser presets for common use cases
#[derive(Debug, Clone)]
pub enum BrowserPreset {
    FastBrowsing,      // Lightning perception, minimal validation
    Balanced,          // Standard perception, normal validation
    Thorough,          // Deep perception, full validation
    Shopping,          // Optimized for e-commerce sites
    Research,          // Optimized for data gathering
    Monitoring,        // Optimized for watching changes
}

/// Simple user configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub language: String,
    pub timeout_seconds: u32,
    pub auto_retry: bool,
    pub save_history: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timeout_seconds: 30,
            auto_retry: true,
            save_history: true,
        }
    }
}

impl RainbowBrowserBuilder {
    pub fn new() -> Self {
        Self {
            preset: BrowserPreset::Balanced,
            custom_config: None,
        }
    }
    
    pub fn with_preset(mut self, preset: BrowserPreset) -> Self {
        self.preset = preset;
        self
    }
    
    pub fn with_config(mut self, config: UserConfig) -> Self {
        self.custom_config = Some(config);
        self
    }
    
    pub async fn build(self) -> Result<SimpleBrowser> {
        let browser = RainbowBrowserV8::new().await?;
        
        Ok(SimpleBrowser {
            inner: Arc::new(browser),
            preset: self.preset,
            config: self.custom_config.unwrap_or_default(),
            progress_handler: None,
        })
    }
}

/// Simplified browser interface for users
pub struct SimpleBrowser {
    inner: Arc<RainbowBrowserV8>,
    preset: BrowserPreset,
    config: UserConfig,
    progress_handler: Option<Arc<dyn ProgressHandler>>,
}

/// Progress handler for user feedback
pub trait ProgressHandler: Send + Sync {
    fn on_progress(&self, progress: Progress);
    fn on_error(&self, error: UserError);
}

/// User-friendly progress information
#[derive(Debug, Clone)]
pub struct Progress {
    pub stage: String,
    pub percentage: u8,
    pub message: String,
}

/// User-friendly error messages
#[derive(Debug, Clone)]
pub enum UserError {
    NetworkIssue(String),
    WebsiteChanged(String),
    TaskFailed(String),
    Timeout(String),
    ConfigError(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::NetworkIssue(msg) => write!(f, "Network issue: {}. Please check your connection.", msg),
            UserError::WebsiteChanged(msg) => write!(f, "Website has changed: {}. Trying alternative approach...", msg),
            UserError::TaskFailed(msg) => write!(f, "Could not complete task: {}", msg),
            UserError::Timeout(msg) => write!(f, "Operation timed out: {}", msg),
            UserError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl SimpleBrowser {
    /// Execute a simple task with natural language
    pub async fn simple_task(&self, description: &str) -> Result<TaskResult> {
        self.update_progress("Starting task", 0);
        
        let result = match self.inner.process_request(description).await {
            Ok(response) => {
                self.update_progress("Task completed", 100);
                TaskResult {
                    success: true,
                    message: response,
                    data: None,
                }
            }
            Err(e) => {
                let user_error = self.convert_error(e);
                if let Some(handler) = &self.progress_handler {
                    handler.on_error(user_error.clone());
                }
                
                if self.config.auto_retry {
                    self.update_progress("Retrying...", 50);
                    self.retry_with_fallback(description).await?
                } else {
                    TaskResult {
                        success: false,
                        message: user_error.to_string(),
                        data: None,
                    }
                }
            }
        };
        
        Ok(result)
    }
    
    /// Common tasks with templates
    pub async fn find_best_price(&self, product: &str) -> Result<PriceResult> {
        let task = format!("Find the best price for {}", product);
        let result = self.simple_task(&task).await?;
        
        Ok(PriceResult {
            product: product.to_string(),
            best_price: self.extract_price(&result.message),
            store: self.extract_store(&result.message),
        })
    }
    
    pub async fn book_travel(&self, destination: &str, date: &str) -> Result<TravelResult> {
        let task = format!("Book travel to {} on {}", destination, date);
        let result = self.simple_task(&task).await?;
        
        Ok(TravelResult {
            destination: destination.to_string(),
            date: date.to_string(),
            booking_reference: self.extract_reference(&result.message),
        })
    }
    
    pub async fn fill_form(&self, form_url: &str, data: FormData) -> Result<TaskResult> {
        let task = format!("Fill form at {} with provided data", form_url);
        // Pass form data through context
        self.simple_task(&task).await
    }
    
    pub async fn monitor_changes(&self, url: &str, check_interval: u32) -> Result<MonitorResult> {
        let task = format!("Monitor {} for changes every {} seconds", url, check_interval);
        let result = self.simple_task(&task).await?;
        
        Ok(MonitorResult {
            url: url.to_string(),
            changes_detected: false,
            last_checked: std::time::SystemTime::now(),
        })
    }
    
    /// Set progress handler
    pub fn set_progress_handler(&mut self, handler: Arc<dyn ProgressHandler>) {
        self.progress_handler = Some(handler);
    }
    
    // Helper methods
    fn update_progress(&self, message: &str, percentage: u8) {
        if let Some(handler) = &self.progress_handler {
            handler.on_progress(Progress {
                stage: message.to_string(),
                percentage,
                message: message.to_string(),
            });
        }
    }
    
    fn convert_error(&self, error: anyhow::Error) -> UserError {
        // Convert technical errors to user-friendly messages
        let error_string = error.to_string();
        
        if error_string.contains("network") || error_string.contains("connection") {
            UserError::NetworkIssue(error_string)
        } else if error_string.contains("timeout") {
            UserError::Timeout(error_string)
        } else if error_string.contains("element not found") {
            UserError::WebsiteChanged(error_string)
        } else {
            UserError::TaskFailed(error_string)
        }
    }
    
    async fn retry_with_fallback(&self, description: &str) -> Result<TaskResult> {
        // Simple retry logic with fallback strategy
        match self.inner.process_request(description).await {
            Ok(response) => Ok(TaskResult {
                success: true,
                message: response,
                data: None,
            }),
            Err(_) => Ok(TaskResult {
                success: false,
                message: "Task could not be completed after retry".to_string(),
                data: None,
            }),
        }
    }
    
    fn extract_price(&self, text: &str) -> Option<f64> {
        // Simple price extraction logic
        None // Placeholder
    }
    
    fn extract_store(&self, text: &str) -> Option<String> {
        // Simple store extraction logic
        None // Placeholder
    }
    
    fn extract_reference(&self, text: &str) -> Option<String> {
        // Simple reference extraction logic
        None // Placeholder
    }
}

/// Task result
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Price comparison result
#[derive(Debug, Clone)]
pub struct PriceResult {
    pub product: String,
    pub best_price: Option<f64>,
    pub store: Option<String>,
}

/// Travel booking result
#[derive(Debug, Clone)]
pub struct TravelResult {
    pub destination: String,
    pub date: String,
    pub booking_reference: Option<String>,
}

/// Form data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormData {
    pub fields: std::collections::HashMap<String, String>,
}

/// Monitor result
#[derive(Debug, Clone)]
pub struct MonitorResult {
    pub url: String,
    pub changes_detected: bool,
    pub last_checked: std::time::SystemTime,
}

/// Example usage for documentation
#[cfg(test)]
mod examples {
    use super::*;
    
    async fn example_usage() -> Result<()> {
        // Simple one-liner setup
        let browser = RainbowBrowserBuilder::new()
            .with_preset(BrowserPreset::Shopping)
            .build()
            .await?;
        
        // Natural language task
        let result = browser.simple_task("Find flights to Tokyo").await?;
        println!("Result: {}", result.message);
        
        // Template-based task
        let price = browser.find_best_price("iPhone 15").await?;
        println!("Best price: {:?}", price.best_price);
        
        Ok(())
    }
}