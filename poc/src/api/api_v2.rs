//! Enhanced API v2 with Natural Language Understanding
//! 
//! New API endpoints that leverage the enhanced instruction parser and semantic analyzer

use axum::{
    extract::{State, Json, Query},
    response::{Json as JsonResponse, IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::{
    browser::browser_pool::BrowserPool,
    enhanced_executor::{EnhancedExecutor, EnhancedCommandProcessor},
    instruction_parser::Feedback,
    semantic_analyzer::PageType,
};

/// Enhanced API state
pub struct ApiV2State {
    pub browser_pool: Arc<BrowserPool>,
    pub executor: Arc<EnhancedExecutor>,
    pub processor: Arc<EnhancedCommandProcessor>,
}

impl ApiV2State {
    pub fn new(browser_pool: Arc<BrowserPool>) -> Self {
        Self {
            browser_pool,
            executor: Arc::new(EnhancedExecutor::new()),
            processor: Arc::new(EnhancedCommandProcessor::new()),
        }
    }
}

// Request/Response structures

#[derive(Debug, Deserialize)]
pub struct NaturalLanguageRequest {
    pub instruction: String,
    pub session_id: Option<String>,
    pub context: Option<RequestContext>,
}

#[derive(Debug, Deserialize)]
pub struct RequestContext {
    pub current_url: Option<String>,
    pub page_type: Option<String>,
    pub user_goal: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NaturalLanguageResponse {
    pub success: bool,
    pub result: Option<ExecutionResult>,
    pub clarification_needed: Option<Vec<String>>,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub intent: String,
    pub confidence: f32,
    pub actions_performed: Vec<String>,
    pub data_extracted: Option<serde_json::Value>,
    pub screenshot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRequest {
    pub instructions: Vec<String>,
    pub session_id: Option<String>,
    pub stop_on_error: bool,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub success: bool,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub step_results: Vec<StepResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StepResult {
    pub step_number: usize,
    pub instruction: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzePageRequest {
    pub url: String,
    pub deep_analysis: bool,
}

#[derive(Debug, Serialize)]
pub struct AnalyzePageResponse {
    pub page_type: String,
    pub regions_found: Vec<String>,
    pub interaction_points: usize,
    pub semantic_elements: usize,
    pub suggestions: Vec<String>,
    pub data_structures: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub session_id: String,
    pub instruction_id: String,
    pub feedback_type: String, // "success", "failure", "partial"
    pub corrections: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct FeedbackResponse {
    pub acknowledged: bool,
    pub learning_applied: bool,
}

// API Handlers

/// Execute a natural language instruction
pub async fn execute_natural_language(
    State(state): State<Arc<ApiV2State>>,
    Json(request): Json<NaturalLanguageRequest>,
) -> impl IntoResponse {
    info!("Natural language request: {}", request.instruction);
    
    // Get browser from pool
    let browser_handle = match state.browser_pool.acquire().await {
        Ok(handle) => handle,
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(NaturalLanguageResponse {
                    success: false,
                    result: None,
                    clarification_needed: None,
                    suggestions: vec![],
                    error: Some(format!("Browser unavailable: {}", e)),
                }),
            );
        }
    };
    
    let browser = match browser_handle.browser() {
        Some(browser) => browser,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(NaturalLanguageResponse {
                    success: false,
                    result: None,
                    clarification_needed: None,
                    suggestions: vec![],
                    error: Some("Browser not available in handle".to_string()),
                }),
            );
        }
    };
    
    // Execute instruction
    match state.executor.execute_instruction(
        &request.instruction,
        browser,
        None, // Could pass conversation context here
    ).await {
        Ok(result) => {
            let response = NaturalLanguageResponse {
                success: result.success,
                result: if result.success {
                    Some(ExecutionResult {
                        intent: format!("{:?}", result.instruction.as_ref().map(|i| &i.intent)),
                        confidence: result.instruction.as_ref().map(|i| i.confidence).unwrap_or(0.0),
                        actions_performed: result.actions_executed.iter()
                            .map(|a| format!("{:?}", a.action.action_type))
                            .collect(),
                        data_extracted: result.actions_executed.first()
                            .and_then(|a| a.data.clone()),
                        screenshot: result.actions_executed.first()
                            .and_then(|a| a.screenshot_path.clone()),
                    })
                } else {
                    None
                },
                clarification_needed: result.clarification_needed,
                suggestions: result.suggestions,
                error: result.error,
            };
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Execution failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(NaturalLanguageResponse {
                    success: false,
                    result: None,
                    clarification_needed: None,
                    suggestions: vec![],
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

/// Execute a workflow of instructions
pub async fn execute_workflow(
    State(state): State<Arc<ApiV2State>>,
    Json(request): Json<WorkflowRequest>,
) -> impl IntoResponse {
    info!("Workflow request with {} steps", request.instructions.len());
    
    let browser_handle = match state.browser_pool.acquire().await {
        Ok(handle) => handle,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(WorkflowResponse {
                    success: false,
                    steps_completed: 0,
                    total_steps: request.instructions.len(),
                    step_results: vec![],
                    error: Some(format!("Browser unavailable: {}", e)),
                }),
            );
        }
    };
    
    let browser = match browser_handle.browser() {
        Some(browser) => browser,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WorkflowResponse {
                    success: false,
                    steps_completed: 0,
                    total_steps: request.instructions.len(),
                    step_results: vec![],
                    error: Some("Browser not available in handle".to_string()),
                }),
            );
        }
    };
    
    match state.executor.execute_workflow(
        request.instructions.clone(),
        browser,
        None,
    ).await {
        Ok(result) => {
            let step_results = result.step_results.iter().enumerate().map(|(i, r)| {
                StepResult {
                    step_number: i + 1,
                    instruction: request.instructions.get(i)
                        .cloned()
                        .unwrap_or_default(),
                    success: r.success,
                    error: r.error.clone(),
                }
            }).collect();
            
            (
                StatusCode::OK,
                Json(WorkflowResponse {
                    success: result.success,
                    steps_completed: result.steps_completed,
                    total_steps: result.total_steps,
                    step_results,
                    error: None,
                }),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(WorkflowResponse {
                    success: false,
                    steps_completed: 0,
                    total_steps: request.instructions.len(),
                    step_results: vec![],
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

/// Analyze a webpage semantically
pub async fn analyze_page(
    State(state): State<Arc<ApiV2State>>,
    Json(request): Json<AnalyzePageRequest>,
) -> impl IntoResponse {
    info!("Analyzing page: {}", request.url);
    
    let browser_handle = match state.browser_pool.acquire().await {
        Ok(handle) => handle,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(AnalyzePageResponse {
                    page_type: "unknown".to_string(),
                    regions_found: vec![],
                    interaction_points: 0,
                    semantic_elements: 0,
                    suggestions: vec![],
                    data_structures: vec![],
                }),
            );
        }
    };
    
    let browser = match browser_handle.browser() {
        Some(browser) => browser,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AnalyzePageResponse {
                    page_type: "error".to_string(),
                    regions_found: vec!["Browser not available in handle".to_string()],
                    interaction_points: 0,
                    semantic_elements: 0,
                    suggestions: vec![],
                    data_structures: vec![],
                }),
            );
        }
    };
    
    // Navigate to URL
    if let Err(e) = browser.navigate_to(&request.url).await {
        return (
            StatusCode::BAD_REQUEST,
            Json(AnalyzePageResponse {
                page_type: "error".to_string(),
                regions_found: vec![format!("Failed to navigate: {}", e)],
                interaction_points: 0,
                semantic_elements: 0,
                suggestions: vec![],
                data_structures: vec![],
            }),
        );
    }
    
    // Analyze page
    let analyzer = crate::semantic_analyzer::SemanticAnalyzer::new(browser.driver());
    
    match analyzer.analyze().await {
        Ok(page_model) => {
            let regions_found = page_model.regions.iter().map(|r| {
                format!("{:?}", std::mem::discriminant(r))
            }).collect();
            
            let data_structures = page_model.data_structures.iter().map(|d| {
                format!("{}: {:?}", d.name, d.structure_type)
            }).collect();
            
            // Get suggestions
            let suggestions = match state.executor.get_suggestions(browser).await {
                Ok(sugg) => sugg,
                Err(_) => vec![],
            };
            
            (
                StatusCode::OK,
                Json(AnalyzePageResponse {
                    page_type: format!("{:?}", page_model.page_type),
                    regions_found,
                    interaction_points: page_model.interaction_points.len(),
                    semantic_elements: page_model.semantic_elements.len(),
                    suggestions,
                    data_structures,
                }),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AnalyzePageResponse {
                    page_type: "error".to_string(),
                    regions_found: vec![format!("Analysis failed: {}", e)],
                    interaction_points: 0,
                    semantic_elements: 0,
                    suggestions: vec![],
                    data_structures: vec![],
                }),
            )
        }
    }
}

/// Provide feedback on instruction execution
pub async fn provide_feedback(
    State(state): State<Arc<ApiV2State>>,
    Json(request): Json<FeedbackRequest>,
) -> impl IntoResponse {
    info!("Feedback received for session {}: {}", 
        request.session_id, request.feedback_type);
    
    let feedback = match request.feedback_type.as_str() {
        "success" => Feedback::Success,
        "failure" => {
            // Would need to provide corrected intent
            Feedback::Success // Placeholder
        }
        "partial" => {
            if let Some(corrections) = request.corrections {
                Feedback::PartialSuccess { improvements: corrections }
            } else {
                Feedback::Success
            }
        }
        _ => Feedback::Success,
    };
    
    match state.executor.provide_feedback(feedback).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(FeedbackResponse {
                    acknowledged: true,
                    learning_applied: true,
                }),
            )
        }
        Err(e) => {
            error!("Failed to apply feedback: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FeedbackResponse {
                    acknowledged: false,
                    learning_applied: false,
                }),
            )
        }
    }
}

/// Get suggestions for the current page
pub async fn get_suggestions(
    State(state): State<Arc<ApiV2State>>,
    Query(params): Query<SessionParams>,
) -> impl IntoResponse {
    info!("Getting suggestions for session: {:?}", params.session_id);
    
    let browser_handle = match state.browser_pool.acquire().await {
        Ok(handle) => handle,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(SuggestionsResponse {
                    suggestions: vec![],
                    page_type: None,
                    error: Some(format!("Browser unavailable: {}", e)),
                }),
            );
        }
    };
    
    let browser = match browser_handle.browser() {
        Some(browser) => browser,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SuggestionsResponse {
                    suggestions: vec![],
                    page_type: None,
                    error: Some("Browser not available in handle".to_string()),
                }),
            );
        }
    };
    
    match state.executor.get_suggestions(browser).await {
        Ok(suggestions) => {
            // Also get page type
            let analyzer = crate::semantic_analyzer::SemanticAnalyzer::new(browser.driver());
            let page_type = analyzer.analyze().await
                .map(|m| format!("{:?}", m.page_type))
                .ok();
            
            (
                StatusCode::OK,
                Json(SuggestionsResponse {
                    suggestions,
                    page_type,
                    error: None,
                }),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SuggestionsResponse {
                    suggestions: vec![],
                    page_type: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SessionParams {
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<String>,
    pub page_type: Option<String>,
    pub error: Option<String>,
}

/// Create V2 API routes
pub fn create_v2_routes() -> axum::Router<Arc<ApiV2State>> {
    use axum::routing::{post, get};
    
    axum::Router::new()
        .route("/v2/execute", post(execute_natural_language))
        .route("/v2/workflow", post(execute_workflow))
        .route("/v2/analyze", post(analyze_page))
        .route("/v2/feedback", post(provide_feedback))
        .route("/v2/suggestions", get(get_suggestions))
}

/// Health check for V2 API
pub async fn health_check_v2() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "2.0",
        "features": [
            "natural_language_understanding",
            "semantic_page_analysis",
            "context_aware_execution",
            "workflow_automation",
            "learning_from_feedback"
        ]
    }))
}