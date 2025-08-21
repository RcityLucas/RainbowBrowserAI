// Advanced Automation Tools Module
// Phase 3 Implementation - Tools 14-17

// Re-export all advanced automation tools (will be implemented progressively)
pub use smart_actions::*;       // Week 9 - COMPLETE
pub use workflow_orchestrator::*;  // Week 10 - COMPLETE
pub use visual_validator::*;    // Week 11 - COMPLETE
pub use performance_monitor::*; // Week 12 - COMPLETE

// Module declarations
pub mod smart_actions;          // Week 9 - Intelligent form filling and interactions
pub mod workflow_orchestrator; // Week 10 - Complex automation sequences
pub mod visual_validator;      // Week 11 - UI testing and visual validation
pub mod performance_monitor;   // Week 12 - Performance metrics and monitoring

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ====================== Phase 3 Advanced Automation Framework ======================

/// Automation action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    /// Click interaction
    Click,
    /// Text input
    Type,
    /// Select option from dropdown
    Select,
    /// Form submission
    Submit,
    /// Navigation action
    Navigate,
    /// Wait condition
    Wait,
    /// Validation check
    Validate,
    /// Screenshot capture
    Screenshot,
    /// Custom JavaScript execution
    JavaScript,
    /// Complex workflow step
    Workflow,
}

/// Automation context for intelligent decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationContext {
    /// Current page URL
    pub current_url: String,
    
    /// Page title
    pub page_title: String,
    
    /// Previously executed actions
    pub action_history: Vec<ExecutedAction>,
    
    /// Form data collected from previous steps
    pub collected_data: HashMap<String, serde_json::Value>,
    
    /// User preferences and settings
    pub user_preferences: HashMap<String, String>,
    
    /// Session timing information
    pub session_start: DateTime<Utc>,
    pub last_action_time: DateTime<Utc>,
}

/// Record of an executed action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedAction {
    /// Action type performed
    pub action_type: ActionType,
    
    /// Target element selector
    pub target_selector: Option<String>,
    
    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Execution duration
    pub duration_ms: u64,
    
    /// Whether action was successful
    pub success: bool,
    
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Intelligent action suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSuggestion {
    /// Suggested action type
    pub action_type: ActionType,
    
    /// Target element information
    pub target: ElementTarget,
    
    /// Suggested parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    
    /// Reasoning for the suggestion
    pub reasoning: String,
    
    /// Alternative suggestions
    pub alternatives: Vec<ActionSuggestion>,
}

/// Target element specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementTarget {
    /// CSS selector for the element
    pub selector: String,
    
    /// Alternative selectors (fallbacks)
    pub fallback_selectors: Vec<String>,
    
    /// Expected element properties
    pub expected_properties: HashMap<String, String>,
    
    /// Element context information
    pub context: Option<String>,
}

/// Automation execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationResult<T> {
    /// The result data
    pub data: T,
    
    /// Execution context
    pub context: AutomationContext,
    
    /// Actions performed
    pub actions_performed: Vec<ExecutedAction>,
    
    /// Success status
    pub success: bool,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Performance metrics
    pub metrics: AutomationMetrics,
}

/// Performance metrics for automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationMetrics {
    /// Total execution time
    pub total_duration_ms: u64,
    
    /// Number of actions performed
    pub actions_count: usize,
    
    /// Number of failed actions
    pub failed_actions: usize,
    
    /// Page load times
    pub page_loads: Vec<u64>,
    
    /// Element wait times
    pub wait_times: Vec<u64>,
    
    /// Memory usage (if available)
    pub memory_usage_mb: Option<f64>,
}

impl Default for AutomationContext {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            current_url: String::new(),
            page_title: String::new(),
            action_history: Vec::new(),
            collected_data: HashMap::new(),
            user_preferences: HashMap::new(),
            session_start: now,
            last_action_time: now,
        }
    }
}

impl Default for AutomationMetrics {
    fn default() -> Self {
        Self {
            total_duration_ms: 0,
            actions_count: 0,
            failed_actions: 0,
            page_loads: Vec::new(),
            wait_times: Vec::new(),
            memory_usage_mb: None,
        }
    }
}

impl<T> AutomationResult<T> {
    /// Create a successful automation result
    pub fn success(data: T, context: AutomationContext, actions: Vec<ExecutedAction>, metrics: AutomationMetrics) -> Self {
        Self {
            data,
            context,
            actions_performed: actions,
            success: true,
            error_message: None,
            metrics,
        }
    }
    
    /// Create a failed automation result
    pub fn failure(data: T, context: AutomationContext, actions: Vec<ExecutedAction>, error: String, metrics: AutomationMetrics) -> Self {
        Self {
            data,
            context,
            actions_performed: actions,
            success: false,
            error_message: Some(error),
            metrics,
        }
    }
}

/// Utility functions for automation
pub mod automation_utils {
    use super::*;
    
    /// Generate a unique element selector with fallbacks
    pub fn generate_robust_selector(element_info: &HashMap<String, String>) -> ElementTarget {
        let mut selectors = Vec::new();
        let mut properties = HashMap::new();
        
        // Primary selector (ID if available)
        if let Some(id) = element_info.get("id") {
            selectors.push(format!("#{}", id));
            properties.insert("id".to_string(), id.clone());
        }
        
        // Secondary selector (class-based)
        if let Some(classes) = element_info.get("class") {
            let class_selector = classes.split_whitespace()
                .map(|c| format!(".{}", c))
                .collect::<Vec<_>>()
                .join("");
            if !class_selector.is_empty() {
                selectors.push(class_selector);
            }
        }
        
        // Tertiary selector (attribute-based)
        if let Some(name) = element_info.get("name") {
            selectors.push(format!("[name='{}']", name));
            properties.insert("name".to_string(), name.clone());
        }
        
        // Quaternary selector (tag + text content)
        if let Some(tag) = element_info.get("tagName") {
            if let Some(text) = element_info.get("textContent") {
                if !text.trim().is_empty() && text.len() < 50 {
                    selectors.push(format!("{}:contains('{}')", tag.to_lowercase(), text.trim()));
                }
            }
        }
        
        ElementTarget {
            selector: selectors.first().cloned().unwrap_or_else(|| "*".to_string()),
            fallback_selectors: selectors.into_iter().skip(1).collect(),
            expected_properties: properties,
            context: element_info.get("context").cloned(),
        }
    }
    
    /// Calculate confidence score for an action suggestion
    pub fn calculate_confidence_score(factors: &HashMap<String, f64>) -> f64 {
        let weights = [
            ("element_visibility", 0.3),
            ("selector_uniqueness", 0.2),
            ("context_relevance", 0.2),
            ("historical_success", 0.15),
            ("user_preference", 0.15),
        ];
        
        let mut score = 0.0;
        let mut total_weight = 0.0;
        
        for (factor, weight) in weights.iter() {
            if let Some(value) = factors.get(*factor) {
                score += value * weight;
                total_weight += weight;
            }
        }
        
        if total_weight > 0.0 {
            (score / total_weight).min(1.0).max(0.0)
        } else {
            0.5 // Default confidence
        }
    }
    
    /// Determine best action type based on element properties
    pub fn suggest_action_type(element_info: &HashMap<String, String>) -> ActionType {
        let tag_name = element_info.get("tagName").map(|s| s.to_lowercase()).unwrap_or_default();
        let input_type = element_info.get("type").map(|s| s.to_lowercase()).unwrap_or_default();
        
        match tag_name.as_str() {
            "button" => ActionType::Click,
            "a" => ActionType::Click,
            "input" => {
                match input_type.as_str() {
                    "text" | "email" | "password" | "search" | "tel" | "url" => ActionType::Type,
                    "submit" | "button" => ActionType::Click,
                    "checkbox" | "radio" => ActionType::Click,
                    "file" => ActionType::Click,
                    _ => ActionType::Click,
                }
            },
            "select" => ActionType::Select,
            "textarea" => ActionType::Type,
            "form" => ActionType::Submit,
            _ => {
                // Check for clickable indicators
                if element_info.get("onclick").is_some() || 
                   element_info.get("class").map(|c| c.contains("btn") || c.contains("button")).unwrap_or(false) {
                    ActionType::Click
                } else {
                    ActionType::Click // Default fallback
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::automation_utils::*;
    
    #[test]
    fn test_generate_robust_selector() {
        let mut element_info = HashMap::new();
        element_info.insert("id".to_string(), "submit-btn".to_string());
        element_info.insert("class".to_string(), "btn btn-primary".to_string());
        element_info.insert("name".to_string(), "submit".to_string());
        
        let target = generate_robust_selector(&element_info);
        assert_eq!(target.selector, "#submit-btn");
        assert!(target.fallback_selectors.contains(&".btn.btn-primary".to_string()));
    }
    
    #[test]
    fn test_confidence_calculation() {
        let mut factors = HashMap::new();
        factors.insert("element_visibility".to_string(), 1.0);
        factors.insert("selector_uniqueness".to_string(), 0.8);
        factors.insert("context_relevance".to_string(), 0.9);
        
        let confidence = calculate_confidence_score(&factors);
        assert!(confidence > 0.8 && confidence <= 1.0);
    }
    
    #[test]
    fn test_action_type_suggestion() {
        let mut button_info = HashMap::new();
        button_info.insert("tagName".to_string(), "button".to_string());
        assert_eq!(suggest_action_type(&button_info), ActionType::Click);
        
        let mut text_input_info = HashMap::new();
        text_input_info.insert("tagName".to_string(), "input".to_string());
        text_input_info.insert("type".to_string(), "text".to_string());
        assert_eq!(suggest_action_type(&text_input_info), ActionType::Type);
    }
}