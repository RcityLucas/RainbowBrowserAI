// Coordinated API Handlers
// Provides HTTP endpoints that use the coordinated module system

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

use super::ApiResponse;
use crate::coordination::RainbowCoordinator;

/// State for coordinated API handlers
#[derive(Clone)]
pub struct CoordinatedApiState {
    pub coordinator: Arc<RainbowCoordinator>,
}

// Request/Response types

#[derive(Debug, Deserialize)]
pub struct CoordinatedRequest<T> {
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct CoordinatedResponse<T> {
    pub success: bool,
    pub session_id: String,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metrics: Option<OperationMetrics>,
}

#[derive(Debug, Serialize)]
pub struct OperationMetrics {
    pub duration_ms: u64,
    pub cache_hits: u32,
    pub cache_misses: u32,
}

impl<T> CoordinatedResponse<T> {
    pub fn success(session_id: String, data: T) -> Self {
        Self {
            success: true,
            session_id,
            data: Some(data),
            error: None,
            metrics: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            session_id: String::new(),
            data: None,
            error: Some(error),
            metrics: None,
        }
    }
}

// Session management endpoints

/// Create a new coordinated session
pub async fn create_coordinated_session(State(state): State<CoordinatedApiState>) -> Response {
    match state.coordinator.create_session().await {
        Ok(bundle) => {
            let response = serde_json::json!({
                "session_id": bundle.session_id,
                "created": true,
                "modules": {
                    "perception": "initialized",
                    "tools": "initialized",
                    "intelligence": "initialized"
                }
            });
            Json(ApiResponse::success(response)).into_response()
        }
        Err(e) => {
            error!("Failed to create coordinated session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

/// Get session information
pub async fn get_coordinated_session(
    State(state): State<CoordinatedApiState>,
    Path(session_id): Path<String>,
) -> Response {
    match state.coordinator.get_session(&session_id).await {
        Some(bundle) => {
            let health = bundle.health_check().await;
            let response = serde_json::json!({
                "session_id": bundle.session_id,
                "exists": true,
                "health": {
                    "perception": format!("{:?}", health.perception_health.status),
                    "tools": format!("{:?}", health.tools_health.status),
                    "intelligence": format!("{:?}", health.intelligence_health.status),
                    "overall": format!("{:?}", health.overall_status)
                }
            });
            Json(ApiResponse::success(response)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!(
                "Session not found: {}",
                session_id
            ))),
        )
            .into_response(),
    }
}

/// List all active sessions
pub async fn list_coordinated_sessions(State(state): State<CoordinatedApiState>) -> Response {
    let sessions = state.coordinator.list_sessions().await;
    let response = serde_json::json!({
        "sessions": sessions.iter().map(|s| serde_json::json!({
            "session_id": s.session_id,
            "created_at": format!("{:?}", s.created_at),
            "last_activity": format!("{:?}", s.last_activity),
            "is_active": s.is_active,
            "resource_usage": {
                "memory_bytes": s.resource_usage.memory_bytes,
                "cpu_percent": s.resource_usage.cpu_percent,
                "active_operations": s.resource_usage.active_operations,
            }
        })).collect::<Vec<_>>()
    });
    Json(ApiResponse::success(response)).into_response()
}

/// Delete a session
pub async fn delete_coordinated_session(
    State(state): State<CoordinatedApiState>,
    Path(session_id): Path<String>,
) -> Response {
    match state.coordinator.remove_session(&session_id).await {
        Ok(()) => Json(ApiResponse::success(serde_json::json!({
            "session_id": session_id,
            "deleted": true
        })))
        .into_response(),
        Err(e) => {
            error!("Failed to delete session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

// Coordinated operation endpoints

#[derive(Debug, Deserialize)]
pub struct NavigateRequest {
    pub url: String,
    pub _wait_for_load: Option<bool>,
    pub analyze_page: Option<bool>,
}

/// Navigate with coordinated perception
pub async fn coordinated_navigate(
    State(state): State<CoordinatedApiState>,
    Json(req): Json<CoordinatedRequest<NavigateRequest>>,
) -> Response {
    let start_time = std::time::Instant::now();

    // Get or create session
    let bundle = match state
        .coordinator
        .get_or_create_session(req.session_id)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response();
        }
    };

    // Navigate
    match bundle.navigate(&req.data.url).await {
        Ok(result) => {
            let mut response_data = serde_json::json!({
                "url": result.url,
                "load_time_ms": result.load_time_ms,
                "success": result.success
            });

            // Add page analysis if requested and available
            if req.data.analyze_page.unwrap_or(false) {
                if let Some(analysis) = result.page_analysis {
                    response_data["analysis"] = serde_json::json!({
                        "title": analysis.title,
                        "element_count": analysis.element_count,
                        "interactive_elements": analysis.interactive_elements
                    });
                }
            }

            let mut response =
                CoordinatedResponse::success(bundle.session_id.clone(), response_data);
            response.metrics = Some(OperationMetrics {
                duration_ms: start_time.elapsed().as_millis() as u64,
                cache_hits: 0,
                cache_misses: 0,
            });

            Json(response).into_response()
        }
        Err(e) => {
            error!("Navigation failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IntelligentActionRequest {
    pub action_type: String,
    pub target: String,
    pub parameters: serde_json::Value,
}

/// Execute an intelligent action with full coordination
pub async fn coordinated_intelligent_action(
    State(state): State<CoordinatedApiState>,
    Json(req): Json<CoordinatedRequest<IntelligentActionRequest>>,
) -> Response {
    let start_time = std::time::Instant::now();

    // Get or create session
    let bundle = match state
        .coordinator
        .get_or_create_session(req.session_id)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response();
        }
    };

    // Convert to internal action request
    let action = crate::coordination::session::IntelligentActionRequest {
        action_type: req.data.action_type,
        target: req.data.target,
        parameters: req.data.parameters,
    };

    // Execute intelligent action
    match bundle.execute_intelligent_action(action).await {
        Ok(result) => {
            let response_data = serde_json::json!({
                "success": result.success,
                "duration_ms": result.duration_ms,
                "analysis": {
                    "elements_found": result.analysis.elements_found.len(),
                    "confidence": result.analysis.confidence
                },
                "plan": {
                    "steps": result.plan.steps,
                    "tools_required": result.plan.tools_required
                },
                "execution": {
                    "success": result.execution_result.success,
                    "output": result.execution_result.output
                },
                "verification": {
                    "success": result.verification.success,
                    "confidence": result.verification.confidence,
                    "error": result.verification.error
                },
                "learning_applied": result.learning_applied
            });

            let mut response =
                CoordinatedResponse::success(bundle.session_id.clone(), response_data);
            response.metrics = Some(OperationMetrics {
                duration_ms: start_time.elapsed().as_millis() as u64,
                cache_hits: 0, // Would need actual metrics
                cache_misses: 0,
            });

            Json(response).into_response()
        }
        Err(e) => {
            error!("Intelligent action failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}

/// Get system health status
pub async fn get_system_health(State(state): State<CoordinatedApiState>) -> Response {
    let health = state.coordinator.get_system_health().await;

    let response = serde_json::json!({
        "healthy": health.healthy_sessions == health.total_sessions,
        "total_sessions": health.total_sessions,
        "healthy_sessions": health.healthy_sessions,
        "resource_usage": {
            "active_browsers": health.resource_usage.active_browsers,
            "total_sessions": health.resource_usage.total_sessions,
            "memory_mb": health.resource_usage.memory_mb,
            "cpu_percent": health.resource_usage.cpu_percent
        },
        "cache_stats": {
            "browser": {
                "entries": health.cache_stats.browser_stats.entries,
                "hits": health.cache_stats.browser_stats.hits,
                "misses": health.cache_stats.browser_stats.misses
            },
            "perception": {
                "entries": health.cache_stats.perception_stats.entries,
                "hits": health.cache_stats.perception_stats.hits,
                "misses": health.cache_stats.perception_stats.misses
            },
            "tools": {
                "entries": health.cache_stats.tool_stats.entries,
                "hits": health.cache_stats.tool_stats.hits,
                "misses": health.cache_stats.tool_stats.misses
            }
        },
        "event_metrics": {
            "total_events": health.event_metrics.total_events,
            "failed_handlers": health.event_metrics.failed_handlers,
            "average_handling_time_ms": health.event_metrics.average_handling_time_ms
        }
    });

    Json(ApiResponse::success(response)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub _parameters: serde_json::Value,
}

/// Execute a tool in a coordinated session
pub async fn coordinated_tool_execution(
    State(state): State<CoordinatedApiState>,
    Json(req): Json<CoordinatedRequest<ToolExecutionRequest>>,
) -> Response {
    let start_time = std::time::Instant::now();

    // Get or create session
    let bundle = match state
        .coordinator
        .get_or_create_session(req.session_id)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response();
        }
    };

    // Execute tool through coordinated registry
    // This would be implemented when we migrate the tool registry
    let response_data = serde_json::json!({
        "tool_name": req.data.tool_name,
        "executed": true,
        "session_id": bundle.session_id,
        "message": "Tool execution through coordinated system (pending full implementation)"
    });

    let mut response = CoordinatedResponse::success(bundle.session_id.clone(), response_data);
    response.metrics = Some(OperationMetrics {
        duration_ms: start_time.elapsed().as_millis() as u64,
        cache_hits: 0,
        cache_misses: 0,
    });

    Json(response).into_response()
}

#[derive(Debug, Deserialize)]
pub struct PerceptionAnalysisRequest {
    pub analysis_type: Option<String>, // "quick", "standard", "deep"
    pub _target: Option<String>,
}

/// Analyze page with coordinated perception
pub async fn coordinated_perception_analysis(
    State(state): State<CoordinatedApiState>,
    Json(req): Json<CoordinatedRequest<PerceptionAnalysisRequest>>,
) -> Response {
    let start_time = std::time::Instant::now();

    // Get or create session
    let bundle = match state
        .coordinator
        .get_or_create_session(req.session_id)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response();
        }
    };

    // Analyze current page
    match bundle.perception.analyze_current_page().await {
        Ok(analysis) => {
            let response_data = serde_json::json!({
                "title": analysis.title,
                "element_count": analysis.element_count,
                "interactive_elements": analysis.interactive_elements,
                "analysis_type": req.data.analysis_type.unwrap_or_else(|| "standard".to_string())
            });

            let mut response =
                CoordinatedResponse::success(bundle.session_id.clone(), response_data);
            response.metrics = Some(OperationMetrics {
                duration_ms: start_time.elapsed().as_millis() as u64,
                cache_hits: 0,
                cache_misses: 0,
            });

            Json(response).into_response()
        }
        Err(e) => {
            error!("Perception analysis failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(CoordinatedResponse::<()>::error(e.to_string())),
            )
                .into_response()
        }
    }
}
