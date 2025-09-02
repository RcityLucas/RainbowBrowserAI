// Browser automation module
pub mod browser;
pub mod browser_pool;
pub mod chromedriver_manager;
pub mod mock; // Mock browser for testing

// Re-export main types
pub use browser::*;
pub use browser_pool::{BrowserPool, PooledBrowserHandle};
pub use chromedriver_manager::*;