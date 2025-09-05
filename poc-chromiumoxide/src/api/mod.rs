use anyhow::Result;
use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod perception_handlers;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing::{info, error};
use crate::browser::{BrowserOps, pool::BrowserPool, SessionManager};

#[derive(Clone)]
struct AppState {
    browser_pool: Arc<BrowserPool>,
    session_manager: Arc<SessionManager>,
}

pub async fn serve(port: u16, browser_pool: BrowserPool) -> Result<()> {
    let session_manager = SessionManager::default();
    let state = AppState {
        browser_pool: Arc::new(browser_pool),
        session_manager: Arc::new(session_manager),
    };
    
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        
        // Session management
        .route("/api/session/create", post(create_session))
        .route("/api/session/:id", get(get_session).delete(delete_session))
        .route("/api/sessions", get(list_sessions))
        
        // Browser actions
        .route("/api/navigate", post(navigate))
        .route("/api/screenshot", post(screenshot))
        .route("/api/click", post(click))
        .route("/api/type", post(type_text))
        .route("/api/execute", post(execute_script))
        .route("/api/find", post(find_elements))
        .route("/api/get_text", post(get_text))
        .route("/api/scroll", post(scroll))
        .route("/api/zoom", post(set_zoom_level))
        .route("/api/fix_scaling", post(fix_content_scaling))
        .route("/api/fix_window", post(fix_window_completely))
        
        // Workflow
        .route("/api/workflow", post(execute_workflow))
        
        // Tools API endpoints
        .route("/api/tools", get(list_tools))
        .route("/api/tools/execute", post(execute_tool))
        
        // Perception API endpoints
        .route("/api/perception/analyze", post(perception_handlers::analyze_page))
        .route("/api/perception/find", post(perception_handlers::intelligent_find_element))
        .route("/api/perception/command", post(perception_handlers::execute_intelligent_command))
        .route("/api/perception/forms/analyze", post(perception_handlers::analyze_form))
        .route("/api/perception/forms/fill", post(perception_handlers::auto_fill_form))
        
        // Static files (serve our migrated interface)
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(dashboard))
        
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    let addr = format!("0.0.0.0:{}", port);
    info!("API server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "rainbow-poc-chromiumoxide",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Dashboard HTML - serve our migrated interface
async fn dashboard() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(_) => {
            // Fallback HTML if static file doesn't exist
            let fallback_html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>RainbowBrowserAI - Chromiumoxide</title>
    <style>
        body { font-family: sans-serif; text-align: center; padding: 50px; }
        .error { color: #e11d48; }
    </style>
</head>
<body>
    <h1>🌈 RainbowBrowserAI - Chromiumoxide Edition</h1>
    <p class="error">⚠️  Static files not found. Please ensure static/index.html exists.</p>
    <p>Alternative: Access the API directly at <code>/api/health</code></p>
</body>
</html>
            "#;
            axum::response::Html(fallback_html).into_response()
        }
    }
}

// Request/Response types
#[derive(Deserialize)]
struct NavigateRequest {
    url: String,
}

#[derive(Deserialize)]
struct ScreenshotRequest {
    url: Option<String>,
    full_page: Option<bool>,
    format: Option<String>,
}

#[derive(Deserialize)]
struct ClickRequest {
    selector: String,
}

#[derive(Deserialize)]
struct TypeRequest {
    selector: String,
    text: String,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    script: String,
}

#[derive(Deserialize)]
struct FindRequest {
    selector: String,
}

#[derive(Deserialize)]
struct GetTextRequest {
    selector: String,
}

#[derive(Deserialize)]
struct ScrollRequest {
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
struct ZoomRequest {
    zoom_factor: f64,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

// API Handlers
async fn navigate(
    State(state): State<AppState>,
    Json(req): Json<NavigateRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.navigate_to(&req.url).await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "url": req.url,
                        "status": "navigated"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Navigation failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, 
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn screenshot(
    State(state): State<AppState>,
    Json(req): Json<ScreenshotRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            // Navigate if URL provided
            if let Some(url) = req.url {
                if let Err(e) = browser.navigate_to(&url).await {
                    return (StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(e.to_string()))).into_response();
                }
            }
            
            let mut options = crate::browser::ScreenshotOptions::default();
            if let Some(full_page) = req.full_page {
                options.full_page = full_page;
            }
            if let Some(format) = req.format {
                options.format = format;
            }
            
            match browser.screenshot(options).await {
                Ok(data) => {
                    use base64::Engine;
            let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    Json(ApiResponse::success(serde_json::json!({
                        "screenshot": base64,
                        "size": data.len()
                    }))).into_response()
                }
                Err(e) => {
                    error!("Screenshot failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn click(
    State(state): State<AppState>,
    Json(req): Json<ClickRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.click(&req.selector).await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "selector": req.selector,
                        "action": "clicked"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Click failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn type_text(
    State(state): State<AppState>,
    Json(req): Json<TypeRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.type_text(&req.selector, &req.text).await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "selector": req.selector,
                        "text": req.text,
                        "action": "typed"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Type failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn execute_script(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.execute_script(&req.script).await {
                Ok(result) => {
                    Json(ApiResponse::success(result)).into_response()
                }
                Err(e) => {
                    error!("Script execution failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn find_elements(
    State(state): State<AppState>,
    Json(req): Json<FindRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.find_elements(&req.selector).await {
                Ok(elements) => {
                    Json(ApiResponse::success(elements)).into_response()
                }
                Err(e) => {
                    error!("Find elements failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn get_text(
    State(state): State<AppState>,
    Json(req): Json<GetTextRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.get_text(&req.selector).await {
                Ok(text) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "selector": req.selector,
                        "text": text
                    }))).into_response()
                }
                Err(e) => {
                    error!("Get text failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn scroll(
    State(state): State<AppState>,
    Json(req): Json<ScrollRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.scroll_to(req.x, req.y).await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "x": req.x,
                        "y": req.y,
                        "action": "scrolled"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Scroll failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn set_zoom_level(
    State(state): State<AppState>,
    Json(req): Json<ZoomRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.set_zoom_level(req.zoom_factor).await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "zoom_factor": req.zoom_factor,
                        "action": "zoom_set"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Set zoom level failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn fix_content_scaling(
    State(state): State<AppState>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.fix_content_scaling().await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "action": "content_scaling_fixed",
                        "message": "Viewport and content scaling fixed"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Fix content scaling failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn fix_window_completely(
    State(state): State<AppState>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match browser.fix_window_completely().await {
                Ok(_) => {
                    Json(ApiResponse::success(serde_json::json!({
                        "action": "window_completely_fixed",
                        "message": "Screenshot trigger + content scaling applied"
                    }))).into_response()
                }
                Err(e) => {
                    error!("Complete window fix failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn execute_workflow(
    State(state): State<AppState>,
    Json(workflow): Json<serde_json::Value>,
) -> Response {
    // TODO: Implement workflow execution
    Json(ApiResponse::<()>::error("Workflow execution not yet implemented".to_string())).into_response()
}

// Session management handlers
async fn create_session(
    State(state): State<AppState>,
) -> Response {
    match state.session_manager.create_session().await {
        Ok(session_id) => {
            Json(ApiResponse::success(serde_json::json!({
                "session_id": session_id,
                "created": true
            }))).into_response()
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn get_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    if let Some(session) = state.session_manager.get_session(&id).await {
        let session_guard = session.read().await;
        Json(ApiResponse::success(serde_json::json!({
            "id": session_guard.id,
            "created_at": session_guard.created_at,
            "last_used": session_guard.last_used,
            "current_url": session_guard.current_url,
            "history": session_guard.history,
            "age_seconds": session_guard.age_seconds(),
            "idle_seconds": session_guard.idle_seconds(),
        }))).into_response()
    } else {
        (StatusCode::NOT_FOUND,
         Json(ApiResponse::<()>::error(format!("Session not found: {}", id)))).into_response()
    }
}

async fn delete_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match state.session_manager.remove_session(&id).await {
        Ok(_) => {
            Json(ApiResponse::success(serde_json::json!({
                "deleted": true,
                "session_id": id
            }))).into_response()
        }
        Err(e) => {
            error!("Failed to delete session: {}", e);
            (StatusCode::NOT_FOUND,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Response {
    let sessions = state.session_manager.list_sessions().await;
    Json(ApiResponse::success(sessions)).into_response()
}

// Tools API handlers
async fn list_tools() -> Response {
    let tools = serde_json::json!({
        "navigation_tools": [
            {"name": "navigate_to_url", "description": "Navigate to a specific URL"},
            {"name": "scroll", "description": "Scroll the page"},
            {"name": "refresh", "description": "Refresh the current page"},
            {"name": "go_back", "description": "Go back in browser history"},
            {"name": "go_forward", "description": "Go forward in browser history"}
        ],
        "interaction_tools": [
            {"name": "click", "description": "Click on an element"},
            {"name": "type_text", "description": "Type text into an input field"},
            {"name": "select_option", "description": "Select an option from a dropdown"},
            {"name": "hover", "description": "Hover over an element"},
            {"name": "focus", "description": "Focus on an element"}
        ],
        "extraction_tools": [
            {"name": "extract_text", "description": "Extract text from elements"},
            {"name": "extract_links", "description": "Extract all links from the page"},
            {"name": "extract_data", "description": "Extract structured data"},
            {"name": "extract_table", "description": "Extract table data"},
            {"name": "extract_form", "description": "Extract form data"}
        ],
        "synchronization_tools": [
            {"name": "wait_for_element", "description": "Wait for an element to appear"},
            {"name": "wait_for_condition", "description": "Wait for a condition to be met"}
        ],
        "memory_tools": [
            {"name": "screenshot", "description": "Take a screenshot"},
            {"name": "session_memory", "description": "Access session memory"},
            {"name": "get_element_info", "description": "Get element information"},
            {"name": "history_tracker", "description": "Track page history"},
            {"name": "persistent_cache", "description": "Access persistent cache"}
        ]
    });
    
    Json(ApiResponse::success(tools)).into_response()
}

#[derive(Deserialize)]
struct ExecuteToolRequest {
    tool_name: String,
    parameters: serde_json::Value,
}

async fn execute_tool(
    State(state): State<AppState>,
    Json(req): Json<ExecuteToolRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let result = match req.tool_name.as_str() {
                "navigate_to_url" => {
                    if let Some(url) = req.parameters.get("url").and_then(|v| v.as_str()) {
                        browser.navigate_to(url).await
                            .map(|_| serde_json::json!({"status": "navigated", "url": url}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: url"))
                    }
                }
                "click" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        browser.click(selector).await
                            .map(|_| serde_json::json!({"status": "clicked", "selector": selector}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "type_text" => {
                    if let (Some(selector), Some(text)) = (
                        req.parameters.get("selector").and_then(|v| v.as_str()),
                        req.parameters.get("text").and_then(|v| v.as_str())
                    ) {
                        browser.type_text(selector, text).await
                            .map(|_| serde_json::json!({"status": "typed", "selector": selector, "text": text}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameters: selector, text"))
                    }
                }
                "screenshot" => {
                    let options = crate::browser::ScreenshotOptions::default();
                    browser.screenshot(options).await
                        .map(|data| {
                            use base64::Engine;
                            let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                            serde_json::json!({"status": "captured", "screenshot": base64, "size": data.len()})
                        })
                        .map_err(|e| anyhow::anyhow!(e))
                }
                "extract_text" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        browser.get_text(selector).await
                            .map(|text| serde_json::json!({"status": "extracted", "selector": selector, "text": text}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "scroll" => {
                    if let (Some(x), Some(y)) = (
                        req.parameters.get("x").and_then(|v| v.as_i64()),
                        req.parameters.get("y").and_then(|v| v.as_i64())
                    ) {
                        browser.scroll_to(x as i32, y as i32).await
                            .map(|_| serde_json::json!({"status": "scrolled", "x": x, "y": y}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameters: x, y"))
                    }
                }
                
                // Navigation tools
                "refresh" | "refresh_page" => {
                    browser.refresh().await
                        .map(|_| serde_json::json!({"status": "refreshed"}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                "go_back" => {
                    browser.go_back().await
                        .map(|_| serde_json::json!({"status": "went_back"}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                "go_forward" => {
                    browser.go_forward().await
                        .map(|_| serde_json::json!({"status": "went_forward"}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                
                // Interaction tools
                "hover" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        browser.hover(selector).await
                            .map(|_| serde_json::json!({"status": "hovered", "selector": selector}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "focus" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        browser.focus(selector).await
                            .map(|_| serde_json::json!({"status": "focused", "selector": selector}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "select_option" => {
                    if let (Some(selector), Some(value)) = (
                        req.parameters.get("selector").and_then(|v| v.as_str()),
                        req.parameters.get("value").and_then(|v| v.as_str())
                    ) {
                        browser.select_option(selector, value).await
                            .map(|_| serde_json::json!({"status": "selected", "selector": selector, "value": value}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameters: selector, value"))
                    }
                }
                
                // Data extraction tools
                "extract_links" => {
                    let selector = req.parameters.get("selector")
                        .and_then(|v| v.as_str())
                        .unwrap_or("a");
                    browser.extract_links(selector).await
                        .map(|links| serde_json::json!({"status": "extracted", "links": links}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                "extract_data" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        let attributes = req.parameters.get("attributes")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<_>>())
                            .unwrap_or_default();
                        
                        browser.extract_attributes(selector, &attributes).await
                            .map(|data| serde_json::json!({"status": "extracted", "data": data}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "extract_table" => {
                    let selector = req.parameters.get("selector")
                        .and_then(|v| v.as_str())
                        .unwrap_or("table");
                    browser.extract_table(selector).await
                        .map(|table| serde_json::json!({"status": "extracted", "table": table}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                "extract_form" => {
                    let selector = req.parameters.get("selector")
                        .and_then(|v| v.as_str())
                        .unwrap_or("form");
                    browser.extract_form(selector).await
                        .map(|form| serde_json::json!({"status": "extracted", "form": form}))
                        .map_err(|e| anyhow::anyhow!(e))
                }
                
                // Synchronization tools
                "wait_for_element" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        let timeout = req.parameters.get("timeout")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(5000);
                        
                        browser.wait_for_selector(selector, std::time::Duration::from_millis(timeout)).await
                            .map(|_| serde_json::json!({"status": "element_found", "selector": selector}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "wait_for_condition" => {
                    if let Some(condition) = req.parameters.get("condition").and_then(|v| v.as_str()) {
                        let timeout = req.parameters.get("timeout")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(5000);
                        
                        browser.wait_for_condition(condition, std::time::Duration::from_millis(timeout)).await
                            .map(|_| serde_json::json!({"status": "condition_met", "condition": condition}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: condition"))
                    }
                }
                
                // Memory tools
                "session_memory" => {
                    let action = req.parameters.get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("get");
                    
                    match action {
                        "get" => {
                            if let Some(key) = req.parameters.get("key").and_then(|v| v.as_str()) {
                                Ok(serde_json::json!({"status": "retrieved", "key": key, "value": null}))
                            } else {
                                Ok(serde_json::json!({"status": "retrieved", "data": {}}))
                            }
                        }
                        "set" => {
                            if let (Some(key), Some(value)) = (
                                req.parameters.get("key").and_then(|v| v.as_str()),
                                req.parameters.get("value")
                            ) {
                                Ok(serde_json::json!({"status": "stored", "key": key}))
                            } else {
                                Err(anyhow::anyhow!("Missing required parameters: key, value"))
                            }
                        }
                        "clear" => {
                            Ok(serde_json::json!({"status": "cleared"}))
                        }
                        _ => Err(anyhow::anyhow!("Invalid action: {}", action))
                    }
                }
                "get_element_info" => {
                    if let Some(selector) = req.parameters.get("selector").and_then(|v| v.as_str()) {
                        browser.get_element_info(selector).await
                            .map(|info| serde_json::json!({"status": "retrieved", "element": info}))
                            .map_err(|e| anyhow::anyhow!(e))
                    } else {
                        Err(anyhow::anyhow!("Missing required parameter: selector"))
                    }
                }
                "history_tracker" => {
                    let action = req.parameters.get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("get");
                    
                    match action {
                        "get" => Ok(serde_json::json!({"status": "retrieved", "history": []})),
                        "clear" => Ok(serde_json::json!({"status": "cleared"})),
                        _ => Err(anyhow::anyhow!("Invalid action: {}", action))
                    }
                }
                "persistent_cache" => {
                    let action = req.parameters.get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("get");
                    
                    match action {
                        "get" => {
                            if let Some(key) = req.parameters.get("key").and_then(|v| v.as_str()) {
                                Ok(serde_json::json!({"status": "retrieved", "key": key, "value": null}))
                            } else {
                                Ok(serde_json::json!({"status": "retrieved", "cache": {}}))
                            }
                        }
                        "set" => {
                            if let (Some(key), Some(value)) = (
                                req.parameters.get("key").and_then(|v| v.as_str()),
                                req.parameters.get("value")
                            ) {
                                Ok(serde_json::json!({"status": "stored", "key": key}))
                            } else {
                                Err(anyhow::anyhow!("Missing required parameters: key, value"))
                            }
                        }
                        "clear" => Ok(serde_json::json!({"status": "cleared"})),
                        _ => Err(anyhow::anyhow!("Invalid action: {}", action))
                    }
                }
                
                _ => Err(anyhow::anyhow!("Unknown tool: {}", req.tool_name))
            };

            match result {
                Ok(data) => Json(ApiResponse::success(data)).into_response(),
                Err(e) => {
                    error!("Tool execution failed: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR,
                     Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
        }
    }
}