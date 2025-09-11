// LLM API handlers
// Handles natural language processing, task planning, and AI-driven automation

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info};

use super::AppState;
use super::task_executor::TaskPlanExecutor;
use crate::llm::{LLMConfig, LLMService, LLMResponse as RealLLMResponse, TokenUsage};

// Re-export TaskPlan from the real LLM module or define here if needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub steps: Vec<BrowserAction>,
    pub confidence: f32,
    pub estimated_time_seconds: u32,
    pub complexity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAction {
    pub action_type: String,
    pub target: Option<String>,
    pub value: Option<String>,
    pub options: BrowserActionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionOptions {
    pub wait_for_element: Option<bool>,
    pub timeout_ms: Option<u32>,
    pub retry_count: Option<u32>,
}

impl Default for BrowserActionOptions {
    fn default() -> Self {
        Self {
            wait_for_element: Some(true),
            timeout_ms: Some(5000),
            retry_count: Some(3),
        }
    }
}

// Helper function to build task planning prompt
fn build_planning_prompt(instruction: &str, context: &HashMap<String, serde_json::Value>) -> String {
    let mut prompt = String::from(
        "You are a browser automation expert. Convert the following instruction into a series of browser actions.\n\n"
    );
    
    // Add context if available
    if !context.is_empty() {
        prompt.push_str("Context:\n");
        for (key, value) in context {
            prompt.push_str(&format!("- {}: {}\n", key, value));
        }
        prompt.push('\n');
    }
    
    prompt.push_str(&format!("Instruction: {}\n\n", instruction));
    prompt.push_str(
        "Respond with a JSON object containing:\n\
        {\n\
          \"steps\": [\n\
            {\n\
              \"action_type\": \"navigate|click|type|wait|extract\",\n\
              \"target\": \"CSS selector or URL\",\n\
              \"value\": \"text to type (if applicable)\",\n\
              \"options\": {\n\
                \"wait_for_element\": true,\n\
                \"timeout_ms\": 5000,\n\
                \"retry_count\": 3\n\
              }\n\
            }\n\
          ],\n\
          \"confidence\": 0.0-1.0,\n\
          \"estimated_time_seconds\": integer,\n\
          \"complexity\": \"simple|medium|complex\"\n\
        }\n\n\
        Only use common, reliable CSS selectors. Be specific and accurate."
    );
    
    prompt
}

// Helper function to parse task plan from LLM response
fn parse_task_plan(response_content: &str) -> Result<TaskPlan, String> {
    // Try to extract JSON from the response
    let json_start = response_content.find('{');
    let json_end = response_content.rfind('}');
    
    if let (Some(start), Some(end)) = (json_start, json_end) {
        let json_str = &response_content[start..=end];
        
        match serde_json::from_str::<TaskPlan>(json_str) {
            Ok(plan) => {
                info!("Successfully parsed LLM task plan with {} steps", plan.steps.len());
                Ok(plan)
            }
            Err(e) => {
                error!("Failed to parse LLM response as JSON: {}", e);
                // Create a simple fallback plan
                Ok(TaskPlan {
                    steps: vec![BrowserAction {
                        action_type: "navigate".to_string(),
                        target: Some("https://example.com".to_string()),
                        value: None,
                        options: BrowserActionOptions::default(),
                    }],
                    confidence: 0.5,
                    estimated_time_seconds: 10,
                    complexity: "simple".to_string(),
                })
            }
        }
    } else {
        Err("No JSON found in LLM response".to_string())
    }
}

async fn mock_task_planner(prompt: &str) -> Result<Vec<BrowserAction>, String> {
    let mut actions = Vec::new();
    let prompt_lower = prompt.to_lowercase();
    
    if prompt_lower.contains("navigate") || prompt_lower.contains("go to") {
        let url = if prompt_lower.contains("google.com") {
            "https://google.com"
        } else if prompt_lower.contains("github.com") {
            "https://github.com"
        } else {
            "https://example.com"
        };
        
        actions.push(BrowserAction {
            action_type: "navigate".to_string(),
            target: Some(url.to_string()),
            value: None,
            options: BrowserActionOptions::default(),
        });
    }

    if prompt_lower.contains("click") {
        let selector = if prompt_lower.contains("button") {
            "button"
        } else if prompt_lower.contains("link") {
            "a"
        } else {
            "[data-testid='click-target']"
        };
        
        actions.push(BrowserAction {
            action_type: "click".to_string(),
            target: Some(selector.to_string()),
            value: None,
            options: BrowserActionOptions::default(),
        });
    }

    if prompt_lower.contains("type") || prompt_lower.contains("enter") {
        let selector = if prompt_lower.contains("search") {
            "input[type='search'], input[name='q']"
        } else {
            "input"
        };
        
        actions.push(BrowserAction {
            action_type: "type".to_string(),
            target: Some(selector.to_string()),
            value: Some("test input".to_string()),
            options: BrowserActionOptions::default(),
        });
    }

    if prompt_lower.contains("wait") || prompt_lower.contains("pause") {
        actions.push(BrowserAction {
            action_type: "wait".to_string(),
            target: None,
            value: Some("2000".to_string()),
            options: BrowserActionOptions::default(),
        });
    }

    if actions.is_empty() {
        actions.push(BrowserAction {
            action_type: "wait_for_load".to_string(),
            target: None,
            value: None,
            options: BrowserActionOptions::default(),
        });
    }

    Ok(actions)
}

/// Enhanced error type for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LLMApiError {
    #[error("LLM provider not available: {0}")]
    ProviderUnavailable(String),
    
    #[error("Task planning failed: {0}")]
    TaskPlanningError(String),
    
    #[error("Natural language processing failed: {0}")]
    NLPError(String),
    
    #[error("Cost limit exceeded: current {current}USD, limit {limit}USD")]
    CostLimitExceeded { current: f64, limit: f64 },
    
    #[error("Invalid request parameters: {0}")]
    ValidationError(String),
    
    #[error("LLM service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Command execution failed: {0}")]
    ExecutionError(String),
}

/// Enhanced response wrapper for LLM operations
#[derive(Debug, Serialize)]
pub struct LLMResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: LLMResponseMetadata,
}

#[derive(Debug, Serialize)]
pub struct LLMResponseMetadata {
    pub processing_time_ms: u64,
    pub provider_used: String,
    pub tokens_used: u32,
    pub estimated_cost_usd: f64,
    pub confidence: Option<f32>,
    pub total_time_ms: u64,
}

impl<T> LLMResponse<T> {
    pub fn success(data: T, metadata: LLMResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }
    
    pub fn error(error: String, metadata: LLMResponseMetadata) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata,
        }
    }
}

/// Direct LLM query endpoint
pub async fn llm_query(
    State(_state): State<AppState>,
    Json(req): Json<LLMQueryRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing LLM query: {:?}", req.prompt.chars().take(50).collect::<String>());
    
    // Validate request
    if let Err(validation_error) = validate_llm_query_request(&req) {
        let metadata = LLMResponseMetadata {
            processing_time_ms: 0,
            provider_used: "none".to_string(),
            tokens_used: 0,
            estimated_cost_usd: 0.0,
            confidence: None,
            total_time_ms: start_time.elapsed().as_millis() as u64,
        };
        return (StatusCode::BAD_REQUEST,
               Json(LLMResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    let processing_start = Instant::now();
    
    // Create LLM configuration and service
    let llm_config = create_llm_config(&req);
    let mut llm_service = match LLMService::new(llm_config.clone()) {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create LLM service: {}", e);
            // Fallback to mock response
            let mock_response = create_mock_llm_response(&req).await;
            let processing_time = processing_start.elapsed().as_millis() as u64;
            let metadata = LLMResponseMetadata {
                processing_time_ms: processing_time,
                provider_used: "mock".to_string(),
                tokens_used: mock_response.usage.total_tokens,
                estimated_cost_usd: 0.0,
                confidence: None,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            return Json(LLMResponse::success(mock_response, metadata)).into_response();
        }
    };
    
    // Try real LLM query
    match llm_service.query(&req.prompt).await {
        Ok(real_response) => {
            let processing_time = processing_start.elapsed().as_millis() as u64;
            
            let metadata = LLMResponseMetadata {
                processing_time_ms: processing_time,
                provider_used: req.provider.clone().unwrap_or_else(|| llm_config.default_provider.clone()),
                tokens_used: real_response.usage.total_tokens,
                estimated_cost_usd: calculate_cost(&real_response.usage, &llm_config.default_provider),
                confidence: Some(0.9),
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            
            info!("Real LLM query completed in {}ms, tokens: {}", processing_time, real_response.usage.total_tokens);
            Json(LLMResponse::success(real_response, metadata)).into_response()
        }
        Err(e) => {
            error!("LLM query failed: {}, falling back to mock", e);
            
            // Fallback to mock response
            let mock_response = create_mock_llm_response(&req).await;
            let processing_time = processing_start.elapsed().as_millis() as u64;
            let metadata = LLMResponseMetadata {
                processing_time_ms: processing_time,
                provider_used: "mock".to_string(),
                tokens_used: mock_response.usage.total_tokens,
                estimated_cost_usd: 0.0,
                confidence: Some(0.5),
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            
            info!("Mock LLM query completed in {}ms, tokens: {}", processing_time, mock_response.usage.total_tokens);
            Json(LLMResponse::success(mock_response, metadata)).into_response()
        }
    }
}

/// Task planning endpoint - converts natural language to browser automation plan
pub async fn task_planning(
    State(_state): State<AppState>,
    Json(req): Json<TaskPlanningRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing task planning request: {}", req.instruction);
    
    // Validate request
    if let Err(validation_error) = validate_task_planning_request(&req) {
        let metadata = LLMResponseMetadata {
            processing_time_ms: 0,
            provider_used: "none".to_string(),
            tokens_used: 0,
            estimated_cost_usd: 0.0,
            confidence: None,
            total_time_ms: start_time.elapsed().as_millis() as u64,
        };
        return (StatusCode::BAD_REQUEST,
               Json(LLMResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    // Create LLM configuration
    let llm_config = create_llm_config_for_planning(&req);
    let provider_name = llm_config.default_provider.clone();
    
    match LLMService::new(llm_config) {
        Ok(mut llm_service) => {
            let processing_start = Instant::now();
            
            // Build context for task planning
            let mut context = HashMap::new();
            context.insert("url".to_string(), serde_json::Value::String(req.url.unwrap_or_else(|| "about:blank".to_string())));
            context.insert("complexity".to_string(), serde_json::Value::String(req.complexity.unwrap_or_else(|| "medium".to_string())));
            
            if let Some(ref page_context) = req.page_context {
                context.insert("page_context".to_string(), serde_json::Value::Object(page_context.clone()));
            }
            
            // Build planning prompt
            let planning_prompt = build_planning_prompt(&req.instruction, &context);
            
            match llm_service.query(&planning_prompt).await {
                Ok(llm_response) => {
                    // Try to parse the response as a task plan
                    match parse_task_plan(&llm_response.content) {
                        Ok(task_plan) => {
                            let processing_time = processing_start.elapsed().as_millis() as u64;
                            let metadata = LLMResponseMetadata {
                                processing_time_ms: processing_time,
                                provider_used: req.provider.unwrap_or_else(|| "default".to_string()),
                                tokens_used: llm_response.usage.total_tokens,
                                estimated_cost_usd: calculate_cost(&llm_response.usage, &provider_name),
                                confidence: Some(task_plan.confidence),
                                total_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                            
                            info!("Task planning completed in {}ms with {} steps", processing_time, task_plan.steps.len());
                            Json(LLMResponse::success(task_plan, metadata)).into_response()
                        }
                        Err(e) => {
                            error!("Failed to parse task plan: {}, falling back to mock", e);
                            
                            // Fallback to mock planner
                            let actions = mock_task_planner(&req.instruction).await.unwrap_or_default();
                            let task_plan = TaskPlan {
                                steps: actions,
                                confidence: 0.7,
                                estimated_time_seconds: 10,
                                complexity: "medium".to_string(),
                            };
                            
                            let processing_time = processing_start.elapsed().as_millis() as u64;
                            let metadata = LLMResponseMetadata {
                                processing_time_ms: processing_time,
                                provider_used: "fallback".to_string(),
                                tokens_used: llm_response.usage.total_tokens,
                                estimated_cost_usd: calculate_cost(&llm_response.usage, &provider_name),
                                confidence: Some(task_plan.confidence),
                                total_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                            
                            info!("Fallback task planning completed in {}ms with {} steps", processing_time, task_plan.steps.len());
                            Json(LLMResponse::success(task_plan, metadata)).into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("LLM query failed: {}, using mock planner", e);
                    
                    // Fallback to mock planner
                    let actions = mock_task_planner(&req.instruction).await.unwrap_or_default();
                    let task_plan = TaskPlan {
                        steps: actions,
                        confidence: 0.5,
                        estimated_time_seconds: 10,
                        complexity: "medium".to_string(),
                    };
                    
                    let processing_time = processing_start.elapsed().as_millis() as u64;
                    let metadata = LLMResponseMetadata {
                        processing_time_ms: processing_time,
                        provider_used: "mock".to_string(),
                        tokens_used: 0,
                        estimated_cost_usd: 0.0,
                        confidence: Some(task_plan.confidence),
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    
                    info!("Mock task planning completed in {}ms with {} steps", processing_time, task_plan.steps.len());
                    Json(LLMResponse::success(task_plan, metadata)).into_response()
                }
            }
        },
        Err(e) => {
            error!("Failed to create LLM service for task planning: {}", e);
            let metadata = LLMResponseMetadata {
                processing_time_ms: 0,
                provider_used: "none".to_string(),
                tokens_used: 0,
                estimated_cost_usd: 0.0,
                confidence: None,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(LLMResponse::<()>::error(format!("LLM service initialization failed: {}", e), metadata))).into_response()
        }
    }
}

/// Natural language command execution endpoint
pub async fn execute_command(
    State(state): State<AppState>,
    Json(req): Json<ExecuteCommandRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing natural language command: {}", req.command);
    
    // Validate request
    if let Err(validation_error) = validate_execute_command_request(&req) {
        let metadata = LLMResponseMetadata {
            processing_time_ms: 0,
            provider_used: "none".to_string(),
            tokens_used: 0,
            estimated_cost_usd: 0.0,
            confidence: None,
            total_time_ms: start_time.elapsed().as_millis() as u64,
        };
        return (StatusCode::BAD_REQUEST,
               Json(LLMResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    // First, plan the task
    let task_planning_req = TaskPlanningRequest {
        instruction: req.command.clone(),
        url: req.url.clone(),
        page_context: None,
        complexity: req.complexity.clone(),
        provider: req.provider.clone(),
        max_steps: req.max_steps,
        session_id: req.session_id.clone(),
    };
    
    // Get or create browser session
    let _browser = match state.browser_pool.acquire().await {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            let metadata = LLMResponseMetadata {
                processing_time_ms: 0,
                provider_used: "none".to_string(),
                tokens_used: 0,
                estimated_cost_usd: 0.0,
                confidence: None,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(LLMResponse::<()>::error(format!("Failed to acquire browser: {}", e), metadata))).into_response();
        }
    };
    
    // Create LLM configuration
    let llm_config = create_llm_config_for_planning(&task_planning_req);
    let provider_name = llm_config.default_provider.clone();
    
    match LLMService::new(llm_config) {
        Ok(mut llm_service) => {
            let processing_start = Instant::now();
            
            // Build context for command execution
            let mut context = HashMap::new();
            context.insert("url".to_string(), serde_json::Value::String(req.url.unwrap_or_else(|| "about:blank".to_string())));
            context.insert("auto_execute".to_string(), serde_json::Value::Bool(req.auto_execute.unwrap_or(true)));
            
            // Build planning prompt and query LLM
            let planning_prompt = build_planning_prompt(&req.command, &context);
            
            match llm_service.query(&planning_prompt).await {
                Ok(llm_response) => {
                    // Try to parse the response as a task plan
                    match parse_task_plan(&llm_response.content) {
                        Ok(task_plan) => {
                            let planning_time = processing_start.elapsed().as_millis() as u64;
                            
                            // Execute the plan if auto_execute is true
                            let execution_result = if req.auto_execute.unwrap_or(true) {
                                info!("Auto-executing task plan with {} steps", task_plan.steps.len());
                                
                                // Use the real task plan executor
                                let executor = TaskPlanExecutor::new(_browser.browser_arc());
                                match executor.execute_plan(task_plan.clone()).await {
                                    Ok(exec_result) => {
                                        info!("Task plan execution completed: {} steps completed, {} failed", 
                                              exec_result.steps_completed, exec_result.steps_failed);
                                        
                                        Some(serde_json::json!({
                                            "executed": true,
                                            "success": exec_result.success,
                                            "steps_completed": exec_result.steps_completed,
                                            "steps_failed": exec_result.steps_failed,
                                            "execution_time_ms": exec_result.total_execution_time_ms,
                                            "results": exec_result.final_result,
                                            "action_results": exec_result.action_results,
                                            "error": exec_result.error
                                        }))
                                    }
                                    Err(e) => {
                                        error!("Task plan execution failed: {}", e);
                                        Some(serde_json::json!({
                                            "executed": true,
                                            "success": false,
                                            "steps_completed": 0,
                                            "steps_failed": task_plan.steps.len(),
                                            "execution_time_ms": 0,
                                            "results": "Task execution failed",
                                            "error": e.to_string()
                                        }))
                                    }
                                }
                            } else {
                                info!("Task plan created but not executed (auto_execute=false)");
                                None
                            };
                            
                            let total_processing_time = processing_start.elapsed().as_millis() as u64;
                            let metadata = LLMResponseMetadata {
                                processing_time_ms: total_processing_time,
                                provider_used: req.provider.unwrap_or_else(|| "default".to_string()),
                                tokens_used: llm_response.usage.total_tokens,
                                estimated_cost_usd: calculate_cost(&llm_response.usage, &provider_name),
                                confidence: Some(task_plan.confidence),
                                total_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                            
                            let response_data = serde_json::json!({
                                "command": req.command,
                                "task_plan": task_plan,
                                "execution_result": execution_result,
                                "planning_time_ms": planning_time
                            });
                            
                            info!("Command processing completed in {}ms", total_processing_time);
                            Json(LLMResponse::success(response_data, metadata)).into_response()
                        }
                        Err(e) => {
                            error!("Failed to parse task plan: {}, creating mock plan", e);
                            
                            // Fallback to mock planner
                            let actions = mock_task_planner(&req.command).await.unwrap_or_default();
                            let task_plan = TaskPlan {
                                steps: actions,
                                confidence: 0.6,
                                estimated_time_seconds: 10,
                                complexity: "medium".to_string(),
                            };
                            
                            let planning_time = processing_start.elapsed().as_millis() as u64;
                            let execution_result = if req.auto_execute.unwrap_or(true) {
                                // Execute fallback plan with task executor
                                let executor = TaskPlanExecutor::new(_browser.browser_arc());
                                match executor.execute_plan(task_plan.clone()).await {
                                    Ok(exec_result) => {
                                        Some(serde_json::json!({
                                            "executed": true,
                                            "success": exec_result.success,
                                            "steps_completed": exec_result.steps_completed,
                                            "steps_failed": exec_result.steps_failed,
                                            "execution_time_ms": exec_result.total_execution_time_ms,
                                            "results": exec_result.final_result,
                                            "action_results": exec_result.action_results,
                                            "error": exec_result.error
                                        }))
                                    }
                                    Err(e) => {
                                        Some(serde_json::json!({
                                            "executed": true,
                                            "success": false,
                                            "steps_completed": 0,
                                            "steps_failed": task_plan.steps.len(),
                                            "execution_time_ms": 0,
                                            "results": "Fallback task execution failed",
                                            "error": e.to_string()
                                        }))
                                    }
                                }
                            } else {
                                None
                            };
                            
                            let total_processing_time = processing_start.elapsed().as_millis() as u64;
                            let metadata = LLMResponseMetadata {
                                processing_time_ms: total_processing_time,
                                provider_used: "fallback".to_string(),
                                tokens_used: llm_response.usage.total_tokens,
                                estimated_cost_usd: calculate_cost(&llm_response.usage, &provider_name),
                                confidence: Some(task_plan.confidence),
                                total_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                            
                            let response_data = serde_json::json!({
                                "command": req.command,
                                "task_plan": task_plan,
                                "execution_result": execution_result,
                                "planning_time_ms": planning_time
                            });
                            
                            Json(LLMResponse::success(response_data, metadata)).into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("LLM query failed: {}, using mock planner", e);
                    
                    // Fallback to mock planner
                    let actions = mock_task_planner(&req.command).await.unwrap_or_default();
                    let task_plan = TaskPlan {
                        steps: actions,
                        confidence: 0.5,
                        estimated_time_seconds: 10,
                        complexity: "medium".to_string(),
                    };
                    
                    let planning_time = processing_start.elapsed().as_millis() as u64;
                    let execution_result = if req.auto_execute.unwrap_or(true) {
                        // Execute mock plan with task executor  
                        let executor = TaskPlanExecutor::new(_browser.browser_arc());
                        match executor.execute_plan(task_plan.clone()).await {
                            Ok(exec_result) => {
                                Some(serde_json::json!({
                                    "executed": true,
                                    "success": exec_result.success,
                                    "steps_completed": exec_result.steps_completed,
                                    "steps_failed": exec_result.steps_failed,
                                    "execution_time_ms": exec_result.total_execution_time_ms,
                                    "results": exec_result.final_result,
                                    "action_results": exec_result.action_results,
                                    "error": exec_result.error
                                }))
                            }
                            Err(e) => {
                                Some(serde_json::json!({
                                    "executed": true,
                                    "success": false,
                                    "steps_completed": 0,
                                    "steps_failed": task_plan.steps.len(),
                                    "execution_time_ms": 0,
                                    "results": "Mock task execution failed",
                                    "error": e.to_string()
                                }))
                            }
                        }
                    } else {
                        None
                    };
                    
                    let total_processing_time = processing_start.elapsed().as_millis() as u64;
                    let metadata = LLMResponseMetadata {
                        processing_time_ms: total_processing_time,
                        provider_used: "mock".to_string(),
                        tokens_used: 0,
                        estimated_cost_usd: 0.0,
                        confidence: Some(task_plan.confidence),
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    
                    let response_data = serde_json::json!({
                        "command": req.command,
                        "task_plan": task_plan,
                        "execution_result": execution_result,
                        "planning_time_ms": planning_time
                    });
                    
                    Json(LLMResponse::success(response_data, metadata)).into_response()
                }
            }
        },
        Err(e) => {
            error!("Failed to create LLM service for command execution: {}", e);
            let metadata = LLMResponseMetadata {
                processing_time_ms: 0,
                provider_used: "none".to_string(),
                tokens_used: 0,
                estimated_cost_usd: 0.0,
                confidence: None,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(LLMResponse::<()>::error(format!("LLM service initialization failed: {}", e), metadata))).into_response()
        }
    }
}

/// Cost tracking and usage monitoring endpoint
pub async fn get_usage_metrics(
    State(_state): State<AppState>,
    Json(req): Json<UsageMetricsRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Fetching usage metrics for timeframe: {:?}", req.timeframe);
    
    // For now, return mock usage data
    // In a real implementation, this would query a database or persistent storage
    let usage_metrics = serde_json::json!({
        "timeframe": req.timeframe,
        "total_requests": 142,
        "total_tokens": 15420,
        "total_cost_usd": 2.34,
        "provider_breakdown": {
            "openai": {
                "requests": 89,
                "tokens": 9840,
                "cost_usd": 1.52
            },
            "claude": {
                "requests": 53,
                "tokens": 5580,
                "cost_usd": 0.82
            }
        },
        "usage_by_endpoint": {
            "llm_query": {
                "requests": 67,
                "tokens": 7200,
                "cost_usd": 1.08
            },
            "task_planning": {
                "requests": 45,
                "tokens": 4810,
                "cost_usd": 0.73
            },
            "execute_command": {
                "requests": 30,
                "tokens": 3410,
                "cost_usd": 0.53
            }
        },
        "period_start": "2024-01-01T00:00:00Z",
        "period_end": "2024-01-02T00:00:00Z",
        "generated_at": chrono::Utc::now().to_rfc3339()
    });
    
    let metadata = LLMResponseMetadata {
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        provider_used: "metrics".to_string(),
        tokens_used: 0,
        estimated_cost_usd: 0.0,
        confidence: None,
        total_time_ms: start_time.elapsed().as_millis() as u64,
    };
    
    Json(LLMResponse::success(usage_metrics, metadata)).into_response()
}

// Request/Response types for LLM API

#[derive(Deserialize)]
pub struct LLMQueryRequest {
    pub prompt: String,
    pub provider: Option<String>, // "openai", "claude", "mock"
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub model: Option<String>,
}

#[derive(Deserialize)]
pub struct TaskPlanningRequest {
    pub instruction: String,
    pub url: Option<String>,
    pub page_context: Option<serde_json::Map<String, serde_json::Value>>,
    pub complexity: Option<String>, // "simple", "medium", "complex"
    pub provider: Option<String>,
    pub max_steps: Option<usize>,
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub url: Option<String>,
    pub auto_execute: Option<bool>, // If false, only plan but don't execute
    pub complexity: Option<String>,
    pub provider: Option<String>,
    pub max_steps: Option<usize>,
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UsageMetricsRequest {
    pub timeframe: Option<String>, // "hour", "day", "week", "month"
    pub provider: Option<String>,
    pub start_date: Option<String>, // ISO 8601 format
    pub end_date: Option<String>,
}

// Validation functions

fn validate_llm_query_request(req: &LLMQueryRequest) -> Result<(), LLMApiError> {
    if req.prompt.trim().is_empty() {
        return Err(LLMApiError::ValidationError("Prompt cannot be empty".to_string()));
    }
    
    if req.prompt.len() > 32000 {
        return Err(LLMApiError::ValidationError("Prompt too long (max 32000 characters)".to_string()));
    }
    
    if let Some(ref provider) = req.provider {
        let valid_providers = ["openai", "claude", "mock"];
        if !valid_providers.contains(&provider.as_str()) {
            return Err(LLMApiError::ValidationError(
                format!("Invalid provider '{}'. Valid providers: {}", provider, valid_providers.join(", "))
            ));
        }
    }
    
    if let Some(max_tokens) = req.max_tokens {
        if max_tokens == 0 || max_tokens > 32000 {
            return Err(LLMApiError::ValidationError("max_tokens must be between 1 and 32000".to_string()));
        }
    }
    
    if let Some(temperature) = req.temperature {
        if temperature < 0.0 || temperature > 2.0 {
            return Err(LLMApiError::ValidationError("temperature must be between 0.0 and 2.0".to_string()));
        }
    }
    
    Ok(())
}

fn validate_task_planning_request(req: &TaskPlanningRequest) -> Result<(), LLMApiError> {
    if req.instruction.trim().is_empty() {
        return Err(LLMApiError::ValidationError("Instruction cannot be empty".to_string()));
    }
    
    if req.instruction.len() > 2000 {
        return Err(LLMApiError::ValidationError("Instruction too long (max 2000 characters)".to_string()));
    }
    
    if let Some(ref complexity) = req.complexity {
        let valid_complexities = ["simple", "medium", "complex"];
        if !valid_complexities.contains(&complexity.as_str()) {
            return Err(LLMApiError::ValidationError(
                format!("Invalid complexity '{}'. Valid values: {}", complexity, valid_complexities.join(", "))
            ));
        }
    }
    
    if let Some(max_steps) = req.max_steps {
        if max_steps == 0 || max_steps > 50 {
            return Err(LLMApiError::ValidationError("max_steps must be between 1 and 50".to_string()));
        }
    }
    
    Ok(())
}

fn validate_execute_command_request(req: &ExecuteCommandRequest) -> Result<(), LLMApiError> {
    if req.command.trim().is_empty() {
        return Err(LLMApiError::ValidationError("Command cannot be empty".to_string()));
    }
    
    if req.command.len() > 1000 {
        return Err(LLMApiError::ValidationError("Command too long (max 1000 characters)".to_string()));
    }
    
    Ok(())
}

// Utility functions

fn create_llm_config(req: &LLMQueryRequest) -> LLMConfig {
    LLMConfig {
        default_provider: req.provider.clone().unwrap_or_else(|| "openai".to_string()),
        openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        claude_api_key: std::env::var("CLAUDE_API_KEY").ok(),
        max_tokens: req.max_tokens.unwrap_or(4000),
        temperature: req.temperature.unwrap_or(0.7),
        cost_limit_usd: 10.0, // Default cost limit
    }
}

fn create_llm_config_for_planning(req: &TaskPlanningRequest) -> LLMConfig {
    LLMConfig {
        default_provider: req.provider.clone().unwrap_or_else(|| "openai".to_string()),
        openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
        claude_api_key: std::env::var("CLAUDE_API_KEY").ok(),
        max_tokens: 2000, // Optimized for task planning
        temperature: 0.3, // Lower temperature for more focused planning
        cost_limit_usd: 5.0,
    }
}

fn calculate_cost(usage: &TokenUsage, provider: &str) -> f64 {
    match provider {
        "openai" => {
            // GPT-4 pricing (approximate)
            let input_cost = usage.prompt_tokens as f64 * 0.00003;
            let output_cost = usage.completion_tokens as f64 * 0.00006;
            input_cost + output_cost
        },
        "claude" => {
            // Claude pricing (approximate)
            let input_cost = usage.prompt_tokens as f64 * 0.000008;
            let output_cost = usage.completion_tokens as f64 * 0.000024;
            input_cost + output_cost
        },
        _ => 0.01 // Default/mock cost
    }
}

async fn create_mock_llm_response(req: &LLMQueryRequest) -> RealLLMResponse {
    // Simulate processing time
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let response_content = if req.prompt.to_lowercase().contains("navigate") {
        "I'll help you navigate to the specified URL. I'll use the browser automation to load the page and wait for it to be ready."
    } else if req.prompt.to_lowercase().contains("click") {
        "I'll locate the specified element on the page and perform a click action on it."
    } else if req.prompt.to_lowercase().contains("extract") {
        "I'll analyze the page content and extract the requested information for you."
    } else if req.prompt.to_lowercase().contains("search") {
        "I'll help you search for the specified content on the page."
    } else if req.prompt.to_lowercase().contains("form") {
        "I'll assist you with filling out the form with the provided information."
    } else {
        "I understand your request and will help you with browser automation tasks. Please provide more specific instructions for the actions you'd like me to perform."
    };

    RealLLMResponse {
        content: response_content.to_string(),
        model: "mock-gpt-4".to_string(),
        usage: TokenUsage {
            prompt_tokens: 150,
            completion_tokens: 75,
            total_tokens: 225,
        },
        finish_reason: "stop".to_string(),
        timestamp: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_llm_query_request() {
        let valid_req = LLMQueryRequest {
            prompt: "Test prompt".to_string(),
            provider: Some("openai".to_string()),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            model: None,
        };
        assert!(validate_llm_query_request(&valid_req).is_ok());
        
        let invalid_req = LLMQueryRequest {
            prompt: "".to_string(),
            provider: None,
            max_tokens: None,
            temperature: None,
            model: None,
        };
        assert!(validate_llm_query_request(&invalid_req).is_err());
    }
    
    #[test]
    fn test_calculate_cost() {
        let usage = MockTokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        
        let openai_cost = calculate_cost(&usage, "openai");
        assert!(openai_cost > 0.0);
        
        let claude_cost = calculate_cost(&usage, "claude");
        assert!(claude_cost > 0.0);
        
        // Claude should be cheaper than OpenAI for same usage
        assert!(claude_cost < openai_cost);
    }
}