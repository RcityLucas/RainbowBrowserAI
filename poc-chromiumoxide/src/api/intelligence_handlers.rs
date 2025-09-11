// Intelligence API handlers
// Provides advanced AI-driven automation with learning and adaptation capabilities

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info};

use super::AppState;
use crate::intelligence::{
    IntelligenceService, IntelligenceConfig, IntelligenceAnalysis, ActionRecommendation,
    PageContext, ViewportInfo,
};

/// Enhanced error type for intelligence operations
#[derive(Debug, thiserror::Error)]
pub enum IntelligenceApiError {
    #[error("Intelligence service not available: {0}")]
    ServiceUnavailable(String),
    
    #[error("Analysis failed: {0}")]
    AnalysisError(String),
    
    #[error("Learning data invalid: {0}")]
    LearningDataError(String),
    
    #[error("Pattern recognition failed: {0}")]
    PatternRecognitionError(String),
    
    #[error("Adaptation failed: {0}")]
    AdaptationError(String),
    
    #[error("Invalid request parameters: {0}")]
    ValidationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Browser integration failed: {0}")]
    BrowserIntegrationError(String),
}

/// Enhanced response wrapper for intelligence operations
#[derive(Debug, Serialize)]
pub struct IntelligenceResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: IntelligenceResponseMetadata,
}

#[derive(Debug, Serialize)]
pub struct IntelligenceResponseMetadata {
    pub processing_time_ms: u64,
    pub analysis_depth: String,
    pub confidence: Option<f32>,
    pub components_used: Vec<String>,
    pub total_time_ms: u64,
    pub intelligence_version: String,
}

impl<T> IntelligenceResponse<T> {
    pub fn success(data: T, metadata: IntelligenceResponseMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }
    
    pub fn error(error: String, metadata: IntelligenceResponseMetadata) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata,
        }
    }
}

/// Comprehensive intelligence analysis endpoint
pub async fn analyze_situation(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeSituationRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing intelligence situation analysis: {}", req.user_intent);
    
    // Validate request
    if let Err(validation_error) = validate_analyze_situation_request(&req) {
        let metadata = IntelligenceResponseMetadata {
            processing_time_ms: 0,
            analysis_depth: "none".to_string(),
            confidence: None,
            components_used: vec![],
            total_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_version: "1.0.0".to_string(),
        };
        return (StatusCode::BAD_REQUEST,
               Json(IntelligenceResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    // Get browser instance
    let browser = match state.browser_pool.acquire().await {
        Ok(browser) => browser,
        Err(e) => {
            error!("Failed to acquire browser for intelligence analysis: {}", e);
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: 0,
                analysis_depth: "none".to_string(),
                confidence: None,
                components_used: vec![],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR,
                   Json(IntelligenceResponse::<()>::error(format!("Browser acquisition failed: {}", e), metadata))).into_response();
        }
    };

    let processing_start = Instant::now();
    
    // Create intelligence service with specified configuration
    let config = req.config.unwrap_or_else(|| IntelligenceConfig::default());
    let intelligence_service = IntelligenceService::new(config.clone());
    
    // Create page context from request
    let domain = req.url.split("://").nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or(&req.url).to_string();
    
    let page_context = PageContext {
        url: req.url.clone(),
        domain,
        title: req.page_title.unwrap_or_else(|| "Untitled".to_string()),
        html_content: None, // This would be populated from actual browser content
        viewport_info: ViewportInfo {
            width: 1920,
            height: 1080,
            device_pixel_ratio: 1.0,
            is_mobile: false,
        },
        performance_metrics: None,
    };
    
    // Perform comprehensive intelligence analysis
    match intelligence_service.analyze_situation(&page_context, &req.user_intent, &browser).await {
        Ok(analysis) => {
            let processing_time = processing_start.elapsed().as_millis() as u64;
            
            // Determine components used
            let mut components_used = vec!["organic_perception".to_string()];
            if config.learning_enabled {
                components_used.push("learning_engine".to_string());
            }
            if config.adaptation_enabled {
                components_used.push("adaptation_manager".to_string());
            }
            if config.pattern_recognition_enabled {
                components_used.push("pattern_recognition".to_string());
            }
            components_used.push("decision_maker".to_string());
            
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_time,
                analysis_depth: config.organic_perception_mode,
                confidence: Some(analysis.confidence as f32),
                components_used,
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            
            info!("Intelligence analysis completed in {}ms with confidence: {:.2}", 
                  processing_time, analysis.confidence);
            
            Json(IntelligenceResponse::success(analysis, metadata)).into_response()
        },
        Err(e) => {
            error!("Intelligence analysis failed: {}", e);
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_start.elapsed().as_millis() as u64,
                analysis_depth: "failed".to_string(),
                confidence: None,
                components_used: vec![],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(IntelligenceResponse::<()>::error(format!("Analysis failed: {}", e), metadata))).into_response()
        }
    }
}

/// Get intelligent action recommendation endpoint
pub async fn recommend_action(
    State(_state): State<AppState>,
    Json(req): Json<RecommendActionRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing action recommendation request");
    
    // Validate request
    if let Err(validation_error) = validate_recommend_action_request(&req) {
        let metadata = IntelligenceResponseMetadata {
            processing_time_ms: 0,
            analysis_depth: "none".to_string(),
            confidence: None,
            components_used: vec![],
            total_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_version: "1.0.0".to_string(),
        };
        return (StatusCode::BAD_REQUEST,
               Json(IntelligenceResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    let processing_start = Instant::now();
    
    // Create intelligence service
    let config = req.config.unwrap_or_else(|| IntelligenceConfig::default());
    let intelligence_service = IntelligenceService::new(config);
    
    // Generate action recommendation from analysis
    match intelligence_service.recommend_action(&req.analysis).await {
        Ok(recommendation) => {
            let processing_time = processing_start.elapsed().as_millis() as u64;
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_time,
                analysis_depth: "recommendation".to_string(),
                confidence: Some(recommendation.confidence as f32),
                components_used: vec!["decision_maker".to_string(), "pattern_recognition".to_string()],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            
            info!("Action recommendation generated in {}ms with confidence: {:.2}", 
                  processing_time, recommendation.confidence);
            
            Json(IntelligenceResponse::success(recommendation, metadata)).into_response()
        },
        Err(e) => {
            error!("Action recommendation failed: {}", e);
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_start.elapsed().as_millis() as u64,
                analysis_depth: "failed".to_string(),
                confidence: None,
                components_used: vec![],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(IntelligenceResponse::<()>::error(format!("Recommendation failed: {}", e), metadata))).into_response()
        }
    }
}

/// Learning feedback endpoint - allows the system to learn from results
pub async fn submit_learning_feedback(
    State(_state): State<AppState>,
    Json(req): Json<LearningFeedbackRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Processing learning feedback: success={}", req.success);
    
    // Validate request
    if let Err(validation_error) = validate_learning_feedback_request(&req) {
        let metadata = IntelligenceResponseMetadata {
            processing_time_ms: 0,
            analysis_depth: "none".to_string(),
            confidence: None,
            components_used: vec![],
            total_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_version: "1.0.0".to_string(),
        };
        return (StatusCode::BAD_REQUEST,
               Json(IntelligenceResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    let processing_start = Instant::now();
    
    // Create intelligence service
    let config = req.config.unwrap_or_else(|| IntelligenceConfig::default());
    let intelligence_service = IntelligenceService::new(config);
    
    // Submit learning feedback
    match intelligence_service.learn_from_result(
        &req.action_recommendation,
        &req.actual_result,
        req.success,
        req.execution_time_ms,
    ).await {
        Ok(()) => {
            let processing_time = processing_start.elapsed().as_millis() as u64;
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_time,
                analysis_depth: "learning".to_string(),
                confidence: None,
                components_used: vec!["learning_engine".to_string(), "pattern_recognition".to_string()],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            
            let response_data = serde_json::json!({
                "learned": true,
                "feedback_processed": true,
                "learning_enabled": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            info!("Learning feedback processed successfully in {}ms", processing_time);
            Json(IntelligenceResponse::success(response_data, metadata)).into_response()
        },
        Err(e) => {
            error!("Learning feedback failed: {}", e);
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_start.elapsed().as_millis() as u64,
                analysis_depth: "failed".to_string(),
                confidence: None,
                components_used: vec![],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(IntelligenceResponse::<()>::error(format!("Learning failed: {}", e), metadata))).into_response()
        }
    }
}

/// Get intelligence service statistics endpoint
pub async fn get_intelligence_statistics(
    State(_state): State<AppState>,
    Json(req): Json<StatisticsRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Fetching intelligence statistics");
    
    let processing_start = Instant::now();
    
    // Create intelligence service
    let config = req.config.unwrap_or_else(|| IntelligenceConfig::default());
    let intelligence_service = IntelligenceService::new(config);
    
    // Get statistics
    match intelligence_service.get_statistics().await {
        Ok(stats) => {
            let processing_time = processing_start.elapsed().as_millis() as u64;
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_time,
                analysis_depth: "statistics".to_string(),
                confidence: Some(stats.average_confidence as f32),
                components_used: vec!["learning_engine".to_string(), "pattern_recognition".to_string()],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            
            info!("Intelligence statistics retrieved in {}ms", processing_time);
            Json(IntelligenceResponse::success(stats, metadata)).into_response()
        },
        Err(e) => {
            error!("Failed to get intelligence statistics: {}", e);
            let metadata = IntelligenceResponseMetadata {
                processing_time_ms: processing_start.elapsed().as_millis() as u64,
                analysis_depth: "failed".to_string(),
                confidence: None,
                components_used: vec![],
                total_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_version: "1.0.0".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(IntelligenceResponse::<()>::error(format!("Statistics retrieval failed: {}", e), metadata))).into_response()
        }
    }
}

/// Update intelligence configuration endpoint
pub async fn update_intelligence_config(
    State(_state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    info!("Updating intelligence configuration");
    
    // Validate request
    if let Err(validation_error) = validate_config_update_request(&req) {
        let metadata = IntelligenceResponseMetadata {
            processing_time_ms: 0,
            analysis_depth: "none".to_string(),
            confidence: None,
            components_used: vec![],
            total_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_version: "1.0.0".to_string(),
        };
        return (StatusCode::BAD_REQUEST,
               Json(IntelligenceResponse::<()>::error(validation_error.to_string(), metadata))).into_response();
    }
    
    let processing_start = Instant::now();
    
    // Update configuration (in a real implementation, this would persist the config)
    let new_config = req.config.clone();
    let mut intelligence_service = IntelligenceService::new(req.config);
    intelligence_service.update_config(new_config.clone()).await;
    
    let processing_time = processing_start.elapsed().as_millis() as u64;
    let metadata = IntelligenceResponseMetadata {
        processing_time_ms: processing_time,
        analysis_depth: "configuration".to_string(),
        confidence: None,
        components_used: vec!["intelligence_service".to_string()],
        total_time_ms: start_time.elapsed().as_millis() as u64,
        intelligence_version: "1.0.0".to_string(),
    };
    
    let response_data = serde_json::json!({
        "updated": true,
        "new_config": new_config,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    info!("Intelligence configuration updated successfully in {}ms", processing_time);
    Json(IntelligenceResponse::success(response_data, metadata)).into_response()
}

// Request/Response types for Intelligence API

#[derive(Deserialize)]
pub struct AnalyzeSituationRequest {
    pub user_intent: String,
    pub url: String,
    pub page_title: Option<String>,
    pub meta_description: Option<String>,
    pub page_type: Option<String>, // "form", "list", "article", "navigation", etc.
    pub complexity_score: Option<f64>,
    pub dynamic_content: Option<bool>,
    pub form_elements: Option<u32>,
    pub interactive_elements: Option<u32>,
    pub config: Option<IntelligenceConfig>,
}

#[derive(Deserialize)]
pub struct RecommendActionRequest {
    pub analysis: IntelligenceAnalysis,
    pub config: Option<IntelligenceConfig>,
}

#[derive(Deserialize)]
pub struct LearningFeedbackRequest {
    pub action_recommendation: ActionRecommendation,
    pub actual_result: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub additional_context: Option<HashMap<String, serde_json::Value>>,
    pub config: Option<IntelligenceConfig>,
}

#[derive(Deserialize)]
pub struct StatisticsRequest {
    pub config: Option<IntelligenceConfig>,
    pub include_detailed_metrics: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub config: IntelligenceConfig,
    pub apply_immediately: Option<bool>,
}

// Validation functions

fn validate_analyze_situation_request(req: &AnalyzeSituationRequest) -> Result<(), IntelligenceApiError> {
    if req.user_intent.trim().is_empty() {
        return Err(IntelligenceApiError::ValidationError("User intent cannot be empty".to_string()));
    }
    
    if req.user_intent.len() > 2000 {
        return Err(IntelligenceApiError::ValidationError("User intent too long (max 2000 characters)".to_string()));
    }
    
    if req.url.trim().is_empty() {
        return Err(IntelligenceApiError::ValidationError("URL cannot be empty".to_string()));
    }
    
    // Validate URL format
    if !req.url.starts_with("http://") && !req.url.starts_with("https://") {
        return Err(IntelligenceApiError::ValidationError("URL must start with http:// or https://".to_string()));
    }
    
    if let Some(complexity) = req.complexity_score {
        if complexity < 0.0 || complexity > 1.0 {
            return Err(IntelligenceApiError::ValidationError("Complexity score must be between 0.0 and 1.0".to_string()));
        }
    }
    
    Ok(())
}

fn validate_recommend_action_request(req: &RecommendActionRequest) -> Result<(), IntelligenceApiError> {
    if req.analysis.confidence < 0.0 || req.analysis.confidence > 1.0 {
        return Err(IntelligenceApiError::ValidationError("Analysis confidence must be between 0.0 and 1.0".to_string()));
    }
    
    if req.analysis.reasoning.trim().is_empty() {
        return Err(IntelligenceApiError::ValidationError("Analysis reasoning cannot be empty".to_string()));
    }
    
    Ok(())
}

fn validate_learning_feedback_request(req: &LearningFeedbackRequest) -> Result<(), IntelligenceApiError> {
    if req.actual_result.trim().is_empty() {
        return Err(IntelligenceApiError::ValidationError("Actual result cannot be empty".to_string()));
    }
    
    if req.execution_time_ms > 300_000 { // 5 minutes max
        return Err(IntelligenceApiError::ValidationError("Execution time too long (max 5 minutes)".to_string()));
    }
    
    if req.action_recommendation.confidence < 0.0 || req.action_recommendation.confidence > 1.0 {
        return Err(IntelligenceApiError::ValidationError("Action recommendation confidence must be between 0.0 and 1.0".to_string()));
    }
    
    Ok(())
}

fn validate_config_update_request(req: &UpdateConfigRequest) -> Result<(), IntelligenceApiError> {
    if req.config.confidence_threshold < 0.0 || req.config.confidence_threshold > 1.0 {
        return Err(IntelligenceApiError::ValidationError("Confidence threshold must be between 0.0 and 1.0".to_string()));
    }
    
    if req.config.adaptation_sensitivity < 0.0 || req.config.adaptation_sensitivity > 1.0 {
        return Err(IntelligenceApiError::ValidationError("Adaptation sensitivity must be between 0.0 and 1.0".to_string()));
    }
    
    let valid_perception_modes = ["standard", "enhanced", "deep"];
    if !valid_perception_modes.contains(&req.config.organic_perception_mode.as_str()) {
        return Err(IntelligenceApiError::ValidationError(
            format!("Invalid perception mode '{}'. Valid modes: {}", 
                   req.config.organic_perception_mode, 
                   valid_perception_modes.join(", "))
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_analyze_situation_request() {
        let valid_req = AnalyzeSituationRequest {
            user_intent: "Click the login button".to_string(),
            url: "https://example.com".to_string(),
            page_title: Some("Example Page".to_string()),
            meta_description: None,
            page_type: Some("form".to_string()),
            complexity_score: Some(0.5),
            dynamic_content: Some(false),
            form_elements: Some(2),
            interactive_elements: Some(5),
            config: None,
        };
        assert!(validate_analyze_situation_request(&valid_req).is_ok());
        
        let invalid_req = AnalyzeSituationRequest {
            user_intent: "".to_string(),
            url: "invalid-url".to_string(),
            page_title: None,
            meta_description: None,
            page_type: None,
            complexity_score: Some(2.0), // Invalid: > 1.0
            dynamic_content: None,
            form_elements: None,
            interactive_elements: None,
            config: None,
        };
        assert!(validate_analyze_situation_request(&invalid_req).is_err());
    }
    
    #[test]
    fn test_intelligence_response_metadata() {
        let metadata = IntelligenceResponseMetadata {
            processing_time_ms: 150,
            analysis_depth: "enhanced".to_string(),
            confidence: Some(0.85),
            components_used: vec!["learning_engine".to_string(), "pattern_recognition".to_string()],
            total_time_ms: 200,
            intelligence_version: "1.0.0".to_string(),
        };
        
        assert_eq!(metadata.processing_time_ms, 150);
        assert_eq!(metadata.analysis_depth, "enhanced");
        assert_eq!(metadata.confidence, Some(0.85));
        assert_eq!(metadata.components_used.len(), 2);
    }
}