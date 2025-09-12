use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
// use futures::future::BoxFuture; // Unused import
use std::sync::Arc;

/// Tool categories for organization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ToolCategory {
    Navigation,
    Interaction,
    DataExtraction,
    Synchronization,
    Memory,
    Intelligence,
    MetaCognitive,
    AdvancedAutomation,
    Workflow,
}

/// Metadata about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub version: String,
    pub author: String,
    pub input_schema: Value,
    pub output_schema: Value,
}

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    type Input: Serialize + DeserializeOwned + Send + Sync;
    type Output: Serialize + Send + Sync;
    
    /// Get the name of this tool
    fn name(&self) -> &str;
    
    /// Get a description of what this tool does
    fn description(&self) -> &str;
    
    /// Get the category this tool belongs to
    fn category(&self) -> ToolCategory;
    
    /// Execute the tool with the given input
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    
    /// Validate the input before execution
    async fn validate_input(&self, _input: &Self::Input) -> Result<()> {
        // Default implementation - override for custom validation
        Ok(())
    }
    
    /// Get metadata about this tool
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: self.name().to_string(),
            description: self.description().to_string(),
            category: self.category(),
            version: "1.0.0".to_string(),
            author: "RainbowBrowserAI".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
        }
    }
}

/// Dynamic tool trait for runtime dispatch
#[async_trait]
pub trait DynamicTool: Send + Sync {
    /// Get the name of this tool
    fn name(&self) -> &str;
    
    /// Get the description of this tool
    fn description(&self) -> &str;
    
    /// Get the category of this tool
    fn category(&self) -> ToolCategory;
    
    /// Execute the tool with JSON input
    async fn execute_json(&self, input: Value) -> Result<Value>;
    
    /// Validate JSON input
    async fn validate_json(&self, input: &Value) -> Result<()>;
    
    /// Get metadata about this tool
    fn metadata(&self) -> ToolMetadata;
}

/// Wrapper to convert a typed tool to a dynamic tool
pub struct DynamicToolWrapper<T: Tool> {
    tool: Arc<T>,
}

impl<T: Tool> DynamicToolWrapper<T> {
    pub fn new(tool: T) -> Self {
        Self {
            tool: Arc::new(tool),
        }
    }
}

#[async_trait]
impl<T> DynamicTool for DynamicToolWrapper<T>
where
    T: Tool + 'static,
{
    fn name(&self) -> &str {
        self.tool.name()
    }
    
    fn description(&self) -> &str {
        self.tool.description()
    }
    
    fn category(&self) -> ToolCategory {
        self.tool.category()
    }
    
    async fn execute_json(&self, input: Value) -> Result<Value> {
        // Deserialize JSON to typed input
        let typed_input: T::Input = serde_json::from_value(input)?;
        
        // Validate input
        self.tool.validate_input(&typed_input).await?;
        
        // Execute tool
        let output = self.tool.execute(typed_input).await?;
        
        // Serialize output to JSON
        Ok(serde_json::to_value(output)?)
    }
    
    async fn validate_json(&self, input: &Value) -> Result<()> {
        // Try to deserialize to validate structure
        let typed_input: T::Input = serde_json::from_value(input.clone())?;
        self.tool.validate_input(&typed_input).await
    }
    
    fn metadata(&self) -> ToolMetadata {
        self.tool.metadata()
    }
}