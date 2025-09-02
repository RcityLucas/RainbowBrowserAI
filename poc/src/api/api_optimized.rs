// Optimized API Layer for Perception Module
// Reduces serialization overhead and improves response times

use axum::{
    extract::{State, Json},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use tracing::{info, debug, warn};

use crate::perception_mvp::{
    PerceptionOrchestrator, UnifiedPerceptionResult, PerceptionLevel,
    BrowserConnection,
};

/// Optimized response format with minimal overhead
#[derive(Serialize, Deserialize)]
pub struct FastResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub timing_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
}

/// Direct perception request without command parsing
#[derive(Deserialize)]
pub struct DirectPerceptionRequest {
    pub level: PerceptionLevel,
    #[serde(default)]
    pub use_cache: bool,
}

/// Lightweight element data
#[derive(Serialize, Deserialize)]
pub struct ElementData {
    pub selector: String,
    pub text: Option<String>,
    pub element_type: String,
    pub visible: bool,
}

/// Optimized perception result
#[derive(Serialize)]
pub struct OptimizedPerceptionResult {
    pub level: PerceptionLevel,
    pub elements: Vec<ElementData>,
    pub stats: PerceptionStats,
}

#[derive(Serialize)]
pub struct PerceptionStats {
    pub total_elements: usize,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
}

/// Response cache for frequently accessed data
pub struct ResponseCache {
    cache: Arc<RwLock<lru::LruCache<String, CachedResponse>>>,
}

#[derive(Clone)]
struct CachedResponse {
    data: Value,
    timestamp: Instant,
    ttl_ms: u64,
}

impl ResponseCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ))),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get(key) {
            if entry.timestamp.elapsed().as_millis() < entry.ttl_ms as u128 {
                debug!("Cache hit for key: {}", key);
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, data: Value, ttl_ms: u64) {
        let mut cache = self.cache.write().await;
        cache.put(key, CachedResponse {
            data,
            timestamp: Instant::now(),
            ttl_ms,
        });
    }
}

/// Optimized API state
pub struct OptimizedApiState {
    pub orchestrator: Arc<PerceptionOrchestrator>,
    pub browser: Arc<BrowserConnection>,
    pub response_cache: Arc<ResponseCache>,
}

// Direct perception endpoints - bypassing command parsing

/// Lightning perception - ultra-fast response
pub async fn lightning_perception_direct(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    // Check cache first
    let cache_key = "lightning_perception";
    if let Some(cached) = state.response_cache.get(cache_key).await {
        return Json(FastResponse {
            data: Some(cached),
            timing_ms: start.elapsed().as_millis() as u64,
            cached: Some(true),
        });
    }

    // Execute lightning perception
    match state.orchestrator.perceive_lightning(&state.browser).await {
        Ok(result) => {
            // Convert to lightweight format
            let elements: Vec<ElementData> = if let Some(lightning) = &result.lightning_data {
                lightning.key_elements.iter().map(|e| ElementData {
                    selector: e.selector.clone(),
                    text: Some(e.text.clone()),
                    element_type: format!("{:?}", e.element_type),
                    visible: e.visible,
                }).collect()
            } else {
                Vec::new()
            };

            let response = serde_json::json!({
                "elements": elements,
                "count": elements.len(),
            });

            // Cache for 1 second
            state.response_cache.set(
                cache_key.to_string(),
                response.clone(),
                1000
            ).await;

            Json(FastResponse {
                data: Some(response),
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
        Err(e) => {
            warn!("Lightning perception failed: {}", e);
            Json(FastResponse::<Value> {
                data: None,
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
    }
}

/// Quick perception - fast interactive element detection
pub async fn quick_perception_direct(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    match state.orchestrator.perceive_quick(&state.browser).await {
        Ok(result) => {
            let mut elements = Vec::new();
            
            if let Some(quick) = &result.quick_data {
                for elem in &quick.interaction_elements {
                    elements.push(ElementData {
                        selector: elem.selector.clone(),
                        text: Some(elem.text.clone()),
                        element_type: format!("{:?}", elem.element_type),
                        visible: true, // All interaction elements are assumed visible
                    });
                }
            }

            Json(FastResponse {
                data: Some(serde_json::json!({
                    "elements": elements,
                    "forms": result.quick_data.as_ref().map(|q| q.form_analysis.len()).unwrap_or(0),
                    "navigation": result.quick_data.as_ref().map(|q| q.navigation_paths.len()).unwrap_or(0),
                })),
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
        Err(e) => {
            warn!("Quick perception failed: {}", e);
            Json(FastResponse::<Value> {
                data: None,
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
    }
}

/// Standard perception - comprehensive analysis
pub async fn standard_perception_direct(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    match state.orchestrator.perceive_standard(&state.browser).await {
        Ok(result) => {
            let response = if let Some(standard) = &result.standard_data {
                serde_json::json!({
                    "content": {
                        "words": standard.content_analysis.text_content.total_words,
                        "images": standard.content_analysis.media_elements.len(),
                        "tables": standard.content_analysis.data_tables.len(),
                    },
                    "links": {
                        "internal": standard.data_extraction.internal_links.len(),
                        "external": standard.data_extraction.external_links.len(),
                    },
                    "patterns": standard.interaction_patterns.len(),
                })
            } else {
                serde_json::json!({})
            };

            Json(FastResponse {
                data: Some(response),
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
        Err(e) => {
            warn!("Standard perception failed: {}", e);
            Json(FastResponse::<Value> {
                data: None,
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
    }
}

/// Deep perception - AI-level analysis  
pub async fn deep_perception_direct(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    match state.orchestrator.perceive_deep(&state.browser).await {
        Ok(result) => {
            let response = if let Some(deep) = &result.deep_data {
                serde_json::json!({
                    "intent": format!("{:?}", deep.ai_understanding.intent_classification.primary_intent),
                    "entities": deep.ai_understanding.entity_recognition.len(),
                    "workflows": deep.ai_understanding.workflow_detection.len(),
                    "automation_opportunities": deep.automation_opportunities.len(),
                    "quality_score": deep.quality_assessment.overall_quality_score,
                })
            } else {
                serde_json::json!({})
            };

            Json(FastResponse {
                data: Some(response),
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
        Err(e) => {
            warn!("Deep perception failed: {}", e);
            Json(FastResponse::<Value> {
                data: None,
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
    }
}

/// Adaptive perception - automatic layer selection
pub async fn adaptive_perception_direct(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let start = Instant::now();
    
    match state.orchestrator.perceive_adaptive(&state.browser).await {
        Ok(result) => {
            Json(FastResponse {
                data: Some(serde_json::json!({
                    "level": format!("{:?}", result.execution_info.actual_level),
                    "quality": result.execution_info.quality_score,
                    "confidence": result.execution_info.confidence_score,
                    "execution_ms": result.execution_info.performance_metrics.total_time_ms,
                })),
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
        Err(e) => {
            warn!("Adaptive perception failed: {}", e);
            Json(FastResponse::<Value> {
                data: None,
                timing_ms: start.elapsed().as_millis() as u64,
                cached: Some(false),
            })
        }
    }
}

/// Batch perception request for multiple operations
#[derive(Deserialize)]
pub struct BatchPerceptionRequest {
    pub operations: Vec<PerceptionOperation>,
}

#[derive(Deserialize)]
pub struct PerceptionOperation {
    pub level: PerceptionLevel,
    pub cache_key: Option<String>,
}

/// Batch perception - execute multiple perception operations in parallel
pub async fn batch_perception(
    State(state): State<Arc<OptimizedApiState>>,
    Json(request): Json<BatchPerceptionRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let mut results = Vec::new();

    // Execute operations in parallel
    let mut handles = Vec::new();
    
    for op in request.operations {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            match op.level {
                PerceptionLevel::Lightning => {
                    state_clone.orchestrator.perceive_lightning(&state_clone.browser).await
                }
                PerceptionLevel::Quick => {
                    state_clone.orchestrator.perceive_quick(&state_clone.browser).await
                }
                PerceptionLevel::Standard => {
                    state_clone.orchestrator.perceive_standard(&state_clone.browser).await
                }
                PerceptionLevel::Deep => {
                    state_clone.orchestrator.perceive_deep(&state_clone.browser).await
                }
                _ => {
                    state_clone.orchestrator.perceive_adaptive(&state_clone.browser).await
                }
            }
        });
        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => {
                results.push(serde_json::json!({
                    "success": true,
                    "level": format!("{:?}", result.execution_info.actual_level),
                    "time_ms": result.execution_info.performance_metrics.total_time_ms,
                }));
            }
            _ => {
                results.push(serde_json::json!({
                    "success": false,
                }));
            }
        }
    }

    Json(FastResponse {
        data: Some(serde_json::json!({
            "results": results,
            "total": results.len(),
        })),
        timing_ms: start.elapsed().as_millis() as u64,
        cached: Some(false),
    })
}

/// Performance monitoring endpoint
pub async fn perception_metrics(
    State(state): State<Arc<OptimizedApiState>>,
) -> impl IntoResponse {
    let stats = state.orchestrator.get_stats().await;
    
    Json(serde_json::json!({
        "total_requests": stats.total_requests,
        "lightning": stats.lightning_requests,
        "quick": stats.quick_requests,
        "standard": stats.standard_requests,
        "deep": stats.deep_requests,
        "cache_hits": stats.cache_hits,
        "errors": stats.errors,
        "avg_response_ms": stats.average_response_time_ms,
        "success_rate": if stats.total_requests > 0 {
            ((stats.total_requests - stats.errors) as f64 / stats.total_requests as f64) * 100.0
        } else {
            0.0
        }
    }))
}

/// Create optimized routes
pub fn create_optimized_routes() -> axum::Router<Arc<OptimizedApiState>> {
    use axum::routing::{get, post};
    
    axum::Router::new()
        // Direct perception endpoints - no command parsing overhead
        .route("/api/v2/perception/lightning", get(lightning_perception_direct))
        .route("/api/v2/perception/quick", get(quick_perception_direct))
        .route("/api/v2/perception/standard", get(standard_perception_direct))
        .route("/api/v2/perception/deep", get(deep_perception_direct))
        .route("/api/v2/perception/adaptive", get(adaptive_perception_direct))
        .route("/api/v2/perception/batch", post(batch_perception))
        .route("/api/v2/perception/metrics", get(perception_metrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_cache_creation() {
        let cache = ResponseCache::new(100);
        // Cache is created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = ResponseCache::new(10);
        
        // Test setting and getting
        let key = "test_key".to_string();
        let data = serde_json::json!({"test": "data"});
        
        cache.set(key.clone(), data.clone(), 1000).await;
        
        let retrieved = cache.get(&key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }
}