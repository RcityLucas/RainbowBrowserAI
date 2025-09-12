use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde_json::Value;
use tracing::{info, debug, error, warn};
use std::time::Instant;
use tokio::sync::RwLock;

use super::traits::{DynamicTool, DynamicToolWrapper, ToolCategory, ToolMetadata};
use super::navigation::{NavigateTool, ScrollTool, RefreshTool, GoBackTool, GoForwardTool};
use super::interaction::{ClickTool, TypeTextTool, SelectOptionTool, HoverTool, FocusTool};
use super::extraction::{ExtractTextTool, ExtractLinksTool, ExtractDataTool, ExtractTableTool, ExtractFormTool};
use super::synchronization::{WaitForElementTool, WaitForConditionTool, WaitForNavigationTool, WaitForNetworkIdleTool};
use super::memory::{ScreenshotTool, SessionMemoryTool, GetElementInfoTool, HistoryTrackerTool, PersistentCacheTool};
use super::cdp_monitoring::{NetworkMonitorTool, PerformanceMetricsTool, CDPNetworkIdleTool};
use super::intelligent_action::IntelligentActionTool;
use super::synthetic_fixtures::CreateTestFixtureTool;
use super::cache::ToolCache;
use super::dependencies::{DependencyManager, ExecutionPlan, ExecutionContext, ExecutionStats};
use crate::browser::Browser;

/// Performance metrics for tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolPerformanceMetric {
    pub tool_name: String,
    pub execution_time_ms: u64,
    pub success: bool,
    pub timestamp: std::time::SystemTime,
    pub input_size_bytes: usize,
    pub output_size_bytes: usize,
    pub error_message: Option<String>,
}

/// Aggregated performance statistics for a tool
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolPerformanceStats {
    pub tool_name: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time_ms: f64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub last_execution: Option<std::time::SystemTime>,
}

/// Central registry for all browser automation tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn DynamicTool>>,
    categories: HashMap<ToolCategory, Vec<String>>,
    performance_metrics: Arc<RwLock<Vec<ToolPerformanceMetric>>>,
    pub cache: Arc<ToolCache>,
    pub dependency_manager: Arc<DependencyManager>,
}

impl ToolRegistry {
    /// Create a new ToolRegistry and populate it with all available tools
    pub fn new(browser: Arc<Browser>) -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
            performance_metrics: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(ToolCache::new()),
            dependency_manager: Arc::new(DependencyManager::new()),
        };

        registry.register_all_tools(browser);
        registry
    }

    /// Register all available tools with the registry
    fn register_all_tools(&mut self, browser: Arc<Browser>) {
        info!("Registering all browser automation tools");

        // Navigation Tools
        self.register_tool(NavigateTool::new(browser.clone()));
        self.register_tool(ScrollTool::new(browser.clone()));
        self.register_tool(RefreshTool::new(browser.clone()));
        self.register_tool(GoBackTool::new(browser.clone()));
        self.register_tool(GoForwardTool::new(browser.clone()));

        // Interaction Tools
        self.register_tool(ClickTool::new(browser.clone()));
        self.register_tool(TypeTextTool::new(browser.clone()));
        self.register_tool(SelectOptionTool::new(browser.clone()));
        self.register_tool(HoverTool::new(browser.clone()));
        self.register_tool(FocusTool::new(browser.clone()));
        
        // Intelligent Action Engine
        self.register_tool(IntelligentActionTool::new(browser.clone()));

        // Data Extraction Tools
        self.register_tool(ExtractTextTool::new(browser.clone()));
        self.register_tool(ExtractLinksTool::new(browser.clone()));
        self.register_tool(ExtractDataTool::new(browser.clone()));
        self.register_tool(ExtractTableTool::new(browser.clone()));
        self.register_tool(ExtractFormTool::new(browser.clone()));

        // Synchronization Tools
        self.register_tool(WaitForElementTool::new(browser.clone()));
        self.register_tool(WaitForConditionTool::new(browser.clone()));
        self.register_tool(WaitForNavigationTool::new(browser.clone()));
        self.register_tool(WaitForNetworkIdleTool::new(browser.clone()));

        // Memory Tools
        self.register_tool(ScreenshotTool::new(browser.clone()));
        self.register_tool(SessionMemoryTool::new(browser.clone()));
        self.register_tool(GetElementInfoTool::new(browser.clone()));
        self.register_tool(HistoryTrackerTool::new(browser.clone()));
        self.register_tool(PersistentCacheTool::new(browser.clone()));

        // CDP Monitoring Tools
        self.register_tool(NetworkMonitorTool::new(browser.clone()));
        self.register_tool(PerformanceMetricsTool::new(browser.clone()));
        self.register_tool(CDPNetworkIdleTool::new(browser.clone()));

        // Synthetic Test Fixtures
        self.register_tool(CreateTestFixtureTool::new(browser.clone()));

        info!("Registered {} tools across {} categories", 
              self.tools.len(), self.categories.len());
    }

    /// Register a single tool
    pub fn register_tool<T>(&mut self, tool: T) 
    where 
        T: super::traits::Tool + 'static 
    {
        let wrapped = DynamicToolWrapper::new(tool);
        let name = wrapped.name().to_string();
        let category = wrapped.category();
        
        debug!("Registering tool: {} (category: {:?})", name, category);
        
        // Add to tools map
        self.tools.insert(name.clone(), Arc::new(wrapped));
        
        // Add to category index
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Register category with dependency manager for intelligent dependency inference
        tokio::spawn({
            let dependency_manager = self.dependency_manager.clone();
            let tool_name = name;
            let tool_category = category;
            async move {
                dependency_manager.register_tool_category(tool_name, tool_category).await;
            }
        });
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn DynamicTool>> {
        self.tools.get(name).cloned()
    }

    /// Execute a tool by name with JSON input
    pub async fn execute_tool(&self, name: &str, input: Value) -> Result<Value> {
        // Check cache first
        if let Some(cached_result) = self.cache.get(name, &input).await {
            debug!("Cache hit for tool '{}', returning cached result", name);
            return Ok(cached_result);
        }

        let tool = self.get_tool(name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", name))?;
        
        debug!("Executing tool: {} with input: {}", name, input);
        
        let start_time = Instant::now();
        let input_size = input.to_string().len();
        let mut success = false;
        let mut error_message = None;
        let mut output_size = 0;

        // Validate input
        let validation_result = tool.validate_json(&input).await.map_err(|e| {
            error!("Tool '{}' input validation failed: {}", name, e);
            anyhow!("Input validation failed for tool '{}': {}", name, e)
        });

        let result = match validation_result {
            Ok(_) => {
                // Execute tool
                match tool.execute_json(input.clone()).await {
                    Ok(result) => {
                        success = true;
                        output_size = result.to_string().len();
                        debug!("Tool '{}' executed successfully", name);
                        
                        // Cache successful results
                        self.cache.set(name, &input, &result).await;
                        
                        Ok(result)
                    }
                    Err(e) => {
                        error_message = Some(e.to_string());
                        error!("Tool '{}' execution failed: {}", name, e);
                        Err(anyhow!("Tool '{}' execution failed: {}", name, e))
                    }
                }
            }
            Err(e) => {
                error_message = Some(e.to_string());
                Err(e)
            }
        };

        // Record performance metrics
        let execution_time = start_time.elapsed();
        let metric = ToolPerformanceMetric {
            tool_name: name.to_string(),
            execution_time_ms: execution_time.as_millis() as u64,
            success,
            timestamp: std::time::SystemTime::now(),
            input_size_bytes: input_size,
            output_size_bytes: output_size,
            error_message,
        };

        // Add metric to performance history (async)
        if let Ok(mut metrics) = self.performance_metrics.try_write() {
            metrics.push(metric);
            
            // Keep only last 1000 metrics to prevent memory growth
            if metrics.len() > 1000 {
                metrics.drain(0..100); // Remove oldest 100 metrics
            }
        }

        result
    }

    /// Get all available tool names
    pub fn get_tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get tools by category
    pub fn get_tools_by_category(&self, category: ToolCategory) -> Vec<String> {
        self.categories.get(&category).cloned().unwrap_or_default()
    }

    /// Get all categories with their tool counts
    pub fn get_categories(&self) -> HashMap<ToolCategory, usize> {
        self.categories
            .iter()
            .map(|(category, tools)| (*category, tools.len()))
            .collect()
    }

    /// Get metadata for a specific tool
    pub fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.get_tool(name).map(|tool| tool.metadata())
    }

    /// Get metadata for all tools
    pub fn get_all_metadata(&self) -> HashMap<String, ToolMetadata> {
        self.tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.metadata()))
            .collect()
    }

    /// Get summary information about the registry
    pub fn get_summary(&self) -> ToolRegistrySummary {
        let total_tools = self.tools.len();
        let categories = self.get_categories();
        let tool_names = self.get_tool_names();

        ToolRegistrySummary {
            total_tools,
            categories,
            tool_names,
        }
    }

    /// Get performance statistics for a specific tool
    pub async fn get_tool_performance_stats(&self, tool_name: &str) -> Option<ToolPerformanceStats> {
        let metrics = self.performance_metrics.read().await;
        let tool_metrics: Vec<_> = metrics.iter()
            .filter(|m| m.tool_name == tool_name)
            .collect();

        if tool_metrics.is_empty() {
            return None;
        }

        let total_executions = tool_metrics.len() as u64;
        let successful_executions = tool_metrics.iter().filter(|m| m.success).count() as u64;
        let failed_executions = total_executions - successful_executions;

        let execution_times: Vec<u64> = tool_metrics.iter()
            .map(|m| m.execution_time_ms)
            .collect();

        let avg_execution_time_ms = execution_times.iter().sum::<u64>() as f64 / execution_times.len() as f64;
        let min_execution_time_ms = *execution_times.iter().min().unwrap_or(&0);
        let max_execution_time_ms = *execution_times.iter().max().unwrap_or(&0);
        let last_execution = tool_metrics.iter()
            .map(|m| m.timestamp)
            .max();

        Some(ToolPerformanceStats {
            tool_name: tool_name.to_string(),
            total_executions,
            successful_executions,
            failed_executions,
            avg_execution_time_ms,
            min_execution_time_ms,
            max_execution_time_ms,
            last_execution,
        })
    }

    /// Get performance statistics for all tools
    pub async fn get_all_performance_stats(&self) -> HashMap<String, ToolPerformanceStats> {
        let mut stats = HashMap::new();
        let tool_names = self.get_tool_names();

        for tool_name in tool_names {
            if let Some(tool_stats) = self.get_tool_performance_stats(&tool_name).await {
                stats.insert(tool_name, tool_stats);
            }
        }

        stats
    }

    /// Get recent performance metrics (last N executions)
    pub async fn get_recent_metrics(&self, limit: usize) -> Vec<ToolPerformanceMetric> {
        let metrics = self.performance_metrics.read().await;
        let start_index = if metrics.len() > limit {
            metrics.len() - limit
        } else {
            0
        };
        metrics[start_index..].to_vec()
    }

    /// Clear performance metrics history
    pub async fn clear_performance_metrics(&self) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.clear();
        info!("Performance metrics cleared");
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> super::cache::CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all cache entries
    pub async fn clear_cache(&self) {
        self.cache.clear_all().await;
    }

    /// Clear cache for a specific tool
    pub async fn clear_tool_cache(&self, tool_name: &str) {
        self.cache.clear_tool_cache(tool_name).await;
    }

    /// Handle navigation event - invalidate navigation-sensitive caches
    pub async fn on_navigation(&self, new_url: &str) {
        self.cache.on_navigation(new_url).await;
    }

    /// Set cache configuration for a specific tool
    pub async fn set_tool_cache_config(&self, tool_name: &str, config: super::cache::CacheConfig) {
        self.cache.set_tool_config(tool_name, config).await;
    }

    /// Create execution plan for multiple tools considering dependencies
    pub async fn create_execution_plan(&self, tool_names: Vec<String>) -> Result<ExecutionPlan> {
        self.dependency_manager.create_execution_plan(tool_names).await
    }

    /// Register dependencies for a tool
    pub async fn register_tool_dependencies(&self, tool_name: String, dependencies: Vec<super::dependencies::ToolDependency>) {
        self.dependency_manager.register_dependencies(tool_name, dependencies).await;
    }

    /// Get dependency information for a tool
    pub async fn get_tool_dependencies(&self, tool_name: &str) -> Vec<super::dependencies::ToolDependency> {
        self.dependency_manager.get_tool_dependencies(tool_name).await
    }

    /// Get tools that depend on a specific tool
    pub async fn get_dependent_tools(&self, tool_name: &str) -> Vec<String> {
        self.dependency_manager.get_dependent_tools(tool_name).await
    }

    /// Execute multiple tools with dependency resolution
    pub async fn execute_tools_with_dependencies(&self, tool_names: Vec<String>) -> Result<ExecutionContext> {
        // Create execution plan
        let plan = self.create_execution_plan(tool_names).await?;
        let mut context = ExecutionContext {
            completed_tools: HashMap::new(),
            failed_tools: std::collections::HashSet::new(),
            execution_times: HashMap::new(),
            stage_results: Vec::new(),
        };

        info!("Executing {} stages with dependency resolution", plan.stages.len());

        // Execute each stage
        for (stage_idx, stage) in plan.stages.iter().enumerate() {
            let stage_start = Instant::now();
            let mut stage_completed = Vec::new();
            let mut stage_failed = Vec::new();

            debug!("Executing stage {} with {} tools (parallel: {})", 
                   stage_idx, stage.tools.len(), stage.can_run_parallel);

            if stage.can_run_parallel && stage.tools.len() > 1 {
                // Execute tools in parallel
                let mut handles = Vec::new();

                for tool_name in &stage.tools {
                    let registry_clone = self.clone_registry_for_execution();
                    let tool_name_clone = tool_name.clone();
                    let input = serde_json::json!({}); // Default empty input - could be parameterized

                    let handle = tokio::spawn(async move {
                        let start = Instant::now();
                        let result = registry_clone.execute_tool(&tool_name_clone, input).await;
                        let duration = start.elapsed().as_millis() as u64;
                        (tool_name_clone, result, duration)
                    });
                    handles.push(handle);
                }

                // Collect results
                for handle in handles {
                    match handle.await {
                        Ok((tool_name, result, duration)) => {
                            context.execution_times.insert(tool_name.clone(), duration);
                            match result {
                                Ok(value) => {
                                    context.completed_tools.insert(tool_name.clone(), value);
                                    stage_completed.push(tool_name);
                                }
                                Err(e) => {
                                    error!("Tool '{}' failed: {}", tool_name, e);
                                    context.failed_tools.insert(tool_name.clone());
                                    stage_failed.push(tool_name);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to execute tool task: {}", e);
                        }
                    }
                }
            } else {
                // Execute tools sequentially
                for tool_name in &stage.tools {
                    let start = Instant::now();
                    let input = serde_json::json!({}); // Default empty input
                    
                    match self.execute_tool(tool_name, input).await {
                        Ok(value) => {
                            let duration = start.elapsed().as_millis() as u64;
                            context.execution_times.insert(tool_name.clone(), duration);
                            context.completed_tools.insert(tool_name.clone(), value);
                            stage_completed.push(tool_name.clone());
                        }
                        Err(e) => {
                            let duration = start.elapsed().as_millis() as u64;
                            context.execution_times.insert(tool_name.clone(), duration);
                            error!("Tool '{}' failed: {}", tool_name, e);
                            context.failed_tools.insert(tool_name.clone());
                            stage_failed.push(tool_name.clone());
                        }
                    }
                }
            }

            let stage_duration = stage_start.elapsed().as_millis() as u64;
            context.stage_results.push(super::dependencies::StageResult {
                stage_number: stage_idx,
                completed_tools: stage_completed,
                failed_tools: stage_failed,
                stage_duration_ms: stage_duration,
            });

            // Check if we should continue if some tools failed
            if !context.failed_tools.is_empty() {
                warn!("Stage {} completed with {} failed tools", stage_idx, context.failed_tools.len());
            }
        }

        // Record execution for analytics
        self.dependency_manager.record_execution(context.clone()).await;

        info!("Dependency execution completed: {} successful, {} failed tools", 
              context.completed_tools.len(), context.failed_tools.len());

        Ok(context)
    }

    /// Get dependency execution statistics
    pub async fn get_dependency_stats(&self) -> ExecutionStats {
        self.dependency_manager.get_execution_stats().await
    }

    /// Helper method to create a registry clone for parallel execution
    /// Note: This is a simplified approach - in production you might want more sophisticated cloning
    fn clone_registry_for_execution(&self) -> Self {
        // For now, we'll create a reference-based clone
        // This works because all our internal data is Arc<> wrapped
        Self {
            tools: self.tools.clone(),
            categories: self.categories.clone(),
            performance_metrics: self.performance_metrics.clone(),
            cache: self.cache.clone(),
            dependency_manager: self.dependency_manager.clone(),
        }
    }

    /// Get performance metrics for a specific tool (last N executions)
    pub async fn get_tool_metrics(&self, tool_name: &str, limit: Option<usize>) -> Vec<ToolPerformanceMetric> {
        let metrics = self.performance_metrics.read().await;
        let tool_metrics: Vec<_> = metrics.iter()
            .filter(|m| m.tool_name == tool_name)
            .cloned()
            .collect();

        match limit {
            Some(n) => {
                let start_index = if tool_metrics.len() > n {
                    tool_metrics.len() - n
                } else {
                    0
                };
                tool_metrics[start_index..].to_vec()
            }
            None => tool_metrics,
        }
    }

    /// Validate that all tools are properly registered
    pub async fn validate_registry(&self) -> Result<RegistryValidationReport> {
        let mut report = RegistryValidationReport {
            total_tools: self.tools.len(),
            valid_tools: 0,
            invalid_tools: Vec::new(),
            missing_metadata: Vec::new(),
            validation_errors: Vec::new(),
        };

        for (name, tool) in &self.tools {
            // Check metadata
            let metadata = tool.metadata();
            if metadata.name.is_empty() || metadata.description.is_empty() {
                report.missing_metadata.push(name.clone());
                continue;
            }

            // Try to validate with empty input (should fail gracefully)
            match tool.validate_json(&Value::Null).await {
                Ok(_) => {
                    // This might be unexpected - tools should usually require some input
                    debug!("Tool '{}' accepts null input", name);
                }
                Err(_) => {
                    // Expected for most tools
                    debug!("Tool '{}' properly rejects invalid input", name);
                }
            }

            report.valid_tools += 1;
        }

        Ok(report)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        // Create registry without tools (for testing or manual registration)
        Self {
            tools: HashMap::new(),
            categories: HashMap::new(),
            performance_metrics: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(ToolCache::new()),
            dependency_manager: Arc::new(DependencyManager::new()),
        }
    }
}

/// Summary information about the tool registry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolRegistrySummary {
    pub total_tools: usize,
    pub categories: HashMap<ToolCategory, usize>,
    pub tool_names: Vec<String>,
}

/// Validation report for the tool registry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryValidationReport {
    pub total_tools: usize,
    pub valid_tools: usize,
    pub invalid_tools: Vec<String>,
    pub missing_metadata: Vec<String>,
    pub validation_errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::browser::Browser;

    #[tokio::test]
    async fn test_registry_creation() {
        // This test would require a mock browser
        // For now, test the default registry
        let registry = ToolRegistry::default();
        assert_eq!(registry.get_tool_names().len(), 0);
    }

    #[tokio::test]
    async fn test_registry_summary() {
        let registry = ToolRegistry::default();
        let summary = registry.get_summary();
        assert_eq!(summary.total_tools, 0);
        assert_eq!(summary.tool_names.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_retrieval() {
        let registry = ToolRegistry::default();
        assert!(registry.get_tool("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_registry_validation() {
        let registry = ToolRegistry::default();
        let report = registry.validate_registry().await.unwrap();
        assert_eq!(report.total_tools, 0);
        assert_eq!(report.valid_tools, 0);
    }
}