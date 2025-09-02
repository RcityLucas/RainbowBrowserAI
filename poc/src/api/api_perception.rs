// Perception API Endpoints
// Provides REST API for testing perception module capabilities

use axum::{
    extract::{State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use anyhow::Result;
use std::time::Instant;

use crate::{
    perception_mvp::{
        browser_connection::{BrowserConnection, BrowserConfig},
        lightning_real::{RealLightningPerception, CachedLightningPerception, LightningData},
        PerceptionEngineMVP, PageType,
    },
    api::ApiState,
};

/// Perception test request
#[derive(Debug, Deserialize)]
pub struct PerceiveRequest {
    pub url: String,
    #[serde(default)]
    pub mode: PerceptionMode,
    #[serde(default)]
    pub use_cache: bool,
}

/// Perception mode selection
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PerceptionMode {
    Lightning,
    Quick,
    Standard,
    Deep,
    Adaptive,
}

impl Default for PerceptionMode {
    fn default() -> Self {
        Self::Lightning
    }
}

/// Perception response
#[derive(Debug, Serialize)]
pub struct PerceiveResponse {
    pub success: bool,
    pub mode: PerceptionMode,
    pub url: String,
    pub scan_time_ms: u64,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// Element finding request
#[derive(Debug, Deserialize)]
pub struct FindElementRequest {
    pub url: String,
    pub description: String,
}

/// Element finding response
#[derive(Debug, Serialize)]
pub struct FindElementResponse {
    pub success: bool,
    pub found: bool,
    pub selector: Option<String>,
    pub element_type: Option<String>,
    pub text: Option<String>,
    pub confidence: Option<f32>,
    pub error: Option<String>,
}

/// Context analysis request
#[derive(Debug, Deserialize)]
pub struct ContextAnalysisRequest {
    pub urls: Vec<String>,
}

/// Context analysis response
#[derive(Debug, Serialize)]
pub struct ContextAnalysisResponse {
    pub results: Vec<PageContextResult>,
}

#[derive(Debug, Serialize)]
pub struct PageContextResult {
    pub url: String,
    pub page_type: String,
    pub analysis_time_ms: u64,
}

/// Performance metrics response
#[derive(Debug, Serialize)]
pub struct PerceptionMetricsResponse {
    pub total_scans: u64,
    pub average_scan_time_ms: f64,
    pub cache_hit_rate: f64,
    pub lightning_success_rate: f64,
    pub modes_usage: std::collections::HashMap<String, u64>,
}

/// Shared perception state
pub struct PerceptionState {
    browser_connection: Arc<RwLock<Option<Arc<BrowserConnection>>>>,
    lightning_perception: Arc<CachedLightningPerception>,
    metrics: Arc<RwLock<PerceptionMetrics>>,
}

#[derive(Default)]
struct PerceptionMetrics {
    total_scans: u64,
    total_scan_time_ms: u64,
    cache_hits: u64,
    cache_misses: u64,
    lightning_success: u64,
    lightning_failures: u64,
    mode_counts: std::collections::HashMap<String, u64>,
}

impl PerceptionState {
    pub fn new() -> Self {
        Self {
            browser_connection: Arc::new(RwLock::new(None)),
            lightning_perception: Arc::new(CachedLightningPerception::new()),
            metrics: Arc::new(RwLock::new(PerceptionMetrics::default())),
        }
    }
    
    /// Get or create browser connection
    async fn get_browser(&self) -> Result<Arc<BrowserConnection>> {
        let mut conn_lock = self.browser_connection.write().await;
        
        if let Some(conn) = &*conn_lock {
            if conn.is_connected().await {
                return Ok(Arc::clone(conn));
            }
        }
        
        // Create new connection
        info!("Creating new browser connection for perception");
        let config = BrowserConfig::default();
        let new_conn = Arc::new(BrowserConnection::new(config).await?);
        *conn_lock = Some(Arc::clone(&new_conn));
        
        Ok(new_conn)
    }
}

/// Create perception API routes
pub fn create_perception_routes(state: Arc<PerceptionState>) -> Router {
    Router::new()
        .route("/perceive/lightning", post(perceive_lightning))
        .route("/perceive/quick", post(perceive_quick))
        .route("/perceive/standard", post(perceive_standard))
        .route("/perceive/deep", post(perceive_deep))
        .route("/perceive/adaptive", post(perceive_adaptive))
        .route("/perceive/full", post(perceive_full))
        .route("/find/element", post(find_element))
        .route("/analyze/context", post(analyze_context))
        .route("/test/browser-connection", get(test_browser_connection))
        .route("/metrics/perception", get(get_perception_metrics))
        .route("/cache/clear", post(clear_perception_cache))
        .with_state(state)
}

/// Test browser connection
async fn test_browser_connection(
    State(state): State<Arc<PerceptionState>>,
) -> impl IntoResponse {
    match state.get_browser().await {
        Ok(browser) => {
            match browser.navigate("https://example.com").await {
                Ok(_) => {
                    let title = browser.title().await.unwrap_or_default();
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Browser connection successful",
                        "title": title,
                        "chromedriver_port": std::env::var("CHROMEDRIVER_PORT").unwrap_or_else(|_| "9515".to_string())
                    })))
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": format!("Navigation failed: {}", e)
                    })))
                }
            }
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to connect to browser: {}", e)
            })))
        }
    }
}

/// Lightning perception endpoint (<50ms)
async fn perceive_lightning(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    // Get browser connection
    let browser = match state.get_browser().await {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(PerceiveResponse {
                success: false,
                mode: PerceptionMode::Lightning,
                url: request.url,
                scan_time_ms: start.elapsed().as_millis() as u64,
                data: serde_json::Value::Null,
                error: Some(format!("Browser connection failed: {}", e)),
            }));
        }
    };
    
    // Navigate to URL
    if let Err(e) = browser.navigate(&request.url).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(PerceiveResponse {
            success: false,
            mode: PerceptionMode::Lightning,
            url: request.url,
            scan_time_ms: start.elapsed().as_millis() as u64,
            data: serde_json::Value::Null,
            error: Some(format!("Navigation failed: {}", e)),
        }));
    }
    
    // Perform Lightning perception
    let result = if request.use_cache {
        state.lightning_perception.scan_page(&*browser).await
    } else {
        let perception = RealLightningPerception::new();
        perception.scan_page(&*browser).await
    };
    
    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.total_scans += 1;
        metrics.total_scan_time_ms += start.elapsed().as_millis() as u64;
        *metrics.mode_counts.entry("lightning".to_string()).or_insert(0) += 1;
        
        if result.is_ok() {
            metrics.lightning_success += 1;
        } else {
            metrics.lightning_failures += 1;
        }
    }
    
    match result {
        Ok(data) => {
            (StatusCode::OK, Json(PerceiveResponse {
                success: true,
                mode: PerceptionMode::Lightning,
                url: request.url,
                scan_time_ms: data.scan_time_ms,
                data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
                error: None,
            }))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(PerceiveResponse {
                success: false,
                mode: PerceptionMode::Lightning,
                url: request.url,
                scan_time_ms: start.elapsed().as_millis() as u64,
                data: serde_json::Value::Null,
                error: Some(format!("Perception failed: {}", e)),
            }))
        }
    }
}

/// Quick perception endpoint (<200ms)
async fn perceive_quick(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    // TODO: Implement Quick perception
    (StatusCode::NOT_IMPLEMENTED, Json(PerceiveResponse {
        success: false,
        mode: PerceptionMode::Quick,
        url: request.url,
        scan_time_ms: 0,
        data: serde_json::Value::Null,
        error: Some("Quick perception not yet implemented".to_string()),
    }))
}

/// Standard perception endpoint (<500ms)
async fn perceive_standard(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    // TODO: Implement Standard perception
    (StatusCode::NOT_IMPLEMENTED, Json(PerceiveResponse {
        success: false,
        mode: PerceptionMode::Standard,
        url: request.url,
        scan_time_ms: 0,
        data: serde_json::Value::Null,
        error: Some("Standard perception not yet implemented".to_string()),
    }))
}

/// Deep perception endpoint (<1000ms)
async fn perceive_deep(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    // TODO: Implement Deep perception
    (StatusCode::NOT_IMPLEMENTED, Json(PerceiveResponse {
        success: false,
        mode: PerceptionMode::Deep,
        url: request.url,
        scan_time_ms: 0,
        data: serde_json::Value::Null,
        error: Some("Deep perception not yet implemented".to_string()),
    }))
}

/// Adaptive perception endpoint
async fn perceive_adaptive(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    // For now, use Lightning as default
    perceive_lightning(State(state), Json(request)).await
}

/// Full perception test (all modes)
async fn perceive_full(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<PerceiveRequest>,
) -> impl IntoResponse {
    // Run Lightning perception for now
    perceive_lightning(State(state), Json(request)).await
}

/// Find element by description
async fn find_element(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<FindElementRequest>,
) -> impl IntoResponse {
    // TODO: Implement element finding with PerceptionEngineMVP
    (StatusCode::NOT_IMPLEMENTED, Json(FindElementResponse {
        success: false,
        found: false,
        selector: None,
        element_type: None,
        text: None,
        confidence: None,
        error: Some("Element finding not yet implemented".to_string()),
    }))
}

/// Analyze page context
async fn analyze_context(
    State(state): State<Arc<PerceptionState>>,
    Json(request): Json<ContextAnalysisRequest>,
) -> impl IntoResponse {
    let mut results = Vec::new();
    
    for url in request.urls {
        // TODO: Implement context analysis
        results.push(PageContextResult {
            url,
            page_type: "Unknown".to_string(),
            analysis_time_ms: 0,
        });
    }
    
    (StatusCode::OK, Json(ContextAnalysisResponse { results }))
}

/// Get perception metrics
async fn get_perception_metrics(
    State(state): State<Arc<PerceptionState>>,
) -> impl IntoResponse {
    let metrics = state.metrics.read().await;
    
    let avg_scan_time = if metrics.total_scans > 0 {
        metrics.total_scan_time_ms as f64 / metrics.total_scans as f64
    } else {
        0.0
    };
    
    let cache_total = metrics.cache_hits + metrics.cache_misses;
    let cache_hit_rate = if cache_total > 0 {
        metrics.cache_hits as f64 / cache_total as f64
    } else {
        0.0
    };
    
    let lightning_total = metrics.lightning_success + metrics.lightning_failures;
    let lightning_success_rate = if lightning_total > 0 {
        metrics.lightning_success as f64 / lightning_total as f64
    } else {
        0.0
    };
    
    (StatusCode::OK, Json(PerceptionMetricsResponse {
        total_scans: metrics.total_scans,
        average_scan_time_ms: avg_scan_time,
        cache_hit_rate,
        lightning_success_rate,
        modes_usage: metrics.mode_counts.clone(),
    }))
}

/// Clear perception cache
async fn clear_perception_cache(
    State(state): State<Arc<PerceptionState>>,
) -> impl IntoResponse {
    state.lightning_perception.clear_cache().await;
    
    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Perception cache cleared"
    })))
}