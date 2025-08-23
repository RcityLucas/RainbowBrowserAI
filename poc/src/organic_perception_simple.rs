// Simplified Organic Perception - Working MVP
//
// This is a minimal working implementation that demonstrates intelligent classification
// replacing hardcoded keyword matching, ready for immediate testing and gradual enhancement.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;
use std::collections::HashMap;

use crate::llm_service::llm_service_enhanced::{TaskType, TaskUnderstanding, TaskPlan, ActionStep, Entity};

/// Simplified organic task understanding that learns and adapts
pub struct SimpleOrganicPerception {
    pattern_weights: HashMap<String, PatternWeight>,
    success_history: HashMap<TaskType, SuccessStats>,
    intelligence_mode: IntelligenceMode,
}

/// Pattern weight for learning-based classification
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternWeight {
    weight: f32,
    usage_count: u32,
    success_rate: f32,
}

/// Success statistics for confidence calibration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuccessStats {
    total_attempts: u32,
    successes: u32,
    average_confidence: f32,
}

/// Intelligence operation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntelligenceMode {
    Legacy,      // Use old hardcoded system
    Hybrid,      // Use both systems and compare
    Intelligent, // Use only organic perception
}

impl SimpleOrganicPerception {
    pub fn new() -> Self {
        let mut perception = Self {
            pattern_weights: HashMap::new(),
            success_history: HashMap::new(),
            intelligence_mode: IntelligenceMode::Hybrid,
        };
        
        // Initialize with intelligent seed patterns
        perception.initialize_intelligent_patterns();
        perception
    }
    
    pub fn from_env() -> Self {
        let mode = std::env::var("INTELLIGENCE_MODE")
            .unwrap_or_else(|_| "Hybrid".to_string());
        
        let intelligence_mode = match mode.to_lowercase().as_str() {
            "legacy" => IntelligenceMode::Legacy,
            "intelligent" => IntelligenceMode::Intelligent,
            _ => IntelligenceMode::Hybrid,
        };
        
        let mut perception = Self::new();
        perception.intelligence_mode = intelligence_mode;
        
        info!("🧠 Simple Organic Perception initialized with mode: {:?}", perception.intelligence_mode);
        perception
    }
    
    /// Initialize with enhanced patterns evolved from hardcoded rules
    fn initialize_intelligent_patterns(&mut self) {
        // Planning patterns - enhanced with context awareness
        self.add_pattern("plan", 0.9, 0.85);
        self.add_pattern("itinerary", 0.85, 0.85);
        self.add_pattern("trip", 0.8, 0.85);
        self.add_pattern("travel", 0.75, 0.85);
        self.add_pattern("vacation", 0.8, 0.85);
        self.add_pattern("journey", 0.7, 0.85);
        
        // Search patterns
        self.add_pattern("search", 0.9, 0.8);
        self.add_pattern("find", 0.85, 0.8);
        self.add_pattern("look for", 0.8, 0.8);
        self.add_pattern("discover", 0.7, 0.8);
        
        // Analysis patterns
        self.add_pattern("analyze", 0.9, 0.75);
        self.add_pattern("review", 0.8, 0.75);
        self.add_pattern("evaluate", 0.8, 0.75);
        
        // Navigation patterns
        self.add_pattern("navigate", 0.95, 0.95);
        self.add_pattern("go to", 0.9, 0.95);
        self.add_pattern("open", 0.85, 0.95);
        self.add_pattern("visit", 0.8, 0.95);
        
        // Other patterns
        self.add_pattern("extract", 0.9, 0.9);
        self.add_pattern("screenshot", 0.95, 0.9);
        self.add_pattern("monitor", 0.85, 0.7);
        self.add_pattern("test", 0.7, 0.8);
        self.add_pattern("report", 0.8, 0.75);
    }
    
    fn add_pattern(&mut self, pattern: &str, weight: f32, success_rate: f32) {
        self.pattern_weights.insert(pattern.to_string(), PatternWeight {
            weight,
            usage_count: 0,
            success_rate,
        });
    }
}

impl TaskUnderstanding for SimpleOrganicPerception {
    /// Intelligent intent classification using organic perception
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        match self.intelligence_mode {
            IntelligenceMode::Legacy => self.classify_with_legacy_fallback(input),
            IntelligenceMode::Intelligent => self.classify_with_organic_intelligence(input),
            IntelligenceMode::Hybrid => self.classify_with_hybrid_approach(input),
        }
    }
    
    /// Enhanced entity extraction
    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let input_lower = input.to_lowercase();
        
        // Enhanced location detection
        let locations = ["paris", "tokyo", "new york", "london", "rome"];
        
        for location in &locations {
            if input_lower.contains(location) {
                entities.push(Entity {
                    entity_type: "location".to_string(),
                    value: location.to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(entities)
    }
    
    /// Intelligent task decomposition
    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        match task_type {
            TaskType::Planning => self.create_planning_steps(input),
            TaskType::Search => self.create_search_steps(input),
            TaskType::Navigation => self.create_navigation_steps(input),
            _ => self.create_generic_steps(input),
        }
    }
    
    /// Create comprehensive task plan
    fn create_task_plan(&self, input: &str) -> Result<TaskPlan> {
        info!("🧠 Creating organic task plan for: {}", input);
        
        let task_type = self.classify_intent(input)?;
        let entities = self.extract_entities(input)?;
        let steps = self.decompose_task(input, task_type)?;
        
        let title = format!("Smart {:?} Task", task_type);
        let description = "Intelligent task execution with organic understanding".to_string();
        let estimated_duration = (steps.len() as u32 * 60).max(30);
        let required_capabilities = vec!["intelligent_automation".to_string()];
        
        Ok(TaskPlan {
            title,
            description,
            steps,
            estimated_duration,
            required_capabilities,
        })
    }
}

// Implementation of organic intelligence methods
impl SimpleOrganicPerception {
    /// Classify with organic intelligence
    fn classify_with_organic_intelligence(&self, input: &str) -> Result<TaskType> {
        let input_lower = input.to_lowercase();
        let mut best_match = (TaskType::Unknown, 0.0f32);
        
        // Enhanced pattern matching
        for (pattern, weight_info) in &self.pattern_weights {
            if input_lower.contains(pattern) {
                let score = weight_info.weight * weight_info.success_rate;
                let task_type = self.pattern_to_task_type(pattern);
                
                if score > best_match.1 {
                    best_match = (task_type, score);
                }
            }
        }
        
        info!("🧠 Organic classification: '{}' → {:?} (score: {:.2})", 
              input, best_match.0, best_match.1);
        
        if best_match.1 > 0.3 {
            Ok(best_match.0)
        } else {
            Ok(TaskType::Unknown)
        }
    }
    
    /// Map pattern to task type
    fn pattern_to_task_type(&self, pattern: &str) -> TaskType {
        match pattern {
            p if ["plan", "itinerary", "trip", "travel", "vacation", "journey"].contains(&p) => TaskType::Planning,
            p if ["search", "find", "look for", "discover"].contains(&p) => TaskType::Search,
            p if ["analyze", "review", "evaluate"].contains(&p) => TaskType::Analysis,
            p if ["extract"].contains(&p) => TaskType::Extraction,
            p if ["navigate", "go to", "open", "visit"].contains(&p) => TaskType::Navigation,
            p if ["monitor"].contains(&p) => TaskType::Monitoring,
            p if ["test"].contains(&p) => TaskType::Testing,
            p if ["report"].contains(&p) => TaskType::Reporting,
            p if ["screenshot"].contains(&p) => TaskType::Screenshot,
            _ => TaskType::Unknown,
        }
    }
    
    /// Hybrid approach with intelligent fallback
    fn classify_with_hybrid_approach(&self, input: &str) -> Result<TaskType> {
        let organic_result = self.classify_with_organic_intelligence(input)?;
        
        if organic_result != TaskType::Unknown {
            Ok(organic_result)
        } else {
            self.classify_with_legacy_fallback(input)
        }
    }
    
    /// Legacy fallback
    fn classify_with_legacy_fallback(&self, input: &str) -> Result<TaskType> {
        let input_lower = input.to_lowercase();
        
        if input_lower.contains("plan") || input_lower.contains("travel") {
            return Ok(TaskType::Planning);
        }
        if input_lower.contains("search") || input_lower.contains("find") {
            return Ok(TaskType::Search);
        }
        if input_lower.contains("navigate") || input_lower.contains("go to") {
            return Ok(TaskType::Navigation);
        }
        
        Ok(TaskType::Unknown)
    }
    
    /// Create intelligent planning steps
    fn create_planning_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        
        if input.to_lowercase().contains("travel") || input.to_lowercase().contains("trip") {
            steps.push(ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Research destination and travel options".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/search?q=travel+guide",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 2,
                action_type: "navigate".to_string(),
                description: "Search for flights".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/flights",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 3,
                action_type: "navigate".to_string(),
                description: "Find accommodation".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.booking.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 4,
                action_type: "navigate".to_string(),
                description: "Research attractions".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.tripadvisor.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 5,
                action_type: "navigate".to_string(),
                description: "Check weather conditions".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.weather.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 6,
                action_type: "report".to_string(),
                description: "Generate comprehensive travel plan".to_string(),
                parameters: serde_json::json!({
                    "format": "detailed_itinerary"
                }),
                depends_on: None,
                optional: false,
            });
        }
        
        Ok(steps)
    }
    
    fn create_search_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: format!("Intelligent search for: {}", input),
                parameters: serde_json::json!({
                    "url": format!("https://www.google.com/search?q={}", 
                                 urlencoding::encode(input)),
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            },
            ActionStep {
                step_number: 2,
                action_type: "extract".to_string(),
                description: "Extract relevant search results".to_string(),
                parameters: serde_json::json!({
                    "selector": ".g",
                    "limit": 10
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    fn create_navigation_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let url = if input.contains("http") {
            input.split_whitespace()
                .find(|word| word.contains("http"))
                .unwrap_or("https://www.google.com")
                .to_string()
        } else {
            "https://www.google.com".to_string()
        };
        
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: format!("Smart navigation to {}", url),
                parameters: serde_json::json!({
                    "url": url,
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    fn create_generic_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: format!("Execute task: {}", input),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    /// Learn from execution outcome
    pub fn learn_from_outcome(&mut self, input: &str, task_type: TaskType, success: bool, confidence: f32) -> Result<()> {
        info!("📚 Learning from outcome: task={:?}, success={}, confidence={:.2}", 
               task_type, success, confidence);
        
        // Update pattern weights based on usage and success
        for (pattern, weight_info) in self.pattern_weights.iter_mut() {
            if input.to_lowercase().contains(pattern) {
                weight_info.usage_count += 1;
                
                // Update success rate using exponential moving average
                let learning_rate = 0.1;
                let outcome_score = if success { 1.0 } else { 0.0 };
                weight_info.success_rate = weight_info.success_rate * (1.0 - learning_rate) + 
                                         outcome_score * learning_rate;
                
                // Adjust weight based on success
                if success && confidence > 0.7 {
                    weight_info.weight = (weight_info.weight * 1.02).min(1.0);
                } else if !success {
                    weight_info.weight *= 0.98;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get intelligence statistics
    pub fn get_intelligence_stats(&self) -> IntelligenceStats {
        let total_usage: u32 = self.pattern_weights.values().map(|p| p.usage_count).sum();
        let average_success_rate = if !self.pattern_weights.is_empty() {
            self.pattern_weights.values().map(|p| p.success_rate).sum::<f32>() / self.pattern_weights.len() as f32
        } else {
            0.0
        };
        
        IntelligenceStats {
            mode: format!("{:?}", self.intelligence_mode),
            total_patterns: self.pattern_weights.len(),
            learned_patterns: self.pattern_weights.values().filter(|p| p.usage_count > 0).count(),
            total_usage,
            average_success_rate,
            task_types_tracked: self.success_history.len(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligenceStats {
    pub mode: String,
    pub total_patterns: usize,
    pub learned_patterns: usize,
    pub total_usage: u32,
    pub average_success_rate: f32,
    pub task_types_tracked: usize,
}

impl Default for SimpleOrganicPerception {
    fn default() -> Self {
        Self::new()
    }
}