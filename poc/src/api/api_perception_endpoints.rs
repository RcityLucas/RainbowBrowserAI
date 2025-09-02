// API Endpoints for Perception Testing
// Provides REST API for all perception features

use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
    Router,
    routing::{get, post},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn, error};

use crate::perception_mvp::{
    browser_connection::{BrowserConnection, BrowserConfig},
    lightning_real::RealLightningPerception,
    quick_real::RealQuickPerception,
    cache_system::PerceptionCache,
    natural_language::NaturalLanguageFinder,
};

/// Shared state for perception API
pub struct PerceptionState {
    browser: Arc<RwLock<Option<BrowserConnection>>>,
    cache: Arc<PerceptionCache>,
    nl_finder: Arc<RwLock<NaturalLanguageFinder>>,
}

impl PerceptionState {
    pub fn new() -> Self {
        Self {
            browser: Arc::new(RwLock::new(None)),
            cache: Arc::new(PerceptionCache::new()),
            nl_finder: Arc::new(RwLock::new(NaturalLanguageFinder::new())),
        }
    }
    
    async fn ensure_browser(&self) -> Result<()> {
        let mut browser_lock = self.browser.write().await;
        if browser_lock.is_none() {
            let config = BrowserConfig::default();
            let browser = BrowserConnection::new(config).await?;
            *browser_lock = Some(browser);
        }
        Ok(())
    }
    
    async fn get_browser(&self) -> Option<BrowserConnection> {
        self.browser.read().await.clone()
    }
}

/// Health check endpoint
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "perception",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Test browser connection
async fn test_browser_connection(
    State(state): State<Arc<PerceptionState>>
) -> impl IntoResponse {
    match state.ensure_browser().await {
        Ok(_) => {
            if let Some(browser) = state.get_browser().await {
                match browser.is_connected().await {
                    true => {
                        Json(serde_json::json!({
                            "status": "connected",
                            "browser": "Chrome",
                            "version": "120.0",
                            "chromedriver": "http://localhost:9515"
                        }))
                    }
                    false => {
                        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                            "status": "disconnected",
                            "error": "Browser connection lost"
                        })))
                    }
                }
            } else {
                (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                    "status": "error",
                    "error": "No browser instance"
                })))
            }
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "status": "error",
                "error": e.to_string()
            })))
        }
    }
}

/// Navigate to URL request
#[derive(Deserialize)]
struct NavigateRequest {
    url: String,
}

/// Navigate to a URL
async fn navigate(
    State(state): State<Arc<PerceptionState>>,
    Json(req): Json<NavigateRequest>,
) -> impl IntoResponse {
    if let Err(e) = state.ensure_browser().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        })));
    }
    
    if let Some(browser) = state.get_browser().await {
        match browser.navigate(&req.url).await {
            Ok(_) => {
                info!("Navigated to {}", req.url);
                Json(serde_json::json!({
                    "status": "success",
                    "url": req.url
                }))
            }
            Err(e) => {
                error!("Navigation failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "No browser instance"
        })))
    }
}

/// Perception request
#[derive(Deserialize)]
struct PerceptionRequest {
    url: Option<String>,
}

/// Lightning perception endpoint
async fn perceive_lightning(
    State(state): State<Arc<PerceptionState>>,
    Json(req): Json<PerceptionRequest>,
) -> impl IntoResponse {
    if let Err(e) = state.ensure_browser().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        })));
    }
    
    if let Some(browser) = state.get_browser().await {
        // Navigate if URL provided
        if let Some(url) = &req.url {
            if let Err(e) = browser.navigate(url).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Navigation failed: {}", e)
                })));
            }
        }
        
        // Check cache first
        let current_url = browser.current_url().await.unwrap_or_default();
        if let Some(cached) = state.cache.get_lightning(&current_url).await {
            info!("Lightning cache hit for {}", current_url);
            return Json(serde_json::json!({
                "key_elements": cached.key_elements,
                "page_status": cached.page_status,
                "urgent_signals": cached.urgent_signals,
                "cached": true,
                "timestamp": cached.timestamp
            }));
        }
        
        // Perform Lightning perception
        let perception = RealLightningPerception::new();
        match perception.scan_page(&browser).await {
            Ok(data) => {
                // Cache the result
                let cache_entry = crate::perception_mvp::cache_system::LightningCacheEntry {
                    key_elements: data.key_elements.iter()
                        .map(|e| (e.selector.clone(), e.text.clone()))
                        .collect(),
                    page_status: crate::perception_mvp::cache_system::PageStatus {
                        is_loading: data.page_status.is_loading,
                        has_errors: data.page_status.has_errors,
                        ready_state: data.page_status.ready_state.clone(),
                    },
                    urgent_signals: data.urgent_signals.clone(),
                    url: current_url.clone(),
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                };
                state.cache.set_lightning(current_url, cache_entry).await;
                
                Json(serde_json::json!({
                    "key_elements": data.key_elements,
                    "page_status": data.page_status,
                    "urgent_signals": data.urgent_signals,
                    "scan_time_ms": data.scan_time_ms,
                    "cached": false
                }))
            }
            Err(e) => {
                error!("Lightning perception failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "No browser instance"
        })))
    }
}

/// Quick perception endpoint
async fn perceive_quick(
    State(state): State<Arc<PerceptionState>>,
    Json(req): Json<PerceptionRequest>,
) -> impl IntoResponse {
    if let Err(e) = state.ensure_browser().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        })));
    }
    
    if let Some(browser) = state.get_browser().await {
        // Navigate if URL provided
        if let Some(url) = &req.url {
            if let Err(e) = browser.navigate(url).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": format!("Navigation failed: {}", e)
                })));
            }
        }
        
        // Check cache first
        let current_url = browser.current_url().await.unwrap_or_default();
        if let Some(cached) = state.cache.get_quick(&current_url).await {
            info!("Quick cache hit for {}", current_url);
            return Json(serde_json::json!({
                "interaction_elements": cached.interactions,
                "layout_structure": cached.layout,
                "navigation_paths": cached.navigation,
                "form_analysis": cached.forms,
                "cached": true
            }));
        }
        
        // Perform Quick perception
        let perception = RealQuickPerception::new();
        match perception.scan_page(&browser).await {
            Ok(data) => {
                // Prepare cache entry
                let cache_entry = crate::perception_mvp::cache_system::QuickCacheEntry {
                    lightning_data: crate::perception_mvp::cache_system::LightningCacheEntry {
                        key_elements: data.lightning_data.key_elements.iter()
                            .map(|e| (e.selector.clone(), e.text.clone()))
                            .collect(),
                        page_status: crate::perception_mvp::cache_system::PageStatus {
                            is_loading: data.lightning_data.page_status.is_loading,
                            has_errors: data.lightning_data.page_status.has_errors,
                            ready_state: data.lightning_data.page_status.ready_state.clone(),
                        },
                        urgent_signals: data.lightning_data.urgent_signals.clone(),
                        url: current_url.clone(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    },
                    interactions: data.interaction_elements.iter()
                        .map(|e| crate::perception_mvp::cache_system::InteractionData {
                            selector: e.selector.clone(),
                            interaction_type: format!("{:?}", e.interaction_type),
                            confidence: e.confidence,
                        })
                        .collect(),
                    layout: crate::perception_mvp::cache_system::LayoutData {
                        has_header: data.layout_structure.has_header,
                        has_navigation: data.layout_structure.has_navigation,
                        has_sidebar: data.layout_structure.has_sidebar,
                        layout_type: format!("{:?}", data.layout_structure.layout_type),
                    },
                    navigation: data.navigation_paths.iter()
                        .map(|n| crate::perception_mvp::cache_system::NavItem {
                            text: n.text.clone(),
                            url: n.url.clone(),
                            is_active: n.is_active,
                        })
                        .collect(),
                    forms: data.form_analysis.iter()
                        .map(|f| crate::perception_mvp::cache_system::FormData {
                            selector: f.form_selector.clone(),
                            field_count: f.fields.len(),
                            has_submit: f.submit_button.is_some(),
                        })
                        .collect(),
                };
                state.cache.set_quick(current_url, cache_entry).await;
                
                Json(serde_json::json!({
                    "interaction_elements": data.interaction_elements,
                    "layout_structure": data.layout_structure,
                    "navigation_paths": data.navigation_paths,
                    "form_analysis": data.form_analysis,
                    "scan_time_ms": data.scan_time_ms,
                    "cached": false
                }))
            }
            Err(e) => {
                error!("Quick perception failed: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "No browser instance"
        })))
    }
}

/// Natural language find request
#[derive(Deserialize)]
struct NaturalLanguageFindRequest {
    description: String,
}

/// Find element using natural language
async fn find_natural(
    State(state): State<Arc<PerceptionState>>,
    Json(req): Json<NaturalLanguageFindRequest>,
) -> impl IntoResponse {
    if let Err(e) = state.ensure_browser().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        })));
    }
    
    if let Some(browser) = state.get_browser().await {
        let mut finder = state.nl_finder.write().await;
        
        // Get Quick perception data for context
        let perception = RealQuickPerception::new();
        let quick_data = perception.scan_page(&browser).await.ok();
        
        match finder.find_element(&req.description, &browser, quick_data.as_ref()).await {
            Ok(result) => {
                Json(serde_json::json!({
                    "selector": result.selector,
                    "element_type": result.element_type,
                    "confidence": result.confidence,
                    "alternatives": result.alternatives,
                    "interpretation": result.interpretation
                }))
            }
            Err(e) => {
                warn!("Natural language find failed: {}", e);
                (StatusCode::NOT_FOUND, Json(serde_json::json!({
                    "error": e.to_string(),
                    "description": req.description
                })))
            }
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "No browser instance"
        })))
    }
}

/// Get cache statistics
async fn cache_stats(
    State(state): State<Arc<PerceptionState>>
) -> impl IntoResponse {
    let stats = state.cache.get_stats().await;
    
    let hit_rate = if stats.total_hits + stats.total_misses > 0 {
        (stats.total_hits as f64) / ((stats.total_hits + stats.total_misses) as f64) * 100.0
    } else {
        0.0
    };
    
    Json(serde_json::json!({
        "total_hits": stats.total_hits,
        "total_misses": stats.total_misses,
        "hit_rate_percent": hit_rate,
        "total_evictions": stats.total_evictions,
        "total_invalidations": stats.total_invalidations,
        "current_memory_mb": stats.current_memory_mb,
        "peak_memory_mb": stats.peak_memory_mb
    }))
}

/// Clear all caches
async fn clear_cache(
    State(state): State<Arc<PerceptionState>>
) -> impl IntoResponse {
    state.cache.clear_all().await;
    
    Json(serde_json::json!({
        "status": "success",
        "message": "All caches cleared"
    }))
}

/// Create perception API router
pub fn create_perception_routes() -> Router {
    let state = Arc::new(PerceptionState::new());
    
    Router::new()
        .route("/health", get(health))
        .route("/test/browser-connection", get(test_browser_connection))
        .route("/navigate", post(navigate))
        .route("/perceive/lightning", post(perceive_lightning))
        .route("/perceive/quick", post(perceive_quick))
        .route("/find/natural", post(find_natural))
        .route("/cache/stats", get(cache_stats))
        .route("/cache/clear", post(clear_cache))
        .with_state(state)
}

/// Start the perception API server
pub async fn start_perception_api(port: u16) -> Result<()> {
    let app = create_perception_routes();
    
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting Perception API server on {}", addr);
    
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}