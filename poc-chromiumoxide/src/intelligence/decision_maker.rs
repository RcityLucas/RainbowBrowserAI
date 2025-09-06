// Decision Maker Module
// Makes intelligent decisions based on perception, patterns, and context

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Makes intelligent decisions for browser automation
#[derive(Debug)]
pub struct DecisionMaker {
    decision_history: Vec<Decision>,
    confidence_threshold: f64,
}

/// A decision made by the intelligence system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub action_type: String,
    pub target_element: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: Confidence,
    pub reasoning: String,
    pub expected_outcome: String,
    pub alternatives: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Confidence level with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confidence {
    pub value: f64,
    pub factors: HashMap<String, f64>,
    pub uncertainty_sources: Vec<String>,
}

/// Context for making decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub user_intent: String,
    pub page_state: HashMap<String, serde_json::Value>,
    pub available_actions: Vec<String>,
    pub constraints: Vec<String>,
}

impl DecisionMaker {
    pub fn new() -> Self {
        Self {
            decision_history: Vec::new(),
            confidence_threshold: 0.7,
        }
    }
    
    pub async fn make_decision(
        &self,
        user_intent: &str,
        perception_result: &super::organic_perception::PerceptionResult,
        _learned_patterns: &[super::pattern_recognition::SuccessPattern],
        _adaptations: &[super::adaptation_manager::AdaptationStrategy],
    ) -> Result<Decision> {
        // Simple decision making logic
        let action_type = self.infer_action_type(user_intent);
        let target_element = self.select_best_element(perception_result);
        
        let confidence = Confidence {
            value: perception_result.confidence * 0.8, // Adjust based on perception confidence
            factors: {
                let mut factors = HashMap::new();
                factors.insert("perception".to_string(), perception_result.confidence);
                factors.insert("element_quality".to_string(), 0.8);
                factors
            },
            uncertainty_sources: vec!["Limited pattern data".to_string()],
        };
        
        Ok(Decision {
            action_type,
            target_element,
            parameters: HashMap::new(),
            confidence,
            reasoning: "Based on perception analysis and user intent".to_string(),
            expected_outcome: "Action should complete successfully".to_string(),
            alternatives: vec!["retry".to_string(), "fallback_selector".to_string()],
            timestamp: chrono::Utc::now(),
        })
    }
    
    fn infer_action_type(&self, intent: &str) -> String {
        let intent_lower = intent.to_lowercase();
        if intent_lower.contains("click") {
            "click".to_string()
        } else if intent_lower.contains("type") || intent_lower.contains("enter") {
            "type".to_string()
        } else if intent_lower.contains("navigate") || intent_lower.contains("go to") {
            "navigate".to_string()
        } else {
            "generic_action".to_string()
        }
    }
    
    fn select_best_element(&self, perception_result: &super::organic_perception::PerceptionResult) -> Option<String> {
        perception_result.elements.first().map(|e| e.selector.clone())
    }
}

impl Default for DecisionMaker {
    fn default() -> Self {
        Self::new()
    }
}