pub mod browser;
pub mod api;
pub mod tools;
pub mod perception;
pub mod llm;
pub mod intelligence;
pub mod coordination;  // New coordination module

// Re-export commonly used types
pub use browser::{Browser, BrowserOps, ElementInfo, ScreenshotOptions};
pub use browser::pool::BrowserPool;

// Re-export coordination types for easy access
pub use coordination::{
    RainbowCoordinator,
    SessionBundle,
    Event, EventBus, EventType,
    UnifiedStateManager,
};

// Re-export perception types (currently not all may be used)
// pub use perception::{
//     PerceptionEngine, PerceivedElement, ElementType, PageType,
//     PerceptionAwareBrowser, IntelligentCommand, IntelligentCommandResult
// };