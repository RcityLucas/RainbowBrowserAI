use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Default timeout for tool operations
    pub default_timeout: Duration,
    
    /// Maximum number of retries for failed operations
    pub max_retries: u32,
    
    /// Delay between retries
    pub retry_delay: Duration,
    
    /// Enable debug logging for tools
    pub debug_mode: bool,
    
    /// Maximum execution time for a single tool
    pub max_execution_time: Duration,
    
    /// Enable performance tracking
    pub track_performance: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            debug_mode: false,
            max_execution_time: Duration::from_secs(60),
            track_performance: true,
        }
    }
}