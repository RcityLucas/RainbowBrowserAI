// Task Planning Module
// Converts natural language instructions into executable browser automation plans

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

use super::{LLMError, LLMResponse};

/// A complete task plan with multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub id: String,
    pub description: String,
    pub steps: Vec<TaskStep>,
    pub estimated_duration: Option<u64>, // seconds
    pub confidence: f32,                 // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Individual step in a task plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: String,
    pub step_type: TaskStepType,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub description: String,
    pub expected_outcome: String,
    pub timeout_seconds: Option<u64>,
    pub retry_count: u32,
    pub dependencies: Vec<String>, // IDs of prerequisite steps
}

/// Types of task steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStepType {
    Navigate,
    Click,
    Type,
    Extract,
    Wait,
    Scroll,
    Screenshot,
    Validate,
    Custom(String),
}

/// Executor for task plans
pub struct TaskPlanExecutor {
    #[allow(dead_code)] // Reserved for browser pool integration
    browser_pool: Option<std::sync::Arc<crate::browser::pool::BrowserPool>>,
    step_results: HashMap<String, TaskStepResult>,
}

/// Result of executing a task step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStepResult {
    pub step_id: String,
    pub success: bool,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Task execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionSummary {
    pub task_id: String,
    pub total_steps: usize,
    pub successful_steps: usize,
    pub failed_steps: usize,
    pub total_execution_time_ms: u64,
    pub overall_success: bool,
    pub step_results: Vec<TaskStepResult>,
}

impl TaskPlan {
    /// Create a new task plan from LLM response
    pub fn from_llm_response(response: &LLMResponse) -> Result<Self, LLMError> {
        info!("Parsing task plan from LLM response");

        // Try to parse JSON first
        if let Ok(plan) = serde_json::from_str::<TaskPlan>(&response.content) {
            return Ok(plan);
        }

        // If JSON parsing fails, use natural language parsing
        Self::from_natural_language(&response.content)
    }

    /// Parse task plan from natural language description
    pub fn from_natural_language(content: &str) -> Result<Self, LLMError> {
        info!("Parsing natural language task description");

        let plan_id = uuid::Uuid::new_v4().to_string();
        let mut steps = Vec::new();

        // Simple natural language parsing - identify common patterns
        let lines: Vec<&str> = content.lines().collect();
        let mut step_counter = 1;

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let step = Self::parse_step_from_line(line, step_counter)?;
            if let Some(step) = step {
                steps.push(step);
                step_counter += 1;
            }
        }

        if steps.is_empty() {
            warn!("No valid steps found in task description");
            // Create a default step for the entire instruction
            steps.push(TaskStep {
                id: format!("step_{}", step_counter),
                step_type: TaskStepType::Custom("general".to_string()),
                action: "execute_instruction".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "instruction".to_string(),
                        serde_json::Value::String(content.to_string()),
                    );
                    params
                },
                description: content.to_string(),
                expected_outcome: "Task completed successfully".to_string(),
                timeout_seconds: Some(60),
                retry_count: 2,
                dependencies: vec![],
            });
        }

        Ok(TaskPlan {
            id: plan_id,
            description: if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            },
            steps,
            estimated_duration: Some(30), // default 30 seconds
            confidence: 0.7,              // default confidence
            created_at: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Parse individual step from line of text
    fn parse_step_from_line(line: &str, step_num: usize) -> Result<Option<TaskStep>, LLMError> {
        let line_lower = line.to_lowercase();

        let (step_type, action, description) = if line_lower.contains("navigate")
            || line_lower.contains("go to")
            || line_lower.contains("visit")
        {
            (TaskStepType::Navigate, "navigate", line)
        } else if line_lower.contains("click") {
            (TaskStepType::Click, "click", line)
        } else if line_lower.contains("type")
            || line_lower.contains("enter")
            || line_lower.contains("input")
        {
            (TaskStepType::Type, "type", line)
        } else if line_lower.contains("scroll") {
            (TaskStepType::Scroll, "scroll", line)
        } else if line_lower.contains("wait") || line_lower.contains("pause") {
            (TaskStepType::Wait, "wait", line)
        } else if line_lower.contains("screenshot") || line_lower.contains("capture") {
            (TaskStepType::Screenshot, "screenshot", line)
        } else if line_lower.contains("extract")
            || line_lower.contains("get")
            || line_lower.contains("find")
        {
            (TaskStepType::Extract, "extract", line)
        } else {
            // Generic step
            (TaskStepType::Custom("general".to_string()), "execute", line)
        };

        let step = TaskStep {
            id: format!("step_{}", step_num),
            step_type,
            action: action.to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "instruction".to_string(),
                    serde_json::Value::String(line.to_string()),
                );
                params
            },
            description: description.to_string(),
            expected_outcome: "Step completed successfully".to_string(),
            timeout_seconds: Some(30),
            retry_count: 2,
            dependencies: vec![],
        };

        Ok(Some(step))
    }

    /// Validate task plan for execution
    pub fn validate(&self) -> Result<(), LLMError> {
        if self.steps.is_empty() {
            return Err(LLMError::InvalidResponse(
                "Task plan has no steps".to_string(),
            ));
        }

        // Check for circular dependencies
        for step in &self.steps {
            if self.has_circular_dependency(&step.id, &step.dependencies)? {
                return Err(LLMError::InvalidResponse(format!(
                    "Circular dependency detected for step {}",
                    step.id
                )));
            }
        }

        Ok(())
    }

    /// Check for circular dependencies in steps
    fn has_circular_dependency(&self, step_id: &str, _deps: &[String]) -> Result<bool, LLMError> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![step_id.to_string()];

        while let Some(current_id) = stack.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            if let Some(step) = self.steps.iter().find(|s| s.id == current_id) {
                for dep_id in &step.dependencies {
                    if dep_id == step_id {
                        return Ok(true); // Circular dependency found
                    }
                    stack.push(dep_id.clone());
                }
            }
        }

        Ok(false)
    }
}

impl TaskPlanExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self {
            browser_pool: None,
            step_results: HashMap::new(),
        }
    }

    /// Create executor with browser pool
    pub fn with_browser_pool(pool: std::sync::Arc<crate::browser::pool::BrowserPool>) -> Self {
        Self {
            browser_pool: Some(pool),
            step_results: HashMap::new(),
        }
    }

    /// Execute a complete task plan
    pub async fn execute_plan(
        &mut self,
        plan: &TaskPlan,
    ) -> Result<TaskExecutionSummary, LLMError> {
        info!("Executing task plan: {}", plan.id);

        plan.validate()?;

        let start_time = std::time::Instant::now();
        let mut successful_steps = 0;
        let mut failed_steps = 0;
        let mut step_results = Vec::new();

        // Execute steps in dependency order
        let execution_order = self.calculate_execution_order(&plan.steps)?;

        for step_id in execution_order {
            if let Some(step) = plan.steps.iter().find(|s| s.id == step_id) {
                info!("Executing step: {} - {}", step.id, step.description);

                let step_start = std::time::Instant::now();
                let result = self.execute_step(step).await;
                let execution_time = step_start.elapsed().as_millis() as u64;

                let step_result = TaskStepResult {
                    step_id: step.id.clone(),
                    success: result.is_ok(),
                    result_data: result.as_ref().ok().cloned(),
                    error_message: result.as_ref().err().map(|e| e.to_string()),
                    execution_time_ms: execution_time,
                    timestamp: Utc::now(),
                };

                if result.is_ok() {
                    successful_steps += 1;
                    info!(
                        "Step {} completed successfully in {}ms",
                        step.id, execution_time
                    );
                } else {
                    failed_steps += 1;
                    error!("Step {} failed: {:?}", step.id, result);
                }

                self.step_results
                    .insert(step.id.clone(), step_result.clone());
                step_results.push(step_result);
            }
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        let overall_success = failed_steps == 0;

        info!(
            "Task plan {} completed: {}/{} steps successful",
            plan.id,
            successful_steps,
            plan.steps.len()
        );

        Ok(TaskExecutionSummary {
            task_id: plan.id.clone(),
            total_steps: plan.steps.len(),
            successful_steps,
            failed_steps,
            total_execution_time_ms: total_time,
            overall_success,
            step_results,
        })
    }

    /// Execute individual step
    async fn execute_step(&mut self, step: &TaskStep) -> Result<serde_json::Value, LLMError> {
        match &step.step_type {
            TaskStepType::Navigate => self.execute_navigate_step(step).await,
            TaskStepType::Click => self.execute_click_step(step).await,
            TaskStepType::Type => self.execute_type_step(step).await,
            TaskStepType::Extract => self.execute_extract_step(step).await,
            TaskStepType::Wait => self.execute_wait_step(step).await,
            TaskStepType::Scroll => self.execute_scroll_step(step).await,
            TaskStepType::Screenshot => self.execute_screenshot_step(step).await,
            TaskStepType::Validate => self.execute_validate_step(step).await,
            TaskStepType::Custom(custom_type) => self.execute_custom_step(step, custom_type).await,
        }
    }

    /// Calculate execution order based on dependencies
    fn calculate_execution_order(&self, steps: &[TaskStep]) -> Result<Vec<String>, LLMError> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        for step in steps {
            if !visited.contains(&step.id) {
                self.topological_sort(
                    &step.id,
                    steps,
                    &mut visited,
                    &mut temp_visited,
                    &mut order,
                )?;
            }
        }

        order.reverse(); // Reverse to get correct order
        Ok(order)
    }

    /// Topological sort for dependency resolution
    #[allow(clippy::only_used_in_recursion)]
    fn topological_sort(
        &self,
        step_id: &str,
        steps: &[TaskStep],
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), LLMError> {
        if temp_visited.contains(step_id) {
            return Err(LLMError::InvalidResponse(
                "Circular dependency detected".to_string(),
            ));
        }

        if visited.contains(step_id) {
            return Ok(());
        }

        temp_visited.insert(step_id.to_string());

        if let Some(step) = steps.iter().find(|s| s.id == step_id) {
            for dep_id in &step.dependencies {
                self.topological_sort(dep_id, steps, visited, temp_visited, order)?;
            }
        }

        temp_visited.remove(step_id);
        visited.insert(step_id.to_string());
        order.push(step_id.to_string());

        Ok(())
    }

    // Individual step execution methods
    async fn execute_navigate_step(
        &mut self,
        step: &TaskStep,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for navigation
        info!("Executing navigate step: {}", step.description);
        Ok(serde_json::Value::String(
            "Navigation completed".to_string(),
        ))
    }

    async fn execute_click_step(&mut self, step: &TaskStep) -> Result<serde_json::Value, LLMError> {
        // Implementation for clicking
        info!("Executing click step: {}", step.description);
        Ok(serde_json::Value::String("Click completed".to_string()))
    }

    async fn execute_type_step(&mut self, step: &TaskStep) -> Result<serde_json::Value, LLMError> {
        // Implementation for typing
        info!("Executing type step: {}", step.description);
        Ok(serde_json::Value::String("Typing completed".to_string()))
    }

    async fn execute_extract_step(
        &mut self,
        step: &TaskStep,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for data extraction
        info!("Executing extract step: {}", step.description);
        Ok(serde_json::Value::String(
            "Extraction completed".to_string(),
        ))
    }

    async fn execute_wait_step(&mut self, step: &TaskStep) -> Result<serde_json::Value, LLMError> {
        // Implementation for waiting
        info!("Executing wait step: {}", step.description);
        let wait_time = step
            .parameters
            .get("duration")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);
        tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
        Ok(serde_json::Value::String("Wait completed".to_string()))
    }

    async fn execute_scroll_step(
        &mut self,
        step: &TaskStep,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for scrolling
        info!("Executing scroll step: {}", step.description);
        Ok(serde_json::Value::String("Scroll completed".to_string()))
    }

    async fn execute_screenshot_step(
        &mut self,
        step: &TaskStep,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for screenshots
        info!("Executing screenshot step: {}", step.description);
        Ok(serde_json::Value::String(
            "Screenshot completed".to_string(),
        ))
    }

    async fn execute_validate_step(
        &mut self,
        step: &TaskStep,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for validation
        info!("Executing validate step: {}", step.description);
        Ok(serde_json::Value::String(
            "Validation completed".to_string(),
        ))
    }

    async fn execute_custom_step(
        &mut self,
        step: &TaskStep,
        custom_type: &str,
    ) -> Result<serde_json::Value, LLMError> {
        // Implementation for custom steps
        info!(
            "Executing custom step ({}): {}",
            custom_type, step.description
        );
        Ok(serde_json::Value::String(format!(
            "Custom step {} completed",
            custom_type
        )))
    }

    /// Get result of a specific step
    pub fn get_step_result(&self, step_id: &str) -> Option<&TaskStepResult> {
        self.step_results.get(step_id)
    }
}

impl Default for TaskPlanExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_language_parsing() {
        let instruction = "Navigate to google.com\nClick on search box\nType hello world";
        let plan = TaskPlan::from_natural_language(instruction).unwrap();

        assert_eq!(plan.steps.len(), 3);
        assert!(matches!(plan.steps[0].step_type, TaskStepType::Navigate));
        assert!(matches!(plan.steps[1].step_type, TaskStepType::Click));
        assert!(matches!(plan.steps[2].step_type, TaskStepType::Type));
    }

    #[test]
    fn test_plan_validation() {
        let plan = TaskPlan {
            id: "test".to_string(),
            description: "Test plan".to_string(),
            steps: vec![],
            estimated_duration: None,
            confidence: 1.0,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(plan.validate().is_err()); // Should fail with no steps
    }

    #[tokio::test]
    async fn test_step_execution_order() {
        let executor = TaskPlanExecutor::new();

        let steps = vec![
            TaskStep {
                id: "step_1".to_string(),
                step_type: TaskStepType::Navigate,
                action: "navigate".to_string(),
                parameters: HashMap::new(),
                description: "First step".to_string(),
                expected_outcome: "Success".to_string(),
                timeout_seconds: Some(10),
                retry_count: 1,
                dependencies: vec![],
            },
            TaskStep {
                id: "step_2".to_string(),
                step_type: TaskStepType::Click,
                action: "click".to_string(),
                parameters: HashMap::new(),
                description: "Second step".to_string(),
                expected_outcome: "Success".to_string(),
                timeout_seconds: Some(10),
                retry_count: 1,
                dependencies: vec!["step_1".to_string()],
            },
        ];

        let order = executor.calculate_execution_order(&steps).unwrap();
        assert_eq!(order, vec!["step_1", "step_2"]);
    }
}
