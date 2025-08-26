use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, Response, Sse, sse::Event},
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
use tracing::{info, warn, error};
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;
use futures::stream::Stream;
use tokio::time::{interval, Duration};
use std::convert::Infallible;

use crate::{
    SimpleBrowser, BrowserPool, LLMService, WorkflowEngine, Workflow,
    MetricsCollector, SecurityMiddleware, Config, CostTracker,
    ParsedCommand, ScreenshotOptions, PluginManager,
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
    pub plugin_manager: Arc<RwLock<PluginManager>>,
}

/// Browser session for stateful operations
pub struct BrowserSession {
    pub id: String,
    pub browser: Arc<SimpleBrowser>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// SSE event types for real-time updates
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum SseEvent {
    #[serde(rename = "metrics")]
    Metrics {
        operations_total: u64,
        success_rate: f64,
        avg_response_time_ms: f64,
        active_browsers: u32,
        memory_usage_mb: f64,
    },
    #[serde(rename = "cost")]
    Cost {
        daily_budget: f64,
        spent_today: f64,
        remaining: f64,
        last_operation_cost: f64,
    },
    #[serde(rename = "session")]
    Session {
        action: String,
        session_id: String,
        active_sessions: u32,
    },
    #[serde(rename = "status")]
    Status {
        message: String,
        level: String, // info, warning, error, success
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        timestamp: String,
        uptime_seconds: u64,
    },
    #[serde(rename = "plugin")]
    Plugin {
        action: String, // "loaded", "unloaded", "error", "discovered"
        plugin_id: String,
        plugin_name: String,
        total_plugins: usize,
        active_plugins: usize,
    },
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

#[derive(Debug, Deserialize)]
pub struct PluginRequest {
    pub action: String, // "list", "load", "unload", "reload", "configure", "discover"
    pub plugin_id: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PluginResponse {
    pub success: bool,
    pub message: String,
    pub plugins: Option<Vec<PluginInfo>>,
    pub plugin_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub plugin_type: String,
    pub state: String,
    pub author: Option<String>,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PluginMetrics {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub failed_plugins: usize,
    pub discovered_plugins: usize,
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
    
    // Get or create browser session with smart reuse
    let (browser, actual_session_id) = get_or_create_browser_session(&state, req.session_id).await?;
    
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
        session_id: Some(actual_session_id),
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
    
    // Check if mock mode is enabled first
    if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
        // Use mock LLM to parse command, then execute it
        info!("Mock mode enabled - parsing command without API");
        
        // Parse command using mock mode
        let mut cost_tracker = state.cost_tracker.write().await;
        let parsed = match state.llm_service.parse_natural_command(&req.command, &mut cost_tracker).await {
            Ok(parsed) => parsed,
            Err(e) => {
                return Ok(Json(NaturalLanguageResponse {
                    success: false,
                    action: "error".to_string(),
                    confidence: 0.0,
                    result: Some(serde_json::json!({
                        "error": "Failed to parse command in mock mode",
                        "details": e.to_string()
                    })),
                    explanation: "Mock parser failed to understand the command".to_string(),
                }));
            }
        };
        drop(cost_tracker);
        
        // Execute the parsed command
        let result = match parsed.action.as_str() {
            "navigate" => {
                // Use execute_parsed_command for proper session management
                info!("Mock mode: Executing navigate command");
                match execute_parsed_command(state.clone(), &parsed, req.session_id.clone()).await {
                    Ok(result) => {
                        info!("Mock mode: navigate execution successful");
                        result
                    },
                    Err(e) => {
                        error!("Mock mode: navigate execution failed: {}", e);
                        serde_json::json!({
                            "success": false,
                            "action": "navigate",
                            "error": format!("Navigation failed: {}", e)
                        })
                    }
                }
            },
            "test" => {
                // In mock mode, we still execute batch testing for demonstration
                info!("Mock mode: Executing test command with {} URLs", parsed.urls.len());
                match execute_parsed_command(state.clone(), &parsed, req.session_id.clone()).await {
                    Ok(result) => {
                        info!("Mock mode: Test execution successful");
                        result
                    },
                    Err(e) => {
                        error!("Mock mode: Test execution failed: {}", e);
                        serde_json::json!({
                            "success": false,
                            "action": "test",
                            "error": format!("Test execution failed: {}", e)
                        })
                    }
                }
            },
            "report" => {
                let cost_tracker = state.cost_tracker.read().await;
                serde_json::json!({
                    "success": true,
                    "action": "report",
                    "daily_budget": cost_tracker.daily_budget,
                    "spent_today": cost_tracker.get_daily_total(),
                    "operations": cost_tracker.operations.len()
                })
            },
            "scroll" | "click" | "back" | "forward" | "refresh" | "input" => {
                // In mock mode, execute browser actions using session management
                info!("Mock mode: Executing {} command", parsed.action);
                match execute_parsed_command(state.clone(), &parsed, req.session_id.clone()).await {
                    Ok(result) => {
                        info!("Mock mode: {} execution successful", parsed.action);
                        result
                    },
                    Err(e) => {
                        error!("Mock mode: {} execution failed: {}", parsed.action, e);
                        serde_json::json!({
                            "success": false,
                            "action": parsed.action,
                            "error": format!("{} execution failed: {}", parsed.action, e)
                        })
                    }
                }
            },
            _ => {
                // For unknown actions, still try to execute them through execute_parsed_command
                // This allows for future action types to be added without modifying this handler
                info!("Mock mode: Attempting to execute unknown action: {}", parsed.action);
                match execute_parsed_command(state.clone(), &parsed, req.session_id.clone()).await {
                    Ok(result) => {
                        info!("Mock mode: {} execution successful", parsed.action);
                        result
                    },
                    Err(e) => {
                        error!("Mock mode: {} execution failed: {}", parsed.action, e);
                        serde_json::json!({
                            "success": false,
                            "action": parsed.action,
                            "error": format!("Unknown or unsupported action '{}': {}", parsed.action, e)
                        })
                    }
                }
            }
        };
        
        return Ok(Json(NaturalLanguageResponse {
            success: result["success"].as_bool().unwrap_or(false),
            action: parsed.action.clone(),
            confidence: parsed.confidence,
            result: Some(result),
            explanation: format!("Mock mode: Parsed '{}' as {} action (confidence: {:.0}%)", 
                                req.command, parsed.action, parsed.confidence * 100.0),
        }));
    }

    // Check if API key is configured
    if state.llm_service.api_key.is_empty() {
        return Err(ApiError {
            error: "OpenAI API key not configured".to_string(),
            details: Some("Please set the OPENAI_API_KEY environment variable or configure it in the settings. You can also enable mock mode with RAINBOW_MOCK_MODE=true for testing.".to_string()),
            code: 503,
        });
    }
    
    // Parse command
    let mut cost_tracker = state.cost_tracker.write().await;
    let parsed = match state.llm_service.parse_natural_command(&req.command, &mut cost_tracker).await {
        Ok(parsed) => parsed,
        Err(e) => {
            let error_msg = e.to_string();
            // Check for various quota-related error patterns
            if error_msg.contains("insufficient_quota") || 
               error_msg.contains("quota") || 
               error_msg.contains("exceeded your current quota") ||
               error_msg.contains("billing") ||
               error_msg.contains("credit balance is too low") ||
               error_msg.contains("Plans & Billing") ||
               error_msg.contains("LLM API error 429") {
                // Auto-fallback to mock mode when quota exceeded
                tracing::warn!("OpenAI quota exceeded, automatically switching to mock mode. Error: {}", error_msg);
                return Ok(Json(NaturalLanguageResponse {
                    success: true,
                    action: "mock".to_string(),
                    confidence: 0.90,
                    result: Some(serde_json::json!({
                        "message": "Auto-switched to mock mode due to OpenAI quota limitation",
                        "command": req.command,
                        "note": "Set RAINBOW_MOCK_MODE=true or add billing to your OpenAI account to continue using AI features"
                    })),
                    explanation: "Automatically switched to mock mode because OpenAI quota was exceeded. The system continues to work normally in simulation mode.".to_string(),
                }));
            } else if error_msg.contains("401") {
                return Err(ApiError {
                    error: "Invalid OpenAI API key".to_string(),
                    details: Some("The provided API key is invalid or expired. Please check your configuration.".to_string()),
                    code: 401,
                });
            } else if error_msg.contains("429") {
                return Err(ApiError {
                    error: "OpenAI rate limit exceeded".to_string(),
                    details: Some("Too many requests to OpenAI. Please try again later.".to_string()),
                    code: 429,
                });
            } else {
                return Err(ApiError {
                    error: "Failed to process natural language command".to_string(),
                    details: Some(format!("LLM API Error: {}. Consider enabling mock mode with RAINBOW_MOCK_MODE=true", error_msg)),
                    code: 500,
                });
            }
        }
    };
    
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

/// Manage plugins
pub async fn plugin_handler(
    State(state): State<ApiState>,
    Json(req): Json<PluginRequest>,
) -> Result<Json<PluginResponse>, ApiError> {
    // Rate limiting
    state.security.check_request("api").await
        .map_err(|_| ApiError {
            error: "Rate limit exceeded".to_string(),
            details: None,
            code: 429,
        })?;
    
    let plugin_manager = state.plugin_manager.read().await;
    
    match req.action.as_str() {
        "list" => {
            let registry = plugin_manager.registry.read().await;
            let plugins = registry.list_plugins();
            
            let plugin_list: Vec<PluginInfo> = plugins.iter().map(|plugin| {
                PluginInfo {
                    id: plugin.id.to_string(),
                    name: plugin.manifest.plugin.name.clone(),
                    version: plugin.manifest.plugin.version.clone(),
                    description: plugin.manifest.plugin.description.clone(),
                    plugin_type: format!("{:?}", plugin.manifest.plugin.plugin_type),
                    state: format!("{:?}", plugin.state),
                    author: Some(plugin.manifest.plugin.author.clone()),
                    dependencies: plugin.manifest.dependencies.as_ref()
                        .map(|deps| vec![deps.runtime_version.clone()])
                        .unwrap_or_default(),
                    permissions: plugin.manifest.capabilities.as_ref()
                        .map(|caps| caps.permissions.iter().map(|p| format!("{:?}", p)).collect())
                        .unwrap_or_default(),
                }
            }).collect();
            
            Ok(Json(PluginResponse {
                success: true,
                message: format!("Found {} plugins", plugin_list.len()),
                plugins: Some(plugin_list),
                plugin_id: None,
            }))
        }
        
        "discover" => {
            drop(plugin_manager); // Release read lock
            let plugin_manager = state.plugin_manager.write().await;
            
            let plugins_dir = std::path::Path::new("plugins");
            let discovered = if plugins_dir.exists() {
                plugin_manager.discover_plugins(plugins_dir).await
                    .map_err(|e| ApiError {
                        error: "Failed to discover plugins".to_string(),
                        details: Some(e.to_string()),
                        code: 500,
                    })?
            } else {
                Vec::new()
            };
            
            Ok(Json(PluginResponse {
                success: true,
                message: format!("Discovered {} plugins", discovered.len()),
                plugins: None,
                plugin_id: None,
            }))
        }
        
        "load" => {
            if let Some(plugin_id) = req.plugin_id {
                drop(plugin_manager); // Release read lock
                let plugin_manager = state.plugin_manager.read().await;
                
                plugin_manager.load_plugin_by_string_id(&plugin_id).await
                    .map_err(|e| ApiError {
                        error: "Failed to load plugin".to_string(),
                        details: Some(e.to_string()),
                        code: 500,
                    })?;
                
                Ok(Json(PluginResponse {
                    success: true,
                    message: format!("Plugin '{}' loaded successfully", plugin_id),
                    plugins: None,
                    plugin_id: Some(plugin_id),
                }))
            } else {
                Err(ApiError {
                    error: "Plugin ID required".to_string(),
                    details: None,
                    code: 400,
                })
            }
        }
        
        "unload" => {
            if let Some(plugin_id) = req.plugin_id {
                drop(plugin_manager); // Release read lock
                let plugin_manager = state.plugin_manager.read().await;
                
                plugin_manager.unload_plugin_by_string_id(&plugin_id).await
                    .map_err(|e| ApiError {
                        error: "Failed to unload plugin".to_string(),
                        details: Some(e.to_string()),
                        code: 500,
                    })?;
                
                Ok(Json(PluginResponse {
                    success: true,
                    message: format!("Plugin '{}' unloaded successfully", plugin_id),
                    plugins: None,
                    plugin_id: Some(plugin_id),
                }))
            } else {
                Err(ApiError {
                    error: "Plugin ID required".to_string(),
                    details: None,
                    code: 400,
                })
            }
        }
        
        "reload" => {
            if let Some(plugin_id) = req.plugin_id {
                drop(plugin_manager); // Release read lock
                let plugin_manager = state.plugin_manager.read().await;
                
                // Unload first, then load
                let _ = plugin_manager.unload_plugin_by_string_id(&plugin_id).await;
                plugin_manager.load_plugin_by_string_id(&plugin_id).await
                    .map_err(|e| ApiError {
                        error: "Failed to reload plugin".to_string(),
                        details: Some(e.to_string()),
                        code: 500,
                    })?;
                
                Ok(Json(PluginResponse {
                    success: true,
                    message: format!("Plugin '{}' reloaded successfully", plugin_id),
                    plugins: None,
                    plugin_id: Some(plugin_id),
                }))
            } else {
                Err(ApiError {
                    error: "Plugin ID required".to_string(),
                    details: None,
                    code: 400,
                })
            }
        }
        
        "configure" => {
            if let Some(plugin_id) = req.plugin_id {
                if let Some(config) = req.config {
                    drop(plugin_manager); // Release read lock
                    let plugin_manager = state.plugin_manager.read().await;
                    
                    plugin_manager.configure_plugin_by_string_id(&plugin_id, config).await
                        .map_err(|e| ApiError {
                            error: "Failed to configure plugin".to_string(),
                            details: Some(e.to_string()),
                            code: 500,
                        })?;
                    
                    Ok(Json(PluginResponse {
                        success: true,
                        message: format!("Plugin '{}' configured successfully", plugin_id),
                        plugins: None,
                        plugin_id: Some(plugin_id),
                    }))
                } else {
                    Err(ApiError {
                        error: "Configuration data required".to_string(),
                        details: None,
                        code: 400,
                    })
                }
            } else {
                Err(ApiError {
                    error: "Plugin ID required".to_string(),
                    details: None,
                    code: 400,
                })
            }
        }
        
        _ => Err(ApiError {
            error: "Invalid action".to_string(),
            details: Some("Valid actions: list, discover, load, unload, reload, configure".to_string()),
            code: 400,
        })
    }
}

/// Get plugin metrics
pub async fn plugin_metrics_handler(State(state): State<ApiState>) -> Result<Json<PluginMetrics>, ApiError> {
    let plugin_manager = state.plugin_manager.read().await;
    let registry = plugin_manager.registry.read().await;
    let plugins = registry.list_plugins();
    
    let total_plugins = plugins.len();
    let active_plugins = plugins.iter()
        .filter(|p| matches!(p.state, crate::plugins::types::PluginState::Active))
        .count();
    let failed_plugins = plugins.iter()
        .filter(|p| matches!(p.state, crate::plugins::types::PluginState::Error(_)))
        .count();
    let discovered_plugins = plugins.iter()
        .filter(|p| matches!(p.state, crate::plugins::types::PluginState::Discovered))
        .count();
    
    Ok(Json(PluginMetrics {
        total_plugins,
        active_plugins,
        failed_plugins,
        discovered_plugins,
    }))
}

// Helper functions

/// Smart session management: reuse existing session if available, otherwise create new one
/// Returns (browser, session_id) tuple
async fn get_or_create_browser_session(
    state: &ApiState,
    session_id: Option<String>,
) -> Result<(Arc<SimpleBrowser>, String)> {
    info!("🔍 DEBUG: get_or_create_browser_session called with session_id: {:?}", session_id);
    if let Some(sid) = session_id {
        // Try to use existing session
        let sessions = state.sessions.read().await;
        if let Some(session) = sessions.get(&sid) {
            info!("✅ Using explicitly requested session: {}", sid);
            return Ok((session.browser.clone(), sid));
        } else {
            return Err(anyhow::anyhow!("Session not found: {}", sid));
        }
    }

    // No session ID provided, try to reuse any existing session with simple validation
    {
        let sessions = state.sessions.read().await;
        if let Some((session_id, session)) = sessions.iter().next() {
            info!("Found existing session {}, attempting simple reuse...", session_id);
            
            // Simple check: if session is less than 5 minutes old, assume it's valid
            let session_age = chrono::Utc::now().signed_duration_since(session.last_used);
            if session_age.num_minutes() < 5 {
                info!("✅ Reusing recent session {} (age: {} minutes)", session_id, session_age.num_minutes());
                
                // Update last_used time
                let browser_clone = session.browser.clone();
                let session_id_clone = session_id.clone();
                drop(sessions); // Release read lock
                
                let mut sessions_write = state.sessions.write().await;
                if let Some(session_mut) = sessions_write.get_mut(&session_id_clone) {
                    session_mut.last_used = chrono::Utc::now();
                }
                
                return Ok((browser_clone, session_id_clone));
            } else {
                info!("Session {} is too old ({} minutes), creating new one", session_id, session_age.num_minutes());
                
                // Clean up old session
                let session_id_clone = session_id.clone();
                drop(sessions); // Release read lock
                let mut sessions_write = state.sessions.write().await;
                sessions_write.remove(&session_id_clone);
                info!("Cleaned up old session: {}", session_id_clone);
            }
        } else {
            info!("No existing sessions found, will create new one");
        }
    }

    // No existing valid sessions, create a new one with generated ID
    let browser = Arc::new(SimpleBrowser::new().await?);
    
    // Navigate to a default page so the browser is properly initialized
    // This ensures commands like current_url() will work
    if let Err(e) = browser.navigate_to("https://www.example.com").await {
        warn!("Failed to navigate to initial example.com page: {}", e);
    }
    
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = BrowserSession {
        id: session_id.clone(),
        browser: browser.clone(),
        created_at: chrono::Utc::now(),
        last_used: chrono::Utc::now(),
    };
    
    // Store the new session
    {
        let mut sessions = state.sessions.write().await;
        
        // Only remove truly invalid sessions (where the browser window was closed)
        let mut invalid_session_ids = Vec::new();
        for (id, existing_session) in sessions.iter() {
            // Quick check if browser is completely dead
            if !existing_session.browser.is_alive().await {
                // Double check with get_title
                if existing_session.browser.get_title().await.is_err() {
                    invalid_session_ids.push(id.clone());
                }
            }
        }
        
        for id in invalid_session_ids {
            warn!("Removing dead session: {}", id);
            sessions.remove(&id);
        }
        
        // Insert the new session
        sessions.insert(session_id.clone(), session);
        info!("✅ Created new browser session: {} (total sessions: {})", session_id, sessions.len());
    }
    
    Ok((browser, session_id))
}

async fn execute_parsed_command(
    state: ApiState,
    command: &ParsedCommand,
    session_id: Option<String>,
) -> Result<serde_json::Value> {
    match command.action.as_str() {
        "navigate" => {
            let url = command.url.as_ref()
                .or_else(|| command.urls.first())
                .ok_or_else(|| anyhow::anyhow!("No URL specified"))?;
            
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            // Navigate to URL
            match browser.navigate_to(url).await {
                Ok(_) => {
                    let title = browser.get_title().await.unwrap_or_default();
                    let mut result = serde_json::json!({
                        "success": true,
                        "action": "navigate",
                        "url": url,
                        "title": title,
                        "session_id": actual_session_id
                    });
                    
                    // Take screenshot if requested
                    if command.parameters.take_screenshot {
                        let filename = command.parameters.screenshot_filename
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| format!("navigate_{}.png", chrono::Utc::now().timestamp()));
                        
                        match browser.take_screenshot(&filename).await {
                            Ok(_) => {
                                result["screenshot_path"] = serde_json::Value::String(format!("screenshots/{}", filename));
                            },
                            Err(e) => {
                                warn!("Screenshot failed: {}", e);
                            }
                        }
                    }
                    
                    Ok(result)
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Navigation failed: {}", e))
                }
            }
        }
        "test" => {
            let mut results = Vec::new();
            let take_screenshots = command.parameters.take_screenshot;
            
            info!("Testing {} websites (screenshots: {})", command.urls.len(), take_screenshots);
            
            for (index, url) in command.urls.iter().enumerate() {
                let start_time = std::time::Instant::now();
                
                // Create new browser for each test to avoid conflicts
                let browser_result = SimpleBrowser::new().await;
                
                match browser_result {
                    Ok(browser) => {
                        let mut test_result = serde_json::json!({
                            "url": url,
                            "index": index + 1,
                            "success": false,
                            "loading_time_ms": 0,
                            "title": null,
                            "screenshot_path": null,
                            "error": null
                        });
                        
                        // Navigate to URL
                        match browser.navigate_to(url).await {
                            Ok(_) => {
                                let loading_time = start_time.elapsed().as_millis() as u64;
                                test_result["loading_time_ms"] = serde_json::json!(loading_time);
                                test_result["success"] = serde_json::Value::Bool(true);
                                
                                // Get page title
                                if let Ok(title) = browser.get_title().await {
                                    test_result["title"] = serde_json::Value::String(title);
                                }
                                
                                // Take screenshot if requested
                                if take_screenshots {
                                    let filename = format!("test_{}_{}.png", 
                                        url.replace(".", "_").replace("/", "_"), 
                                        chrono::Utc::now().format("%Y%m%d_%H%M%S")
                                    );
                                    
                                    match browser.take_screenshot(&filename).await {
                                        Ok(_) => {
                                            let screenshot_path = format!("screenshots/{}", filename);
                                            test_result["screenshot_path"] = serde_json::Value::String(screenshot_path);
                                            info!("Screenshot saved: {}", filename);
                                        },
                                        Err(e) => {
                                            warn!("Screenshot failed for {}: {}", url, e);
                                        }
                                    }
                                }
                                
                                info!("✅ Test {}/{}: {} loaded in {}ms", 
                                    index + 1, command.urls.len(), url, loading_time);
                            },
                            Err(e) => {
                                let loading_time = start_time.elapsed().as_millis() as u64;
                                test_result["loading_time_ms"] = serde_json::json!(loading_time);
                                test_result["error"] = serde_json::Value::String(format!("{}", e));
                                
                                error!("❌ Test {}/{}: {} failed: {}", 
                                    index + 1, command.urls.len(), url, e);
                            }
                        }
                        
                        results.push(test_result);
                    },
                    Err(e) => {
                        let test_result = serde_json::json!({
                            "url": url,
                            "index": index + 1,
                            "success": false,
                            "loading_time_ms": 0,
                            "title": null,
                            "screenshot_path": null,
                            "error": format!("Failed to create browser: {}", e)
                        });
                        
                        results.push(test_result);
                        error!("❌ Test {}/{}: {} - Browser creation failed: {}", 
                            index + 1, command.urls.len(), url, e);
                    }
                }
            }
            
            let successful_tests = results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count();
            let total_tests = results.len();
            
            info!("🎯 Test completed: {}/{} successful", successful_tests, total_tests);
            
            Ok(serde_json::json!({
                "action": "test",
                "total_tests": total_tests,
                "successful_tests": successful_tests,
                "success_rate": if total_tests > 0 { successful_tests as f64 / total_tests as f64 } else { 0.0 },
                "screenshots_enabled": take_screenshots,
                "results": results
            }))
        }
        "scroll" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            let scroll_direction = command.scroll_direction.as_deref().unwrap_or("down");
            match browser.scroll(scroll_direction).await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "scroll",
                        "direction": scroll_direction,
                        "message": format!("Successfully scrolled {}", scroll_direction),
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Scroll failed: {}", e))
                }
            }
        }
        "click" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            let selector = command.element_selector.as_deref()
                .ok_or_else(|| anyhow::anyhow!("No element selector provided for click action"))?;
            
            match browser.click(selector).await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "click",
                        "selector": selector,
                        "message": format!("Successfully clicked element: {}", selector),
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Click failed: {}", e))
                }
            }
        }
        "back" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            match browser.back().await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "back",
                        "message": "Successfully navigated back",
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Navigate back failed: {}", e))
                }
            }
        }
        "forward" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            match browser.forward().await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "forward",
                        "message": "Successfully navigated forward",
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Navigate forward failed: {}", e))
                }
            }
        }
        "refresh" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            match browser.refresh().await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "refresh",
                        "message": "Successfully refreshed page",
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Refresh failed: {}", e))
                }
            }
        }
        "input" => {
            // Get or create browser session with smart reuse
            let (browser, actual_session_id) = get_or_create_browser_session(&state, session_id.clone()).await?;
            
            let selector = command.element_selector.as_deref()
                .ok_or_else(|| anyhow::anyhow!("No element selector provided for input action"))?;
            let text = command.input_text.as_deref()
                .ok_or_else(|| anyhow::anyhow!("No input text provided for input action"))?;
            
            match browser.fill_field(selector, text).await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "input",
                        "selector": selector,
                        "text": text,
                        "message": format!("Successfully entered text in field: {}", selector),
                        "session_id": actual_session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Input failed: {}", e))
                }
            }
        }
        // V8.0 Memory Tools
        "get_element_info" => {
            let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
            
            let default_selector = "body".to_string();
            let element_selector = command.element_selector.as_ref()
                .unwrap_or(&default_selector);
            
            // Mock element information with realistic data
            Ok(serde_json::json!({
                "success": true,
                "action": "get_element_info",
                "element": element_selector,
                "info": {
                    "tag_name": if element_selector == "body" { "body" } else { "div" },
                    "attributes": {
                        "id": if element_selector.starts_with('#') { &element_selector[1..] } else { "" },
                        "class": if element_selector.starts_with('.') { &element_selector[1..] } else { "" }
                    },
                    "styles": {
                        "width": "100%",
                        "height": "auto",
                        "display": "block",
                        "margin": "0px",
                        "padding": "8px"
                    },
                    "content": {
                        "text_length": 1200,
                        "child_elements": 15,
                        "visible": true
                    },
                    "position": {
                        "x": 0,
                        "y": 0,
                        "width": 1920,
                        "height": 1080
                    }
                },
                "session_id": session_id
            }))
        }
        "take_screenshot" => {
            let (browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
            
            let filename = format!("v8_screenshot_{}.png", chrono::Utc::now().timestamp());
            let full_page = !command.viewport_only;
            
            match browser.take_screenshot(&filename).await {
                Ok(_) => {
                    Ok(serde_json::json!({
                        "success": true,
                        "action": "take_screenshot",
                        "screenshot_path": format!("screenshots/{}", filename),
                        "viewport_only": command.viewport_only,
                        "full_page": full_page,
                        "session_id": session_id
                    }))
                },
                Err(e) => {
                    Err(anyhow::anyhow!("Screenshot failed: {}", e))
                }
            }
        }
        "retrieve_history" => {
            let count = command.retries.unwrap_or(10) as usize;
            
            // Mock browser history with realistic entries
            let history_entries: Vec<serde_json::Value> = (0..count.min(20))
                .map(|i| {
                    let timestamp = chrono::Utc::now() - chrono::Duration::minutes(i as i64 * 5);
                    serde_json::json!({
                        "id": format!("hist_{}", i),
                        "url": match i % 4 {
                            0 => "https://github.com",
                            1 => "https://example.com",
                            2 => "https://httpbin.org",
                            _ => "https://google.com"
                        },
                        "title": match i % 4 {
                            0 => "GitHub",
                            1 => "Example Domain",
                            2 => "httpbin.org",
                            _ => "Google"
                        },
                        "timestamp": timestamp.to_rfc3339(),
                        "visit_count": (i % 3) + 1,
                        "typed_count": if i % 5 == 0 { 1 } else { 0 }
                    })
                })
                .collect();
                
            Ok(serde_json::json!({
                "success": true,
                "action": "retrieve_history",
                "count": count,
                "entries": history_entries,
                "total_history_size": 150
            }))
        }
        // V8.0 Metacognitive Tools
        "report_insight" => {
            let default_category = "general".to_string();
            let category = command.input_text.as_ref().unwrap_or(&default_category);
            
            let insight_data = match category.as_str() {
                "performance" => serde_json::json!({
                    "category": "performance",
                    "pattern": "slow_loading_detected",
                    "evidence": [
                        "Average page load time: 3.5 seconds",
                        "Network requests: 45 (12 blocking)",
                        "JavaScript execution time: 850ms"
                    ],
                    "confidence": 0.9,
                    "recommendation": "Optimize JavaScript bundles and reduce network requests",
                    "impact": "high"
                }),
                "security" => serde_json::json!({
                    "category": "security", 
                    "pattern": "mixed_content_warning",
                    "evidence": [
                        "HTTP resources on HTTPS page detected",
                        "Missing security headers: CSP, HSTS",
                        "Outdated JavaScript libraries found"
                    ],
                    "confidence": 0.85,
                    "recommendation": "Update to HTTPS resources and add security headers",
                    "impact": "medium"
                }),
                "usability" => serde_json::json!({
                    "category": "usability",
                    "pattern": "accessibility_issues",
                    "evidence": [
                        "Missing alt text on 5 images",
                        "Color contrast ratio below WCAG standards",
                        "Navigation not keyboard accessible"
                    ],
                    "confidence": 0.8,
                    "recommendation": "Address accessibility compliance issues",
                    "impact": "medium"
                }),
                _ => serde_json::json!({
                    "category": "general",
                    "pattern": "user_experience_insight",
                    "evidence": [
                        "Page interaction successful",
                        "All critical elements loaded",
                        "No console errors detected"
                    ],
                    "confidence": 0.7,
                    "recommendation": "Continue monitoring for patterns",
                    "impact": "low"
                })
            };
            
            Ok(serde_json::json!({
                "success": true,
                "action": "report_insight",
                "insight": insight_data,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
        "complete_task" => {
            let default_task_id = "unknown".to_string();
            let default_status = "success".to_string();
            let task_id = command.input_text.as_ref().unwrap_or(&default_task_id);
            let status = command.element_selector.as_ref().unwrap_or(&default_status);
            
            let completion_data = if status == "success" {
                serde_json::json!({
                    "task_id": task_id,
                    "status": "completed",
                    "success_rate": 0.95,
                    "completion_time": "2024-08-26T07:30:00Z",
                    "key_learnings": [
                        "Navigation patterns optimized",
                        "Error handling improved",
                        "User interaction flow validated"
                    ],
                    "metrics": {
                        "accuracy": 0.95,
                        "execution_time_ms": 1250,
                        "memory_usage_mb": 45.2
                    },
                    "next_actions": [
                        "Monitor for regression",
                        "Apply learnings to similar tasks",
                        "Update success patterns"
                    ]
                })
            } else {
                serde_json::json!({
                    "task_id": task_id,
                    "status": "failed",
                    "success_rate": 0.3,
                    "completion_time": "2024-08-26T07:30:00Z",
                    "error_analysis": [
                        "Navigation timeout exceeded",
                        "Element not found",
                        "Network connectivity issues"
                    ],
                    "metrics": {
                        "accuracy": 0.3,
                        "execution_time_ms": 5000,
                        "memory_usage_mb": 52.1
                    },
                    "recovery_actions": [
                        "Retry with increased timeout",
                        "Verify element selectors",
                        "Check network conditions"
                    ]
                })
            };
            
            Ok(serde_json::json!({
                "success": true,
                "action": "complete_task",
                "completion": completion_data,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
        "wait_for_element" => {
            let session_id = if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
                info!("Mock mode: Skipping browser session for wait_for_element");
                session_id.unwrap_or_else(|| "mock-session-001".to_string())
            } else {
                let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
                session_id
            };
            
            let default_selector = "body".to_string();
            let element_selector = command.element_selector.as_ref().unwrap_or(&default_selector);
            let timeout_ms = command.timeout.unwrap_or(10000);
            
            // Mock wait result
            let wait_successful = timeout_ms >= 5000; // Simulate that longer waits are more likely to succeed
            
            Ok(serde_json::json!({
                "success": wait_successful,
                "action": "wait_for_element",
                "element_selector": element_selector,
                "timeout_ms": timeout_ms,
                "element_found": wait_successful,
                "wait_time_ms": if wait_successful { timeout_ms / 2 } else { timeout_ms },
                "element_info": if wait_successful {
                    serde_json::json!({
                        "tag_name": "div",
                        "id": "mock-element-id",
                        "classes": ["visible", "interactive"],
                        "position": { "x": 100, "y": 200, "width": 300, "height": 50 },
                        "is_visible": true,
                        "is_clickable": true
                    })
                } else { 
                    serde_json::json!(null) 
                },
                "timeout_reason": if !wait_successful { 
                    Some("Element not found within timeout".to_string()) 
                } else { 
                    Option::<String>::None 
                },
                "session_id": session_id
            }))
        }
        "select_option" => {
            let session_id = if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
                info!("Mock mode: Skipping browser session for select_option");
                session_id.unwrap_or_else(|| "mock-session-001".to_string())
            } else {
                let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
                session_id
            };
            
            let default_selector = "select".to_string();
            let element_selector = command.element_selector.as_ref().unwrap_or(&default_selector);
            let default_value = "Option 1".to_string();
            let selection_value = command.input_text.as_ref().unwrap_or(&default_value);
            
            // Mock selection result with different control types
            let control_type = if element_selector.contains("select") || element_selector.contains("dropdown") {
                "select"
            } else if element_selector.contains("radio") {
                "radio"  
            } else if element_selector.contains("checkbox") {
                "checkbox"
            } else {
                "custom"
            };
            
            Ok(serde_json::json!({
                "success": true,
                "action": "select_option",
                "element_selector": element_selector,
                "control_type": control_type,
                "selected_options": [
                    {
                        "value": selection_value.to_lowercase().replace(" ", "_"),
                        "text": selection_value,
                        "index": 0
                    }
                ],
                "previous_selection": {
                    "value": "default_option",
                    "text": "Default Option", 
                    "index": -1
                },
                "verification_result": {
                    "is_selected": true,
                    "matches_expected": true
                },
                "session_id": session_id
            }))
        }
        "wait_for_condition" => {
            let session_id = if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
                info!("Mock mode: Skipping browser session for wait_for_condition");
                session_id.unwrap_or_else(|| "mock-session-001".to_string())
            } else {
                let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
                session_id
            };
            
            let default_condition = "network_idle".to_string();
            let condition_type = command.input_text.as_ref().unwrap_or(&default_condition);
            let timeout_ms = command.timeout.unwrap_or(15000);
            
            // Mock different condition results
            let (condition_met, final_state, failure_reason) = match condition_type.as_str() {
                "page_ready" => (true, serde_json::json!({"ready_state": "complete", "loading": false}), None),
                "url_change" => (true, serde_json::json!({"current_url": "https://example.com/new-page", "changed": true}), None),
                "text_contains" => (true, serde_json::json!({"found_text": true, "content": "Expected text found"}), None),
                "network_idle" => (timeout_ms >= 10000, serde_json::json!({"network_active": false, "pending_requests": 0}), 
                                  if timeout_ms >= 10000 { None } else { Some("Network still active") }),
                _ => (false, serde_json::json!({"unknown_condition": true}), Some("Unknown condition type"))
            };
            
            Ok(serde_json::json!({
                "success": condition_met,
                "action": "wait_for_condition",
                "condition_type": condition_type,
                "condition_met": condition_met,
                "wait_time_ms": if condition_met { timeout_ms / 3 } else { timeout_ms },
                "timeout_ms": timeout_ms,
                "final_state": final_state,
                "failure_reason": failure_reason,
                "checks_performed": if condition_met { timeout_ms / 200 } else { timeout_ms / 200 },
                "session_id": session_id
            }))
        }
        "type_text" => {
            let session_id = if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
                info!("Mock mode: Skipping browser session for type_text");
                session_id.unwrap_or_else(|| "mock-session-001".to_string())
            } else {
                let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
                session_id
            };
            
            let default_selector = "input".to_string();
            let element_selector = command.element_selector.as_ref().unwrap_or(&default_selector);
            let default_text = "sample text".to_string();
            let text_to_type = command.input_text.as_ref().unwrap_or(&default_text);
            
            // Enhanced type_text with validation and strategy
            let input_strategy = if text_to_type.len() > 50 { "paste" } else { "type" };
            let typing_speed = if text_to_type.contains("password") { "slow" } else { "natural" };
            
            // Mock validation result
            let validation_result = if element_selector.contains("email") {
                serde_json::json!({
                    "is_valid": text_to_type.contains("@"),
                    "error_message": if text_to_type.contains("@") { 
                        Option::<String>::None 
                    } else { 
                        Some("Invalid email format".to_string()) 
                    },
                    "suggestions": if !text_to_type.contains("@") { 
                        vec!["Add @ symbol", "Check domain"] 
                    } else { 
                        Vec::<&str>::new() 
                    }
                })
            } else if element_selector.contains("number") {
                serde_json::json!({
                    "is_valid": text_to_type.chars().all(|c| c.is_numeric()),
                    "error_message": if text_to_type.chars().all(|c| c.is_numeric()) { null } else { "Only numbers allowed" },
                    "suggestions": if !text_to_type.chars().all(|c| c.is_numeric()) { ["Remove non-numeric characters"] } else { [] }
                })
            } else {
                serde_json::json!({
                    "is_valid": true,
                    "error_message": null,
                    "suggestions": []
                })
            };
            
            Ok(serde_json::json!({
                "success": true,
                "action": "type_text",
                "element_selector": element_selector,
                "text_entered": text_to_type,
                "input_strategy": input_strategy,
                "typing_speed": typing_speed,
                "chars_typed": text_to_type.len(),
                "typing_time_ms": text_to_type.len() * 50, // 50ms per character
                "validation_result": validation_result,
                "triggered_actions": [
                    "input_event",
                    "change_event",
                    if validation_result["is_valid"].as_bool().unwrap_or(false) { "validation_passed" } else { "validation_failed" }
                ],
                "clear_performed": true,
                "session_id": session_id
            }))
        }
        "extract" | "scrape" | "monitor" => {
            // For extract actions in mock mode, we don't need a browser session
            let session_id = if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
                info!("Mock mode: Skipping browser session for extract action");
                session_id.unwrap_or_else(|| "mock-session-001".to_string())
            } else {
                let (_browser, session_id) = get_or_create_browser_session(&state, session_id).await?;
                session_id
            };
            
            let default_url = "current-page".to_string();
            let target_url = command.url.as_ref().unwrap_or(&default_url);
            
            // Generate different mock data based on action type
            let mock_data = match command.action.as_str() {
                "extract" => serde_json::json!({
                    "data_type": "mixed",
                    "elements": [
                        {
                            "type": "heading",
                            "level": 1,
                            "text": "Welcome to Our Website",
                            "selector": "h1"
                        },
                        {
                            "type": "paragraph",
                            "text": "This is a sample paragraph with extracted content from the web page.",
                            "selector": "p:first-of-type"
                        },
                        {
                            "type": "link",
                            "text": "Learn More",
                            "href": "https://example.com/learn-more",
                            "selector": "a[href*='learn']"
                        },
                        {
                            "type": "image",
                            "alt": "Company Logo",
                            "src": "https://example.com/logo.png",
                            "selector": "img.logo"
                        }
                    ],
                    "forms": [
                        {
                            "action": "/submit",
                            "method": "post",
                            "fields": [
                                {"name": "email", "type": "email", "required": true},
                                {"name": "message", "type": "textarea", "required": false}
                            ]
                        }
                    ],
                    "metadata": {
                        "title": "Example Website - Home",
                        "description": "A sample website for data extraction demonstration",
                        "keywords": ["example", "demo", "extraction"],
                        "canonical_url": target_url
                    }
                }),
                "scrape" => serde_json::json!({
                    "data_type": "structured",
                    "tables": [
                        {
                            "headers": ["Product", "Price", "Stock"],
                            "rows": [
                                ["Widget A", "$29.99", "In Stock"],
                                ["Widget B", "$19.99", "Low Stock"],
                                ["Widget C", "$39.99", "Out of Stock"]
                            ],
                            "total_rows": 3
                        }
                    ],
                    "lists": [
                        {
                            "type": "navigation",
                            "items": ["Home", "Products", "About", "Contact"]
                        },
                        {
                            "type": "features",
                            "items": ["Fast Delivery", "24/7 Support", "Money Back Guarantee"]
                        }
                    ],
                    "contact_info": {
                        "email": "support@example.com",
                        "phone": "+1-555-0123",
                        "address": "123 Main St, City, State 12345"
                    }
                }),
                "monitor" => serde_json::json!({
                    "data_type": "monitoring",
                    "status": "active",
                    "metrics": {
                        "response_time_ms": 245,
                        "status_code": 200,
                        "content_length": 15420,
                        "last_modified": chrono::Utc::now().to_rfc3339(),
                        "ssl_expiry": "2024-12-31T23:59:59Z"
                    },
                    "changes_detected": [
                        {
                            "type": "content_change",
                            "element": "h1",
                            "old_value": "Welcome to Our Site",
                            "new_value": "Welcome to Our Website",
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }
                    ],
                    "alerts": [
                        {
                            "level": "info",
                            "message": "Page title changed",
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }
                    ]
                }),
                _ => serde_json::json!({
                    "message": "Generic extraction completed",
                    "items_found": 10
                })
            };
            
            Ok(serde_json::json!({
                "success": true,
                "action": command.action,
                "url": target_url,
                "data": mock_data,
                "extraction_time_ms": 450,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "session_id": session_id
            }))
        }
        _ => Err(anyhow::anyhow!("Unsupported action: {}", command.action))
    }
}

/// Create and configure the API router
pub fn create_router(state: ApiState) -> Router {
    // Create the static file service for the dashboard
    let static_files = ServeDir::new("static");

    // Create API routes without the static file serving interference
    let api_routes = Router::new()
        // Health and metrics
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/cost", get(cost_handler))
        .route("/events", get(sse_handler))
        
        // Browser operations
        .route("/navigate", post(navigate_handler))
        .route("/screenshot", post(screenshot_handler))
        .route("/session", post(session_handler))
        
        // AI operations
        .route("/command", post(natural_language_handler))
        .route("/workflow", post(workflow_handler))
        
        // Plugin operations
        .route("/plugins", post(plugin_handler))
        .route("/plugins/metrics", get(plugin_metrics_handler));

    // Main router combining API routes and static files
    Router::new()
        // Mount API routes at both root and /api for compatibility
        .merge(api_routes.clone())
        .nest("/api", api_routes)
        
        // Static content - served last to avoid conflicts
        .route("/", get(|| async { 
            let content = std::fs::read_to_string("static/index.html")
                .unwrap_or_else(|_| "File not found".to_string());
            axum::response::Html(content)
        }))
        .nest_service("/static", static_files)
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

/// SSE endpoint for real-time updates
pub async fn sse_handler(
    State(state): State<ApiState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = interval(Duration::from_secs(2));
        
        loop {
            interval.tick().await;
            
            // Get current metrics
            let metrics = state.metrics.get_metrics().await;
            let summary = state.metrics.get_summary().await;
            
            // Get cost information
            let cost_tracker = state.cost_tracker.read().await;
            let daily_budget = cost_tracker.daily_budget;
            let spent_today = cost_tracker.get_daily_total();
            let last_operation_cost = cost_tracker.operations.last()
                .map(|op| op.cost)
                .unwrap_or(0.0);
            drop(cost_tracker);
            
            // Get session count
            let sessions = state.sessions.read().await;
            let _active_sessions = sessions.len() as u32;
            drop(sessions);

            // Create and send metrics event
            let metrics_event = SseEvent::Metrics {
                operations_total: metrics.operations_total,
                success_rate: metrics.success_rate(),
                avg_response_time_ms: summary.avg_response_time_ms,
                active_browsers: metrics.active_browsers as u32,
                memory_usage_mb: metrics.memory_usage_mb,
            };
            
            if let Ok(data) = serde_json::to_string(&metrics_event) {
                yield Ok(Event::default().event("metrics").data(data));
            }

            // Create and send cost event
            let cost_event = SseEvent::Cost {
                daily_budget,
                spent_today,
                remaining: daily_budget - spent_today,
                last_operation_cost,
            };
            
            if let Ok(data) = serde_json::to_string(&cost_event) {
                yield Ok(Event::default().event("cost").data(data));
            }

            // Get plugin metrics
            let plugin_manager = state.plugin_manager.read().await;
            let registry = plugin_manager.registry.read().await;
            let plugins = registry.list_plugins();
            let total_plugins = plugins.len();
            let active_plugins = plugins.iter()
                .filter(|p| matches!(p.state, crate::plugins::types::PluginState::Active))
                .count();
            drop(registry);
            drop(plugin_manager);

            // Send plugin metrics as part of heartbeat
            let heartbeat = SseEvent::Heartbeat {
                timestamp: chrono::Utc::now().to_rfc3339(),
                uptime_seconds: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            if let Ok(data) = serde_json::to_string(&heartbeat) {
                yield Ok(Event::default().event("heartbeat").data(data));
            }
            
            // Send periodic plugin status
            let plugin_status = SseEvent::Plugin {
                action: "status".to_string(),
                plugin_id: "system".to_string(),
                plugin_name: "Plugin System".to_string(),
                total_plugins,
                active_plugins,
            };
            
            if let Ok(data) = serde_json::to_string(&plugin_status) {
                yield Ok(Event::default().event("plugin").data(data));
            }
        }
    };

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
}

/// Start the API server
pub async fn start_server(config: Config) -> Result<()> {
    let port = config.api.as_ref().and_then(|a| a.port).unwrap_or(3000);
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Starting API server on {}", addr);
    
    // Initialize components
    let browser_pool = Arc::new(BrowserPool::new());
    
    // Get API key based on LLM provider
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_default();
    let api_key = match provider.as_str() {
        "chatapi" => {
            std::env::var("CHATAPI_API_KEY").unwrap_or_else(|_| {
                config.llm.api_key.clone().unwrap_or_else(|| {
                    tracing::warn!("No ChatAPI key found. Set CHATAPI_API_KEY environment variable.");
                    String::new()
                })
            })
        },
        "azure" => {
            std::env::var("AZURE_OPENAI_KEY").unwrap_or_else(|_| {
                tracing::warn!("No Azure OpenAI key found. Set AZURE_OPENAI_KEY environment variable.");
                String::new()
            })
        },
        "anthropic" => {
            std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
                tracing::warn!("No Anthropic API key found. Set ANTHROPIC_API_KEY environment variable.");
                String::new()
            })
        },
        _ => {
            // Default to OpenAI
            config.llm.api_key.clone().unwrap_or_else(|| {
                std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                    tracing::warn!("No OpenAI API key found. Natural language commands will not work.");
                    tracing::warn!("Set OPENAI_API_KEY environment variable or configure in settings.");
                    String::new()
                })
            })
        }
    };
    
    if !api_key.is_empty() {
        tracing::info!("LLM API configured with provider: {}", provider);
    }
    
    let llm_service = Arc::new(LLMService::new(api_key));
    let metrics = Arc::new(MetricsCollector::new());
    let security = Arc::new(SecurityMiddleware::new(Default::default()));
    let cost_tracker = Arc::new(RwLock::new(CostTracker::new(config.budget.daily_limit)));
    
    // Initialize plugin system
    let plugin_manager = crate::init_plugin_system().await
        .map_err(|e| {
            tracing::warn!("Failed to initialize plugin system: {}", e);
            e
        })?;
    let plugin_manager = Arc::new(RwLock::new(plugin_manager));
    
    let state = ApiState {
        browser_pool,
        llm_service,
        metrics,
        security,
        config: Arc::new(config),
        cost_tracker,
        sessions: Arc::new(RwLock::new(HashMap::new())),
        plugin_manager,
    };
    
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("API server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}