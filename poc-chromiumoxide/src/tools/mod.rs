// Tool module for browser automation tools
// Currently most tools are placeholders for future implementation

pub mod traits;
// pub mod registry;  // Removed - unused
pub mod navigation;
pub mod interaction;
pub mod extraction;
pub mod synchronization;
pub mod memory;
pub mod config;

// Re-exports commented out until tools are actually used
// pub use traits::{Tool, DynamicTool, ToolCategory, ToolMetadata};
// pub use config::ToolConfig;
// pub use navigation::{NavigateTool, ScrollTool, RefreshTool};
// pub use interaction::{ClickTool, TypeTextTool, SelectOptionTool};
// pub use extraction::{ExtractTextTool, ExtractLinksTool, ExtractDataTool};
// pub use synchronization::{WaitForElementTool, WaitForConditionTool};
// pub use memory::{ScreenshotTool, SessionMemoryTool};