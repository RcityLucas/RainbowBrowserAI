// Intelligence Module
// Provides advanced AI-driven automation with learning and adaptation capabilities

pub mod adaptation_manager;
pub mod decision_maker;
pub mod learning_engine;
pub mod organic_perception;
pub mod pattern_recognition;

// Re-exports for public API
pub use adaptation_manager::{AdaptationManager, AdaptationStrategy, EnvironmentContext};
pub use decision_maker::{Confidence, Decision, DecisionContext, DecisionMaker};
pub use learning_engine::{ActionPattern, LearningData, LearningEngine, PerformanceMetrics};
pub use organic_perception::{
    ElementInsight, OrganicPerceptionEngine, PageContext, PerceptionResult, ViewportInfo,
};
pub use pattern_recognition::{ActionSequence, PatternMatch, PatternRecognizer, SuccessPattern};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Main intelligence service coordinator
#[derive(Debug)]
pub struct IntelligenceService {
    organic_perception: Arc<RwLock<OrganicPerceptionEngine>>,
    learning_engine: Arc<RwLock<LearningEngine>>,
    adaptation_manager: Arc<RwLock<AdaptationManager>>,
    pattern_recognizer: Arc<RwLock<PatternRecognizer>>,
    decision_maker: Arc<RwLock<DecisionMaker>>,
    config: IntelligenceConfig,
}

/// Configuration for intelligence services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub learning_enabled: bool,
    pub adaptation_enabled: bool,
    pub pattern_recognition_enabled: bool,
    pub organic_perception_mode: String, // "standard", "enhanced", "deep"
    pub confidence_threshold: f64,
    pub max_learning_samples: usize,
    pub adaptation_sensitivity: f64,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            learning_enabled: true,
            adaptation_enabled: true,
            pattern_recognition_enabled: true,
            organic_perception_mode: "enhanced".to_string(),
            confidence_threshold: 0.7,
            max_learning_samples: 10000,
            adaptation_sensitivity: 0.8,
        }
    }
}

/// Combined intelligence analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceAnalysis {
    pub perception_result: PerceptionResult,
    pub learned_patterns: Vec<SuccessPattern>,
    pub adaptation_suggestions: Vec<AdaptationStrategy>,
    pub decision: Decision,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Intelligence-driven action recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    pub action_type: String,
    pub target_selector: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub expected_outcome: String,
    pub alternative_actions: Vec<AlternativeAction>,
    pub risk_assessment: RiskAssessment,
}

/// Alternative action if primary fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeAction {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub reason: String,
}

/// Risk assessment for actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: String, // "low", "medium", "high"
    pub potential_issues: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub success_probability: f64,
}

impl IntelligenceService {
    /// Create new intelligence service
    pub fn new(config: IntelligenceConfig) -> Self {
        info!(
            "Initializing Intelligence Service with mode: {}",
            config.organic_perception_mode
        );

        Self {
            organic_perception: Arc::new(RwLock::new(OrganicPerceptionEngine::new())),
            learning_engine: Arc::new(RwLock::new(LearningEngine::new())),
            adaptation_manager: Arc::new(RwLock::new(AdaptationManager::new())),
            pattern_recognizer: Arc::new(RwLock::new(PatternRecognizer::new())),
            decision_maker: Arc::new(RwLock::new(DecisionMaker::new())),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(IntelligenceConfig::default())
    }

    /// Perform comprehensive intelligence analysis of a page/situation
    pub async fn analyze_situation(
        &self,
        page_context: &PageContext,
        user_intent: &str,
        browser: &crate::browser::Browser,
    ) -> Result<IntelligenceAnalysis> {
        info!(
            "Starting comprehensive intelligence analysis for intent: {}",
            user_intent
        );

        // 1. Organic perception analysis
        let perception_result = {
            let mut perception = self.organic_perception.write().await;
            perception
                .analyze_page_deeply(page_context, browser)
                .await?
        };

        // 2. Pattern recognition
        let learned_patterns = if self.config.pattern_recognition_enabled {
            let pattern_recognizer = self.pattern_recognizer.read().await;
            pattern_recognizer
                .find_relevant_patterns(user_intent, &perception_result)
                .await
        } else {
            Vec::new()
        };

        // 3. Adaptation analysis
        let adaptation_suggestions = if self.config.adaptation_enabled {
            let adaptation_manager = self.adaptation_manager.read().await;
            adaptation_manager
                .suggest_adaptations(page_context, &learned_patterns)
                .await?
        } else {
            Vec::new()
        };

        // 4. Decision making
        let decision = {
            let decision_maker = self.decision_maker.read().await;
            decision_maker
                .make_decision(
                    user_intent,
                    &perception_result,
                    &learned_patterns,
                    &adaptation_suggestions,
                )
                .await?
        };

        let confidence = decision.confidence.value;
        let reasoning = format!(
            "Analysis based on {} elements perceived, {} patterns matched, {} adaptations suggested",
            perception_result.elements.len(),
            learned_patterns.len(),
            adaptation_suggestions.len()
        );

        debug!(
            "Intelligence analysis completed with confidence: {:.2}",
            confidence
        );

        Ok(IntelligenceAnalysis {
            perception_result,
            learned_patterns,
            adaptation_suggestions,
            decision,
            confidence,
            reasoning,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get intelligent action recommendation
    pub async fn recommend_action(
        &self,
        analysis: &IntelligenceAnalysis,
    ) -> Result<ActionRecommendation> {
        debug!("Generating action recommendation from intelligence analysis");

        let decision = &analysis.decision;
        let confidence = analysis.confidence;

        // Extract primary action from decision
        let action_type = decision.action_type.clone();
        let target_selector = decision.target_element.clone();
        let parameters = decision.parameters.clone();

        // Generate alternatives based on patterns and adaptations
        let mut alternative_actions = Vec::new();

        // Add alternatives from learned patterns
        for pattern in &analysis.learned_patterns {
            if pattern.confidence > 0.6 && !pattern.action_sequence.is_empty() {
                let alt_action = &pattern.action_sequence[0];
                alternative_actions.push(AlternativeAction {
                    action_type: alt_action.action_type.clone(),
                    parameters: alt_action.parameters.clone(),
                    confidence: pattern.confidence,
                    reason: format!("Based on successful pattern: {}", pattern.name),
                });
            }
        }

        // Add alternatives from adaptation suggestions
        for adaptation in &analysis.adaptation_suggestions {
            if let Some(fallback) = &adaptation.fallback_strategy {
                alternative_actions.push(AlternativeAction {
                    action_type: fallback.clone(),
                    parameters: HashMap::new(),
                    confidence: adaptation.confidence,
                    reason: "Adaptation fallback strategy".to_string(),
                });
            }
        }

        // Risk assessment
        let risk_assessment = self
            .assess_action_risk(&action_type, confidence, &analysis.perception_result)
            .await;

        Ok(ActionRecommendation {
            action_type,
            target_selector,
            parameters,
            confidence,
            expected_outcome: decision.expected_outcome.clone(),
            alternative_actions,
            risk_assessment,
        })
    }

    /// Learn from action results
    pub async fn learn_from_result(
        &self,
        action_recommendation: &ActionRecommendation,
        actual_result: &str,
        success: bool,
        execution_time_ms: u64,
    ) -> Result<()> {
        if !self.config.learning_enabled {
            return Ok(());
        }

        info!(
            "Learning from action result: success={}, time={}ms",
            success, execution_time_ms
        );

        let learning_data = LearningData {
            action_type: action_recommendation.action_type.clone(),
            parameters: action_recommendation.parameters.clone(),
            expected_outcome: action_recommendation.expected_outcome.clone(),
            actual_outcome: actual_result.to_string(),
            success,
            execution_time_ms,
            confidence: action_recommendation.confidence,
            timestamp: chrono::Utc::now(),
        };

        // Update learning engine
        {
            let mut learning_engine = self.learning_engine.write().await;
            learning_engine.record_learning_data(learning_data).await?;
        }

        // Update pattern recognizer
        if success {
            let mut pattern_recognizer = self.pattern_recognizer.write().await;
            pattern_recognizer
                .reinforce_successful_pattern(&action_recommendation.action_type)
                .await;
        }

        Ok(())
    }

    /// Assess risk of performing an action
    async fn assess_action_risk(
        &self,
        action_type: &str,
        confidence: f64,
        perception_result: &PerceptionResult,
    ) -> RiskAssessment {
        let mut potential_issues = Vec::new();
        let mut mitigation_strategies = Vec::new();

        // Base risk on confidence
        let risk_level = if confidence >= 0.9 {
            "low"
        } else if confidence >= 0.7 {
            "medium"
        } else {
            "high"
        };

        // Action-specific risks
        match action_type {
            "click" => {
                if perception_result.dynamic_elements > 5 {
                    potential_issues
                        .push("Page has many dynamic elements, target may move".to_string());
                    mitigation_strategies
                        .push("Wait for page stability before clicking".to_string());
                }
            }
            "type" => {
                potential_issues.push("Input validation may reject text".to_string());
                mitigation_strategies.push("Validate input format before typing".to_string());
            }
            "navigate" => {
                potential_issues.push("Navigation may timeout or fail".to_string());
                mitigation_strategies.push("Set appropriate timeout and retry logic".to_string());
            }
            _ => {}
        }

        // General risks based on page complexity
        if perception_result.page_complexity > 0.8 {
            potential_issues.push("High page complexity increases failure risk".to_string());
            mitigation_strategies.push("Use multiple fallback selectors".to_string());
        }

        let success_probability =
            (confidence * 0.7) + (0.3 * (1.0 - perception_result.page_complexity));

        RiskAssessment {
            risk_level: risk_level.to_string(),
            potential_issues,
            mitigation_strategies,
            success_probability,
        }
    }

    /// Get intelligence service statistics
    pub async fn get_statistics(&self) -> Result<IntelligenceStatistics> {
        let learning_stats = {
            let learning_engine = self.learning_engine.read().await;
            learning_engine.get_statistics().await
        };

        let pattern_stats = {
            let pattern_recognizer = self.pattern_recognizer.read().await;
            pattern_recognizer.get_statistics().await
        };

        Ok(IntelligenceStatistics {
            learning_samples: learning_stats.total_samples,
            success_rate: learning_stats.success_rate,
            patterns_learned: pattern_stats.total_patterns,
            average_confidence: learning_stats.average_confidence,
            adaptations_applied: 0, // TODO: Track this
        })
    }

    /// Update intelligence configuration
    pub async fn update_config(&mut self, new_config: IntelligenceConfig) {
        info!("Updating intelligence configuration");
        self.config = new_config;

        // Update sub-components with new config
        // This could be expanded to pass specific config to each component
    }
}

impl Default for IntelligenceService {
    fn default() -> Self {
        Self::new(IntelligenceConfig::default())
    }
}

/// Statistics about intelligence service performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceStatistics {
    pub learning_samples: usize,
    pub success_rate: f64,
    pub patterns_learned: usize,
    pub average_confidence: f64,
    pub adaptations_applied: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_intelligence_service_creation() {
        let service = IntelligenceService::default();
        let stats = service.get_statistics().await.unwrap();

        assert_eq!(stats.learning_samples, 0);
        assert_eq!(stats.patterns_learned, 0);
    }

    #[test]
    fn test_intelligence_config() {
        let config = IntelligenceConfig::default();

        assert!(config.learning_enabled);
        assert!(config.adaptation_enabled);
        assert_eq!(config.organic_perception_mode, "enhanced");
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let service = IntelligenceService::default();

        let perception_result = PerceptionResult {
            elements: Vec::new(),
            page_complexity: 0.5,
            dynamic_elements: 3,
            confidence: 0.8,
            processing_time_ms: 100,
        };

        let risk = service
            .assess_action_risk("click", 0.8, &perception_result)
            .await;

        assert_eq!(risk.risk_level, "medium");
        assert!(risk.success_probability > 0.0);
    }
}
