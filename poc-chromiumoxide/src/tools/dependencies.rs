use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use serde_json::Value;
use tokio::sync::RwLock;

use super::traits::ToolCategory;

/// Dependency relationship between tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Tool must complete successfully before dependent can run
    Required,
    /// Tool should run before dependent, but failure is acceptable
    Preferred,
    /// Tools cannot run simultaneously (mutual exclusion)
    Exclusive,
    /// Tool provides context/data that enhances dependent's execution
    Contextual,
}

/// Tool dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    pub tool_name: String,
    pub dependency_type: DependencyType,
    pub condition: Option<DependencyCondition>,
    pub timeout_seconds: Option<u64>,
    pub retry_attempts: Option<u32>,
}

/// Conditions that must be met for dependency to be satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyCondition {
    /// Dependency result must match expected value
    ResultEquals(Value),
    /// Dependency result must contain specific field
    ResultContains(String),
    /// Dependency must complete within time limit
    CompletedWithin(u64),
    /// Custom condition expression (future enhancement)
    Expression(String),
}

/// Execution plan for tools with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub stages: Vec<ExecutionStage>,
    pub estimated_duration_ms: u64,
    pub parallel_opportunities: usize,
    pub dependency_graph: HashMap<String, Vec<String>>,
}

/// Single stage in execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStage {
    pub stage_number: usize,
    pub tools: Vec<String>,
    pub can_run_parallel: bool,
    pub estimated_duration_ms: u64,
    pub dependencies_satisfied: Vec<String>,
}

/// Tool execution context with dependency results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub completed_tools: HashMap<String, Value>,
    pub failed_tools: HashSet<String>,
    pub execution_times: HashMap<String, u64>,
    pub stage_results: Vec<StageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub stage_number: usize,
    pub completed_tools: Vec<String>,
    pub failed_tools: Vec<String>,
    pub stage_duration_ms: u64,
}

/// Tool dependency manager
pub struct DependencyManager {
    dependencies: Arc<RwLock<HashMap<String, Vec<ToolDependency>>>>,
    tool_categories: Arc<RwLock<HashMap<String, ToolCategory>>>,
    execution_history: Arc<RwLock<Vec<ExecutionContext>>>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            tool_categories: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register tool dependencies
    pub async fn register_dependencies(&self, tool_name: String, dependencies: Vec<ToolDependency>) {
        let mut deps = self.dependencies.write().await;
        deps.insert(tool_name.clone(), dependencies.clone());
        info!("Registered {} dependencies for tool '{}'", dependencies.len(), tool_name);
        
        for dep in dependencies {
            debug!("  - {:?} dependency on '{}' ({:?})", 
                   dep.dependency_type, dep.tool_name, dep.condition);
        }
    }

    /// Register tool category for intelligent dependency inference
    pub async fn register_tool_category(&self, tool_name: String, category: ToolCategory) {
        let mut categories = self.tool_categories.write().await;
        categories.insert(tool_name, category);
    }

    /// Create execution plan for multiple tools considering dependencies
    pub async fn create_execution_plan(&self, tool_names: Vec<String>) -> Result<ExecutionPlan> {
        let dependencies = self.dependencies.read().await;
        let categories = self.tool_categories.read().await;
        
        // Build dependency graph
        let mut graph = HashMap::new();
        let mut all_tools = HashSet::new();
        
        for tool_name in &tool_names {
            all_tools.insert(tool_name.clone());
            graph.insert(tool_name.clone(), Vec::new());
            
            if let Some(tool_deps) = dependencies.get(tool_name) {
                for dep in tool_deps {
                    if dep.dependency_type == DependencyType::Required || 
                       dep.dependency_type == DependencyType::Preferred {
                        graph.get_mut(tool_name).unwrap().push(dep.tool_name.clone());
                        all_tools.insert(dep.tool_name.clone());
                    }
                }
            }
        }

        // Add intelligent category-based dependencies
        self.infer_category_dependencies(&mut graph, &all_tools, &categories).await;

        // Detect cycles
        if let Some(cycle) = self.detect_cycles(&graph) {
            return Err(anyhow!("Circular dependency detected: {}", cycle.join(" -> ")));
        }

        // Topological sort to create execution stages
        let stages = self.create_execution_stages(&graph, &all_tools).await?;
        
        // Calculate timing estimates
        let estimated_duration = self.estimate_execution_time(&stages).await;
        let parallel_opportunities = stages.iter().filter(|s| s.can_run_parallel).count();

        Ok(ExecutionPlan {
            stages,
            estimated_duration_ms: estimated_duration,
            parallel_opportunities,
            dependency_graph: graph,
        })
    }

    /// Infer dependencies based on tool categories and common patterns
    async fn infer_category_dependencies(
        &self,
        graph: &mut HashMap<String, Vec<String>>,
        all_tools: &HashSet<String>,
        categories: &HashMap<String, ToolCategory>,
    ) {
        let navigation_tools: HashSet<_> = all_tools.iter()
            .filter(|tool| categories.get(*tool) == Some(&ToolCategory::Navigation))
            .collect();

        let interaction_tools: HashSet<_> = all_tools.iter()
            .filter(|tool| categories.get(*tool) == Some(&ToolCategory::Interaction))
            .collect();

        let extraction_tools: HashSet<_> = all_tools.iter()
            .filter(|tool| categories.get(*tool) == Some(&ToolCategory::DataExtraction))
            .collect();

        let memory_tools: HashSet<_> = all_tools.iter()
            .filter(|tool| categories.get(*tool) == Some(&ToolCategory::Memory))
            .collect();

        // Navigation tools should run before interaction tools
        for interaction_tool in &interaction_tools {
            for navigation_tool in &navigation_tools {
                if !graph.get(*interaction_tool).unwrap().contains(*navigation_tool) {
                    graph.get_mut(*interaction_tool).unwrap().push((*navigation_tool).clone());
                }
            }
        }

        // Interaction tools should run before extraction tools
        for extraction_tool in &extraction_tools {
            for interaction_tool in &interaction_tools {
                if !graph.get(*extraction_tool).unwrap().contains(*interaction_tool) {
                    graph.get_mut(*extraction_tool).unwrap().push((*interaction_tool).clone());
                }
            }
        }

        // Memory tools (like screenshots) can run after any stage
        // No automatic dependencies added for memory tools

        debug!("Inferred category-based dependencies: {} navigation, {} interaction, {} extraction, {} memory tools",
               navigation_tools.len(), interaction_tools.len(), extraction_tools.len(), memory_tools.len());
    }

    /// Detect circular dependencies in the graph
    fn detect_cycles(&self, graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) = self.dfs_cycle_detection(node, graph, &mut visited, &mut rec_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs_cycle_detection(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.dfs_cycle_detection(neighbor, graph, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found cycle, extract it from path
                    let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.to_string()); // Close the cycle
                    return Some(cycle);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// Create execution stages using topological sort
    async fn create_execution_stages(
        &self,
        graph: &HashMap<String, Vec<String>>,
        all_tools: &HashSet<String>,
    ) -> Result<Vec<ExecutionStage>> {
        let mut in_degree = HashMap::new();
        let mut stages = Vec::new();

        // Calculate in-degrees
        for tool in all_tools {
            in_degree.insert(tool.clone(), 0);
        }
        
        for (_tool, dependencies) in graph {
            for dep in dependencies {
                *in_degree.get_mut(dep).unwrap() += 1;
            }
        }

        let mut stage_number = 0;
        
        while !in_degree.is_empty() {
            // Find all nodes with in-degree 0
            let ready_tools: Vec<String> = in_degree.iter()
                .filter(|(_, &degree)| degree == 0)
                .map(|(tool, _)| tool.clone())
                .collect();

            if ready_tools.is_empty() {
                return Err(anyhow!("Unable to resolve dependencies - possible circular reference"));
            }

            // Remove ready tools from in_degree
            for tool in &ready_tools {
                in_degree.remove(tool);
            }

            // Update in-degrees for dependent tools
            for tool in &ready_tools {
                if let Some(dependencies) = graph.get(tool) {
                    for dep in dependencies {
                        if let Some(degree) = in_degree.get_mut(dep) {
                            *degree -= 1;
                        }
                    }
                }
            }

            // Create execution stage
            let can_run_parallel = ready_tools.len() > 1;
            let estimated_duration = if can_run_parallel { 2000 } else { ready_tools.len() as u64 * 1000 };

            stages.push(ExecutionStage {
                stage_number,
                tools: ready_tools.clone(),
                can_run_parallel,
                estimated_duration_ms: estimated_duration,
                dependencies_satisfied: Vec::new(), // Will be populated during execution
            });

            stage_number += 1;
        }

        info!("Created execution plan with {} stages", stages.len());
        for (i, stage) in stages.iter().enumerate() {
            debug!("Stage {}: {} tools (parallel: {}): [{}]", 
                   i, stage.tools.len(), stage.can_run_parallel, stage.tools.join(", "));
        }

        Ok(stages)
    }

    /// Estimate total execution time for stages
    async fn estimate_execution_time(&self, stages: &[ExecutionStage]) -> u64 {
        // Simple estimation - sum of stage times
        stages.iter().map(|s| s.estimated_duration_ms).sum()
    }

    /// Check if dependency condition is satisfied
    pub fn check_dependency_condition(
        &self,
        condition: &DependencyCondition,
        result: &Value,
        execution_time: u64,
    ) -> bool {
        match condition {
            DependencyCondition::ResultEquals(expected) => result == expected,
            DependencyCondition::ResultContains(field) => {
                result.as_object().map_or(false, |obj| obj.contains_key(field))
            },
            DependencyCondition::CompletedWithin(max_time) => execution_time <= *max_time * 1000,
            DependencyCondition::Expression(_expr) => {
                // Future enhancement: implement expression evaluation
                warn!("Expression conditions not yet implemented");
                true
            }
        }
    }

    /// Get dependency information for a tool
    pub async fn get_tool_dependencies(&self, tool_name: &str) -> Vec<ToolDependency> {
        let dependencies = self.dependencies.read().await;
        dependencies.get(tool_name).cloned().unwrap_or_default()
    }

    /// Get tools that depend on a specific tool
    pub async fn get_dependent_tools(&self, tool_name: &str) -> Vec<String> {
        let dependencies = self.dependencies.read().await;
        let mut dependents = Vec::new();

        for (tool, deps) in dependencies.iter() {
            for dep in deps {
                if dep.tool_name == tool_name {
                    dependents.push(tool.clone());
                }
            }
        }

        dependents
    }

    /// Record execution result for dependency tracking
    pub async fn record_execution(&self, context: ExecutionContext) {
        let mut history = self.execution_history.write().await;
        history.push(context);

        // Keep only last 100 execution contexts
        if history.len() > 100 {
            history.drain(0..20);
        }
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        let history = self.execution_history.read().await;
        let dependencies = self.dependencies.read().await;

        let total_executions = history.len();
        let avg_stages = if total_executions > 0 {
            history.iter().map(|ctx| ctx.stage_results.len()).sum::<usize>() as f64 / total_executions as f64
        } else {
            0.0
        };

        let total_tools_with_deps = dependencies.len();
        let avg_dependencies = if total_tools_with_deps > 0 {
            dependencies.values().map(|deps| deps.len()).sum::<usize>() as f64 / total_tools_with_deps as f64
        } else {
            0.0
        };

        ExecutionStats {
            total_executions,
            avg_stages_per_execution: avg_stages,
            total_tools_with_dependencies: total_tools_with_deps,
            avg_dependencies_per_tool: avg_dependencies,
            successful_executions: history.iter().filter(|ctx| ctx.failed_tools.is_empty()).count(),
            failed_executions: history.iter().filter(|ctx| !ctx.failed_tools.is_empty()).count(),
        }
    }

    /// Clear all dependencies (for testing/reset)
    pub async fn clear_dependencies(&self) {
        let mut dependencies = self.dependencies.write().await;
        let mut categories = self.tool_categories.write().await;
        let mut history = self.execution_history.write().await;
        
        dependencies.clear();
        categories.clear();
        history.clear();
        
        info!("Cleared all dependency information");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_executions: usize,
    pub avg_stages_per_execution: f64,
    pub total_tools_with_dependencies: usize,
    pub avg_dependencies_per_tool: f64,
    pub successful_executions: usize,
    pub failed_executions: usize,
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-defined dependency patterns for common tool combinations
pub struct DependencyPatterns;

impl DependencyPatterns {
    /// Create dependencies for web form automation workflow
    pub fn web_form_workflow() -> HashMap<String, Vec<ToolDependency>> {
        let mut patterns = HashMap::new();

        // Navigate must complete before any interaction
        patterns.insert("click".to_string(), vec![
            ToolDependency {
                tool_name: "navigate".to_string(),
                dependency_type: DependencyType::Required,
                condition: Some(DependencyCondition::ResultContains("url".to_string())),
                timeout_seconds: Some(30),
                retry_attempts: Some(3),
            }
        ]);

        patterns.insert("type_text".to_string(), vec![
            ToolDependency {
                tool_name: "navigate".to_string(),
                dependency_type: DependencyType::Required,
                condition: None,
                timeout_seconds: Some(30),
                retry_attempts: Some(3),
            }
        ]);

        // Screenshot can run after page load
        patterns.insert("screenshot".to_string(), vec![
            ToolDependency {
                tool_name: "navigate".to_string(),
                dependency_type: DependencyType::Preferred,
                condition: None,
                timeout_seconds: Some(15),
                retry_attempts: Some(1),
            }
        ]);

        patterns
    }

    /// Create dependencies for data extraction workflow
    pub fn data_extraction_workflow() -> HashMap<String, Vec<ToolDependency>> {
        let mut patterns = HashMap::new();

        // Wait for elements before extracting
        patterns.insert("extract_text".to_string(), vec![
            ToolDependency {
                tool_name: "wait_for_element".to_string(),
                dependency_type: DependencyType::Required,
                condition: None,
                timeout_seconds: Some(10),
                retry_attempts: Some(2),
            }
        ]);

        patterns.insert("extract_links".to_string(), vec![
            ToolDependency {
                tool_name: "wait_for_element".to_string(),
                dependency_type: DependencyType::Preferred,
                condition: None,
                timeout_seconds: Some(10),
                retry_attempts: Some(2),
            }
        ]);

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dependency_manager_creation() {
        let manager = DependencyManager::new();
        let stats = manager.get_execution_stats().await;
        assert_eq!(stats.total_executions, 0);
    }

    #[tokio::test]
    async fn test_simple_execution_plan() {
        let manager = DependencyManager::new();
        
        manager.register_dependencies("click".to_string(), vec![
            ToolDependency {
                tool_name: "navigate".to_string(),
                dependency_type: DependencyType::Required,
                condition: None,
                timeout_seconds: Some(30),
                retry_attempts: Some(3),
            }
        ]).await;

        let plan = manager.create_execution_plan(vec![
            "navigate".to_string(),
            "click".to_string(),
        ]).await.unwrap();

        assert_eq!(plan.stages.len(), 2);
        assert!(plan.stages[0].tools.contains(&"navigate".to_string()));
        assert!(plan.stages[1].tools.contains(&"click".to_string()));
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let manager = DependencyManager::new();
        
        manager.register_dependencies("tool_a".to_string(), vec![
            ToolDependency {
                tool_name: "tool_b".to_string(),
                dependency_type: DependencyType::Required,
                condition: None,
                timeout_seconds: Some(30),
                retry_attempts: Some(3),
            }
        ]).await;

        manager.register_dependencies("tool_b".to_string(), vec![
            ToolDependency {
                tool_name: "tool_a".to_string(),
                dependency_type: DependencyType::Required,
                condition: None,
                timeout_seconds: Some(30),
                retry_attempts: Some(3),
            }
        ]).await;

        let result = manager.create_execution_plan(vec![
            "tool_a".to_string(),
            "tool_b".to_string(),
        ]).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }

    #[tokio::test]
    async fn test_dependency_condition_check() {
        let manager = DependencyManager::new();
        
        let condition = DependencyCondition::ResultContains("success".to_string());
        let result = serde_json::json!({"success": true, "url": "https://example.com"});
        
        assert!(manager.check_dependency_condition(&condition, &result, 1000));
        
        let result2 = serde_json::json!({"error": "failed"});
        assert!(!manager.check_dependency_condition(&condition, &result2, 1000));
    }
}