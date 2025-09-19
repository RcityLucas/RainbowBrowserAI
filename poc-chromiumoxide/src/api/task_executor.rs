// Task Plan Execution Engine
// Bridges LLM-generated task plans to actual browser operations

use crate::api::llm_handlers::{BrowserAction, TaskPlan};
use crate::browser::Browser;
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Execution result for a single action
#[derive(Debug, Clone, Serialize)]
pub struct ActionResult {
    pub action_type: String,
    pub target: Option<String>,
    pub success: bool,
    pub execution_time_ms: u64,
    pub result_data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Overall execution result for a task plan
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub total_execution_time_ms: u64,
    pub steps_completed: usize,
    pub steps_failed: usize,
    pub action_results: Vec<ActionResult>,
    pub final_result: serde_json::Value,
    pub error: Option<String>,
}

/// Task Plan Executor
pub struct TaskPlanExecutor {
    browser: Arc<Browser>,
}

impl TaskPlanExecutor {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }

    /// Execute a complete task plan
    pub async fn execute_plan(&self, plan: TaskPlan) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        let mut action_results = Vec::new();
        let mut steps_completed = 0;
        let mut steps_failed = 0;
        let mut final_result = serde_json::json!({
            "message": "Task execution started",
            "plan_confidence": plan.confidence,
            "estimated_time": plan.estimated_time_seconds
        });

        info!(
            "Executing task plan with {} steps (confidence: {:.2})",
            plan.steps.len(),
            plan.confidence
        );

        // Execute each step in sequence
        for (index, action) in plan.steps.iter().enumerate() {
            info!(
                "Executing step {} of {}: {} {:?}",
                index + 1,
                plan.steps.len(),
                action.action_type,
                action.target
            );

            let action_start = Instant::now();
            match self.execute_action(action).await {
                Ok(mut result) => {
                    result.execution_time_ms = action_start.elapsed().as_millis() as u64;
                    steps_completed += 1;

                    // Update final result based on action type
                    self.update_final_result(&mut final_result, action, &result);

                    action_results.push(result);
                    info!(
                        "Step {} completed successfully in {}ms",
                        index + 1,
                        action_start.elapsed().as_millis()
                    );
                }
                Err(e) => {
                    steps_failed += 1;
                    let error_result = ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: false,
                        execution_time_ms: action_start.elapsed().as_millis() as u64,
                        result_data: None,
                        error: Some(e.to_string()),
                    };
                    action_results.push(error_result);

                    warn!("Step {} failed: {}", index + 1, e);

                    // Decide whether to continue or stop on failure
                    if self.should_stop_on_failure(&action.action_type) {
                        error!("Critical action failed, stopping execution: {}", e);
                        final_result = serde_json::json!({
                            "message": "Task execution failed",
                            "error": e.to_string(),
                            "steps_completed": steps_completed,
                            "steps_failed": steps_failed
                        });
                        break;
                    }
                }
            }

            // Small delay between actions to prevent overwhelming the browser
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        let overall_success = steps_failed == 0 || steps_completed > steps_failed;

        if overall_success {
            final_result["message"] = serde_json::Value::String(format!(
                "Task completed successfully. {} steps completed, {} failed",
                steps_completed, steps_failed
            ));
        }

        info!(
            "Task plan execution completed: {} successful, {} failed, {}ms total",
            steps_completed, steps_failed, total_time
        );

        Ok(ExecutionResult {
            success: overall_success,
            total_execution_time_ms: total_time,
            steps_completed,
            steps_failed,
            action_results,
            final_result,
            error: if overall_success {
                None
            } else {
                Some(format!(
                    "Task partially failed: {} of {} steps failed",
                    steps_failed,
                    plan.steps.len()
                ))
            },
        })
    }

    /// Execute a single browser action
    async fn execute_action(&self, action: &BrowserAction) -> Result<ActionResult> {
        let timeout = Duration::from_millis(action.options.timeout_ms.unwrap_or(5000) as u64);

        match action.action_type.as_str() {
            "navigate" => {
                if let Some(ref url) = action.target {
                    self.browser.navigate_to(url).await?;

                    // Wait for page load if requested
                    if action.options.wait_for_element.unwrap_or(true) {
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }

                    Ok(ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: true,
                        execution_time_ms: 0, // Will be set by caller
                        result_data: Some(serde_json::json!({
                            "url": url,
                            "action": "navigated"
                        })),
                        error: None,
                    })
                } else {
                    Err(anyhow::anyhow!("Navigate action requires target URL"))
                }
            }

            "click" => {
                if let Some(ref selector) = action.target {
                    // Wait for element if requested
                    if action.options.wait_for_element.unwrap_or(true) {
                        self.browser.wait_for_selector(selector, timeout).await?;
                    }

                    self.browser.click(selector).await?;

                    Ok(ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: true,
                        execution_time_ms: 0,
                        result_data: Some(serde_json::json!({
                            "selector": selector,
                            "action": "clicked"
                        })),
                        error: None,
                    })
                } else {
                    Err(anyhow::anyhow!("Click action requires target selector"))
                }
            }

            "type" => {
                if let (Some(ref selector), Some(ref text)) = (&action.target, &action.value) {
                    // Wait for element if requested
                    if action.options.wait_for_element.unwrap_or(true) {
                        self.browser.wait_for_selector(selector, timeout).await?;
                    }

                    self.browser.type_text(selector, text).await?;

                    Ok(ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: true,
                        execution_time_ms: 0,
                        result_data: Some(serde_json::json!({
                            "selector": selector,
                            "text": text,
                            "action": "typed"
                        })),
                        error: None,
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "Type action requires both target selector and text value"
                    ))
                }
            }

            "wait" => {
                let wait_time = if let Some(ref time_str) = action.value {
                    time_str.parse::<u64>().unwrap_or(1000)
                } else {
                    1000
                };

                tokio::time::sleep(Duration::from_millis(wait_time)).await;

                Ok(ActionResult {
                    action_type: action.action_type.clone(),
                    target: action.target.clone(),
                    success: true,
                    execution_time_ms: 0,
                    result_data: Some(serde_json::json!({
                        "wait_time_ms": wait_time,
                        "action": "waited"
                    })),
                    error: None,
                })
            }

            "extract" => {
                if let Some(ref selector) = action.target {
                    // Wait for element if requested
                    if action.options.wait_for_element.unwrap_or(true) {
                        self.browser.wait_for_selector(selector, timeout).await?;
                    }

                    let text = self.browser.get_text(selector).await?;

                    Ok(ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: true,
                        execution_time_ms: 0,
                        result_data: Some(serde_json::json!({
                            "selector": selector,
                            "extracted_text": text,
                            "action": "extracted"
                        })),
                        error: None,
                    })
                } else {
                    Err(anyhow::anyhow!("Extract action requires target selector"))
                }
            }

            "wait_for_element" => {
                if let Some(ref selector) = action.target {
                    self.browser.wait_for_selector(selector, timeout).await?;

                    Ok(ActionResult {
                        action_type: action.action_type.clone(),
                        target: action.target.clone(),
                        success: true,
                        execution_time_ms: 0,
                        result_data: Some(serde_json::json!({
                            "selector": selector,
                            "action": "element_found"
                        })),
                        error: None,
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "Wait for element action requires target selector"
                    ))
                }
            }

            "wait_for_load" => {
                // Simple page load wait
                tokio::time::sleep(Duration::from_millis(2000)).await;

                Ok(ActionResult {
                    action_type: action.action_type.clone(),
                    target: action.target.clone(),
                    success: true,
                    execution_time_ms: 0,
                    result_data: Some(serde_json::json!({
                        "action": "page_loaded"
                    })),
                    error: None,
                })
            }

            "screenshot" => {
                let screenshot_data = self
                    .browser
                    .screenshot(crate::browser::ScreenshotOptions::default())
                    .await?;

                Ok(ActionResult {
                    action_type: action.action_type.clone(),
                    target: action.target.clone(),
                    success: true,
                    execution_time_ms: 0,
                    result_data: Some(serde_json::json!({
                        "screenshot_size": screenshot_data.len(),
                        "action": "screenshot_taken"
                    })),
                    error: None,
                })
            }

            _ => {
                warn!("Unknown action type: {}", action.action_type);
                Err(anyhow::anyhow!(
                    "Unknown action type: {}",
                    action.action_type
                ))
            }
        }
    }

    /// Update the final result based on the completed action
    fn update_final_result(
        &self,
        final_result: &mut serde_json::Value,
        action: &BrowserAction,
        result: &ActionResult,
    ) {
        if let Some(result_data) = &result.result_data {
            match action.action_type.as_str() {
                "navigate" => {
                    if let Some(url) = result_data.get("url") {
                        final_result["current_url"] = url.clone();
                    }
                }
                "extract" => {
                    if let Some(extracted) = result_data.get("extracted_text") {
                        if final_result["extracted_data"].is_null() {
                            final_result["extracted_data"] = serde_json::json!([]);
                        }
                        final_result["extracted_data"].as_array_mut().unwrap().push(
                            serde_json::json!({
                                "selector": action.target,
                                "text": extracted
                            }),
                        );
                    }
                }
                "screenshot" => {
                    final_result["screenshot_taken"] = serde_json::Value::Bool(true);
                }
                _ => {}
            }
        }
    }

    /// Determine if execution should stop on failure for this action type
    fn should_stop_on_failure(&self, action_type: &str) -> bool {
        matches!(action_type, "navigate" | "wait_for_element")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::llm_handlers::BrowserAction;

    #[tokio::test]
    async fn test_task_plan_structure() {
        // Test that we can create a task plan with actions
        let actions = vec![
            BrowserAction {
                action_type: "navigate".to_string(),
                target: Some("https://example.com".to_string()),
                value: None,
                options: crate::api::llm_handlers::BrowserActionOptions::default(),
            },
            BrowserAction {
                action_type: "wait_for_element".to_string(),
                target: Some("h1".to_string()),
                value: None,
                options: crate::api::llm_handlers::BrowserActionOptions::default(),
            },
        ];

        let task_plan = TaskPlan {
            steps: actions,
            confidence: 0.9,
            estimated_time_seconds: 10,
            complexity: "medium".to_string(),
        };

        assert_eq!(task_plan.steps.len(), 2);
        assert_eq!(task_plan.confidence, 0.9);
    }
}
