// Advanced Perception Engine - Integration Layer
// Combines all perception capabilities into a unified, intelligent system

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use thirtyfour::{WebDriver, WebElement};
use tracing::{debug, info, warn, error};

use super::perception_orchestrator::{PerceptionOrchestrator, UnifiedPerceptionResult, PerceptionLevel};
use super::enhanced_error_recovery::{EnhancedErrorRecovery, RecoveryResult, RecoveryStats};
use super::enhanced_form_handler::{EnhancedFormHandler, FormInteractionResult};
use crate::smart_element_detector::{SmartElementDetector, ElementDescriptor, ElementType, detect_element_type};
use super::browser_connection::BrowserConnection;

/// Advanced Perception Engine that unifies all perception capabilities
pub struct AdvancedPerceptionEngine {
    orchestrator: PerceptionOrchestrator,
    error_recovery: EnhancedErrorRecovery,
    form_handler: EnhancedFormHandler,
    element_detector: SmartElementDetector,
    browser_connection: BrowserConnection,
    stats: Arc<RwLock<PerceptionStats>>,
    config: AdvancedPerceptionConfig,
}

#[derive(Debug, Clone)]
pub struct AdvancedPerceptionConfig {
    pub intelligent_layer_selection: bool,
    pub auto_error_recovery: bool,
    pub smart_form_handling: bool,
    pub performance_optimization: bool,
    pub adaptive_learning: bool,
    pub real_time_validation: bool,
    pub context_awareness: bool,
}

impl Default for AdvancedPerceptionConfig {
    fn default() -> Self {
        Self {
            intelligent_layer_selection: true,
            auto_error_recovery: true,
            smart_form_handling: true,
            performance_optimization: true,
            adaptive_learning: true,
            real_time_validation: true,
            context_awareness: true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PerceptionStats {
    pub total_requests: u64,
    pub successful_interactions: u64,
    pub error_recoveries: u64,
    pub form_interactions: u64,
    pub average_response_time_ms: f64,
    pub intelligence_usage: IntelligenceUsage,
    pub success_rate: f64,
}

#[derive(Debug, Default, Clone)]
pub struct IntelligenceUsage {
    pub lightning_layer_uses: u64,
    pub quick_layer_uses: u64,
    pub standard_layer_uses: u64,
    pub deep_layer_uses: u64,
    pub error_recovery_uses: u64,
    pub smart_detection_uses: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPerceptionResult<T> {
    pub result: Option<T>,
    pub success: bool,
    pub confidence: f32,
    pub strategy_used: PerceptionStrategy,
    pub execution_time_ms: u64,
    pub intelligence_level: PerceptionLevel,
    pub error_message: Option<String>,
    pub suggestions: Vec<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionStrategy {
    DirectPerception,
    IntelligentRecovery,
    SmartFormHandling,
    AdaptiveLayer,
    FallbackDetection,
    ContextAware,
}

impl AdvancedPerceptionEngine {
    /// Create new Advanced Perception Engine with all capabilities
    pub async fn new(
        driver: WebDriver,
        config: Option<AdvancedPerceptionConfig>,
    ) -> Result<Self> {
        let browser_connection = BrowserConnection::new(driver.clone()).await?;
        let orchestrator = PerceptionOrchestrator::new(browser_connection.clone()).await?;
        let error_recovery = EnhancedErrorRecovery::new(driver.clone(), None);
        let form_handler = EnhancedFormHandler::new(driver.clone(), None);
        let element_detector = SmartElementDetector::new(driver.clone());
        
        Ok(Self {
            orchestrator,
            error_recovery,
            form_handler,
            element_detector,
            browser_connection,
            stats: Arc::new(RwLock::new(PerceptionStats::default())),
            config: config.unwrap_or_default(),
        })
    }

    /// Find element using the most appropriate strategy
    pub async fn find_element_intelligently(
        &self,
        description: &str,
    ) -> AdvancedPerceptionResult<WebElement> {
        let start_time = Instant::now();
        self.update_request_stats().await;

        info!("Advanced perception: finding element '{}'", description);

        // Step 1: Analyze the request to determine optimal strategy
        let element_type = detect_element_type(description);
        let complexity = self.assess_complexity(description, &element_type).await;
        
        // Step 2: Try intelligent layer selection first
        if self.config.intelligent_layer_selection {
            if let Some(result) = self.try_perception_layers(description, complexity).await {
                return self.create_success_result(
                    result,
                    PerceptionStrategy::DirectPerception,
                    PerceptionLevel::Lightning, // Will be updated based on actual layer used
                    start_time,
                ).await;
            }
        }

        // Step 3: Try smart element detection with error recovery
        if self.config.auto_error_recovery {
            let descriptor = ElementDescriptor {
                description: description.to_string(),
                element_type: element_type.clone(),
                attributes: std::collections::HashMap::new(),
                context: None,
            };

            let recovery_result = self.error_recovery.find_element_with_recovery(&descriptor).await;
            
            if let Some(element) = recovery_result.result {
                self.update_success_stats().await;
                return AdvancedPerceptionResult {
                    result: Some(element),
                    success: true,
                    confidence: recovery_result.confidence,
                    strategy_used: PerceptionStrategy::IntelligentRecovery,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    intelligence_level: PerceptionLevel::Quick,
                    error_message: None,
                    suggestions: vec![],
                    metadata: self.create_metadata(&recovery_result).await,
                };
            }
        }

        // Step 4: Final attempt with context-aware fallback
        if self.config.context_awareness {
            if let Some(result) = self.try_context_aware_detection(description, &element_type).await {
                return self.create_success_result(
                    result,
                    PerceptionStrategy::ContextAware,
                    PerceptionLevel::Standard,
                    start_time,
                ).await;
            }
        }

        // Step 5: Complete failure - return detailed error information
        self.update_failure_stats().await;
        AdvancedPerceptionResult {
            result: None,
            success: false,
            confidence: 0.0,
            strategy_used: PerceptionStrategy::FallbackDetection,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_level: PerceptionLevel::Deep,
            error_message: Some(format!("Could not find element '{}' using any available strategy", description)),
            suggestions: self.generate_failure_suggestions(description, &element_type).await,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Fill form field with advanced intelligence
    pub async fn fill_form_field_intelligently(
        &mut self,
        field_description: &str,
        value: &str,
    ) -> AdvancedPerceptionResult<FormInteractionResult> {
        let start_time = Instant::now();
        self.update_request_stats().await;

        info!("Advanced form handling: filling '{}' with value", field_description);

        if !self.config.smart_form_handling {
            return AdvancedPerceptionResult {
                result: None,
                success: false,
                confidence: 0.0,
                strategy_used: PerceptionStrategy::SmartFormHandling,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                intelligence_level: PerceptionLevel::Quick,
                error_message: Some("Smart form handling is disabled".to_string()),
                suggestions: vec!["Enable smart form handling in configuration".to_string()],
                metadata: std::collections::HashMap::new(),
            };
        }

        // Use enhanced form handler
        match self.form_handler.fill_field(field_description, value).await {
            Ok(form_result) => {
                self.update_form_interaction_stats().await;
                
                if form_result.success {
                    self.update_success_stats().await;
                }

                AdvancedPerceptionResult {
                    result: Some(form_result.clone()),
                    success: form_result.success,
                    confidence: if form_result.success { 0.9 } else { 0.3 },
                    strategy_used: PerceptionStrategy::SmartFormHandling,
                    execution_time_ms: form_result.execution_time_ms,
                    intelligence_level: PerceptionLevel::Standard,
                    error_message: form_result.error_message,
                    suggestions: form_result.suggestions,
                    metadata: self.create_form_metadata(&form_result).await,
                }
            },
            Err(e) => {
                self.update_failure_stats().await;
                AdvancedPerceptionResult {
                    result: None,
                    success: false,
                    confidence: 0.0,
                    strategy_used: PerceptionStrategy::SmartFormHandling,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    intelligence_level: PerceptionLevel::Standard,
                    error_message: Some(e.to_string()),
                    suggestions: vec![
                        "Check if the field is visible and interactable".to_string(),
                        "Verify the field description is accurate".to_string(),
                    ],
                    metadata: std::collections::HashMap::new(),
                }
            }
        }
    }

    /// Perform comprehensive page analysis
    pub async fn analyze_page_comprehensively(
        &self,
        analysis_level: PerceptionLevel,
    ) -> AdvancedPerceptionResult<UnifiedPerceptionResult> {
        let start_time = Instant::now();
        self.update_request_stats().await;

        info!("Comprehensive page analysis at level: {:?}", analysis_level);

        match self.orchestrator.execute_perception(analysis_level).await {
            Ok(perception_result) => {
                self.update_success_stats().await;
                
                AdvancedPerceptionResult {
                    result: Some(perception_result.clone()),
                    success: true,
                    confidence: 0.95,
                    strategy_used: PerceptionStrategy::DirectPerception,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    intelligence_level: analysis_level,
                    error_message: None,
                    suggestions: vec![],
                    metadata: self.create_perception_metadata(&perception_result).await,
                }
            },
            Err(e) => {
                self.update_failure_stats().await;
                AdvancedPerceptionResult {
                    result: None,
                    success: false,
                    confidence: 0.0,
                    strategy_used: PerceptionStrategy::DirectPerception,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    intelligence_level: analysis_level,
                    error_message: Some(e.to_string()),
                    suggestions: vec![
                        "Try a different perception level".to_string(),
                        "Check if the page is fully loaded".to_string(),
                    ],
                    metadata: std::collections::HashMap::new(),
                }
            }
        }
    }

    /// Get comprehensive system statistics
    pub async fn get_comprehensive_stats(&self) -> ComprehensiveStats {
        let perception_stats = self.stats.read().await.clone();
        let recovery_stats = self.error_recovery.get_stats().await;
        
        ComprehensiveStats {
            perception: perception_stats,
            recovery: recovery_stats,
            system_health: self.assess_system_health().await,
            recommendations: self.generate_system_recommendations().await,
        }
    }

    // Private helper methods

    async fn assess_complexity(&self, description: &str, element_type: &ElementType) -> f32 {
        let mut complexity = 0.5; // Base complexity
        
        // Increase complexity for certain element types
        match element_type {
            ElementType::Select => complexity += 0.2,
            ElementType::Form => complexity += 0.3,
            ElementType::Unknown => complexity += 0.4,
            _ => {}
        }
        
        // Increase complexity for ambiguous descriptions
        let words = description.split_whitespace().count();
        if words < 2 {
            complexity += 0.2;
        }
        if words > 5 {
            complexity += 0.1;
        }
        
        complexity.min(1.0)
    }

    async fn try_perception_layers(&self, description: &str, complexity: f32) -> Option<WebElement> {
        // Select appropriate layer based on complexity
        let layer = if complexity < 0.3 {
            PerceptionLevel::Lightning
        } else if complexity < 0.6 {
            PerceptionLevel::Quick
        } else if complexity < 0.8 {
            PerceptionLevel::Standard
        } else {
            PerceptionLevel::Deep
        };

        // Try the selected layer
        if let Ok(result) = self.orchestrator.execute_perception(layer).await {
            // Look for matching elements in the perception result
            // This is a simplified implementation - in practice, you'd want more sophisticated matching
            None // Placeholder - would implement element extraction from perception result
        } else {
            None
        }
    }

    async fn try_context_aware_detection(
        &self,
        description: &str,
        element_type: &ElementType,
    ) -> Option<WebElement> {
        // Implement context-aware detection logic
        // This would use page context, user history, and intelligent patterns
        None // Placeholder for now
    }

    async fn create_success_result(
        &self,
        element: WebElement,
        strategy: PerceptionStrategy,
        level: PerceptionLevel,
        start_time: Instant,
    ) -> AdvancedPerceptionResult<WebElement> {
        self.update_success_stats().await;
        
        AdvancedPerceptionResult {
            result: Some(element),
            success: true,
            confidence: 0.9,
            strategy_used: strategy,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            intelligence_level: level,
            error_message: None,
            suggestions: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    async fn generate_failure_suggestions(
        &self,
        description: &str,
        element_type: &ElementType,
    ) -> Vec<String> {
        let mut suggestions = vec![
            "Check if the element is visible on the current page".to_string(),
            "Try scrolling to make the element visible".to_string(),
            "Verify the element description matches the actual element".to_string(),
        ];

        match element_type {
            ElementType::Button => {
                suggestions.push("Look for submit buttons or clickable elements".to_string());
            },
            ElementType::Input => {
                suggestions.push("Check for input fields, text areas, or form elements".to_string());
            },
            ElementType::Select => {
                suggestions.push("Look for dropdown menus or select elements".to_string());
            },
            _ => {
                suggestions.push("Try using a more specific description".to_string());
            }
        }

        suggestions
    }

    async fn create_metadata(
        &self,
        recovery_result: &RecoveryResult<WebElement>,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert(
            "recovery_strategy".to_string(),
            serde_json::json!(recovery_result.recovery_strategy_used),
        );
        metadata.insert(
            "confidence".to_string(),
            serde_json::json!(recovery_result.confidence),
        );
        
        if let Some(partial_data) = &recovery_result.partial_data {
            metadata.insert("partial_data".to_string(), partial_data.clone());
        }
        
        metadata
    }

    async fn create_form_metadata(
        &self,
        form_result: &FormInteractionResult,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();
        
        metadata.insert(
            "field_updated".to_string(),
            serde_json::json!(form_result.field_updated),
        );
        metadata.insert(
            "validation_passed".to_string(),
            serde_json::json!(form_result.validation_passed),
        );
        
        metadata
    }

    async fn create_perception_metadata(
        &self,
        perception_result: &UnifiedPerceptionResult,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();
        
        // Add perception result details to metadata
        // This would depend on the actual structure of UnifiedPerceptionResult
        metadata.insert(
            "perception_level".to_string(),
            serde_json::json!("comprehensive_analysis"),
        );
        
        metadata
    }

    async fn assess_system_health(&self) -> f32 {
        let stats = self.stats.read().await;
        if stats.total_requests > 0 {
            stats.successful_interactions as f32 / stats.total_requests as f32
        } else {
            1.0
        }
    }

    async fn generate_system_recommendations(&self) -> Vec<String> {
        let health = self.assess_system_health().await;
        let mut recommendations = Vec::new();
        
        if health < 0.8 {
            recommendations.push("System performance is below optimal. Consider adjusting configuration.".to_string());
        }
        
        if health > 0.95 {
            recommendations.push("System performing excellently!".to_string());
        }
        
        recommendations
    }

    // Statistics update methods
    async fn update_request_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
    }

    async fn update_success_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.successful_interactions += 1;
        stats.success_rate = stats.successful_interactions as f64 / stats.total_requests as f64;
    }

    async fn update_failure_stats(&self) {
        // Failure stats are implicitly calculated from success rate
    }

    async fn update_form_interaction_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.form_interactions += 1;
    }
}

#[derive(Debug, Clone)]
pub struct ComprehensiveStats {
    pub perception: PerceptionStats,
    pub recovery: RecoveryStats,
    pub system_health: f32,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_perception_config_default() {
        let config = AdvancedPerceptionConfig::default();
        assert!(config.intelligent_layer_selection);
        assert!(config.auto_error_recovery);
        assert!(config.smart_form_handling);
    }

    #[test]
    fn test_perception_stats_default() {
        let stats = PerceptionStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_interactions, 0);
    }
}