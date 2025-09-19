use anyhow::Result;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

mod coordinated_handlers;
mod intelligence_handlers;
mod llm_handlers;
mod perception_handlers;
mod task_executor;
mod workflow_handlers; // New coordinated handlers
use crate::browser::{pool::BrowserPool, BrowserOps, SessionManager};
use crate::tools::registry::ToolRegistry;
use std::io::ErrorKind;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
struct AppState {
    browser_pool: Arc<BrowserPool>,
    session_manager: Arc<SessionManager>,
    tool_registry: Arc<LazyToolRegistry>,
    recent_nav: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Clone)]
struct LazyToolRegistry {
    inner: Arc<RwLock<Option<Arc<ToolRegistry>>>>,
    // Track the active browser instance used by the registry so other subsystems
    // (e.g., perception) can share the same browser when no session_id is provided.
    active_browser: Arc<RwLock<Option<Arc<crate::browser::Browser>>>>,
    browser_pool: Arc<BrowserPool>,
    session_manager: Arc<SessionManager>,
}

impl LazyToolRegistry {
    fn new(browser_pool: Arc<BrowserPool>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
            active_browser: Arc::new(RwLock::new(None)),
            browser_pool,
            session_manager,
        }
    }

    async fn get(&self) -> anyhow::Result<Arc<ToolRegistry>> {
        // Fast path: already initialized
        if let Some(existing) = self.inner.read().await.as_ref() {
            return Ok(existing.clone());
        }
        // Initialize lazily
        let mut write_guard = self.inner.write().await;
        if let Some(existing) = write_guard.as_ref() {
            return Ok(existing.clone());
        }
        // Try to acquire a browser from the pool with a short timeout to avoid deadlocks
        let browser_arc =
            match timeout(Duration::from_millis(2000), self.browser_pool.acquire()).await {
                Ok(Ok(guard)) => guard.browser_arc(),
                Ok(Err(e)) => {
                    warn!(
                        "Tool registry: pool acquire failed ({}). Attempting session fallback...",
                        e
                    );
                    self.try_session_browser_fallback().await?
                }
                Err(_) => {
                    warn!("Tool registry: pool acquire timed out. Attempting session fallback...");
                    self.try_session_browser_fallback().await?
                }
            };

        let registry = Arc::new(ToolRegistry::new(browser_arc.clone()));

        // Record the active browser used by the registry for cross-module sharing
        {
            let mut ab = self.active_browser.write().await;
            *ab = Some(browser_arc);
        }

        // Start background cache cleanup task once
        {
            use crate::tools::cache::start_cache_cleanup_task;
            let cache = registry.cache.clone();
            tokio::spawn(start_cache_cleanup_task(
                cache,
                std::time::Duration::from_secs(300),
            ));
        }

        *write_guard = Some(registry.clone());
        Ok(registry)
    }

    async fn initialized(&self) -> bool {
        self.inner.read().await.is_some()
    }

    // Expose the active browser if the registry has been initialized
    async fn active_browser(&self) -> Option<Arc<crate::browser::Browser>> {
        self.active_browser.read().await.clone()
    }

    // Allow other subsystems to promote a browser as the active one
    async fn set_active_browser(&self, browser: Arc<crate::browser::Browser>) {
        let mut ab = self.active_browser.write().await;
        *ab = Some(browser);
    }

    async fn try_session_browser_fallback(&self) -> anyhow::Result<Arc<crate::browser::Browser>> {
        // Try to pick an existing session's browser to initialize the registry
        let sessions = self.session_manager.list_sessions().await;
        if sessions.is_empty() {
            anyhow::bail!("Browser unavailable and no active sessions for fallback");
        }
        // Choose the most recently used session (smallest idle_seconds)
        let mut best = &sessions[0];
        for s in &sessions {
            if s.idle_seconds < best.idle_seconds {
                best = s;
            }
        }
        if let Some(session_arc) = self.session_manager.get_session(&best.id).await {
            let lock = session_arc.read().await;
            Ok(lock.browser.clone())
        } else {
            anyhow::bail!("Selected session not found during fallback")
        }
    }
}

fn format_addr(port: u16) -> String {
    format!("127.0.0.1:{}", port)
}

async fn bind_with_retry(base_port: u16, attempts: u16) -> Result<(tokio::net::TcpListener, u16)> {
    for i in 0..attempts {
        let p = base_port + i;
        let addr = format_addr(p);
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => return Ok((listener, p)),
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                warn!("Port {} in use, trying {}", p, p + 1);
                continue;
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to bind {}: {}", addr, e)),
        }
    }
    Err(anyhow::anyhow!(
        "No free port found starting at {} after {} attempts",
        base_port,
        attempts
    ))
}

pub async fn serve(port: u16, browser_pool: BrowserPool) -> Result<()> {
    let browser_pool_arc = Arc::new(browser_pool);

    // Create session manager using the browser pool
    let session_manager = SessionManager::new(
        browser_pool_arc.clone(),
        10,   // max_sessions
        1800, // session_timeout (30 minutes)
    );

    // Create the RainbowCoordinator for coordinated operations
    let coordinator =
        match crate::coordination::RainbowCoordinator::new(browser_pool_arc.clone()).await {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("Failed to create RainbowCoordinator: {}", e);
                // Continue with legacy system if coordinator fails
                return serve_legacy(port, browser_pool_arc, session_manager).await;
            }
        };

    let session_manager_arc = Arc::new(session_manager);
    let state = AppState {
        browser_pool: browser_pool_arc.clone(),
        session_manager: session_manager_arc.clone(),
        tool_registry: Arc::new(LazyToolRegistry::new(
            browser_pool_arc.clone(),
            session_manager_arc.clone(),
        )),
        recent_nav: Arc::new(RwLock::new(HashMap::new())),
    };

    let coordinated_state = coordinated_handlers::CoordinatedApiState {
        coordinator: coordinator.clone(),
    };

    // Create v2 router with coordinated endpoints
    let v2_router = Router::new()
        .route(
            "/session/create",
            post(coordinated_handlers::create_coordinated_session),
        )
        .route(
            "/session/:id",
            get(coordinated_handlers::get_coordinated_session)
                .delete(coordinated_handlers::delete_coordinated_session),
        )
        .route(
            "/sessions",
            get(coordinated_handlers::list_coordinated_sessions),
        )
        .route(
            "/navigate",
            post(coordinated_handlers::coordinated_navigate),
        )
        .route(
            "/intelligent-action",
            post(coordinated_handlers::coordinated_intelligent_action),
        )
        .route(
            "/perception/analyze",
            post(coordinated_handlers::coordinated_perception_analysis),
        )
        .route(
            "/tool/execute",
            post(coordinated_handlers::coordinated_tool_execution),
        )
        .route("/health", get(coordinated_handlers::get_system_health))
        .with_state(coordinated_state);

    // Static list of important routes for diagnostics
    fn route_list() -> Vec<&'static str> {
        vec![
            "/api/health",
            "/api/diagnostics",
            "/api/session/create",
            "/api/sessions",
            "/api/navigate",
            "/api/perception/analyze",
            "/api/perceive-mode",
            "/api/navigate-perceive",
            "/api/tools/execute",
        ]
    }

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/api/diagnostics", get(diagnostics))
        .route(
            "/api/routes",
            get(|| async move { Json(ApiResponse::success(route_list())) }),
        )
        // Nest v2 coordinated endpoints under /api/v2
        .nest("/api/v2", v2_router)
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
        .route("/api/tools/metadata", get(get_tools_metadata))
        .route("/api/tools/validate", post(validate_registry))
        // Performance Monitoring API endpoints
        .route(
            "/api/tools/performance/stats",
            get(get_all_performance_stats),
        )
        .route(
            "/api/tools/performance/stats/:tool_name",
            get(get_tool_performance_stats),
        )
        .route(
            "/api/tools/performance/metrics",
            get(get_recent_performance_metrics),
        )
        .route(
            "/api/tools/performance/metrics/:tool_name",
            get(get_tool_performance_metrics),
        )
        .route(
            "/api/tools/performance/clear",
            post(clear_performance_metrics),
        )
        // Cache Management API endpoints
        .route("/api/tools/cache/stats", get(get_cache_stats))
        .route("/api/tools/cache/clear", post(clear_all_cache))
        .route("/api/tools/cache/clear/:tool_name", post(clear_tool_cache))
        .route(
            "/api/tools/cache/config/:tool_name",
            post(set_tool_cache_config),
        )
        // Dependency Management API endpoints
        .route("/api/tools/dependencies/plan", post(create_execution_plan))
        .route(
            "/api/tools/dependencies/execute",
            post(execute_with_dependencies),
        )
        .route(
            "/api/tools/dependencies/register",
            post(register_tool_dependencies),
        )
        .route(
            "/api/tools/dependencies/:tool_name",
            get(get_tool_dependencies),
        )
        .route(
            "/api/tools/dependencies/dependents/:tool_name",
            get(get_dependent_tools),
        )
        .route("/api/tools/dependencies/stats", get(get_dependency_stats))
        // Perception API endpoints
        .route(
            "/api/perception/analyze",
            post(perception_handlers::analyze_page),
        )
        .route(
            "/api/perception/find",
            post(perception_handlers::intelligent_find_element),
        )
        .route(
            "/api/perception/command",
            post(perception_handlers::execute_intelligent_command),
        )
        .route(
            "/api/perception/forms/analyze",
            post(perception_handlers::analyze_form),
        )
        .route(
            "/api/perception/forms/fill",
            post(perception_handlers::auto_fill_form),
        )
        // NEW: Layered perception endpoints
        .route(
            "/api/perceive-mode",
            post(perception_handlers::perceive_with_mode),
        )
        .route(
            "/api/navigate-perceive",
            post(perception_handlers::navigate_and_perceive),
        )
        .route("/api/quick-scan", post(perception_handlers::quick_scan))
        .route(
            "/api/smart-element-search",
            post(perception_handlers::smart_element_search),
        )
        .route(
            "/api/perception/smart_search",
            post(perception_handlers::smart_element_search),
        )
        .route(
            "/api/perception/find_element",
            post(perception_handlers::intelligent_find_element),
        )
        // LLM API endpoints
        .route("/api/llm/query", post(llm_handlers::llm_query))
        .route("/api/llm/plan", post(llm_handlers::task_planning))
        .route("/api/llm/execute", post(llm_handlers::execute_command))
        .route("/api/llm/usage", post(llm_handlers::get_usage_metrics))
        // Intelligence API endpoints
        .route(
            "/api/intelligence/analyze",
            post(intelligence_handlers::analyze_situation),
        )
        .route(
            "/api/intelligence/recommend",
            post(intelligence_handlers::recommend_action),
        )
        .route(
            "/api/intelligence/learn",
            post(intelligence_handlers::submit_learning_feedback),
        )
        .route(
            "/api/intelligence/statistics",
            post(intelligence_handlers::get_intelligence_statistics),
        )
        .route(
            "/api/intelligence/config",
            post(intelligence_handlers::update_intelligence_config),
        )
        // Workflow API endpoints
        .route(
            "/api/workflow/intelligent",
            post(workflow_handlers::execute_intelligent_workflow),
        )
        .route(
            "/api/workflow/simple",
            post(workflow_handlers::execute_simple_workflow),
        )
        .route(
            "/api/workflow/status",
            post(workflow_handlers::get_workflow_status),
        )
        // Static files (serve our migrated interface)
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(dashboard))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind to loopback and gracefully retry on EADDRINUSE (Windows 10048)
    let (listener, actual_port) = bind_with_retry(port, 3).await?;
    let addr = format_addr(actual_port);
    info!("API server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

// Legacy serve function for fallback when coordinator fails
async fn serve_legacy(
    port: u16,
    browser_pool_arc: Arc<BrowserPool>,
    session_manager: SessionManager,
) -> Result<()> {
    info!("Starting API server in LEGACY mode (coordinator unavailable)");

    let session_manager_arc = Arc::new(session_manager);
    let state = AppState {
        browser_pool: browser_pool_arc.clone(),
        session_manager: session_manager_arc.clone(),
        tool_registry: Arc::new(LazyToolRegistry::new(browser_pool_arc, session_manager_arc)),
        recent_nav: Arc::new(RwLock::new(HashMap::new())),
    };

    // Build app without coordinated endpoints
    let app = build_legacy_app(state);

    let (listener, actual_port) = bind_with_retry(port, 3).await?;
    let addr = format_addr(actual_port);
    info!("API server (legacy mode) listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

// Build legacy app without coordination
fn build_legacy_app(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .route("/api/diagnostics", get(diagnostics))
        .route(
            "/api/routes",
            get(|| async move {
                Json(ApiResponse::success(vec![
                    "/api/health",
                    "/api/diagnostics",
                    "/api/session/create",
                    "/api/sessions",
                    "/api/navigate",
                    "/api/perception/analyze",
                    "/api/perceive-mode",
                    "/api/navigate-perceive",
                    "/api/tools/execute",
                ]))
            }),
        )
        // All the existing non-coordinated endpoints
        .route("/api/session/create", post(create_session))
        .route("/api/session/:id", get(get_session).delete(delete_session))
        .route("/api/sessions", get(list_sessions))
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
        .route("/api/workflow", post(execute_workflow))
        .route("/api/tools", get(list_tools))
        .route("/api/tools/execute", post(execute_tool))
        .route("/api/tools/metadata", get(get_tools_metadata))
        .route("/api/tools/validate", post(validate_registry))
        .route(
            "/api/tools/performance/stats",
            get(get_all_performance_stats),
        )
        .route(
            "/api/tools/performance/stats/:tool_name",
            get(get_tool_performance_stats),
        )
        .route(
            "/api/tools/performance/metrics",
            get(get_recent_performance_metrics),
        )
        .route(
            "/api/tools/performance/metrics/:tool_name",
            get(get_tool_performance_metrics),
        )
        .route(
            "/api/tools/performance/clear",
            post(clear_performance_metrics),
        )
        .route("/api/tools/cache/stats", get(get_cache_stats))
        .route("/api/tools/cache/clear", post(clear_all_cache))
        .route("/api/tools/cache/clear/:tool_name", post(clear_tool_cache))
        .route(
            "/api/tools/cache/config/:tool_name",
            post(set_tool_cache_config),
        )
        .route("/api/tools/dependencies/plan", post(create_execution_plan))
        .route(
            "/api/tools/dependencies/execute",
            post(execute_with_dependencies),
        )
        .route(
            "/api/tools/dependencies/register",
            post(register_tool_dependencies),
        )
        .route(
            "/api/tools/dependencies/:tool_name",
            get(get_tool_dependencies),
        )
        .route(
            "/api/tools/dependencies/dependents/:tool_name",
            get(get_dependent_tools),
        )
        .route("/api/tools/dependencies/stats", get(get_dependency_stats))
        .route(
            "/api/perception/analyze",
            post(perception_handlers::analyze_page),
        )
        .route(
            "/api/perception/find",
            post(perception_handlers::intelligent_find_element),
        )
        .route(
            "/api/perception/command",
            post(perception_handlers::execute_intelligent_command),
        )
        .route(
            "/api/perception/forms/analyze",
            post(perception_handlers::analyze_form),
        )
        .route(
            "/api/perception/forms/fill",
            post(perception_handlers::auto_fill_form),
        )
        // Combined navigate + perceive (available in legacy mode too)
        .route(
            "/api/navigate-perceive",
            post(perception_handlers::navigate_and_perceive),
        )
        .route(
            "/api/perceive-mode",
            post(perception_handlers::perceive_with_mode),
        )
        .route("/api/quick-scan", post(perception_handlers::quick_scan))
        .route(
            "/api/smart-element-search",
            post(perception_handlers::smart_element_search),
        )
        .route(
            "/api/perception/smart_search",
            post(perception_handlers::smart_element_search),
        )
        .route(
            "/api/perception/find_element",
            post(perception_handlers::intelligent_find_element),
        )
        .route("/api/llm/query", post(llm_handlers::llm_query))
        .route("/api/llm/plan", post(llm_handlers::task_planning))
        .route("/api/llm/execute", post(llm_handlers::execute_command))
        .route("/api/llm/usage", post(llm_handlers::get_usage_metrics))
        .route(
            "/api/intelligence/analyze",
            post(intelligence_handlers::analyze_situation),
        )
        .route(
            "/api/intelligence/recommend",
            post(intelligence_handlers::recommend_action),
        )
        .route(
            "/api/intelligence/learn",
            post(intelligence_handlers::submit_learning_feedback),
        )
        .route(
            "/api/intelligence/statistics",
            post(intelligence_handlers::get_intelligence_statistics),
        )
        .route(
            "/api/intelligence/config",
            post(intelligence_handlers::update_intelligence_config),
        )
        .route(
            "/api/workflow/intelligent",
            post(workflow_handlers::execute_intelligent_workflow),
        )
        .route(
            "/api/workflow/simple",
            post(workflow_handlers::execute_simple_workflow),
        )
        .route(
            "/api/workflow/status",
            post(workflow_handlers::get_workflow_status),
        )
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(dashboard))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "rainbow-poc-chromiumoxide",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "binding": "127.0.0.1",
        "build": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": env!("BUILD_TIMESTAMP"),
            "git": option_env!("BUILD_GIT_HASH").unwrap_or("unknown"),
        }
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

// Diagnostics endpoint
async fn diagnostics(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let probe = params.get("probe").map(|v| v == "true").unwrap_or(false);

    // Is tool registry initialized?
    let initialized = state.tool_registry.initialized().await;

    // Optionally probe browser availability by attempting to acquire once
    let mut browser_status = serde_json::json!({
        "initialized": initialized,
        "probe_attempted": false,
        "can_launch": serde_json::Value::Null,
        "error": serde_json::Value::Null
    });

    if probe {
        match state.browser_pool.acquire().await {
            Ok(_browser) => {
                browser_status["probe_attempted"] = serde_json::json!(true);
                browser_status["can_launch"] = serde_json::json!(true);
            }
            Err(e) => {
                browser_status["probe_attempted"] = serde_json::json!(true);
                browser_status["can_launch"] = serde_json::json!(false);
                browser_status["error"] = serde_json::json!(e.to_string());
            }
        }
    }

    let response = serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "binding": "127.0.0.1",
        "browser": browser_status,
    });

    Json(ApiResponse::success(response)).into_response()
}

// Request/Response types
#[derive(Deserialize)]
struct NavigateRequest {
    url: String,
    #[serde(default)]
    session_id: Option<String>,
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
async fn navigate(State(state): State<AppState>, Json(req): Json<NavigateRequest>) -> Response {
    // If a session_id is provided, navigate using that session's browser and update its state
    if let Some(session_id) = &req.session_id {
        if let Some(session_arc) = state.session_manager.get_session(session_id).await {
            let mut session = session_arc.write().await;
            match session.navigate(&req.url).await {
                Ok(_) => {
                    // Promote this browser as the active tool-registry browser for non-session flows
                    let browser_arc = session.browser.clone();
                    tokio::spawn({
                        let tr = state.tool_registry.clone();
                        async move {
                            tr.set_active_browser(browser_arc).await;
                        }
                    });
                    // Record recent navigation for this session
                    {
                        let mut map = state.recent_nav.write().await;
                        map.insert(session_id.clone(), req.url.clone());
                    }
                    return Json(ApiResponse::success(serde_json::json!({
                        "url": req.url,
                        "status": "navigated",
                        "session_id": session_id,
                        "session": true
                    })))
                    .into_response();
                }
                Err(e) => {
                    error!("Session navigation failed: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(e.to_string())),
                    )
                        .into_response();
                }
            }
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(format!(
                    "Session not found: {}",
                    session_id
                ))),
            )
                .into_response();
        }
    }

    // Fallback: use a pooled browser (legacy behavior)
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.navigate_to(&req.url).await {
            Ok(_) => {
                // Promote this pool browser as the active tool-registry browser for non-session flows
                let browser_arc = browser.browser_arc();
                {
                    let tr = state.tool_registry.clone();
                    tokio::spawn(async move {
                        tr.set_active_browser(browser_arc).await;
                    });
                }
                // Record global recent navigation hint
                {
                    let mut map = state.recent_nav.write().await;
                    map.insert("__global__".to_string(), req.url.clone());
                }
                Json(ApiResponse::success(serde_json::json!({
                    "url": req.url,
                    "status": "navigated",
                    "session": false
                })))
                .into_response()
            }
            Err(e) => {
                error!("Navigation failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn screenshot(State(state): State<AppState>, Json(req): Json<ScreenshotRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            // Navigate if URL provided
            if let Some(url) = req.url {
                if let Err(e) = browser.navigate_to(&url).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(e.to_string())),
                    )
                        .into_response();
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
                    })))
                    .into_response()
                }
                Err(e) => {
                    error!("Screenshot failed: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(e.to_string())),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn click(State(state): State<AppState>, Json(req): Json<ClickRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.click(&req.selector).await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "selector": req.selector,
                "action": "clicked"
            })))
            .into_response(),
            Err(e) => {
                error!("Click failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn type_text(State(state): State<AppState>, Json(req): Json<TypeRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.type_text(&req.selector, &req.text).await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "selector": req.selector,
                "text": req.text,
                "action": "typed"
            })))
            .into_response(),
            Err(e) => {
                error!("Type failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn execute_script(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.execute_script(&req.script).await {
            Ok(result) => Json(ApiResponse::success(result)).into_response(),
            Err(e) => {
                error!("Script execution failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn find_elements(State(state): State<AppState>, Json(req): Json<FindRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.find_elements(&req.selector).await {
            Ok(elements) => Json(ApiResponse::success(elements)).into_response(),
            Err(e) => {
                error!("Find elements failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn get_text(State(state): State<AppState>, Json(req): Json<GetTextRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.get_text(&req.selector).await {
            Ok(text) => Json(ApiResponse::success(serde_json::json!({
                "selector": req.selector,
                "text": text
            })))
            .into_response(),
            Err(e) => {
                error!("Get text failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn scroll(State(state): State<AppState>, Json(req): Json<ScrollRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.scroll_to(req.x, req.y).await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "x": req.x,
                "y": req.y,
                "action": "scrolled"
            })))
            .into_response(),
            Err(e) => {
                error!("Scroll failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn set_zoom_level(State(state): State<AppState>, Json(req): Json<ZoomRequest>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.set_zoom_level(req.zoom_factor).await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "zoom_factor": req.zoom_factor,
                "action": "zoom_set"
            })))
            .into_response(),
            Err(e) => {
                error!("Set zoom level failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn fix_content_scaling(State(state): State<AppState>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.fix_content_scaling().await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "action": "content_scaling_fixed",
                "message": "Viewport and content scaling fixed"
            })))
            .into_response(),
            Err(e) => {
                error!("Fix content scaling failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn fix_window_completely(State(state): State<AppState>) -> Response {
    match state.browser_pool.acquire().await {
        Ok(browser) => match browser.fix_window_completely().await {
            Ok(_) => Json(ApiResponse::success(serde_json::json!({
                "action": "window_completely_fixed",
                "message": "Screenshot trigger + content scaling applied"
            })))
            .into_response(),
            Err(e) => {
                error!("Complete window fix failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn execute_workflow(
    State(state): State<AppState>,
    Json(_workflow): Json<serde_json::Value>,
) -> Response {
    // TODO: Implement workflow execution
    let _ = state; // Suppress unused warning
    Json(ApiResponse::<()>::error(
        "Workflow execution not yet implemented".to_string(),
    ))
    .into_response()
}

// Session management handlers
async fn create_session(State(state): State<AppState>) -> Response {
    match state.session_manager.create_session().await {
        Ok(session_id) => Json(ApiResponse::success(serde_json::json!({
            "session_id": session_id,
            "created": true
        })))
        .into_response(),
        Err(e) => {
            error!("Failed to create session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
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
        })))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!(
                "Session not found: {}",
                id
            ))),
        )
            .into_response()
    }
}

async fn delete_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    match state.session_manager.remove_session(&id).await {
        Ok(_) => Json(ApiResponse::success(serde_json::json!({
            "deleted": true,
            "session_id": id
        })))
        .into_response(),
        Err(e) => {
            error!("Failed to delete session: {}", e);
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn list_sessions(State(state): State<AppState>) -> Response {
    let sessions = state.session_manager.list_sessions().await;
    Json(ApiResponse::success(sessions)).into_response()
}

// Tools API handlers
async fn list_tools(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let summary = registry.get_summary();
    let metadata = registry.get_all_metadata();

    // Group tools by category
    let mut tools_by_category = std::collections::HashMap::new();

    for (tool_name, tool_metadata) in &metadata {
        tools_by_category
            .entry(tool_metadata.category)
            .or_insert_with(Vec::new)
            .push(serde_json::json!({
                "name": tool_name,
                "description": tool_metadata.description,
                "version": tool_metadata.version,
                "author": tool_metadata.author
            }));
    }

    let response = serde_json::json!({
        "summary": {
            "total_tools": summary.total_tools,
            "categories": summary.categories,
        },
        "tools_by_category": tools_by_category,
        "all_tools": summary.tool_names
    });

    Json(ApiResponse::success(response)).into_response()
}

#[derive(Deserialize)]
struct ExecuteToolRequest {
    tool_name: String,
    parameters: serde_json::Value,
    session_id: Option<String>, // Add session_id field
}

async fn execute_tool(
    State(state): State<AppState>,
    Json(req): Json<ExecuteToolRequest>,
) -> Response {
    info!(
        "Executing tool: {} with parameters: {} (session: {:?})",
        req.tool_name, req.parameters, req.session_id
    );

    // If session_id is provided, use the session's browser
    let registry = if let Some(session_id) = &req.session_id {
        // Get the session's browser
        if let Some(session) = state.session_manager.get_session(session_id).await {
            info!(
                "Using browser from session {} for tool execution",
                session_id
            );
            let session_guard = session.read().await;
            // Create a tool registry with the session's browser
            let browser_arc = session_guard.browser.clone();
            // Promote as active browser for cross-module sharing
            let tr = state.tool_registry.clone();
            let browser_arc_for_promotion = browser_arc.clone();
            tokio::spawn(async move {
                tr.set_active_browser(browser_arc_for_promotion).await;
            });
            // Use a separate clone for registry construction
            let browser_arc_for_registry = browser_arc.clone();
            Arc::new(ToolRegistry::new(browser_arc_for_registry))
        } else {
            warn!(
                "Session {} not found, falling back to pool browser",
                session_id
            );
            // Fallback to getting registry from pool
            match state.tool_registry.get().await {
                Ok(r) => r,
                Err(e) => {
                    error!("Tool registry unavailable: {}", e);
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(ApiResponse::<()>::error(
                            "Tool registry not initialized (browser unavailable)".to_string(),
                        )),
                    )
                        .into_response();
                }
            }
        }
    } else {
        // No session_id, use the registry from pool
        match state.tool_registry.get().await {
            Ok(r) => r,
            Err(e) => {
                error!("Tool registry unavailable: {}", e);
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::<()>::error(
                        "Tool registry not initialized (browser unavailable)".to_string(),
                    )),
                )
                    .into_response();
            }
        }
    };

    match registry
        .execute_tool(&req.tool_name, req.parameters.clone())
        .await
    {
        Ok(result) => {
            debug!("Tool '{}' executed successfully", req.tool_name);

            // If this was a session-bound tool execution, update session state to reflect the real page
            if let Some(session_id) = req.session_id.clone() {
                if let Some(session_arc) = state.session_manager.get_session(&session_id).await {
                    let tr = state.tool_registry.clone();
                    tokio::spawn(async move {
                        // Update last_used and current_url from the browser
                        let current_url = {
                            let s = session_arc.read().await;
                            s.browser
                                .current_url()
                                .await
                                .unwrap_or_else(|_| "".to_string())
                        };
                        if !current_url.is_empty() {
                            let mut s = session_arc.write().await;
                            if let Some(prev) = s.current_url.clone() {
                                s.history.push(prev);
                            }
                            s.current_url = Some(current_url);
                            s.touch();
                            // Promote this browser as active for cross-module sharing
                            tr.set_active_browser(s.browser.clone()).await;
                        }
                    });
                }
                // Also record recent navigation if this was navigate_to_url
                if req.tool_name == "navigate_to_url" {
                    // Extract URL from input parameters
                    let url = req
                        .parameters
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !url.is_empty() {
                        let mut map = state.recent_nav.write().await;
                        map.insert(session_id.clone(), url.to_string());
                    }
                }
            }
            // For non-session flows, if it was navigate_to_url, set global recent nav intent
            if req.session_id.is_none() && req.tool_name == "navigate_to_url" {
                let url = req
                    .parameters
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !url.is_empty() {
                    // Promote current active browser if registry exists
                    if let Ok(r) = state.tool_registry.get().await {
                        // registry initialization ensures an active browser is set in LazyToolRegistry
                        let _ = r; // no-op, just ensures get() side effects
                    }
                    let mut map = state.recent_nav.write().await;
                    map.insert("__global__".to_string(), url.to_string());
                }
            }
            Json(ApiResponse::success(result)).into_response()
        }
        Err(e) => {
            error!("Tool '{}' execution failed: {}", req.tool_name, e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

// Tool metadata and validation endpoints
async fn get_tools_metadata(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let metadata = registry.get_all_metadata();

    Json(ApiResponse::success(metadata)).into_response()
}

async fn validate_registry(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    match registry.validate_registry().await {
        Ok(report) => Json(ApiResponse::success(report)).into_response(),
        Err(e) => {
            error!("Registry validation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

// Performance Monitoring API handlers

async fn get_all_performance_stats(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let stats = registry.get_all_performance_stats().await;
    Json(ApiResponse::success(stats)).into_response()
}

async fn get_tool_performance_stats(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    match registry.get_tool_performance_stats(&tool_name).await {
        Some(stats) => Json(ApiResponse::success(stats)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!(
                "No performance data found for tool '{}'",
                tool_name
            ))),
        )
            .into_response(),
    }
}

async fn get_recent_performance_metrics(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50); // Default to 50 recent metrics

    let metrics = registry.get_recent_metrics(limit).await;
    Json(ApiResponse::success(metrics)).into_response()
}

async fn get_tool_performance_metrics(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let limit = params.get("limit").and_then(|s| s.parse::<usize>().ok());

    let metrics = registry.get_tool_metrics(&tool_name, limit).await;
    Json(ApiResponse::success(metrics)).into_response()
}

async fn clear_performance_metrics(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    registry.clear_performance_metrics().await;
    Json(ApiResponse::success(serde_json::json!({
        "message": "Performance metrics cleared successfully"
    })))
    .into_response()
}

// Cache Management API handlers

async fn get_cache_stats(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let stats = registry.get_cache_stats().await;
    Json(ApiResponse::success(stats)).into_response()
}

async fn clear_all_cache(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    registry.clear_cache().await;

    Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": "All cache entries cleared successfully"
    })))
    .into_response()
}

async fn clear_tool_cache(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    registry.clear_tool_cache(&tool_name).await;

    Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": format!("Cache cleared successfully for tool '{}'", tool_name)
    })))
    .into_response()
}

#[derive(Deserialize)]
struct CacheConfigRequest {
    ttl_seconds: Option<u64>,
    max_entries: Option<usize>,
    enabled: Option<bool>,
    invalidate_on_navigation: Option<bool>,
}

async fn set_tool_cache_config(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Json(req): Json<CacheConfigRequest>,
) -> Response {
    use crate::tools::cache::CacheConfig;
    use std::time::Duration;

    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Get current config or use default
    let current_config = registry.cache.get_tool_config(&tool_name).await;

    let new_config = CacheConfig {
        ttl: Duration::from_secs(req.ttl_seconds.unwrap_or(current_config.ttl.as_secs())),
        max_entries: req.max_entries.unwrap_or(current_config.max_entries),
        enabled: req.enabled.unwrap_or(current_config.enabled),
        invalidate_on_navigation: req
            .invalidate_on_navigation
            .unwrap_or(current_config.invalidate_on_navigation),
    };

    registry.set_tool_cache_config(&tool_name, new_config).await;

    Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": format!("Cache configuration updated successfully for tool '{}'", tool_name)
    })))
    .into_response()
}

// Dependency Management API handlers

#[derive(Deserialize)]
struct ExecutionPlanRequest {
    tool_names: Vec<String>,
}

async fn create_execution_plan(
    State(state): State<AppState>,
    Json(req): Json<ExecutionPlanRequest>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    match registry.create_execution_plan(req.tool_names).await {
        Ok(plan) => Json(ApiResponse::success(plan)).into_response(),
        Err(e) => {
            error!("Failed to create execution plan: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

async fn execute_with_dependencies(
    State(state): State<AppState>,
    Json(req): Json<ExecutionPlanRequest>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    match registry
        .execute_tools_with_dependencies(req.tool_names)
        .await
    {
        Ok(context) => Json(ApiResponse::success(context)).into_response(),
        Err(e) => {
            error!("Failed to execute tools with dependencies: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct RegisterDependenciesRequest {
    tool_name: String,
    dependencies: Vec<crate::tools::dependencies::ToolDependency>,
}

async fn register_tool_dependencies(
    State(state): State<AppState>,
    Json(req): Json<RegisterDependenciesRequest>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    registry
        .register_tool_dependencies(req.tool_name.clone(), req.dependencies.clone())
        .await;

    Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": format!("Registered {} dependencies for tool '{}'", req.dependencies.len(), req.tool_name)
    }))).into_response()
}

async fn get_tool_dependencies(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let dependencies = registry.get_tool_dependencies(&tool_name).await;
    Json(ApiResponse::success(dependencies)).into_response()
}

async fn get_dependent_tools(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let dependents = registry.get_dependent_tools(&tool_name).await;
    Json(ApiResponse::success(dependents)).into_response()
}

async fn get_dependency_stats(State(state): State<AppState>) -> Response {
    let registry = match state.tool_registry.get().await {
        Ok(r) => r,
        Err(e) => {
            error!("Tool registry unavailable: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiResponse::<()>::error(
                    "Tool registry not initialized (browser unavailable)".to_string(),
                )),
            )
                .into_response();
        }
    };
    let stats = registry.get_dependency_stats().await;
    Json(ApiResponse::success(stats)).into_response()
}
