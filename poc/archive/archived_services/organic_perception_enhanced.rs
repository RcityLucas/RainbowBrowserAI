//! Enhanced Organic Perception with Adaptive Memory Integration
//!
//! This module enhances the simple organic perception with persistent memory,
//! cross-session learning, and advanced pattern evolution capabilities.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::llm_service::llm_service_enhanced::{TaskType, TaskUnderstanding, TaskPlan, ActionStep, Entity};
use crate::simple_memory::{SimpleMemory, InteractionRecord, SimpleMemoryConfig};

/// Enhanced organic perception with adaptive memory
pub struct EnhancedOrganicPerception {
    simple_patterns: HashMap<String, PatternWeight>,
    success_history: HashMap<TaskType, SuccessStats>,
    intelligence_mode: IntelligenceMode,
    memory_system: Option<Arc<SimpleMemory>>,
    session_interactions: Vec<SessionInteraction>,
}

/// Pattern weight for immediate session learning
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

/// Intelligence operation mode with memory integration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntelligenceMode {
    Legacy,      // Use old hardcoded system
    Hybrid,      // Use simple patterns + memory system
    Memory,      // Use memory system as primary
    Learning,    // Full adaptive learning mode
}

/// Session interaction tracking
#[derive(Debug, Clone)]
struct SessionInteraction {
    input: String,
    classified_task: TaskType,
    confidence: f32,
    timestamp: chrono::DateTime<Utc>,
    success: Option<bool>,
}

impl EnhancedOrganicPerception {
    /// Create new enhanced perception system
    pub async fn new() -> Result<Self> {
        let mut perception = Self {
            simple_patterns: HashMap::new(),
            success_history: HashMap::new(),
            intelligence_mode: IntelligenceMode::Hybrid,
            memory_system: None,
            session_interactions: Vec::new(),
        };
        
        // Initialize simple patterns for fallback
        perception.initialize_simple_patterns();
        
        Ok(perception)
    }

    /// Create with adaptive memory system integration
    pub async fn with_memory(memory_config: SimpleMemoryConfig) -> Result<Self> {
        let memory_system = SimpleMemory::new(memory_config).await?;
        
        let mut perception = Self {
            simple_patterns: HashMap::new(),
            success_history: HashMap::new(),
            intelligence_mode: IntelligenceMode::Memory,
            memory_system: Some(Arc::new(memory_system)),
            session_interactions: Vec::new(),
        };
        
        // Initialize simple patterns for fallback
        perception.initialize_simple_patterns();
        
        info!("🧠 Enhanced Organic Perception initialized with adaptive memory");
        Ok(perception)
    }

    /// Create from environment configuration
    pub async fn from_env() -> Result<Self> {
        let mode = std::env::var("INTELLIGENCE_MODE")
            .unwrap_or_else(|_| "Hybrid".to_string());
        
        let intelligence_mode = match mode.to_lowercase().as_str() {
            "legacy" => IntelligenceMode::Legacy,
            "memory" => IntelligenceMode::Memory,
            "learning" => IntelligenceMode::Learning,
            _ => IntelligenceMode::Hybrid,
        };

        let enable_memory = std::env::var("ENABLE_ADAPTIVE_MEMORY")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        let mut perception = if enable_memory {
            Self::with_memory(SimpleMemoryConfig::default()).await?
        } else {
            Self::new().await?
        };

        perception.intelligence_mode = intelligence_mode;
        
        info!("🧠 Enhanced Organic Perception initialized with mode: {:?}", intelligence_mode);
        Ok(perception)
    }

    /// Initialize simple patterns for fallback
    fn initialize_simple_patterns(&mut self) {
        // Enhanced planning patterns
        self.add_simple_pattern("plan", 0.9, 0.85);
        self.add_simple_pattern("itinerary", 0.85, 0.85);
        self.add_simple_pattern("trip", 0.8, 0.85);
        self.add_simple_pattern("travel", 0.75, 0.85);
        self.add_simple_pattern("vacation", 0.8, 0.85);
        self.add_simple_pattern("journey", 0.7, 0.85);
        
        // Search patterns
        self.add_simple_pattern("search", 0.9, 0.8);
        self.add_simple_pattern("find", 0.85, 0.8);
        self.add_simple_pattern("look for", 0.8, 0.8);
        self.add_simple_pattern("discover", 0.7, 0.8);
        
        // Analysis patterns
        self.add_simple_pattern("analyze", 0.9, 0.75);
        self.add_simple_pattern("review", 0.8, 0.75);
        self.add_simple_pattern("evaluate", 0.8, 0.75);
        
        // Navigation patterns
        self.add_simple_pattern("navigate", 0.95, 0.95);
        self.add_simple_pattern("go to", 0.9, 0.95);
        self.add_simple_pattern("open", 0.85, 0.95);
        self.add_simple_pattern("visit", 0.8, 0.95);
        
        // Other patterns
        self.add_simple_pattern("extract", 0.9, 0.9);
        self.add_simple_pattern("screenshot", 0.95, 0.9);
        self.add_simple_pattern("monitor", 0.85, 0.7);
        self.add_simple_pattern("test", 0.7, 0.8);
        self.add_simple_pattern("report", 0.8, 0.75);
    }

    fn add_simple_pattern(&mut self, pattern: &str, weight: f32, success_rate: f32) {
        self.simple_patterns.insert(pattern.to_string(), PatternWeight {
            weight,
            usage_count: 0,
            success_rate,
        });
    }

    /// Record interaction outcome for learning
    pub async fn record_outcome(&mut self, input: &str, task_type: TaskType, confidence: f32, success: bool, execution_time_ms: u64) -> Result<()> {
        // Update session tracking
        if let Some(interaction) = self.session_interactions.iter_mut().find(|i| i.input == input) {
            interaction.success = Some(success);
        }

        // Record in memory system if available
        if let Some(ref memory) = self.memory_system {
            let interaction = InteractionRecord {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                user_input: input.to_string(),
                classified_task: task_type,
                confidence,
                execution_success: success,
                execution_time_ms,
                context_markers: self.extract_context_markers(input),
            };

            memory.record_interaction(interaction).await?;
            
            info!("📚 Recorded learning outcome: {} -> {:?} (success: {}, confidence: {:.2})", 
                  input, task_type, success, confidence);
        }

        // Update simple patterns for immediate learning
        self.update_simple_patterns_from_outcome(input, success);

        Ok(())
    }

    /// Extract context markers from input
    fn extract_context_markers(&self, input: &str) -> Vec<String> {
        let mut markers = Vec::new();
        let input_lower = input.to_lowercase();

        // Time-based markers
        if input_lower.contains("today") || input_lower.contains("now") {
            markers.push("immediate".to_string());
        }
        if input_lower.contains("tomorrow") || input_lower.contains("next") {
            markers.push("future".to_string());
        }

        // Urgency markers
        if input_lower.contains("urgent") || input_lower.contains("quickly") || input_lower.contains("asap") {
            markers.push("urgent".to_string());
        }

        // Complexity markers
        if input_lower.split_whitespace().count() > 8 {
            markers.push("complex".to_string());
        }

        // Domain markers
        if input_lower.contains("travel") || input_lower.contains("trip") {
            markers.push("travel_domain".to_string());
        }
        if input_lower.contains("business") || input_lower.contains("work") {
            markers.push("business_domain".to_string());
        }
        if input_lower.contains("personal") || input_lower.contains("family") {
            markers.push("personal_domain".to_string());
        }

        markers
    }

    /// Update simple patterns based on outcome
    fn update_simple_patterns_from_outcome(&mut self, input: &str, success: bool) {
        let input_lower = input.to_lowercase();
        
        for (pattern_text, pattern_weight) in self.simple_patterns.iter_mut() {
            if input_lower.contains(pattern_text) {
                pattern_weight.usage_count += 1;
                
                // Update success rate using exponential moving average
                let learning_rate = 0.1;
                let outcome_score = if success { 1.0 } else { 0.0 };
                pattern_weight.success_rate = pattern_weight.success_rate * (1.0 - learning_rate) + 
                                            outcome_score * learning_rate;
                
                // Adjust weight based on success
                if success {
                    pattern_weight.weight = (pattern_weight.weight * 1.02).min(1.0);
                } else {
                    pattern_weight.weight *= 0.98;
                }
            }
        }
    }

    /// Record user feedback for additional learning (simplified)
    pub async fn record_user_feedback(&mut self, input: &str, success: bool) -> Result<()> {
        // For the simplified version, we'll just record this as a learning outcome
        if let Some(interaction) = self.session_interactions.iter().find(|i| i.input == input) {
            self.record_outcome(input, interaction.classified_task, interaction.confidence, success, 0).await?;
            info!("👍 Recorded user feedback for: {} (success: {})", input, success);
        }

        Ok(())
    }

    /// Get memory-enhanced statistics
    pub async fn get_enhanced_stats(&self) -> EnhancedIntelligenceStats {
        let mut stats = EnhancedIntelligenceStats {
            mode: format!("{:?}", self.intelligence_mode),
            session_interactions: self.session_interactions.len(),
            simple_patterns: self.simple_patterns.len(),
            memory_enabled: self.memory_system.is_some(),
            memory_stats: None,
        };

        if let Some(ref memory) = self.memory_system {
            let memory_stats = memory.get_memory_stats().await;
            stats.memory_stats = Some(memory_stats);
        }

        stats
    }
}

impl TaskUnderstanding for EnhancedOrganicPerception {
    /// Intelligent intent classification with memory enhancement
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        match self.intelligence_mode {
            IntelligenceMode::Legacy => self.classify_with_legacy_fallback(input),
            IntelligenceMode::Hybrid => self.classify_with_hybrid_approach(input),
            IntelligenceMode::Memory => self.classify_with_memory_primary(input),
            IntelligenceMode::Learning => self.classify_with_full_learning(input),
        }
    }

    /// Enhanced entity extraction
    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let input_lower = input.to_lowercase();
        
        // Enhanced location detection
        let locations = ["paris", "tokyo", "new york", "london", "rome", "barcelona", "berlin", "amsterdam"];
        
        for location in &locations {
            if input_lower.contains(location) {
                entities.push(Entity {
                    entity_type: "location".to_string(),
                    value: location.to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        // Time entities
        let time_expressions = ["today", "tomorrow", "next week", "this weekend", "next month"];
        for time_expr in &time_expressions {
            if input_lower.contains(time_expr) {
                entities.push(Entity {
                    entity_type: "time".to_string(),
                    value: time_expr.to_string(),
                    confidence: 0.85,
                });
            }
        }

        Ok(entities)
    }

    /// Intelligent task decomposition with memory-informed steps
    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        match task_type {
            TaskType::Planning => self.create_memory_informed_planning_steps(input),
            TaskType::Search => self.create_memory_informed_search_steps(input),
            TaskType::Navigation => self.create_memory_informed_navigation_steps(input),
            _ => self.create_adaptive_generic_steps(input),
        }
    }

    /// Create comprehensive task plan with memory integration
    fn create_task_plan(&self, input: &str) -> Result<TaskPlan> {
        info!("🧠 Creating memory-enhanced task plan for: {}", input);
        
        let task_type = self.classify_intent(input)?;
        let entities = self.extract_entities(input)?;
        let steps = self.decompose_task(input, task_type)?;
        
        let title = format!("Adaptive {:?} Task", task_type);
        let description = "Memory-enhanced intelligent task execution with adaptive learning".to_string();
        let estimated_duration = (steps.len() as u32 * 60).max(30);
        let required_capabilities = vec!["adaptive_intelligence".to_string(), "memory_integration".to_string()];
        
        // Note: This is a simplified approach - in a real implementation,
        // we'd use interior mutability or restructure to avoid this issue
        // For now, we'll track this externally
        
        Ok(TaskPlan {
            title,
            description,
            steps,
            estimated_duration,
            required_capabilities,
        })
    }
}

// Implementation of classification methods
impl EnhancedOrganicPerception {
    /// Classify with memory as primary source
    fn classify_with_memory_primary(&self, input: &str) -> Result<TaskType> {
        // TODO: Implement async memory lookup in sync context
        // For now, fall back to hybrid approach
        self.classify_with_hybrid_approach(input)
    }

    /// Classify with full learning mode
    fn classify_with_full_learning(&self, input: &str) -> Result<TaskType> {
        // TODO: Implement advanced learning algorithms
        // For now, use hybrid approach
        self.classify_with_hybrid_approach(input)
    }

    /// Hybrid approach using simple patterns + memory when available
    fn classify_with_hybrid_approach(&self, input: &str) -> Result<TaskType> {
        let simple_result = self.classify_with_simple_patterns(input)?;
        
        // TODO: Enhance with memory system results
        // For now, return simple pattern result
        
        Ok(simple_result)
    }

    /// Classify using simple patterns
    fn classify_with_simple_patterns(&self, input: &str) -> Result<TaskType> {
        let input_lower = input.to_lowercase();
        let mut best_match = (TaskType::Unknown, 0.0f32);
        
        for (pattern, weight_info) in &self.simple_patterns {
            if input_lower.contains(pattern) {
                let score = weight_info.weight * weight_info.success_rate;
                let task_type = self.pattern_to_task_type(pattern);
                
                if score > best_match.1 {
                    best_match = (task_type, score);
                }
            }
        }
        
        info!("🧠 Enhanced classification: '{}' → {:?} (score: {:.2})", 
              input, best_match.0, best_match.1);
        
        if best_match.1 > 0.3 {
            Ok(best_match.0)
        } else {
            Ok(TaskType::Unknown)
        }
    }

    /// Legacy fallback classification
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

    /// Create memory-informed planning steps
    fn create_memory_informed_planning_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        
        if input.to_lowercase().contains("travel") || input.to_lowercase().contains("trip") {
            steps.push(ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Research destination with adaptive insights".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/search?q=travel+guide",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 2,
                action_type: "navigate".to_string(),
                description: "Search for flights with learned preferences".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/flights",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            });

            steps.push(ActionStep {
                step_number: 3,
                action_type: "navigate".to_string(),
                description: "Find accommodation based on memory patterns".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.booking.com",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            });

            steps.push(ActionStep {
                step_number: 4,
                action_type: "navigate".to_string(),
                description: "Research attractions with adaptive filtering".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.tripadvisor.com",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            });

            steps.push(ActionStep {
                step_number: 5,
                action_type: "navigate".to_string(),
                description: "Check weather with contextual awareness".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.weather.com",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            });

            steps.push(ActionStep {
                step_number: 6,
                action_type: "report".to_string(),
                description: "Generate adaptive travel plan with memory insights".to_string(),
                parameters: serde_json::json!({
                    "format": "adaptive_itinerary",
                    "include_memory_insights": true
                }),
                depends_on: None,
                optional: false,
            });
        }
        
        Ok(steps)
    }

    fn create_memory_informed_search_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: format!("Adaptive search for: {}", input),
                parameters: serde_json::json!({
                    "url": format!("https://www.google.com/search?q={}", 
                                 urlencoding::encode(input)),
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            },
            ActionStep {
                step_number: 2,
                action_type: "extract".to_string(),
                description: "Extract relevant results with learned filters".to_string(),
                parameters: serde_json::json!({
                    "selector": ".g",
                    "limit": 10,
                    "adaptive_filtering": true
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }

    fn create_memory_informed_navigation_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
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
                description: format!("Adaptive navigation to {}", url),
                parameters: serde_json::json!({
                    "url": url,
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }

    fn create_adaptive_generic_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: format!("Execute adaptive task: {}", input),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true,
                    "memory_enhanced": true
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
}

/// Enhanced intelligence statistics with memory integration
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedIntelligenceStats {
    pub mode: String,
    pub session_interactions: usize,
    pub simple_patterns: usize,
    pub memory_enabled: bool,
    pub memory_stats: Option<crate::simple_memory::SimpleMemoryStats>,
}

impl Default for EnhancedOrganicPerception {
    fn default() -> Self {
        // Note: This creates a version without memory for sync contexts
        Self {
            simple_patterns: HashMap::new(),
            success_history: HashMap::new(),
            intelligence_mode: IntelligenceMode::Hybrid,
            memory_system: None,
            session_interactions: Vec::new(),
        }
    }
}

/// Create enhanced perception from environment
pub async fn create_enhanced_perception() -> Result<EnhancedOrganicPerception> {
    EnhancedOrganicPerception::from_env().await
}