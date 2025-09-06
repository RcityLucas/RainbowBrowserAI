pub mod core;
pub mod pool;
pub mod session;
pub mod navigation;

// Re-export main types
pub use core::{Browser, BrowserOps, ScreenshotOptions, ElementInfo};
pub use session::SessionManager;
