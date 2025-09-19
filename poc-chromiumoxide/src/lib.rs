pub mod api;
pub mod browser;
pub mod coordination;
pub mod intelligence;
pub mod llm;
pub mod perception;
pub mod tools; // New coordination module

// Re-export commonly used types
pub use browser::pool::BrowserPool;
pub use browser::{Browser, BrowserOps, ElementInfo, ScreenshotOptions};

// Re-export coordination types for easy access
pub use coordination::{
    Event, EventBus, EventType, RainbowCoordinator, SessionBundle, UnifiedStateManager,
};

// Re-export perception types (currently not all may be used)
// pub use perception::{
//     PerceptionEngine, PerceivedElement, ElementType, PageType,
//     PerceptionAwareBrowser, IntelligentCommand, IntelligentCommandResult
// };
