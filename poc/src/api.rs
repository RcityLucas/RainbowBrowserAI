use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing::info;
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;

use crate::{
    SimpleBrowser, BrowserPool, LLMService, WorkflowEngine, Workflow,
    MetricsCollector, SecurityMiddleware, Config, CostTracker,
    ParsedCommand, ScreenshotOptions,
};

/// API state shared across handlers
#[derive(Clone)]
pub struct ApiState {
    pub browser_pool: Arc<BrowserPool>,
    pub llm_service: Arc<LLMService>,
    pub metrics: Arc<MetricsCollector>,
    pub security: Arc<SecurityMiddleware>,
    pub config: Arc<Config>,
    pub cost_tracker: Arc<RwLock<CostTracker>>,
    pub sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
}

/// Browser session for stateful operations
pub struct BrowserSession {
    pub id: String,
    pub browser: Arc<SimpleBrowser>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub details: Option<String>,
    pub code: u16,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Convert anyhow errors to API errors
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError {
            error: "Internal server error".to_string(),
            details: Some(err.to_string()),
            code: 500,
        }
    }
}

/// Convert serde_yaml errors to API errors
impl From<serde_yaml::Error> for ApiError {
    fn from(err: serde_yaml::Error) -> Self {
        ApiError {
            error: "YAML parsing error".to_string(),
            details: Some(err.to_string()),
            code: 400,
        }
    }
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct NavigateRequest {
    pub url: String,
    pub screenshot: Option<bool>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NavigateResponse {
    pub success: bool,
    pub title: Option<String>,
    pub screenshot_path: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NaturalLanguageRequest {
    pub command: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NaturalLanguageResponse {
    pub success: bool,
    pub action: String,
    pub confidence: f32,
    pub result: Option<serde_json::Value>,
    pub explanation: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRequest {
    pub workflow: serde_json::Value,
    pub inputs: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub steps_executed: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ScreenshotRequest {
    pub url: Option<String>,
    pub full_page: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotResponse {
    pub success: bool,
    pub path: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionRequest {
    pub action: String, // "create", "destroy", "list"
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: Option<String>,
    pub sessions: Option<Vec<SessionInfo>>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: String,
    pub last_used: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub operations_total: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub active_browsers: usize,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Serialize)]
pub struct CostResponse {
    pub daily_budget: f64,
    pub spent_today: f64,
    pub remaining: f64,
    pub operations: Vec<CostOperation>,
}

#[derive(Debug, Serialize)]
pub struct CostOperation {
    pub operation: String,
    pub cost: f64,
    pub timestamp: String,
}

// API Handlers

/// Health check endpoint
pub async fn health_handler(State(state): State<ApiState>) -> Result<Json<HealthResponse>, ApiError> {
    let sessions = state.sessions.read().await;
    let metrics = state.metrics.get_metrics().await;
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: metrics.uptime_seconds,
        active_sessions: sessions.len(),
    }))
}

/// Navigate to URL
pub async fn navigate_handler(
    State(state): State<ApiState>,
    Json(req): Json<NavigateRequest>,
) -> Result<Json<NavigateResponse>, ApiError> {
    // Validate input
    let safe_url = state.security.validate_url(&req.url)
        .map_err(|e| ApiError {
            error: "Invalid URL".to_string(),
            details: Some(e.to_string()),
            code: 400,
        })?;
    
    // Rate limiting
    state.security.check_request("api").await
        .map_err(|_| ApiError {
            error: "Rate limit exceeded".to_string(),
            details: None,
            code: 429,
        })?;
    
    // Get or create browser
    let (browser, session_id) = if let Some(sid) = req.session_id {
        // Use existing session
        let sessions = state.sessions.read().await;
        if let Some(session) = sessions.get(&sid) {
            (session.browser.clone(), Some(sid))
        } else {
            return Err(ApiError {
                error: "Session not found".to_string(),
                details: None,
                code: 404,
            });
        }
    } else {
        // Create new browser
        (Arc::new(SimpleBrowser::new().await?), None)
    };
    
    // Navigate
    browser.navigate_to(&safe_url.to_string()).await?;
    
    let title = browser.get_title().await.ok();
    
    // Take screenshot if requested
    let screenshot_path = if req.screenshot.unwrap_or(false) {
        let filename = format!("api_{}.png", Uuid::new_v4());
        browser.take_screenshot(&filename).await?;
        Some(format!("screenshots/{}", filename))
    } else {
        None
    };
    
    // Record metrics
    state.metrics.record_operation(
        std::time::Duration::from_millis(100),
        true,
        0.001
    ).await;
    
    Ok(Json(NavigateResponse {
        success: true,
        title,
        screenshot_path,
        session_id,
    }))
}

/// Process natural language command
pub async fn natural_language_handler(
    State(state): State<ApiState>,
    Json(req): Json<NaturalLanguageRequest>,
) -> Result<Json<NaturalLanguageResponse>, ApiError> {
    // Rate limiting
    state.security.check_request("api").await
        .map_err(|_| ApiError {
            error: "Rate limit exceeded".to_string(),
            details: None,
            code: 429,
        })?;
    
    // Check if API key is configured
    if state.llm_service.api_key.is_empty() {
        return Err(ApiError {
            error: "OpenAI API key not configured".to_string(),
            details: Some("Please set the OPENAI_API_KEY environment variable or configure it in the settings".to_string()),
            code: 503,
        });
    }
    
    // Parse command
    let mut cost_tracker = state.cost_tracker.write().await;
    let parsed = state.llm_service.parse_natural_command(&req.command, &mut cost_tracker).await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("401") {
                ApiError {
                    error: "Invalid OpenAI API key".to_string(),
                    details: Some("The provided API key is invalid or expired. Please check your configuration.".to_string()),
                    code: 401,
                }
            } else if error_msg.contains("429") {
                ApiError {
                    error: "OpenAI rate limit exceeded".to_string(),
                    details: Some("Too many requests to OpenAI. Please try again later.".to_string()),
                    code: 429,
                }
            } else {
                ApiError {
                    error: "Failed to process natural language command".to_string(),
                    details: Some(error_msg),
                    code: 500,
                }
            }
        })?;
    
    // Generate explanation
    let explanation = state.llm_service.explain_command(&parsed).await;
    
    // Execute command (simplified)
    let result = execute_parsed_command(state.clone(), &parsed, req.session_id).await?;
    
    Ok(Json(NaturalLanguageResponse {
        success: true,
        action: parsed.action,
        confidence: parsed.confidence,
        result: Some(result),
        explanation,
    }))
}

/// Execute workflow
pub async fn workflow_handler(
    State(state): State<ApiState>,
    Json(req): Json<WorkflowRequest>,
) -> Result<Json<WorkflowResponse>, ApiError> {
    // Rate limiting
    state.security.check_request("api").await
        .map_err(|_| ApiError {
            error: "Rate limit exceeded".to_string(),
            details: None,
            code: 429,
        })?;
    
    // Parse workflow
    let workflow: Workflow = serde_json::from_value(req.workflow)
        .map_err(|e| ApiError {
            error: "Invalid workflow format".to_string(),
            details: Some(e.to_string()),
            code: 400,
        })?;
    
    // Validate workflow
    let workflow_yaml = serde_yaml::to_string(&workflow)?;
    state.security.validate_workflow(&workflow_yaml)
        .map_err(|e| ApiError {
            error: "Workflow validation failed".to_string(),
            details: Some(e.to_string()),
            code: 400,
        })?;
    
    // Execute workflow
    let start = std::time::Instant::now();
    let mut engine = WorkflowEngine::new_simple();
    
    // Set input variables
    if let Some(inputs) = req.inputs {
        for (key, value) in inputs {
            engine.set_variable(&key, value).await;
        }
    }
    
    let result = engine.execute(&workflow).await?;
    let duration = start.elapsed();
    
    // Record metrics
    state.metrics.record_operation(duration, true, 0.01).await;
    
    Ok(Json(WorkflowResponse {
        success: result.success,
        result: if result.success {
            Some(serde_json::json!(result.variables))
        } else {
            None
        },
        steps_executed: result.steps_executed,
        duration_ms: duration.as_millis() as u64,
    }))
}

/// Take screenshot
pub async fn screenshot_handler(
    State(state): State<ApiState>,
    Json(req): Json<ScreenshotRequest>,
) -> Result<Json<ScreenshotResponse>, ApiError> {
    // Rate limiting
    state.security.check_request("api").await
        .map_err(|_| ApiError {
            error: "Rate limit exceeded".to_string(),
            details: None,
            code: 429,
        })?;
    
    // Get browser
    let browser = if let Some(sid) = req.session_id {
        // Use session browser
        let sessions = state.sessions.read().await;
        sessions.get(&sid)
            .map(|s| s.browser.clone())
            .ok_or_else(|| ApiError {
                error: "Session not found or browser unavailable".to_string(),
                details: None,
                code: 404,
            })?
    } else {
        // Create new browser
        Arc::new(SimpleBrowser::new().await?)
    };
    
    // Navigate if URL provided
    let url = if let Some(url) = req.url {
        let safe_url = state.security.validate_url(&url)?;
        browser.navigate_to(&safe_url.to_string()).await?;
        safe_url.to_string()
    } else {
        browser.get_url().await?
    };
    
    // Take screenshot
    let filename = format!("api_{}.png", Uuid::new_v4());
    let options = ScreenshotOptions {
        full_page: req.full_page.unwrap_or(true),
        viewport_width: req.width.unwrap_or(1920),
        viewport_height: req.height.unwrap_or(1080),
        wait_after_load: std::time::Duration::from_secs(2),
    };
    
    browser.take_screenshot_with_options(&filename, &options).await?;
    
    Ok(Json(ScreenshotResponse {
        success: true,
        path: format!("screenshots/{}", filename),
        url,
    }))
}

/// Manage browser sessions
pub async fn session_handler(
    State(state): State<ApiState>,
    Json(req): Json<SessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    match req.action.as_str() {
        "create" => {
            let session_id = Uuid::new_v4().to_string();
            let browser = SimpleBrowser::new().await?;
            
            let session = BrowserSession {
                id: session_id.clone(),
                browser: Arc::new(browser),
                created_at: chrono::Utc::now(),
                last_used: chrono::Utc::now(),
            };
            
            let mut sessions = state.sessions.write().await;
            sessions.insert(session_id.clone(), session);
            
            Ok(Json(SessionResponse {
                session_id: Some(session_id),
                sessions: None,
                message: "Session created".to_string(),
            }))
        }
        "destroy" => {
            if let Some(sid) = req.session_id {
                let mut sessions = state.sessions.write().await;
                if sessions.remove(&sid).is_some() {
                    // Browser will be cleaned up when Arc drops
                    Ok(Json(SessionResponse {
                        session_id: None,
                        sessions: None,
                        message: "Session destroyed".to_string(),
                    }))
                } else {
                    Err(ApiError {
                        error: "Session not found".to_string(),
                        details: None,
                        code: 404,
                    })
                }
            } else {
                Err(ApiError {
                    error: "Session ID required".to_string(),
                    details: None,
                    code: 400,
                })
            }
        }
        "list" => {
            let sessions = state.sessions.read().await;
            let session_list: Vec<SessionInfo> = sessions.values()
                .map(|s| SessionInfo {
                    id: s.id.clone(),
                    created_at: s.created_at.to_rfc3339(),
                    last_used: s.last_used.to_rfc3339(),
                })
                .collect();
            
            Ok(Json(SessionResponse {
                session_id: None,
                sessions: Some(session_list),
                message: format!("{} active sessions", sessions.len()),
            }))
        }
        _ => Err(ApiError {
            error: "Invalid action".to_string(),
            details: Some("Valid actions: create, destroy, list".to_string()),
            code: 400,
        })
    }
}

/// Get metrics
pub async fn metrics_handler(State(state): State<ApiState>) -> Result<Json<MetricsResponse>, ApiError> {
    let metrics = state.metrics.get_metrics().await;
    let summary = state.metrics.get_summary().await;
    
    Ok(Json(MetricsResponse {
        operations_total: metrics.operations_total,
        success_rate: metrics.success_rate(),
        avg_response_time_ms: summary.avg_response_time_ms,
        active_browsers: metrics.active_browsers,
        memory_usage_mb: metrics.memory_usage_mb,
    }))
}

/// Get cost report
pub async fn cost_handler(State(state): State<ApiState>) -> Result<Json<CostResponse>, ApiError> {
    let tracker = state.cost_tracker.read().await;
    let _report = tracker.generate_daily_report();
    
    // Parse report into structured response
    Ok(Json(CostResponse {
        daily_budget: tracker.daily_budget,
        spent_today: tracker.get_daily_total(),
        remaining: tracker.daily_budget - tracker.get_daily_total(),
        operations: tracker.operations.iter()
            .map(|op| CostOperation {
                operation: op.operation_type.clone(),
                cost: op.cost,
                timestamp: op.timestamp.to_rfc3339(),
            })
            .collect(),
    }))
}

// Helper functions

async fn execute_parsed_command(
    state: ApiState,
    command: &ParsedCommand,
    session_id: Option<String>,
) -> Result<serde_json::Value> {
    match command.action.as_str() {
        "navigate" => {
            if let Some(url) = command.urls.first() {
                let req = NavigateRequest {
                    url: url.clone(),
                    screenshot: Some(command.parameters.take_screenshot),
                    session_id,
                };
                let response = navigate_handler(State(state), Json(req)).await?;
                Ok(serde_json::to_value(response.0)?)
            } else {
                Err(anyhow::anyhow!("No URL specified"))
            }
        }
        "test" => {
            let mut results = Vec::new();
            for url in &command.urls {
                let req = NavigateRequest {
                    url: url.clone(),
                    screenshot: Some(false),
                    session_id: session_id.clone(),
                };
                let response = navigate_handler(State(state.clone()), Json(req)).await;
                results.push(serde_json::json!({
                    "url": url,
                    "success": response.is_ok(),
                    "error": response.err().map(|e| e.error),
                }));
            }
            Ok(serde_json::json!({ "results": results }))
        }
        _ => Err(anyhow::anyhow!("Unsupported action: {}", command.action))
    }
}

/// Create and configure the API router
pub fn create_router(state: ApiState) -> Router {
    // Create the static file service for the dashboard
    let static_files = ServeDir::new("static");

    Router::new()
        // Health and metrics
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/cost", get(cost_handler))
        
        // Browser operations
        .route("/navigate", post(navigate_handler))
        .route("/screenshot", post(screenshot_handler))
        .route("/session", post(session_handler))
        
        // AI operations
        .route("/command", post(natural_language_handler))
        .route("/workflow", post(workflow_handler))
        
        // Serve static files and dashboard at root
        .nest_service("/", static_files)
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

/// Start the API server
pub async fn start_server(config: Config) -> Result<()> {
    let port = config.api.as_ref().and_then(|a| a.port).unwrap_or(3000);
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Starting API server on {}", addr);
    
    // Initialize components
    let browser_pool = Arc::new(BrowserPool::new());
    
    // Check for API key and warn if missing
    let api_key = config.llm.api_key.clone().unwrap_or_else(|| {
        tracing::warn!("No OpenAI API key found. Natural language commands will not work.");
        tracing::warn!("Set OPENAI_API_KEY environment variable or configure in settings.");
        String::new()
    });
    
    let llm_service = Arc::new(LLMService::new(api_key));
    let metrics = Arc::new(MetricsCollector::new());
    let security = Arc::new(SecurityMiddleware::new(Default::default()));
    let cost_tracker = Arc::new(RwLock::new(CostTracker::new(config.budget.daily_limit)));
    
    let state = ApiState {
        browser_pool,
        llm_service,
        metrics,
        security,
        config: Arc::new(config),
        cost_tracker,
        sessions: Arc::new(RwLock::new(HashMap::new())),
    };
    
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}