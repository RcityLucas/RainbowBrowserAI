pub mod core;
pub mod navigation;
pub mod pool;
pub mod session;

// Re-export main types
pub use core::{Browser, BrowserOps, ElementInfo, ScreenshotOptions};
pub use session::SessionManager;
