// Result Verification Engine
// Part of the Intelligent Action Engine

use crate::error::{Result, RainbowError};
use crate::action::{Action, ActionType, BoundingBox, ElementInfo};
use chromiumoxide::{Page, Element};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Verification engine that validates action execution results
#[derive(Debug)]
pub struct VerificationEngine {
    verifiers: Vec<Box<dyn ActionVerifier + Send + Sync>>,
    verification_cache: tokio::sync::RwLock<HashMap<String, VerificationCache>>,
}

#[derive(Debug, Clone)]
struct VerificationCache {
    result: VerificationResult,
    timestamp: Instant,
    action_id: uuid::Uuid,
}

impl VerificationEngine {
    pub fn new() -> Self {
        Self {
            verifiers: vec![
                Box::new(ClickVerifier),
                Box::new(InputVerifier),
                Box::new(NavigationVerifier),
                Box::new(VisibilityVerifier),
                Box::new(StateChangeVerifier),
                Box::new(PerformanceVerifier),
            ],
            verification_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Verify that an action was successfully executed
    pub async fn verify_action_result(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationResult> {
        let cache_key = format!("{}-{:?}", action.id, action.action_type);
        
        // Check cache first
        if let Some(cached) = self.check_cache(&cache_key).await {
            return Ok(cached);
        }

        let start_time = Instant::now();
        let mut verification_result = VerificationResult {
            action_id: action.id,
            success: true,
            confidence: 1.0,
            verification_time: Duration::default(),
            checks_performed: Vec::new(),
            error_details: None,
            element_state_before: None,
            element_state_after: None,
            page_changes: Vec::new(),
        };

        // Get element state before verification
        verification_result.element_state_before = Some(
            self.capture_element_state(element).await?
        );

        // Run appropriate verifiers
        let mut total_confidence = 0.0;
        let mut verifier_count = 0;

        for verifier in &self.verifiers {
            if verifier.can_verify(&action.action_type) {
                let check_result = verifier
                    .verify(page.clone(), action, element)
                    .await;

                match check_result {
                    Ok(check) => {
                        verification_result.checks_performed.push(check.clone());
                        total_confidence += check.confidence;
                        verifier_count += 1;

                        if !check.passed {
                            verification_result.success = false;
                            if verification_result.error_details.is_none() {
                                verification_result.error_details = Some(
                                    check.error_message.unwrap_or_else(|| {
                                        format!("Verification failed: {}", check.check_name)
                                    })
                                );
                            }
                        }
                    }
                    Err(e) => {
                        verification_result.success = false;
                        verification_result.error_details = Some(
                            format!("Verifier error: {}", e)
                        );
                    }
                }
            }
        }

        // Calculate overall confidence
        if verifier_count > 0 {
            verification_result.confidence = total_confidence / verifier_count as f64;
        }

        // Get element state after verification
        verification_result.element_state_after = Some(
            self.capture_element_state(element).await?
        );

        // Detect page changes
        verification_result.page_changes = self.detect_page_changes(page.clone()).await?;

        verification_result.verification_time = start_time.elapsed();

        // Cache the result
        self.cache_result(&cache_key, &verification_result).await;

        Ok(verification_result)
    }

    /// Verify multiple actions in a chain
    pub async fn verify_action_chain(
        &self,
        page: Arc<Page>,
        action_results: &[crate::action::ActionResult],
    ) -> Result<ChainVerificationResult> {
        let mut chain_result = ChainVerificationResult {
            overall_success: true,
            individual_results: Vec::new(),
            chain_integrity_score: 1.0,
            total_verification_time: Duration::default(),
            breaking_point: None,
        };

        let start_time = Instant::now();

        for (index, action_result) in action_results.iter().enumerate() {
            // For chain verification, we focus on state consistency
            let chain_check = self.verify_chain_consistency(
                page.clone(),
                action_result,
                index,
                action_results,
            ).await?;

            chain_result.individual_results.push(chain_check.clone());

            if !chain_check.passed {
                chain_result.overall_success = false;
                if chain_result.breaking_point.is_none() {
                    chain_result.breaking_point = Some(index);
                }
            }
        }

        // Calculate chain integrity score
        let passed_count = chain_result.individual_results
            .iter()
            .filter(|r| r.passed)
            .count();
        
        chain_result.chain_integrity_score = if action_results.is_empty() {
            0.0
        } else {
            passed_count as f64 / action_results.len() as f64
        };

        chain_result.total_verification_time = start_time.elapsed();

        Ok(chain_result)
    }

    async fn check_cache(&self, cache_key: &str) -> Option<VerificationResult> {
        let cache = self.verification_cache.read().await;
        
        if let Some(cached) = cache.get(cache_key) {
            // Cache is valid for 10 seconds
            if cached.timestamp.elapsed() < Duration::from_secs(10) {
                return Some(cached.result.clone());
            }
        }

        None
    }

    async fn cache_result(&self, cache_key: &str, result: &VerificationResult) {
        let mut cache = self.verification_cache.write().await;
        
        cache.insert(cache_key.to_string(), VerificationCache {
            result: result.clone(),
            timestamp: Instant::now(),
            action_id: result.action_id,
        });

        // Clean old cache entries (keep last 100)
        if cache.len() > 100 {
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by_key(|(_, v)| v.timestamp);
            
            let keys_to_remove: Vec<String> = entries
                .iter()
                .take(cache.len() - 100)
                .map(|(k, _)| k.to_string())
                .collect();

            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }

    async fn capture_element_state(&self, element: &Element) -> Result<ElementState> {
        let js_code = r#"
            const elem = arguments[0];
            return {
                tagName: elem.tagName,
                id: elem.id || null,
                className: elem.className || null,
                value: elem.value || null,
                checked: elem.checked || null,
                selected: elem.selected || null,
                disabled: elem.disabled || null,
                hidden: elem.hidden || null,
                textContent: elem.textContent || null,
                innerHTML: elem.innerHTML || null,
                attributes: Array.from(elem.attributes).reduce((acc, attr) => {
                    acc[attr.name] = attr.value;
                    return acc;
                }, {}),
                style: {
                    display: window.getComputedStyle(elem).display,
                    visibility: window.getComputedStyle(elem).visibility,
                    opacity: window.getComputedStyle(elem).opacity
                },
                boundingRect: elem.getBoundingClientRect()
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let state_obj = result.as_object().ok_or_else(|| {
            RainbowError::ExecutionError("Failed to capture element state".to_string())
        })?;

        Ok(ElementState {
            tag_name: state_obj.get("tagName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            attributes: state_obj.get("attributes")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            value: state_obj.get("value")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            text_content: state_obj.get("textContent")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            is_visible: state_obj.get("style")
                .and_then(|style| style.as_object())
                .map(|style| {
                    style.get("display").and_then(|v| v.as_str()) != Some("none") &&
                    style.get("visibility").and_then(|v| v.as_str()) != Some("hidden") &&
                    style.get("opacity").and_then(|v| v.as_str()) != Some("0")
                })
                .unwrap_or(false),
            is_enabled: !state_obj.get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            bounding_rect: state_obj.get("boundingRect")
                .and_then(|rect| serde_json::from_value(rect.clone()).ok())
                .unwrap_or(BoundingBox { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }),
        })
    }

    async fn detect_page_changes(&self, page: Arc<Page>) -> Result<Vec<PageChange>> {
        let js_code = r#"
            return {
                url: window.location.href,
                title: document.title,
                loadState: document.readyState,
                elementCount: document.querySelectorAll('*').length,
                timestamp: Date.now()
            };
        "#;

        let result = page.evaluate(js_code).await?;
        let page_info = result.as_object().ok_or_else(|| {
            RainbowError::ExecutionError("Failed to get page information".to_string())
        })?;

        // For now, just capture basic page info
        // In a real implementation, this would compare against previous state
        Ok(vec![
            PageChange {
                change_type: "info_capture".to_string(),
                description: "Page information captured".to_string(),
                element_selector: None,
                before_value: None,
                after_value: Some(serde_json::to_string(page_info)?),
            }
        ])
    }

    async fn verify_chain_consistency(
        &self,
        page: Arc<Page>,
        action_result: &crate::action::ActionResult,
        index: usize,
        all_results: &[crate::action::ActionResult],
    ) -> Result<VerificationCheck> {
        // Check if the action result is consistent with the chain
        let mut check = VerificationCheck {
            check_name: format!("ChainConsistency_{}", index),
            passed: action_result.success,
            confidence: 0.8,
            details: format!("Action {} in chain", index),
            error_message: None,
            timing: Duration::default(),
        };

        // Additional chain-specific checks
        if index > 0 {
            let previous_result = &all_results[index - 1];
            
            // Check if previous action might have affected this one
            if !previous_result.success && action_result.success {
                check.confidence = 0.6; // Lower confidence if previous failed
                check.details.push_str(" (previous action failed)");
            }
        }

        if !action_result.success {
            check.error_message = action_result.error.clone();
        }

        Ok(check)
    }
}

impl Default for VerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of action verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub action_id: uuid::Uuid,
    pub success: bool,
    pub confidence: f64,
    pub verification_time: Duration,
    pub checks_performed: Vec<VerificationCheck>,
    pub error_details: Option<String>,
    pub element_state_before: Option<ElementState>,
    pub element_state_after: Option<ElementState>,
    pub page_changes: Vec<PageChange>,
}

/// Individual verification check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub check_name: String,
    pub passed: bool,
    pub confidence: f64,
    pub details: String,
    pub error_message: Option<String>,
    pub timing: Duration,
}

/// Element state capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementState {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub value: Option<String>,
    pub text_content: Option<String>,
    pub is_visible: bool,
    pub is_enabled: bool,
    pub bounding_rect: BoundingBox,
}

/// Page change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageChange {
    pub change_type: String,
    pub description: String,
    pub element_selector: Option<String>,
    pub before_value: Option<String>,
    pub after_value: Option<String>,
}

/// Chain verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerificationResult {
    pub overall_success: bool,
    pub individual_results: Vec<VerificationCheck>,
    pub chain_integrity_score: f64,
    pub total_verification_time: Duration,
    pub breaking_point: Option<usize>,
}

/// Trait for specific action verifiers
#[async_trait::async_trait]
trait ActionVerifier: std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn can_verify(&self, action_type: &ActionType) -> bool;
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationCheck>;
}

/// Click action verifier
#[derive(Debug)]
struct ClickVerifier;

#[async_trait::async_trait]
impl ActionVerifier for ClickVerifier {
    fn name(&self) -> &'static str { "ClickVerifier" }
    
    fn can_verify(&self, action_type: &ActionType) -> bool {
        matches!(action_type, 
            ActionType::Click | 
            ActionType::DoubleClick | 
            ActionType::RightClick
        )
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        // Check if element is still clickable
        let js_code = r#"
            const elem = arguments[0];
            const rect = elem.getBoundingClientRect();
            const style = window.getComputedStyle(elem);
            
            const isClickable = rect.width > 0 && 
                               rect.height > 0 && 
                               style.pointerEvents !== 'none' &&
                               !elem.disabled;
            
            return {
                clickable: isClickable,
                rect: rect,
                style: {
                    pointerEvents: style.pointerEvents,
                    display: style.display,
                    visibility: style.visibility
                }
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let check_result = result.as_object().unwrap();
        
        let is_clickable = check_result.get("clickable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(VerificationCheck {
            check_name: "ClickabilityCheck".to_string(),
            passed: is_clickable,
            confidence: if is_clickable { 0.9 } else { 0.1 },
            details: format!("Element clickability verified: {}", is_clickable),
            error_message: if !is_clickable { 
                Some("Element is not clickable".to_string()) 
            } else { 
                None 
            },
            timing: start_time.elapsed(),
        })
    }
}

/// Input action verifier
#[derive(Debug)]
struct InputVerifier;

#[async_trait::async_trait]
impl ActionVerifier for InputVerifier {
    fn name(&self) -> &'static str { "InputVerifier" }
    
    fn can_verify(&self, action_type: &ActionType) -> bool {
        matches!(action_type, ActionType::Type(_) | ActionType::Clear)
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        let js_code = r#"
            const elem = arguments[0];
            return {
                value: elem.value || '',
                readOnly: elem.readOnly,
                disabled: elem.disabled,
                type: elem.type
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let input_state = result.as_object().unwrap();
        
        let current_value = input_state.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let is_readonly = input_state.get("readOnly")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let is_disabled = input_state.get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let verification_passed = match &action.action_type {
            ActionType::Type(expected_text) => {
                current_value.contains(expected_text) && !is_readonly && !is_disabled
            }
            ActionType::Clear => {
                current_value.is_empty() && !is_readonly && !is_disabled
            }
            _ => false,
        };

        Ok(VerificationCheck {
            check_name: "InputVerification".to_string(),
            passed: verification_passed,
            confidence: if verification_passed { 0.95 } else { 0.2 },
            details: format!("Input state: value='{}', readonly={}, disabled={}", 
                           current_value, is_readonly, is_disabled),
            error_message: if !verification_passed {
                Some(format!("Input verification failed. Current value: '{}'", current_value))
            } else {
                None
            },
            timing: start_time.elapsed(),
        })
    }
}

/// Navigation action verifier
#[derive(Debug)]
struct NavigationVerifier;

#[async_trait::async_trait]
impl ActionVerifier for NavigationVerifier {
    fn name(&self) -> &'static str { "NavigationVerifier" }
    
    fn can_verify(&self, action_type: &ActionType) -> bool {
        matches!(action_type, 
            ActionType::Navigate(_) | 
            ActionType::GoBack | 
            ActionType::GoForward | 
            ActionType::Refresh
        )
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        _element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        let current_url = page.url().await.unwrap_or_default();
        
        let verification_passed = match &action.action_type {
            ActionType::Navigate(expected_url) => {
                current_url.contains(expected_url) || 
                expected_url.contains(&current_url)
            }
            ActionType::GoBack | ActionType::GoForward | ActionType::Refresh => {
                // For these actions, we verify that navigation completed
                !current_url.is_empty()
            }
            _ => false,
        };

        Ok(VerificationCheck {
            check_name: "NavigationVerification".to_string(),
            passed: verification_passed,
            confidence: if verification_passed { 0.9 } else { 0.3 },
            details: format!("Current URL: {}", current_url),
            error_message: if !verification_passed {
                Some(format!("Navigation verification failed. Current URL: {}", current_url))
            } else {
                None
            },
            timing: start_time.elapsed(),
        })
    }
}

/// Visibility verifier
#[derive(Debug)]
struct VisibilityVerifier;

#[async_trait::async_trait]
impl ActionVerifier for VisibilityVerifier {
    fn name(&self) -> &'static str { "VisibilityVerifier" }
    
    fn can_verify(&self, _action_type: &ActionType) -> bool {
        true // Can verify any action's element visibility
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        let js_code = r#"
            const elem = arguments[0];
            const rect = elem.getBoundingClientRect();
            const style = window.getComputedStyle(elem);
            
            const isVisible = rect.width > 0 && 
                             rect.height > 0 && 
                             style.visibility !== 'hidden' && 
                             style.display !== 'none' &&
                             style.opacity !== '0';
            
            return {
                visible: isVisible,
                rect: rect,
                computed_style: {
                    display: style.display,
                    visibility: style.visibility,
                    opacity: style.opacity
                }
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let visibility_info = result.as_object().unwrap();
        
        let is_visible = visibility_info.get("visible")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(VerificationCheck {
            check_name: "VisibilityCheck".to_string(),
            passed: is_visible,
            confidence: if is_visible { 0.8 } else { 0.4 },
            details: format!("Element visibility: {}", is_visible),
            error_message: if !is_visible {
                Some("Element is not visible".to_string())
            } else {
                None
            },
            timing: start_time.elapsed(),
        })
    }
}

/// State change verifier
#[derive(Debug)]
struct StateChangeVerifier;

#[async_trait::async_trait]
impl ActionVerifier for StateChangeVerifier {
    fn name(&self) -> &'static str { "StateChangeVerifier" }
    
    fn can_verify(&self, _action_type: &ActionType) -> bool {
        true // Can verify state changes for any action
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        // This would ideally compare before/after states
        // For now, just verify element still exists and is accessible
        let element_exists = element.tag_name().await.is_ok();
        
        Ok(VerificationCheck {
            check_name: "StateChangeCheck".to_string(),
            passed: element_exists,
            confidence: if element_exists { 0.7 } else { 0.2 },
            details: format!("Element exists after action: {}", element_exists),
            error_message: if !element_exists {
                Some("Element no longer exists or accessible".to_string())
            } else {
                None
            },
            timing: start_time.elapsed(),
        })
    }
}

/// Performance verifier
#[derive(Debug)]
struct PerformanceVerifier;

#[async_trait::async_trait]
impl ActionVerifier for PerformanceVerifier {
    fn name(&self) -> &'static str { "PerformanceVerifier" }
    
    fn can_verify(&self, _action_type: &ActionType) -> bool {
        true // Can verify performance for any action
    }
    
    async fn verify(
        &self,
        page: Arc<Page>,
        action: &Action,
        _element: &Element,
    ) -> Result<VerificationCheck> {
        let start_time = Instant::now();
        
        // Check page performance metrics
        let js_code = r#"
            const nav = performance.getEntriesByType('navigation')[0];
            return {
                loadTime: nav ? nav.loadEventEnd - nav.loadEventStart : 0,
                domContentLoaded: nav ? nav.domContentLoadedEventEnd - nav.domContentLoadedEventStart : 0,
                timestamp: Date.now()
            };
        "#;

        let result = page.evaluate(js_code).await?;
        let perf_info = result.as_object().unwrap();
        
        let load_time = perf_info.get("loadTime")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Consider performance acceptable if load time is reasonable
        let performance_ok = load_time < 5000.0; // 5 seconds max

        Ok(VerificationCheck {
            check_name: "PerformanceCheck".to_string(),
            passed: performance_ok,
            confidence: 0.6, // Lower confidence as performance can vary
            details: format!("Page load time: {:.2}ms", load_time),
            error_message: if !performance_ok {
                Some(format!("Performance degraded. Load time: {:.2}ms", load_time))
            } else {
                None
            },
            timing: start_time.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_engine_creation() {
        let engine = VerificationEngine::new();
        assert_eq!(engine.verifiers.len(), 6);
    }

    #[test]
    fn test_verifier_capabilities() {
        let click_verifier = ClickVerifier;
        assert!(click_verifier.can_verify(&ActionType::Click));
        assert!(!click_verifier.can_verify(&ActionType::Type("test".to_string())));

        let input_verifier = InputVerifier;
        assert!(input_verifier.can_verify(&ActionType::Type("test".to_string())));
        assert!(!input_verifier.can_verify(&ActionType::Click));
    }

    #[test]
    fn test_verification_result_creation() {
        let result = VerificationResult {
            action_id: uuid::Uuid::new_v4(),
            success: true,
            confidence: 0.9,
            verification_time: Duration::from_millis(100),
            checks_performed: vec![],
            error_details: None,
            element_state_before: None,
            element_state_after: None,
            page_changes: vec![],
        };

        assert!(result.success);
        assert_eq!(result.confidence, 0.9);
    }
}