pub mod core;
pub mod pool;
pub mod session;
pub mod navigation;

// Re-export main types
pub use core::{Browser, BrowserOps, BrowserOperations, ElementInfo, ElementRect, ScreenshotOptions};
pub use pool::{BrowserPool, BrowserGuard};
pub use session::{BrowserSession, SessionManager, SessionInfo};
pub use navigation::{NavigationOptions, NavigationResult, PageMetrics};