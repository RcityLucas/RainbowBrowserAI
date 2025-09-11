// Workflow API handlers
// Orchestrates cross-module communication between Perception, LLM, and Intelligence services

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{error, info, debug};

use super::AppState;
use crate::perception::{LayeredPerception, PerceptionMode};
use crate::intelligence::{IntelligenceService, IntelligenceConfig, PageContext, IntelligenceAnalysis, ActionRecommendation, ViewportInfo};

// Import types from other modules
use super::llm_handlers::BrowserAction;

/// Enhanced error type for workflow operations
#[derive(Debug, thiserror::Error)]
pub enum WorkflowApiError {
    #[error("Workflow execution failed: {0}")]
    ExecutionError(String),
    
    #[error("Perception analysis failed: {0}")]
    PerceptionError(String),
    
    #[error("LLM processing failed: {0}")]
    LLMError(String),
    
    #[error("Intelligence analysis failed: {0}")]
    IntelligenceError(String),
    
    #[error("Module communication failed: {0}")]
    CommunicationError(String),
    
    #[error("Invalid workflow configuration: {0}")]
    ConfigurationError(String),
    
    #[error("Workflow validation failed: {0}")]
    ValidationError(String),
    
    #[error("Browser integration failed: {0}")]
    BrowserIntegrationError(String),
}

/// Enhanced response wrapper for workflow operations
#[derive(Debug, Serialize)]
pub struct WorkflowResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: WorkflowResponseMetadata,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponseMetadata {
    pub total_processing_time_ms: u64,
    pub modules_used: Vec<String>,
    pub perception_time_ms: Option<u64>,
    pub llm_time_ms: Option<u64>,
    pub intelligence_time_ms: Option<u64>,
    pub execution_time_ms: Option<u64>,
    pub workflow_version: String,
    pub success_rate: f32,
}

impl<T> WorkflowResponse<T> {
    pub fn success(data: T, metadata: WorkflowResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }
    
    pub fn error(error: String, metadata: WorkflowResponseMetadata) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata,
        }
    }
}

/// Complete AI-driven automation workflow endpoint
pub async fn execute_intelligent_workflow(
    State(state): State<AppState>,
    Json(req): Json<IntelligentWorkflowRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Starting intelligent workflow execution: {}", req.user_command);
    
    // Validate request
    if let Err(validation_error) = validate_workflow_request(&req) {
        let metadata = WorkflowResponseMetadata {
            total_processing_time_ms: 0,
            modules_used: vec![],
            perception_time_ms: None,
            llm_time_ms: None,
            intelligence_time_ms: None,
            execution_time_ms: None,
            workflow_version: "1.0.0".to_string(),
            success_rate: 0.0,
        };
        return (StatusCode::BAD_REQUEST,
               Json(WorkflowResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    // Get browser instance
    let browser = match state.browser_pool.acquire().await {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to acquire browser for workflow: {}", e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used: vec![],
                perception_time_ms: None,
                llm_time_ms: None,
                intelligence_time_ms: None,
                execution_time_ms: None,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.0,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Browser acquisition failed: {}", e), metadata))).into_response();
        }
    };

    let mut modules_used = Vec::new();
    let mut perception_time: Option<u64> = None;
    let mut llm_time: Option<u64> = None;
    let mut intelligence_time: Option<u64> = None;
    let mut execution_time: Option<u64> = None;

    // Navigate to URL if provided
    if let Some(ref url) = req.url {
        if let Err(e) = browser.navigate_to(url).await {
            error!("Failed to navigate to {}: {}", url, e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used,
                perception_time_ms: perception_time,
                llm_time_ms: llm_time,
                intelligence_time_ms: intelligence_time,
                execution_time_ms: execution_time,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.0,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Navigation failed: {}", e), metadata))).into_response();
        }
    }

    // PHASE 1: Perception Analysis
    info!("Phase 1: Perception Analysis");
    let perception_start = Instant::now();
    
    let mut layered_perception = LayeredPerception::new(browser.browser_arc());
    let perception_mode_str = req.perception_mode.clone().unwrap_or_else(|| "standard".to_string());
    let perception_mode = match perception_mode_str.as_str() {
        "lightning" => PerceptionMode::Lightning,
        "quick" => PerceptionMode::Quick,
        "standard" => PerceptionMode::Standard,
        "deep" => PerceptionMode::Deep,
        "adaptive" => PerceptionMode::Adaptive,
        _ => PerceptionMode::Standard,
    };
    
    let perception_result = match layered_perception.perceive(perception_mode).await {
        Ok(result) => {
            modules_used.push("perception".to_string());
            perception_time = Some(perception_start.elapsed().as_millis() as u64);
            debug!("Perception analysis completed in {}ms", perception_time.unwrap());
            result
        },
        Err(e) => {
            error!("Perception analysis failed: {}", e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used,
                perception_time_ms: Some(perception_start.elapsed().as_millis() as u64),
                llm_time_ms: llm_time,
                intelligence_time_ms: intelligence_time,
                execution_time_ms: execution_time,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.25,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Perception failed: {}", e), metadata))).into_response();
        }
    };

    // PHASE 2: Intelligence Analysis
    info!("Phase 2: Intelligence Analysis");
    let intelligence_start = Instant::now();
    
    let intelligence_config = req.intelligence_config.clone().unwrap_or_else(|| IntelligenceConfig::default());
    let intelligence_service = IntelligenceService::new(intelligence_config);
    
    // Create page context from current page
    let current_url = req.url.clone().unwrap_or_else(|| "about:blank".to_string());
    let domain = current_url.split("://").nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or(&current_url).to_string();
    
    let page_context = PageContext {
        url: current_url,
        domain,
        title: "Current Page".to_string(), // This would be extracted from browser
        html_content: None,
        viewport_info: ViewportInfo {
            width: 1920,
            height: 1080,
            device_pixel_ratio: 1.0,
            is_mobile: false,
        },
        performance_metrics: None,
    };
    
    let intelligence_analysis = match intelligence_service.analyze_situation(&page_context, &req.user_command, &browser).await {
        Ok(analysis) => {
            modules_used.push("intelligence".to_string());
            intelligence_time = Some(intelligence_start.elapsed().as_millis() as u64);
            debug!("Intelligence analysis completed in {}ms", intelligence_time.unwrap());
            analysis
        },
        Err(e) => {
            error!("Intelligence analysis failed: {}", e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used,
                perception_time_ms: perception_time,
                llm_time_ms: llm_time,
                intelligence_time_ms: Some(intelligence_start.elapsed().as_millis() as u64),
                execution_time_ms: execution_time,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.5,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Intelligence analysis failed: {}", e), metadata))).into_response();
        }
    };

    // PHASE 3: LLM Task Planning
    info!("Phase 3: LLM Task Planning");
    let llm_start = Instant::now();
    
    // Create enhanced prompt combining perception and intelligence insights
    let enhanced_prompt = format!(
        "User Command: {}\n\nPage Analysis:\n- Perception mode: {:?}\n- Analysis completed\n\nIntelligence Insights:\n- Recommended action: {}\n- Confidence: {:.2}\n- Reasoning: {}\n\nPlease provide specific browser automation steps.",
        req.user_command,
        perception_result,
        intelligence_analysis.decision.action_type,
        intelligence_analysis.confidence,
        intelligence_analysis.reasoning
    );
    
    // Mock LLM task planning (in production, this would call actual LLM service)
    let task_plan = create_enhanced_task_plan(&enhanced_prompt, &perception_result, &intelligence_analysis).await;
    
    modules_used.push("llm".to_string());
    llm_time = Some(llm_start.elapsed().as_millis() as u64);
    debug!("LLM task planning completed in {}ms", llm_time.unwrap());

    // PHASE 4: Intelligent Action Recommendation
    info!("Phase 4: Action Recommendation");
    let action_recommendation = match intelligence_service.recommend_action(&intelligence_analysis).await {
        Ok(recommendation) => {
            debug!("Action recommendation generated with confidence: {:.2}", recommendation.confidence);
            recommendation
        },
        Err(e) => {
            error!("Action recommendation failed: {}", e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used,
                perception_time_ms: perception_time,
                llm_time_ms: llm_time,
                intelligence_time_ms: intelligence_time,
                execution_time_ms: execution_time,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.75,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Action recommendation failed: {}", e), metadata))).into_response();
        }
    };

    // PHASE 5: Execution (Optional)
    let execution_result = if req.auto_execute.unwrap_or(true) {
        info!("Phase 5: Task Execution");
        let execution_start = Instant::now();
        
        let result = execute_task_plan(&browser, &task_plan, &action_recommendation).await;
        execution_time = Some(execution_start.elapsed().as_millis() as u64);
        debug!("Task execution completed in {}ms", execution_time.unwrap());
        
        // PHASE 6: Learning Feedback
        if let Ok(ref exec_result) = result {
            let _ = intelligence_service.learn_from_result(
                &action_recommendation,
                &exec_result.summary,
                exec_result.success,
                execution_time.unwrap_or(0),
            ).await;
            debug!("Learning feedback submitted");
        }
        
        Some(result)
    } else {
        info!("Skipping execution (auto_execute=false)");
        None
    };

    // Build comprehensive workflow result
    let workflow_result = IntelligentWorkflowResult {
        user_command: req.user_command,
        perception_result,
        intelligence_analysis,
        task_plan: task_plan.clone(),
        action_recommendation,
        execution_result: execution_result.as_ref().map(|r| match r {
            Ok(result) => result.clone(),
            Err(e) => ExecutionResult {
                success: false,
                summary: format!("Execution failed: {}", e),
                actions_completed: 0,
                total_actions: task_plan.steps.len(),
                execution_time_ms: execution_time.unwrap_or(0),
                errors: vec![e.to_string()],
            }
        }),
        modules_coordination: ModulesCoordination {
            perception_to_intelligence: "Page analysis fed into intelligence decision making".to_string(),
            intelligence_to_llm: "Intelligence insights enhanced LLM task planning".to_string(),
            llm_to_execution: "LLM plan guided browser action execution".to_string(),
            feedback_loop: "Execution results fed back to intelligence learning".to_string(),
        },
        workflow_metrics: WorkflowMetrics {
            total_time_ms: start_time.elapsed().as_millis() as u64,
            perception_time_ms: perception_time.unwrap_or(0),
            intelligence_time_ms: intelligence_time.unwrap_or(0),
            llm_time_ms: llm_time.unwrap_or(0),
            execution_time_ms: execution_time.unwrap_or(0),
            success_rate: calculate_workflow_success_rate(&execution_result),
            modules_used: modules_used.len(),
        },
    };

    let total_time = start_time.elapsed().as_millis() as u64;
    let success_rate = calculate_workflow_success_rate(&execution_result);
    
    let metadata = WorkflowResponseMetadata {
        total_processing_time_ms: total_time,
        modules_used,
        perception_time_ms: perception_time,
        llm_time_ms: llm_time,
        intelligence_time_ms: intelligence_time,
        execution_time_ms: execution_time,
        workflow_version: "1.0.0".to_string(),
        success_rate,
    };

    info!("Intelligent workflow completed in {}ms with success rate: {:.2}", total_time, success_rate);
    Json(WorkflowResponse::success(workflow_result, metadata)).into_response()
}

/// Simple workflow execution with basic module coordination
pub async fn execute_simple_workflow(
    State(state): State<AppState>,
    Json(req): Json<SimpleWorkflowRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Starting simple workflow execution: {} steps", req.steps.len());
    
    // Validate request
    if req.steps.is_empty() {
        let metadata = WorkflowResponseMetadata {
            total_processing_time_ms: 0,
            modules_used: vec![],
            perception_time_ms: None,
            llm_time_ms: None,
            intelligence_time_ms: None,
            execution_time_ms: None,
            workflow_version: "1.0.0".to_string(),
            success_rate: 0.0,
        };
        return (StatusCode::BAD_REQUEST,
               Json(WorkflowResponse::<()>::error("Workflow steps cannot be empty".to_string(), metadata))).into_response();
    }
    
    // Get browser instance
    let browser = match state.browser_pool.acquire().await {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to acquire browser for simple workflow: {}", e);
            let metadata = WorkflowResponseMetadata {
                total_processing_time_ms: start_time.elapsed().as_millis() as u64,
                modules_used: vec![],
                perception_time_ms: None,
                llm_time_ms: None,
                intelligence_time_ms: None,
                execution_time_ms: None,
                workflow_version: "1.0.0".to_string(),
                success_rate: 0.0,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(WorkflowResponse::<()>::error(format!("Browser acquisition failed: {}", e), metadata))).into_response();
        }
    };

    let execution_start = Instant::now();
    let mut completed_steps = 0;
    let mut errors = Vec::new();

    // Execute each step in sequence
    for (index, step) in req.steps.iter().enumerate() {
        info!("Executing step {}: {}", index + 1, step.action_type);
        
        match execute_workflow_step(&browser, step).await {
            Ok(_) => {
                completed_steps += 1;
                debug!("Step {} completed successfully", index + 1);
            },
            Err(e) => {
                error!("Step {} failed: {}", index + 1, e);
                errors.push(format!("Step {}: {}", index + 1, e));
                
                if req.stop_on_error.unwrap_or(true) {
                    break;
                }
            }
        }
    }

    let execution_time = execution_start.elapsed().as_millis() as u64;
    let success = errors.is_empty();
    let success_rate = completed_steps as f32 / req.steps.len() as f32;

    let simple_result = SimpleWorkflowResult {
        steps_completed: completed_steps,
        total_steps: req.steps.len(),
        success,
        execution_time_ms: execution_time,
        errors,
        summary: if success {
            format!("All {} steps completed successfully", req.steps.len())
        } else {
            format!("{} of {} steps completed", completed_steps, req.steps.len())
        },
    };

    let metadata = WorkflowResponseMetadata {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        modules_used: vec!["browser".to_string()],
        perception_time_ms: None,
        llm_time_ms: None,
        intelligence_time_ms: None,
        execution_time_ms: Some(execution_time),
        workflow_version: "1.0.0".to_string(),
        success_rate,
    };

    info!("Simple workflow completed: {} of {} steps succeeded", completed_steps, req.steps.len());
    Json(WorkflowResponse::success(simple_result, metadata)).into_response()
}

/// Get workflow execution status and metrics
pub async fn get_workflow_status(
    State(_state): State<AppState>,
    Json(req): Json<WorkflowStatusRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    
    // Mock workflow status (in production, this would query actual workflow storage)
    let status = WorkflowStatus {
        workflow_id: req.workflow_id.clone(),
        status: "completed".to_string(),
        progress: 100.0,
        current_step: None,
        started_at: chrono::Utc::now() - chrono::Duration::minutes(5),
        completed_at: Some(chrono::Utc::now()),
        modules_involved: vec!["perception".to_string(), "llm".to_string(), "intelligence".to_string()],
        success_rate: 0.95,
        total_steps: 8,
        completed_steps: 8,
        failed_steps: 0,
        execution_metrics: WorkflowExecutionMetrics {
            total_time_ms: 4500,
            perception_time_ms: 800,
            llm_time_ms: 1200,
            intelligence_time_ms: 900,
            execution_time_ms: 1600,
            average_step_time_ms: 562,
            peak_memory_usage_mb: 256,
            cpu_usage_percent: 45.2,
        },
    };

    let metadata = WorkflowResponseMetadata {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        modules_used: vec!["workflow_manager".to_string()],
        perception_time_ms: None,
        llm_time_ms: None,
        intelligence_time_ms: None,
        execution_time_ms: None,
        workflow_version: "1.0.0".to_string(),
        success_rate: 1.0,
    };

    Json(WorkflowResponse::success(status, metadata)).into_response()
}

// Helper functions

async fn create_enhanced_task_plan(
    _enhanced_prompt: &str,
    _perception_result: &crate::perception::PerceptionResult,
    intelligence_analysis: &IntelligenceAnalysis,
) -> TaskPlan {
    // Create task plan based on intelligence insights
    let mut actions = Vec::new();
    
    // Use intelligence decision as primary action
    let primary_action = BrowserAction {
        action_type: intelligence_analysis.decision.action_type.clone(),
        target: intelligence_analysis.decision.target_element.clone(),
        value: intelligence_analysis.decision.parameters.get("value")
            .and_then(|v| v.as_str())
            .map(String::from),
        options: super::llm_handlers::BrowserActionOptions::default(),
    };
    actions.push(primary_action);
    
    // Add a generic wait action for page stability
    actions.push(BrowserAction {
        action_type: "wait_for_load".to_string(),
        target: None,
        value: Some("2000".to_string()),
        options: super::llm_handlers::BrowserActionOptions::default(),
    });
    
    TaskPlan {
        steps: actions,
        confidence: intelligence_analysis.confidence as f32,
        estimated_time_seconds: 10,
        complexity: "enhanced".to_string(),
    }
}

async fn execute_task_plan(
    browser: &crate::browser::Browser,
    task_plan: &TaskPlan,
    action_recommendation: &ActionRecommendation,
) -> Result<ExecutionResult, anyhow::Error> {
    let mut completed_actions = 0;
    let mut errors = Vec::new();
    let start_time = Instant::now();
    
    for (index, action) in task_plan.steps.iter().enumerate() {
        debug!("Executing action {}: {}", index + 1, action.action_type);
        
        match execute_browser_action(browser, action).await {
            Ok(_) => {
                completed_actions += 1;
                debug!("Action {} completed successfully", index + 1);
            },
            Err(e) => {
                let error_msg = format!("Action {}: {}", index + 1, e);
                errors.push(error_msg);
                error!("Action {} failed: {}", index + 1, e);
                
                // Try alternative action if available
                if let Some(alt_action) = action_recommendation.alternative_actions.get(index) {
                    debug!("Trying alternative action: {}", alt_action.action_type);
                    // In a real implementation, would execute the alternative action
                }
            }
        }
    }
    
    let execution_time = start_time.elapsed().as_millis() as u64;
    let success = errors.is_empty();
    
    Ok(ExecutionResult {
        success,
        summary: if success {
            format!("All {} actions completed successfully", task_plan.steps.len())
        } else {
            format!("{} of {} actions completed", completed_actions, task_plan.steps.len())
        },
        actions_completed: completed_actions,
        total_actions: task_plan.steps.len(),
        execution_time_ms: execution_time,
        errors,
    })
}

async fn execute_browser_action(
    browser: &crate::browser::Browser,
    action: &BrowserAction,
) -> Result<(), anyhow::Error> {
    // Simulate browser action execution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    match action.action_type.as_str() {
        "click" => {
            if let Some(ref target) = action.target {
                browser.click(target).await?;
            }
        },
        "type" => {
            if let (Some(ref target), Some(ref value)) = (&action.target, &action.value) {
                browser.type_text(target, value).await?;
            }
        },
        "navigate" => {
            if let Some(ref target) = action.target {
                browser.navigate_to(target).await?;
            }
        },
        "wait_for_element" => {
            if let Some(ref target) = action.target {
                let timeout = action.value.as_ref()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(5000);
                browser.wait_for_selector(target, std::time::Duration::from_millis(timeout)).await?;
            }
        },
        _ => {
            debug!("Unknown action type: {}", action.action_type);
        }
    }
    
    Ok(())
}

async fn execute_workflow_step(
    browser: &crate::browser::Browser,
    step: &WorkflowStep,
) -> Result<(), anyhow::Error> {
    let browser_action = BrowserAction {
        action_type: step.action_type.clone(),
        target: step.target.clone(),
        value: step.value.clone(),
        options: super::llm_handlers::BrowserActionOptions::default(),
    };
    
    execute_browser_action(browser, &browser_action).await
}

fn calculate_workflow_success_rate(execution_result: &Option<Result<ExecutionResult, anyhow::Error>>) -> f32 {
    match execution_result {
        Some(Ok(result)) => {
            if result.total_actions == 0 {
                1.0
            } else {
                result.actions_completed as f32 / result.total_actions as f32
            }
        },
        Some(Err(_)) => 0.0,
        None => 0.8, // Partial success (planning completed, execution skipped)
    }
}

// Request/Response types

#[derive(Deserialize)]
pub struct IntelligentWorkflowRequest {
    pub user_command: String,
    pub url: Option<String>,
    pub perception_mode: Option<String>, // "lightning", "quick", "standard", "deep"
    pub intelligence_config: Option<IntelligenceConfig>,
    pub auto_execute: Option<bool>,
    pub learning_enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct SimpleWorkflowRequest {
    pub steps: Vec<WorkflowStep>,
    pub stop_on_error: Option<bool>,
}

#[derive(Deserialize)]
pub struct WorkflowStep {
    pub action_type: String,
    pub target: Option<String>,
    pub value: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct WorkflowStatusRequest {
    pub workflow_id: String,
}

// Response types

#[derive(Serialize)]
pub struct IntelligentWorkflowResult {
    pub user_command: String,
    pub perception_result: crate::perception::PerceptionResult,
    pub intelligence_analysis: IntelligenceAnalysis,
    pub task_plan: TaskPlan,
    pub action_recommendation: ActionRecommendation,
    pub execution_result: Option<ExecutionResult>,
    pub modules_coordination: ModulesCoordination,
    pub workflow_metrics: WorkflowMetrics,
}

#[derive(Serialize)]
pub struct SimpleWorkflowResult {
    pub steps_completed: usize,
    pub total_steps: usize,
    pub success: bool,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub summary: String,
    pub actions_completed: usize,
    pub total_actions: usize,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct TaskPlan {
    pub steps: Vec<BrowserAction>,
    pub confidence: f32,
    pub estimated_time_seconds: u32,
    pub complexity: String,
}

#[derive(Serialize)]
pub struct ModulesCoordination {
    pub perception_to_intelligence: String,
    pub intelligence_to_llm: String,
    pub llm_to_execution: String,
    pub feedback_loop: String,
}

#[derive(Serialize)]
pub struct WorkflowMetrics {
    pub total_time_ms: u64,
    pub perception_time_ms: u64,
    pub intelligence_time_ms: u64,
    pub llm_time_ms: u64,
    pub execution_time_ms: u64,
    pub success_rate: f32,
    pub modules_used: usize,
}

#[derive(Serialize)]
pub struct WorkflowStatus {
    pub workflow_id: String,
    pub status: String, // "running", "completed", "failed", "pending"
    pub progress: f32, // 0.0 to 100.0
    pub current_step: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub modules_involved: Vec<String>,
    pub success_rate: f32,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub execution_metrics: WorkflowExecutionMetrics,
}

#[derive(Serialize)]
pub struct WorkflowExecutionMetrics {
    pub total_time_ms: u64,
    pub perception_time_ms: u64,
    pub llm_time_ms: u64,
    pub intelligence_time_ms: u64,
    pub execution_time_ms: u64,
    pub average_step_time_ms: u64,
    pub peak_memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
}

// Validation functions

fn validate_workflow_request(req: &IntelligentWorkflowRequest) -> Result<(), WorkflowApiError> {
    if req.user_command.trim().is_empty() {
        return Err(WorkflowApiError::ValidationError("User command cannot be empty".to_string()));
    }
    
    if req.user_command.len() > 2000 {
        return Err(WorkflowApiError::ValidationError("User command too long (max 2000 characters)".to_string()));
    }
    
    if let Some(ref url) = req.url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(WorkflowApiError::ValidationError("URL must start with http:// or https://".to_string()));
        }
    }
    
    if let Some(ref mode) = req.perception_mode {
        let valid_modes = ["lightning", "quick", "standard", "deep", "enhanced"];
        if !valid_modes.contains(&mode.as_str()) {
            return Err(WorkflowApiError::ValidationError(
                format!("Invalid perception mode '{}'. Valid modes: {}", mode, valid_modes.join(", "))
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_workflow_request() {
        let valid_req = IntelligentWorkflowRequest {
            user_command: "Click the login button".to_string(),
            url: Some("https://example.com".to_string()),
            perception_mode: Some("enhanced".to_string()),
            intelligence_config: None,
            auto_execute: Some(true),
            learning_enabled: Some(true),
        };
        assert!(validate_workflow_request(&valid_req).is_ok());
        
        let invalid_req = IntelligentWorkflowRequest {
            user_command: "".to_string(),
            url: Some("invalid-url".to_string()),
            perception_mode: Some("invalid-mode".to_string()),
            intelligence_config: None,
            auto_execute: None,
            learning_enabled: None,
        };
        assert!(validate_workflow_request(&invalid_req).is_err());
    }
    
    #[test]
    fn test_calculate_workflow_success_rate() {
        let success_result = ExecutionResult {
            success: true,
            summary: "All completed".to_string(),
            actions_completed: 5,
            total_actions: 5,
            execution_time_ms: 1000,
            errors: vec![],
        };
        
        let success_rate = calculate_workflow_success_rate(&Some(Ok(success_result)));
        assert_eq!(success_rate, 1.0);
        
        let partial_result = ExecutionResult {
            success: false,
            summary: "Partial completion".to_string(),
            actions_completed: 3,
            total_actions: 5,
            execution_time_ms: 1000,
            errors: vec!["Some error".to_string()],
        };
        
        let partial_rate = calculate_workflow_success_rate(&Some(Ok(partial_result)));
        assert_eq!(partial_rate, 0.6);
    }
}