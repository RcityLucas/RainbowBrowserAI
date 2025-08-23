pub mod services;
pub mod traits;
pub mod utils;

// Re-export commonly used types
pub use traits::*;
pub use services::*;
pub use utils::*;

// Common error type for all shared services
pub type Result<T> = std::result::Result<T, anyhow::Error>;

// Service registry for dependency injection
pub struct ServiceRegistry {
    pub browser: std::sync::Arc<dyn crate::traits::BrowserService>,
    pub llm: std::sync::Arc<dyn crate::traits::LLMService>,
}

impl ServiceRegistry {
    pub fn new(
        browser: std::sync::Arc<dyn crate::traits::BrowserService>,
        llm: std::sync::Arc<dyn crate::traits::LLMService>,
    ) -> Self {
        Self { browser, llm }
    }
}