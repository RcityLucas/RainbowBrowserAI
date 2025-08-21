//! Standard Tools Module for RainbowBrowserAI
//! 
//! This module implements the 12 standard tools as specified in TOOLS.md,
//! providing a unified, intelligent, and predictable interface for AI operations
//! in the digital world.

pub mod types;
pub mod errors;
pub mod config;
pub mod security;  // Security module for input validation
pub mod navigation;
pub mod interaction;
pub mod synchronization;
pub mod data_extraction;
pub mod intelligence;
pub mod advanced_automation;  // Phase 3 - Advanced automation tools
pub mod memory;  // V8.0 Memory tools (Phase 1)
pub mod common;

#[cfg(test)]
mod integration_tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
// Re-export commonly used types
pub use types::*;
pub use errors::*;
pub use config::*;

// Convenience type alias for consistent error handling
pub type Result<T> = anyhow::Result<T>;
pub use navigation::*;
pub use interaction::*;
pub use synchronization::*;
pub use data_extraction::*;
// pub use intelligence::*;     // TODO: Uncomment when Phase 3 tools are implemented
pub use advanced_automation::*;  // Phase 3 - Advanced automation tools
pub use memory::*;  // V8.0 Memory tools

// Re-export common utilities but avoid conflicts
pub use common::{
    BrowserTool, create_webdriver, StandardTimeouts, StandardIntervals
    // ToolError is intentionally not re-exported to avoid conflict with errors::ToolError
};

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Input parameters type for this tool
    type Input: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    
    /// Output result type for this tool
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    
    /// Get the name of this tool
    fn name(&self) -> &str;
    
    /// Get the description of this tool
    fn description(&self) -> &str;
    
    /// Execute the tool with given parameters
    async fn execute(&self, params: Self::Input) -> Result<Self::Output>;
    
    /// Validate input parameters before execution
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        Ok(())
    }
    
    /// Get the JSON schema for input parameters
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    
    /// Get the JSON schema for output
    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

/// Dynamic tool trait for runtime dispatch
#[async_trait]
pub trait DynamicTool: Send + Sync {
    /// Get the name of this tool
    fn name(&self) -> &str;
    
    /// Execute with JSON input and return JSON output
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Get input schema
    fn input_schema(&self) -> serde_json::Value;
    
    /// Get output schema
    fn output_schema(&self) -> serde_json::Value;
}

/// Tool registry for managing all available tools
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Arc<dyn DynamicTool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }
    
    /// Register a tool
    pub fn register<T>(&mut self, tool: T)
    where
        T: Tool + 'static,
        T::Input: 'static,
        T::Output: 'static,
    {
        let name = tool.name().to_string();
        let dynamic_tool = Arc::new(DynamicToolWrapper::new(tool));
        self.tools.insert(name, dynamic_tool);
    }
    
    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn DynamicTool>> {
        self.tools.get(name).cloned()
    }
    
    /// List all available tools
    pub fn list(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

/// Wrapper to convert a typed tool to a dynamic tool
pub struct DynamicToolWrapper<T: Tool> {
    tool: T,
}

impl<T: Tool + 'static> DynamicToolWrapper<T> 
where
    T::Input: 'static,
    T::Output: 'static,
{
    pub fn new(tool: T) -> Self {
        Self { tool }
    }
}

#[async_trait]
impl<T> DynamicTool for DynamicToolWrapper<T>
where
    T: Tool + Send + Sync + 'static,
    T::Input: 'static,
    T::Output: 'static,
{
    fn name(&self) -> &str {
        self.tool.name()
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: T::Input = serde_json::from_value(params)?;
        let output = self.tool.execute(input).await?;
        Ok(serde_json::to_value(output)?)
    }
    
    fn input_schema(&self) -> serde_json::Value {
        self.tool.input_schema()
    }
    
    fn output_schema(&self) -> serde_json::Value {
        self.tool.output_schema()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }
}