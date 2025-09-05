// Perception API handlers

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse},
};
use serde::Deserialize;
use tracing::error;

use super::{ApiResponse, AppState};

pub async fn analyze_page(
    State(state): State<AppState>,
    Json(_req): Json<AnalyzePageRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let mut perception = crate::perception::PerceptionEngine::new(browser.browser_arc());
            match perception.analyze_page().await {
                Ok(analysis) => Json(ApiResponse::success(analysis)).into_response(),
                Err(e) => {
                    error!("Page analysis failed: {}", e);
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

pub async fn intelligent_find_element(
    State(state): State<AppState>,
    Json(req): Json<FindElementRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let mut perception = crate::perception::PerceptionEngine::new(browser.browser_arc());
            match perception.find_element(&req.description).await {
                Ok(element) => Json(ApiResponse::success(element)).into_response(),
                Err(e) => {
                    error!("Element finding failed: {}", e);
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

pub async fn execute_intelligent_command(
    State(state): State<AppState>,
    Json(req): Json<IntelligentCommandRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let mut perception_browser = crate::perception::integration::PerceptionAwareBrowser::new(browser.browser_arc());
            match perception_browser.execute_intelligent_command(req.command).await {
                Ok(result) => Json(ApiResponse::success(result)).into_response(),
                Err(e) => {
                    error!("Intelligent command execution failed: {}", e);
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

pub async fn analyze_form(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeFormRequest>,
) -> impl IntoResponse {
    match state.browser_pool.acquire().await {
        Ok(browser) => {
            let form_handler = crate::perception::smart_forms::SmartFormHandler::new();
            match form_handler.analyze_form(browser.browser(), req.form_selector.as_deref()).await {
                Ok(analysis) => Json(ApiResponse::success(analysis)).into_response(),
                Err(e) => {
                    error!("Form analysis failed: {}", e);
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
            match form_handler.analyze_form(browser.browser(), req.form_selector.as_deref()).await {
                Ok(form_analysis) => {
                    match form_handler.auto_fill_form(browser.browser(), &form_analysis, &req.profile_name).await {
                        Ok(fill_result) => Json(ApiResponse::success(fill_result)).into_response(),
                        Err(e) => {
                            error!("Form auto-fill failed: {}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR,
                             Json(ApiResponse::<()>::error(e.to_string()))).into_response()
                        }
                    }
                }
                Err(e) => {
                    error!("Form analysis failed: {}", e);
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

// Request/Response types for perception API
#[derive(Deserialize)]
pub struct AnalyzePageRequest {
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct FindElementRequest {
    pub description: String,
}

#[derive(Deserialize)]
pub struct IntelligentCommandRequest {
    pub command: crate::perception::integration::IntelligentCommand,
}

#[derive(Deserialize)]
pub struct AnalyzeFormRequest {
    pub form_selector: Option<String>,
}

#[derive(Deserialize)]
pub struct AutoFillFormRequest {
    pub form_selector: Option<String>,
    pub profile_name: String,
    pub user_profile: Option<crate::perception::smart_forms::UserProfile>,
}