use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use futures::future::BoxFuture;

pub mod traits;
pub mod registry;
pub mod navigation;
pub mod interaction;
pub mod extraction;
pub mod synchronization;
pub mod memory;
pub mod config;

// Re-export commonly used types
pub use traits::{Tool, DynamicTool, ToolCategory, ToolMetadata};
pub use registry::ToolRegistry;
pub use config::ToolConfig;

// Re-export specific tools
pub use navigation::{NavigateTool, ScrollTool, RefreshTool};
pub use interaction::{ClickTool, TypeTextTool, SelectOptionTool};
pub use extraction::{ExtractTextTool, ExtractLinksTool, ExtractDataTool};
pub use synchronization::{WaitForElementTool, WaitForConditionTool};
pub use memory::{ScreenshotTool, SessionMemoryTool};