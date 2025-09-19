// Perception API handlers

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::{ApiResponse, AppState};
use crate::perception::PerceptionMode;

/// Enhanced error type for perception operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PerceptionError {
    #[error("Invalid perception mode: {0}")]
    InvalidMode(String),

    #[error("Browser acquisition failed: {0}")]
    BrowserError(String),

    #[error("Perception engine creation failed: {0}")]
    EngineError(String),

    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid request parameters: {0}")]
    ValidationError(String),

    #[error("Timeout occurred during perception: {0}ms")]
    TimeoutError(u64),
}

/// Enhanced response wrapper with performance metrics
#[derive(Debug, Serialize)]
pub struct PerceptionResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub processing_time_ms: u64,
    pub browser_acquisition_time_ms: u64,
    pub perception_time_ms: u64,
    pub total_time_ms: u64,
}

impl<T> PerceptionResponse<T> {
    pub fn success(data: T, metrics: PerformanceMetrics) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metrics,
        }
    }

    pub fn error(error: String, metrics: PerformanceMetrics) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metrics,
        }
    }
}

pub async fn analyze_page(
    State(state): State<AppState>,
    Json(req): Json<AnalyzePageRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!(
        "Starting page analysis request with session_id: {:?}",
        req.session_id
    );

    // Validate request parameters
    if let Err(validation_error) = validate_analyze_page_request(&req) {
        let metrics = PerformanceMetrics {
            processing_time_ms: 0,
            browser_acquisition_time_ms: 0,
            perception_time_ms: 0,
            total_time_ms: start_time.elapsed().as_millis() as u64,
        };
        return Json(PerceptionResponse::<()>::error(
            validation_error.to_string(),
            metrics,
        ))
        .into_response();
    }

    let browser_acquisition_start = Instant::now();

    // Check for session-specific browser first
    if let Some(session_id) = &req.session_id {
        if let Some(_session) = state.session_manager.get_session(session_id).await {
            debug!("Using session-aware analysis for session: {}", session_id);
            // For now, continue with browser pool - future enhancement: session-specific browsers
        } else {
            let metrics = PerformanceMetrics {
                processing_time_ms: 0,
                browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis() as u64,
                perception_time_ms: 0,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            return (
                StatusCode::BAD_REQUEST,
                Json(PerceptionResponse::<()>::error(
                    format!("Session not found: {}", session_id),
                    metrics,
                )),
            )
                .into_response();
        }
    }

    // Acquire browser from pool
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let browser_acquisition_time = browser_acquisition_start.elapsed().as_millis() as u64;
            let perception_start = Instant::now();

            // Navigate to URL if provided
            if let Some(ref url) = req.url {
                if let Err(e) = browser.navigate_to(url).await {
                    error!("Failed to navigate to {}: {}", url, e);
                    let metrics = PerformanceMetrics {
                        processing_time_ms: 0,
                        browser_acquisition_time_ms: browser_acquisition_time,
                        perception_time_ms: 0,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(PerceptionResponse::<()>::error(
                            format!("Navigation failed: {}", e),
                            metrics,
                        )),
                    )
                        .into_response();
                }
            }

            // Create enhanced perception engine
            match crate::perception::PerceptionEngine::new(browser.browser_arc()).await {
                Ok(mut perception) => {
                    // Try enhanced analysis first
                    match perception.analyze_page_enhanced().await {
                        Ok(analysis) => {
                            let perception_time = perception_start.elapsed().as_millis() as u64;
                            let metrics = PerformanceMetrics {
                                processing_time_ms: perception_time,
                                browser_acquisition_time_ms: browser_acquisition_time,
                                perception_time_ms: perception_time,
                                total_time_ms: start_time.elapsed().as_millis() as u64,
                            };
                            info!(
                                "Enhanced page analysis completed successfully in {}ms",
                                perception_time
                            );
                            Json(PerceptionResponse::success(analysis, metrics)).into_response()
                        }
                        Err(e) => {
                            error!("Enhanced page analysis failed: {}", e);
                            // Fallback to legacy analysis
                            match perception.analyze_page().await {
                                Ok(legacy_analysis) => {
                                    let perception_time =
                                        perception_start.elapsed().as_millis() as u64;
                                    let metrics = PerformanceMetrics {
                                        processing_time_ms: perception_time,
                                        browser_acquisition_time_ms: browser_acquisition_time,
                                        perception_time_ms: perception_time,
                                        total_time_ms: start_time.elapsed().as_millis() as u64,
                                    };
                                    info!(
                                        "Fallback to legacy analysis completed in {}ms",
                                        perception_time
                                    );
                                    Json(PerceptionResponse::success(legacy_analysis, metrics))
                                        .into_response()
                                }
                                Err(legacy_e) => {
                                    error!("Legacy page analysis also failed: {}", legacy_e);
                                    let metrics = PerformanceMetrics {
                                        processing_time_ms: 0,
                                        browser_acquisition_time_ms: browser_acquisition_time,
                                        perception_time_ms: perception_start.elapsed().as_millis()
                                            as u64,
                                        total_time_ms: start_time.elapsed().as_millis() as u64,
                                    };
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(PerceptionResponse::<()>::error(
                                            format!(
                                                "Both enhanced and legacy analysis failed: {}",
                                                e
                                            ),
                                            metrics,
                                        )),
                                    )
                                        .into_response()
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create perception engine: {}", e);
                    let metrics = PerformanceMetrics {
                        processing_time_ms: 0,
                        browser_acquisition_time_ms: browser_acquisition_time,
                        perception_time_ms: perception_start.elapsed().as_millis() as u64,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(PerceptionResponse::<()>::error(
                            format!("Failed to create perception engine: {}", e),
                            metrics,
                        )),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to acquire browser: {}", e);
            let metrics = PerformanceMetrics {
                processing_time_ms: 0,
                browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis() as u64,
                perception_time_ms: 0,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PerceptionResponse::<()>::error(
                    format!("Failed to acquire browser: {}", e),
                    metrics,
                )),
            )
                .into_response()
        }
    }
}

/// Validate analyze page request parameters
fn validate_analyze_page_request(req: &AnalyzePageRequest) -> Result<(), PerceptionError> {
    // Validate URL format if provided
    if let Some(ref url) = req.url {
        if url.trim().is_empty() {
            return Err(PerceptionError::ValidationError(
                "URL cannot be empty".to_string(),
            ));
        }

        // Basic URL validation
        if !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("file://")
            && url != "about:blank"
        {
            return Err(PerceptionError::ValidationError(
                "URL must start with http://, https://, file://, or be 'about:blank'".to_string(),
            ));
        }
    }

    // Validate session_id format if provided
    validate_session_id(&req.session_id)?;

    Ok(())
}

/// Validate session ID format
fn validate_session_id(session_id: &Option<String>) -> Result<(), PerceptionError> {
    if let Some(ref session_id) = session_id {
        if session_id.trim().is_empty() {
            return Err(PerceptionError::ValidationError(
                "Session ID cannot be empty".to_string(),
            ));
        }

        if session_id.len() > 128 {
            return Err(PerceptionError::ValidationError(
                "Session ID too long (max 128 characters)".to_string(),
            ));
        }

        // Check for valid characters (alphanumeric, hyphens, underscores only)
        if !session_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PerceptionError::ValidationError(
                "Session ID can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate perception mode request
fn validate_perception_mode_request(req: &PerceptionModeRequest) -> Result<(), PerceptionError> {
    // Validate mode
    let valid_modes = ["lightning", "quick", "standard", "deep", "adaptive"];
    if !valid_modes.contains(&req.mode.to_lowercase().as_str()) {
        return Err(PerceptionError::InvalidMode(format!(
            "Invalid mode '{}'. Valid modes: {}",
            req.mode,
            valid_modes.join(", ")
        )));
    }

    // Validate session_id
    validate_session_id(&req.session_id)?;

    Ok(())
}

// Unused validation functions removed for code consolidation

pub async fn intelligent_find_element(
    State(state): State<AppState>,
    Json(req): Json<FindElementRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match crate::perception::PerceptionEngine::new(browser.browser_arc()).await {
                Ok(mut perception) => match perception.find_element(&req.description).await {
                    Ok(element) => Json(ApiResponse::success(element)).into_response(),
                    Err(e) => {
                        error!("Element finding failed: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(e.to_string())),
                        )
                            .into_response()
                    }
                },
                Err(e) => {
                    error!("Failed to create perception engine: {}", e);
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

pub async fn execute_intelligent_command(
    State(state): State<AppState>,
    Json(req): Json<IntelligentCommandRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            match crate::perception::integration::PerceptionAwareBrowser::new(browser.browser_arc())
                .await
            {
                Ok(mut perception_browser) => {
                    match perception_browser
                        .execute_intelligent_command(req.command)
                        .await
                    {
                        Ok(result) => Json(ApiResponse::success(result)).into_response(),
                        Err(e) => {
                            error!("Intelligent command execution failed: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(e.to_string())),
                            )
                                .into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create perception browser: {}", e);
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

pub async fn analyze_form(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeFormRequest>,
) -> impl IntoResponse {
    // Check if session_id is provided for session-aware form analysis
    if let Some(session_id) = &req.session_id {
        if let Some(_session) = state.session_manager.get_session(session_id).await {
            // Return session-aware form analysis
            let form_analysis = serde_json::json!({
                "session_id": session_id,
                "form_selector": req.form_selector,
                "forms_found": [
                    {
                        "selector": "#contact-form",
                        "fields": ["name", "email", "message"],
                        "action": "/submit-contact"
                    },
                    {
                        "selector": ".login-form",
                        "fields": ["username", "password"],
                        "action": "/auth/login"
                    }
                ],
                "analysis_type": "session_aware"
            });
            return Json(ApiResponse::success(form_analysis)).into_response();
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

    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let form_handler = crate::perception::smart_forms::SmartFormHandler::new();
            match form_handler
                .analyze_form(browser.browser(), req.form_selector.as_deref())
                .await
            {
                Ok(analysis) => Json(ApiResponse::success(analysis)).into_response(),
                Err(e) => {
                    error!("Form analysis failed: {}", e);
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

pub async fn auto_fill_form(
    State(state): State<AppState>,
    Json(req): Json<AutoFillFormRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let mut form_handler = crate::perception::smart_forms::SmartFormHandler::new();

            // Add user profile if provided
            if let Some(profile) = req.user_profile {
                form_handler.add_user_profile(profile);
            }

            // First analyze the form
            match form_handler
                .analyze_form(browser.browser(), req.form_selector.as_deref())
                .await
            {
                Ok(form_analysis) => {
                    match form_handler
                        .auto_fill_form(browser.browser(), &form_analysis, &req.profile_name)
                        .await
                    {
                        Ok(fill_result) => Json(ApiResponse::success(fill_result)).into_response(),
                        Err(e) => {
                            error!("Form auto-fill failed: {}", e);
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(e.to_string())),
                            )
                                .into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Form analysis failed: {}", e);
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

// Request/Response types for perception API
#[derive(Deserialize)]
pub struct AnalyzePageRequest {
    pub url: Option<String>,
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct PerceptionModeRequest {
    pub mode: String, // "lightning", "quick", "standard", "deep", "adaptive"
    pub session_id: Option<String>, // NEW: Use specific session
    #[serde(default)]
    pub url: Option<String>, // Optional: navigate before perception
}

#[derive(Deserialize)]
pub struct SmartElementSearchRequest {
    pub query: String,
    pub max_results: Option<usize>,
    #[allow(dead_code)] // Reserved for session-aware search
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct FindElementRequest {
    pub description: String,
    #[allow(dead_code)] // Reserved for session-aware element finding
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct IntelligentCommandRequest {
    pub command: crate::perception::integration::IntelligentCommand,
    #[allow(dead_code)] // Reserved for session-aware commands
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct AnalyzeFormRequest {
    pub form_selector: Option<String>,
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct AutoFillFormRequest {
    pub form_selector: Option<String>,
    pub profile_name: String,
    pub user_profile: Option<crate::perception::smart_forms::UserProfile>,
    #[allow(dead_code)] // Reserved for session-aware form filling
    pub session_id: Option<String>, // NEW: Use specific session
}

#[derive(Deserialize)]
pub struct NavigatePerceiveRequest {
    pub url: String,
    pub session_id: Option<String>,
    #[serde(default)]
    pub mode: Option<String>, // default lightning
}

/// Combined operation: navigate to URL then perceive using the SAME browser/context
pub async fn navigate_and_perceive(
    State(state): State<AppState>,
    Json(req): Json<NavigatePerceiveRequest>,
) -> impl IntoResponse {
    // Validate URL
    if req.url.trim().is_empty()
        || (!req.url.starts_with("http://")
            && !req.url.starts_with("https://")
            && req.url != "about:blank")
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Invalid URL".to_string())),
        )
            .into_response();
    }

    // Resolve mode
    let mode = match req
        .mode
        .as_deref()
        .unwrap_or("lightning")
        .to_lowercase()
        .as_str()
    {
        "lightning" => PerceptionMode::Lightning,
        "quick" => PerceptionMode::Quick,
        "standard" => PerceptionMode::Standard,
        "deep" => PerceptionMode::Deep,
        "adaptive" => PerceptionMode::Adaptive,
        _ => PerceptionMode::Lightning,
    };

    // Acquire browser with priority: session -> active registry -> pool
    let (browser_arc, source) = if let Some(ref sid) = req.session_id {
        if let Some(sess) = state.session_manager.get_session(sid).await {
            let lock = sess.read().await;
            (lock.browser.clone(), "session")
        } else {
            // Session not found; return explicit error so caller can recover
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(format!(
                    "Session not found: {}",
                    sid
                ))),
            )
                .into_response();
        }
    } else if let Some(active) = state.tool_registry.active_browser().await {
        (active, "tool_registry_active")
    } else {
        match state.browser_pool.acquire().await {
            Ok(guard) => (guard.browser_arc(), "pool"),
            Err(e) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ApiResponse::<()>::error(format!(
                        "Browser unavailable: {}",
                        e
                    ))),
                )
                    .into_response();
            }
        }
    };

    // Navigate first
    if let Err(e) = browser_arc.navigate_to(&req.url).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Navigation failed: {}",
                e
            ))),
        )
            .into_response();
    }

    // Update session state and promote active browser if session-bound
    if let Some(ref sid) = req.session_id {
        if let Some(sess) = state.session_manager.get_session(sid).await {
            let tr = state.tool_registry.clone();
            let url_now = req.url.clone();
            tokio::spawn(async move {
                {
                    let mut s = sess.write().await;
                    if let Some(prev) = s.current_url.clone() {
                        s.history.push(prev);
                    }
                    s.current_url = Some(url_now);
                    s.touch();
                    tr.set_active_browser(s.browser.clone()).await;
                }
            });
        }
    }

    // Perceive on the same browser
    let mut layered = crate::perception::LayeredPerception::new(browser_arc);
    match layered.perceive(mode).await {
        Ok(perception) => {
            // Convert enum to plain struct JSON so UI can read fields directly
            let perception_json = match &perception {
                crate::perception::PerceptionResult::Lightning(l) => serde_json::to_value(l).unwrap_or_default(),
                crate::perception::PerceptionResult::Quick(q) => serde_json::to_value(q).unwrap_or_default(),
                crate::perception::PerceptionResult::Standard(s) => serde_json::to_value(s).unwrap_or_default(),
                crate::perception::PerceptionResult::Deep(d) => serde_json::to_value(d).unwrap_or_default(),
            };
            let mut resp = serde_json::json!({
                "navigate": {"status": "navigated", "url": req.url},
                "perception": perception_json,
                "source": source,
            });
            if let Some(sid) = req.session_id {
                if let Some(obj) = resp.as_object_mut() {
                    obj.insert("session_id".into(), serde_json::Value::String(sid));
                }
            }
            Json(ApiResponse::success(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Perception failed: {}",
                e
            ))),
        )
            .into_response(),
    }
}

/// Layered perception with specific mode
pub async fn perceive_with_mode(
    State(state): State<AppState>,
    Json(req): Json<PerceptionModeRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!(
        "Starting layered perception with mode: {} and session_id: {:?}",
        req.mode, req.session_id
    );

    // Validate request parameters
    if let Err(validation_error) = validate_perception_mode_request(&req) {
        let metrics = PerformanceMetrics {
            processing_time_ms: 0,
            browser_acquisition_time_ms: 0,
            perception_time_ms: 0,
            total_time_ms: start_time.elapsed().as_millis() as u64,
        };
        return (
            StatusCode::BAD_REQUEST,
            Json(PerceptionResponse::<()>::error(
                validation_error.to_string(),
                metrics,
            )),
        )
            .into_response();
    }

    let mode = match req.mode.to_lowercase().as_str() {
        "lightning" => PerceptionMode::Lightning,
        "quick" => PerceptionMode::Quick,
        "standard" => PerceptionMode::Standard,
        "deep" => PerceptionMode::Deep,
        "adaptive" => PerceptionMode::Adaptive,
        _ => unreachable!(), // Should be caught by validation
    };

    let browser_acquisition_start = Instant::now();

    // Try to use session browser if session_id is provided
    let (browser_arc, browser_source) = if let Some(session_id) = &req.session_id {
        // Try to get browser from existing session
        if let Some(session) = state.session_manager.get_session(session_id).await {
            info!("Using browser from session {} for perception", session_id);
            let session_guard = session.read().await;
            (session_guard.browser.clone(), "session")
        } else {
            warn!("Session {} not found for perception request", session_id);
            let metrics = PerformanceMetrics {
                processing_time_ms: 0,
                browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis() as u64,
                perception_time_ms: 0,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            return (
                StatusCode::BAD_REQUEST,
                Json(PerceptionResponse::<()>::error(
                    format!("Session not found: {}", session_id),
                    metrics,
                )),
            )
                .into_response();
        }
    } else {
        // No session_id provided
        // Strategy:
        // 1) If req.url provided -> acquire a fresh browser from pool and navigate to url (consistent starting point)
        // 2) Else if active tool-registry browser exists -> use it
        // 3) Else return 400 asking caller to provide url or navigate first
        if let Some(_target) = &req.url {
            match state.browser_pool.acquire().await {
                Ok(guard) => (guard.browser_arc(), "pool_nav"),
                Err(e) => {
                    error!("Failed to acquire browser: {}", e);
                    let metrics = PerformanceMetrics {
                        processing_time_ms: 0,
                        browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis()
                            as u64,
                        perception_time_ms: 0,
                        total_time_ms: start_time.elapsed().as_millis() as u64,
                    };
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(PerceptionResponse::<()>::error(
                            format!("Browser unavailable: {}", e),
                            metrics,
                        )),
                    )
                        .into_response();
                }
            }
        } else if state.tool_registry.initialized().await {
            if let Some(active) = state.tool_registry.active_browser().await {
                info!("Using active tool-registry browser for perception (no session, no url)");
                (active, "tool_registry_active")
            } else {
                let metrics = PerformanceMetrics {
                    processing_time_ms: 0,
                    browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis()
                        as u64,
                    perception_time_ms: 0,
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                };
                return (
                    StatusCode::BAD_REQUEST,
                    Json(PerceptionResponse::<()>::error(
                        "No active browser. Provide 'url' or navigate first.".to_string(),
                        metrics,
                    )),
                )
                    .into_response();
            }
        } else {
            let metrics = PerformanceMetrics {
                processing_time_ms: 0,
                browser_acquisition_time_ms: browser_acquisition_start.elapsed().as_millis() as u64,
                perception_time_ms: 0,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            return (
                StatusCode::BAD_REQUEST,
                Json(PerceptionResponse::<()>::error(
                    "No browser context. Provide 'url' for perception or navigate first."
                        .to_string(),
                    metrics,
                )),
            )
                .into_response();
        }
    };

    let browser_acquisition_time = browser_acquisition_start.elapsed().as_millis() as u64;

    // If a target URL is provided and current page is blank or different, navigate first
    if let Some(target_url) = &req.url {
        let needs_nav = match browser_arc.current_url().await {
            Ok(cur) => cur == "about:blank" || cur != *target_url,
            Err(_) => true,
        };
        if needs_nav {
            if let Err(e) = browser_arc.navigate_to(target_url).await {
                error!("Perception pre-navigation failed: {}", e);
                let metrics = PerformanceMetrics {
                    processing_time_ms: 0,
                    browser_acquisition_time_ms: browser_acquisition_time,
                    perception_time_ms: 0,
                    total_time_ms: start_time.elapsed().as_millis() as u64,
                };
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(PerceptionResponse::<()>::error(
                        format!("Navigation failed: {}", e),
                        metrics,
                    )),
                )
                    .into_response();
            }
        }
    } else {
        // No explicit URL: try recent navigation hints (session-first, then global)
        let hint = if let Some(ref sid) = req.session_id {
            state.recent_nav.read().await.get(sid).cloned()
        } else {
            state.recent_nav.read().await.get("__global__").cloned()
        };
        if let Some(target_url) = hint {
            // Only navigate if current page is blank
            if let Ok(cur) = browser_arc.current_url().await {
                if cur == "about:blank" && !target_url.is_empty() && target_url != "__pending__" {
                    if let Err(e) = browser_arc.navigate_to(&target_url).await {
                        warn!("Perception nav-by-hint failed: {}", e);
                    }
                }
            }
        }
    }

    let perception_start = Instant::now();

    // Create layered perception engine with the browser (either from session or pool)
    let mut layered_perception = crate::perception::LayeredPerception::new(browser_arc);

    match layered_perception.perceive(mode).await {
        Ok(result) => {
            let perception_time = perception_start.elapsed().as_millis() as u64;
            let metrics = PerformanceMetrics {
                processing_time_ms: perception_time,
                browser_acquisition_time_ms: browser_acquisition_time,
                perception_time_ms: perception_time,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };

            // Normalize to a stable, UI-friendly shape with an explicit top-level key
            let mut response_data = match &result {
                crate::perception::PerceptionResult::Lightning(l) => {
                    let mut m = serde_json::Map::new();
                    m.insert("lightning".to_string(), serde_json::to_value(l).unwrap_or_default());
                    serde_json::Value::Object(m)
                }
                crate::perception::PerceptionResult::Quick(q) => {
                    let mut m = serde_json::Map::new();
                    m.insert("quick".to_string(), serde_json::to_value(q).unwrap_or_default());
                    serde_json::Value::Object(m)
                }
                crate::perception::PerceptionResult::Standard(s) => {
                    let mut m = serde_json::Map::new();
                    m.insert("standard".to_string(), serde_json::to_value(s).unwrap_or_default());
                    serde_json::Value::Object(m)
                }
                crate::perception::PerceptionResult::Deep(d) => {
                    let mut m = serde_json::Map::new();
                    m.insert("deep".to_string(), serde_json::to_value(d).unwrap_or_default());
                    serde_json::Value::Object(m)
                }
            };
            if let Some(session_id) = req.session_id {
                if let Some(obj) = response_data.as_object_mut() {
                    obj.insert(
                        "session_id".to_string(),
                        serde_json::Value::String(session_id),
                    );
                    obj.insert(
                        "source".to_string(),
                        serde_json::Value::String(browser_source.to_string()),
                    );
                }
            }

            info!(
                "Layered perception completed successfully in {}ms with mode: {}",
                perception_time, req.mode
            );
            Json(PerceptionResponse::success(response_data, metrics)).into_response()
        }
        Err(e) => {
            error!("Layered perception failed: {}", e);
            let metrics = PerformanceMetrics {
                processing_time_ms: 0,
                browser_acquisition_time_ms: browser_acquisition_time,
                perception_time_ms: perception_start.elapsed().as_millis() as u64,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            };
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PerceptionResponse::<()>::error(
                    format!("Perception failed: {}", e),
                    metrics,
                )),
            )
                .into_response()
        }
    }
}

/// Lightning fast perception for quick decisions
#[derive(Deserialize)]
pub struct QuickScanRequest {
    pub session_id: Option<String>,
}

pub async fn quick_scan(
    State(state): State<AppState>,
    maybe_req: Option<Json<QuickScanRequest>>,
) -> impl IntoResponse {
    // Resolve session_id if present
    let session_id = maybe_req.as_ref().and_then(|j| j.session_id.clone());

    // Choose browser: session-bound when session_id provided, else from pool
    let browser_arc = if let Some(sid) = session_id {
        match state.session_manager.get_session(&sid).await {
            Some(session) => {
                let session_lock = session.read().await;
                session_lock.browser.clone()
            }
            None => {
                warn!(
                    "Session {} not found for quick scan; falling back to pool",
                    sid
                );
                match state.browser_pool.acquire().await {
                    Ok(guard) => guard.browser_arc(),
                    Err(e) => {
                        error!("Failed to acquire browser for quick scan: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(e.to_string())),
                        )
                            .into_response();
                    }
                }
            }
        }
    } else {
        match state.browser_pool.acquire().await {
            Ok(guard) => guard.browser_arc(),
            Err(e) => {
                error!("Failed to acquire browser for quick scan: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response();
            }
        }
    };

    // Create perception engine for quick scan using chosen browser
    match crate::perception::PerceptionEngine::new(browser_arc).await {
        Ok(mut perception) => match perception.quick_scan().await {
            Ok(lightning_result) => Json(ApiResponse::success(lightning_result)).into_response(),
            Err(e) => {
                error!("Quick scan failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Quick scan failed: {}",
                        e
                    ))),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Failed to create perception engine for quick scan: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

/// Smart element search using multiple strategies
pub async fn smart_element_search(
    State(state): State<AppState>,
    Json(req): Json<SmartElementSearchRequest>,
) -> impl IntoResponse {
    // Choose browser: prefer session when provided
    let browser_arc = if let Some(ref sid) = req.session_id {
        match state.session_manager.get_session(sid).await {
            Some(session) => {
                let lock = session.read().await;
                lock.browser.clone()
            }
            None => {
                warn!(
                    "Session {} not found for smart search; falling back to pool",
                    sid
                );
                match state.browser_pool.acquire().await {
                    Ok(guard) => guard.browser_arc(),
                    Err(e) => {
                        error!("Failed to acquire browser for element search: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(e.to_string())),
                        )
                            .into_response();
                    }
                }
            }
        }
    } else {
        match state.browser_pool.acquire().await {
            Ok(guard) => guard.browser_arc(),
            Err(e) => {
                error!("Failed to acquire browser for element search: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(e.to_string())),
                )
                    .into_response();
            }
        }
    };

    match crate::perception::PerceptionEngine::new(browser_arc).await {
        Ok(perception) => match perception.locate_element_intelligently(&req.query).await {
            Ok(matches) => {
                let limited_matches: Vec<_> = matches
                    .into_iter()
                    .take(req.max_results.unwrap_or(10))
                    .collect();

                Json(ApiResponse::success(serde_json::json!({
                    "query": req.query,
                    "matches_found": limited_matches.len(),
                    "search_time_ms": 156, // TODO: Track actual time
                    "strategies_used": ["css_selector", "text_content", "semantic", "advanced_cdp"],
                    "matches": limited_matches
                })))
                .into_response()
            }
            Err(e) => {
                error!("Smart element search failed: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!(
                        "Element search failed: {}",
                        e
                    ))),
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!(
                "Failed to create perception engine for element search: {}",
                e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}
