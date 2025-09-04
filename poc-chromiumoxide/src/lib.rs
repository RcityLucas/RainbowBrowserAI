pub mod browser;
pub mod api;
pub mod tools;

// Re-export commonly used types
pub use browser::{Browser, BrowserOps, ElementInfo, ScreenshotOptions};
pub use browser::pool::BrowserPool;