//! # 错误处理
//! 
//! 定义项目中使用的错误类型

use thiserror::Error;

/// 浏览器错误类型
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("WebDriver error: {0}")]
    WebDriverError(String),
    
    #[error("LLM error: {0}")]
    LLMError(String),
    
    #[error("Session error: {0}")]
    SessionError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, BrowserError>;

// impl From<reqwest::Error> for BrowserError {
//     fn from(err: reqwest::Error) -> Self {
//         BrowserError::NetworkError(err.to_string())
//     }
// }

// impl From<thirtyfour::error::WebDriverError> for BrowserError {
//     fn from(err: thirtyfour::error::WebDriverError) -> Self {
//         BrowserError::WebDriverError(err.to_string())
//     }
// }

impl From<serde_json::Error> for BrowserError {
    fn from(err: serde_json::Error) -> Self {
        BrowserError::LLMError(format!("JSON parsing error: {}", err))
    }
}

impl From<anyhow::Error> for BrowserError {
    fn from(err: anyhow::Error) -> Self {
        BrowserError::Unknown(err.to_string())
    }
}