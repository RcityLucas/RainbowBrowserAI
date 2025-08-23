//! Error types for tools module

use thiserror::Error;

/// Main error type for tool operations
#[derive(Error, Debug)]
pub enum ToolError {
    /// Tool not found in registry
    #[error("Tool not found: {0}")]
    NotFound(String),
    
    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Invalid output from tool
    #[error("Invalid output: {0}")]
    InvalidOutput(String),
    
    /// Operation timed out
    #[error("Operation timed out after {0} ms")]
    Timeout(u64),
    
    /// Element not found
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    
    /// Navigation failed
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    
    /// Browser error
    #[error("Browser error: {0}")]
    BrowserError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// JavaScript execution error
    #[error("JavaScript error: {0}")]
    JavaScriptError(String),
    
    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Screenshot failed
    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),
    
    /// Condition not met
    #[error("Condition not met: {0}")]
    ConditionNotMet(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Other error with context
    #[error("{0}")]
    Other(String),
    
    /// Wrapped anyhow error
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::SerializationError(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for ToolError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        ToolError::Timeout(0)
    }
}

// Removed: anyhow already implements From<E> for Error where E: std::error::Error
// The ToolError already implements std::error::Error via thiserror, so this is redundant
// and causes a conflict with anyhow's blanket implementation

/// Result type for tool operations
pub type ToolResult<T> = Result<T, ToolError>;