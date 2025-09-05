pub mod browser;
pub mod api;
pub mod tools;
pub mod perception;

// Re-export commonly used types
pub use browser::{Browser, BrowserOps, ElementInfo, ScreenshotOptions};
pub use browser::pool::BrowserPool;
// Re-export perception types (currently not all may be used)
// pub use perception::{
//     PerceptionEngine, PerceivedElement, ElementType, PageType,
//     PerceptionAwareBrowser, IntelligentCommand, IntelligentCommandResult
// };