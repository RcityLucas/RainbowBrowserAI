use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use tracing::{info, warn, debug};
use super::traits::{DynamicTool, ToolCategory, ToolMetadata};
use crate::browser::Browser;

/// Tool execution result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Tool registry for managing all available tools
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn DynamicTool>>>>,
    browser: Arc<Browser>,
    execution_history: Arc<RwLock<Vec<ToolExecutionResult>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            browser,
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a new tool
    pub async fn register(&self, tool: Arc<dyn DynamicTool>) -> Result<()> {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().await;
        
        if tools.contains_key(&name) {
            warn!("Tool {} is already registered, replacing", name);
        }
        
        info!("Registering tool: {} ({})", name, tool.description());
        tools.insert(name, tool);
        Ok(())
    }
    
    /// Unregister a tool
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut tools = self.tools.write().await;
        
        if tools.remove(name).is_some() {
            info!("Unregistered tool: {}", name);
            Ok(())
        } else {
            Err(anyhow!("Tool not found: {}", name))
        }
    }
    
    /// Get a tool by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn DynamicTool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
    
    /// Execute a tool by name with JSON input
    pub async fn execute(&self, name: &str, input: Value) -> Result<ToolExecutionResult> {
        let start = std::time::Instant::now();
        
        // Get the tool
        let tool = self.get(name).await
            .ok_or_else(|| anyhow!("Tool not found: {}", name))?;
        
        debug!("Executing tool: {} with input: {:?}", name, input);
        
        // Execute the tool
        let result = match tool.execute_json(input).await {
            Ok(output) => {
                let execution_time = start.elapsed().as_millis() as u64;
                info!("Tool {} executed successfully in {}ms", name, execution_time);
                
                ToolExecutionResult {
                    tool_name: name.to_string(),
                    success: true,
                    output: Some(output),
                    error: None,
                    execution_time_ms: execution_time,
                }
            }
            Err(e) => {
                let execution_time = start.elapsed().as_millis() as u64;
                warn!("Tool {} failed: {}", name, e);
                
                ToolExecutionResult {
                    tool_name: name.to_string(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                }
            }
        };
        
        // Store in history
        {
            let mut history = self.execution_history.write().await;
            history.push(result.clone());
            
            // Keep only last 100 executions
            if history.len() > 100 {
                let drain_count = history.len() - 100;
                history.drain(0..drain_count);
            }
        }
        
        Ok(result)
    }
    
    /// Execute a chain of tools
    pub async fn execute_chain(&self, chain: Vec<(String, Value)>) -> Result<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();
        
        for (tool_name, input) in chain {
            let result = self.execute(&tool_name, input).await?;
            
            // Stop chain if a tool fails
            if !result.success {
                return Err(anyhow!("Tool chain failed at {}: {:?}", 
                    tool_name, result.error));
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values()
            .map(|tool| tool.metadata())
            .collect()
    }
    
    /// List tools by category
    pub async fn list_by_category(&self, category: ToolCategory) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values()
            .filter(|tool| tool.category() == category)
            .map(|tool| tool.metadata())
            .collect()
    }
    
    /// Get execution history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<ToolExecutionResult> {
        let history = self.execution_history.read().await;
        let limit = limit.unwrap_or(history.len());
        
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Clear execution history
    pub async fn clear_history(&self) {
        let mut history = self.execution_history.write().await;
        history.clear();
        info!("Cleared tool execution history");
    }
    
    /// Get statistics about tool usage
    pub async fn get_statistics(&self) -> HashMap<String, ToolStatistics> {
        let history = self.execution_history.read().await;
        let mut stats: HashMap<String, ToolStatistics> = HashMap::new();
        
        for result in history.iter() {
            let entry = stats.entry(result.tool_name.clone())
                .or_insert_with(|| ToolStatistics::default());
            
            entry.total_executions += 1;
            entry.total_time_ms += result.execution_time_ms;
            
            if result.success {
                entry.successful_executions += 1;
            }
        }
        
        // Calculate averages
        for stat in stats.values_mut() {
            if stat.total_executions > 0 {
                stat.average_time_ms = stat.total_time_ms / stat.total_executions;
                stat.success_rate = (stat.successful_executions as f64) / (stat.total_executions as f64);
            }
        }
        
        stats
    }
}

/// Statistics about tool usage
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ToolStatistics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub total_time_ms: u64,
    pub average_time_ms: u64,
    pub success_rate: f64,
}

/// Builder for registering multiple tools at once
pub struct ToolRegistryBuilder {
    browser: Arc<Browser>,
    tools: Vec<Arc<dyn DynamicTool>>,
}

impl ToolRegistryBuilder {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            tools: Vec::new(),
        }
    }
    
    pub fn with_tool(mut self, tool: Arc<dyn DynamicTool>) -> Self {
        self.tools.push(tool);
        self
    }
    
    pub async fn build(self) -> Result<ToolRegistry> {
        let registry = ToolRegistry::new(self.browser);
        
        for tool in self.tools {
            registry.register(tool).await?;
        }
        
        Ok(registry)
    }
}