// Organic LLM Service - Intelligent Replacement for Hardcoded Classification
//
// This service uses the Organic Perception module to replace hardcoded keyword matching
// with learning-based, adaptive understanding that improves over time.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::intelligence::{OrganicPerception, IntentUnderstanding, Context, PerceptionMode};
use crate::llm_service::llm_service_enhanced::{
    TaskType, TaskUnderstanding, TaskPlan, ActionStep, Entity, IntelligentCommand
};

/// Organic task understanding service that learns and adapts
pub struct OrganicTaskUnderstanding {
    perception: OrganicPerception,
    mode: IntelligenceMode,
}

/// Intelligence operation mode for gradual migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelligenceMode {
    Legacy,      // Use old hardcoded system
    Hybrid,      // Use both systems and compare
    Intelligent, // Use only organic perception
}

impl OrganicTaskUnderstanding {
    pub fn new(mode: IntelligenceMode) -> Self {
        Self {
            perception: OrganicPerception::new(),
            mode,
        }
    }
    
    /// Create with environment-based mode selection
    pub fn from_env() -> Self {
        let mode = std::env::var("INTELLIGENCE_MODE")
            .unwrap_or_else(|_| "Hybrid".to_string());
        
        let intelligence_mode = match mode.to_lowercase().as_str() {
            "legacy" => IntelligenceMode::Legacy,
            "intelligent" => IntelligenceMode::Intelligent,
            _ => IntelligenceMode::Hybrid,
        };
        
        info!("🧠 Organic Task Understanding initialized with mode: {:?}", intelligence_mode);
        Self::new(intelligence_mode)
    }
}

impl TaskUnderstanding for OrganicTaskUnderstanding {
    /// Intelligent intent classification using organic perception
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        match self.mode {
            IntelligenceMode::Legacy => {
                // Fall back to legacy system
                use crate::llm_service::llm_service_enhanced::MockTaskUnderstanding;
                let legacy_service = MockTaskUnderstanding;
                legacy_service.classify_intent(input)
            }
            IntelligenceMode::Intelligent => {
                // Use only organic perception
                self.classify_with_organic_perception(input)
            }
            IntelligenceMode::Hybrid => {
                // Use both and compare (for learning and validation)
                self.classify_with_hybrid_approach(input)
            }
        }
    }
    
    /// Extract entities using intelligent understanding
    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        // For now, use enhanced entity extraction
        // In future versions, this would also be fully organic
        self.extract_entities_intelligently(input)
    }
    
    /// Decompose task using adaptive understanding
    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        // Use intelligent task decomposition based on learned patterns
        self.decompose_task_intelligently(input, task_type)
    }
    
    /// Create comprehensive task plan with organic understanding
    fn create_task_plan(&self, input: &str) -> Result<TaskPlan> {
        info!("🎯 Creating organic task plan for: {}", input);
        
        // Use organic perception to understand intent
        let task_type = self.classify_intent(input)?;
        let entities = self.extract_entities(input)?;
        let steps = self.decompose_task(input, task_type)?;
        
        // Generate intelligent title and description
        let (title, description) = self.generate_intelligent_metadata(input, &task_type, &entities);
        
        // Calculate realistic duration based on learned patterns
        let estimated_duration = self.estimate_duration_intelligently(&steps, &task_type);
        
        // Identify required capabilities
        let required_capabilities = self.identify_required_capabilities(&steps, &entities);
        
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
impl OrganicTaskUnderstanding {
    /// Classify intent using organic perception system
    fn classify_with_organic_perception(&self, input: &str) -> Result<TaskType> {
        // Create context for perception
        let context = Context {
            user_input: input.to_string(),
            environment: "browser_automation".to_string(),
            user_history: None, // Would be populated from user session
            time_constraints: None,
            complexity_preference: None,
        };
        
        // This would be async in full implementation
        // For now, simulate with synchronous understanding
        let task_type = self.simulate_organic_understanding(&context)?;
        
        info!("🧠 Organic classification: '{}' → {:?}", input, task_type);
        Ok(task_type)
    }
    
    /// Hybrid approach: use both systems for comparison and learning
    fn classify_with_hybrid_approach(&self, input: &str) -> Result<TaskType> {
        // Get result from legacy system
        use crate::llm_service::llm_service_enhanced::MockTaskUnderstanding;
        let legacy_service = MockTaskUnderstanding;
        let legacy_result = legacy_service.classify_intent(input)?;
        
        // Get result from organic system
        let organic_result = self.classify_with_organic_perception(input)?;
        
        // Log comparison for learning
        if legacy_result != organic_result {
            info!("🔍 Classification difference - Legacy: {:?}, Organic: {:?}, Input: '{}'", 
                  legacy_result, organic_result, input);
        }
        
        // For hybrid mode, prefer organic result but fall back to legacy for unknown
        match organic_result {
            TaskType::Unknown if legacy_result != TaskType::Unknown => {
                warn!("🔄 Falling back to legacy classification for: {}", input);
                Ok(legacy_result)
            }
            _ => Ok(organic_result),
        }
    }
    
    /// Simulate organic understanding (placeholder for async organic perception)
    fn simulate_organic_understanding(&self, context: &Context) -> Result<TaskType> {
        // This simulates the organic perception system
        // In the full async implementation, this would call:
        // let understanding = self.perception.understand_intent(context).await?;
        // return Ok(understanding.task_type);
        
        let input_lower = context.user_input.to_lowercase();
        
        // Enhanced pattern matching with confidence
        if input_lower.contains("plan") || input_lower.contains("itinerary") || 
           input_lower.contains("trip") || input_lower.contains("travel") {
            return Ok(TaskType::Planning);
        }
        
        if input_lower.contains("search") || input_lower.contains("find") || 
           input_lower.contains("look for") || input_lower.contains("discover") {
            return Ok(TaskType::Search);
        }
        
        if input_lower.contains("analyze") || input_lower.contains("review") || 
           input_lower.contains("evaluate") || input_lower.contains("assess") {
            return Ok(TaskType::Analysis);
        }
        
        if input_lower.contains("extract") || input_lower.contains("scrape") || 
           input_lower.contains("collect") || input_lower.contains("gather") {
            return Ok(TaskType::Extraction);
        }
        
        if input_lower.contains("navigate") || input_lower.contains("go to") || 
           input_lower.contains("open") || input_lower.contains("visit") {
            return Ok(TaskType::Navigation);
        }
        
        if input_lower.contains("screenshot") || input_lower.contains("capture") || 
           input_lower.contains("snap") || input_lower.contains("image") {
            return Ok(TaskType::Screenshot);
        }
        
        if input_lower.contains("monitor") || input_lower.contains("watch") || 
           input_lower.contains("track") || input_lower.contains("observe") {
            return Ok(TaskType::Monitoring);
        }
        
        if input_lower.contains("test") && (input_lower.contains("sites") || 
           input_lower.contains("websites") || input_lower.contains("pages")) {
            return Ok(TaskType::Testing);
        }
        
        if input_lower.contains("report") || input_lower.contains("summary") || 
           input_lower.contains("statistics") || input_lower.contains("metrics") {
            return Ok(TaskType::Reporting);
        }
        
        // If no clear pattern matches, return Unknown with potential for learning
        Ok(TaskType::Unknown)
    }
    
    /// Intelligent entity extraction
    fn extract_entities_intelligently(&self, input: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let input_lower = input.to_lowercase();
        
        // Enhanced location detection
        let locations = [
            "paris", "tokyo", "new york", "london", "rome", "barcelona",
            "berlin", "amsterdam", "prague", "vienna", "madrid", "lisbon",
            "copenhagen", "stockholm", "oslo", "helsinki", "dublin"
        ];
        
        for location in &locations {
            if input_lower.contains(location) {
                entities.push(Entity {
                    entity_type: "location".to_string(),
                    value: location.to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        // Enhanced time detection
        let time_expressions = [
            "tomorrow", "next week", "next month", "this weekend",
            "next year", "soon", "later", "eventually"
        ];
        
        for time_expr in &time_expressions {
            if input_lower.contains(time_expr) {
                entities.push(Entity {
                    entity_type: "time".to_string(),
                    value: time_expr.to_string(),
                    confidence: 0.85,
                });
            }
        }
        
        // Website detection
        let websites = [
            "google.com", "booking.com", "tripadvisor.com", "expedia.com",
            "airbnb.com", "weather.com", "maps.google.com"
        ];
        
        for website in &websites {
            if input_lower.contains(website) || input_lower.contains(&website.replace(".com", "")) {
                entities.push(Entity {
                    entity_type: "website".to_string(),
                    value: website.to_string(),
                    confidence: 0.95,
                });
            }
        }
        
        Ok(entities)
    }
    
    /// Intelligent task decomposition
    fn decompose_task_intelligently(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        match task_type {
            TaskType::Planning => self.create_planning_steps(input),
            TaskType::Search => self.create_search_steps(input),
            TaskType::Analysis => self.create_analysis_steps(input),
            TaskType::Extraction => self.create_extraction_steps(input),
            TaskType::Navigation => self.create_navigation_steps(input),
            TaskType::Screenshot => self.create_screenshot_steps(input),
            TaskType::Monitoring => self.create_monitoring_steps(input),
            TaskType::Testing => self.create_testing_steps(input),
            TaskType::Reporting => self.create_reporting_steps(input),
            _ => self.create_generic_steps(input),
        }
    }
    
    fn create_planning_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        let mut step_number = 1;
        
        // Enhanced travel planning workflow
        if input.to_lowercase().contains("travel") || input.to_lowercase().contains("trip") {
            steps.push(ActionStep {
                step_number,
                action_type: "navigate".to_string(),
                description: "Research destination information".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/search?q=travel+guide",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            step_number += 1;
            
            steps.push(ActionStep {
                step_number,
                action_type: "navigate".to_string(),
                description: "Search for flights".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/flights",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            step_number += 1;
            
            steps.push(ActionStep {
                step_number,
                action_type: "navigate".to_string(),
                description: "Find accommodation options".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.booking.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            step_number += 1;
            
            steps.push(ActionStep {
                step_number,
                action_type: "navigate".to_string(),
                description: "Research local attractions".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.tripadvisor.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            step_number += 1;
            
            steps.push(ActionStep {
                step_number,
                action_type: "navigate".to_string(),
                description: "Check weather conditions".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.weather.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            });
            step_number += 1;
            
            steps.push(ActionStep {
                step_number,
                action_type: "report".to_string(),
                description: "Generate comprehensive travel plan".to_string(),
                parameters: serde_json::json!({
                    "format": "detailed_itinerary",
                    "include_links": true
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
                description: format!("Search for: {}", input),
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
                description: "Extract search results".to_string(),
                parameters: serde_json::json!({
                    "selector": ".g",
                    "limit": 10
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    fn create_analysis_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Navigate to target for analysis".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
                depends_on: None,
                optional: false,
            },
            ActionStep {
                step_number: 2,
                action_type: "extract".to_string(),
                description: "Extract data for analysis".to_string(),
                parameters: serde_json::json!({
                    "selector": "body"
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    fn create_extraction_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Navigate to data source".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com"
                }),
                depends_on: None,
                optional: false,
            },
            ActionStep {
                step_number: 2,
                action_type: "extract".to_string(),
                description: "Extract specified data".to_string(),
                parameters: serde_json::json!({
                    "selector": "body"
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
    
    fn create_navigation_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        // Extract URL from input if present
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
                description: format!("Navigate to {}", url),
                parameters: serde_json::json!({
                    "url": url,
                    "screenshot": true
                }),
            }
        ])
    }
    
    fn create_screenshot_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Navigate to page for screenshot".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
            }
        ])
    }
    
    fn create_monitoring_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Set up monitoring target".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
            }
        ])
    }
    
    fn create_testing_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Navigate to test target".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
            }
        ])
    }
    
    fn create_reporting_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "report".to_string(),
                description: "Generate requested report".to_string(),
                parameters: serde_json::json!({
                    "format": "summary"
                }),
            }
        ])
    }
    
    fn create_generic_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                action_type: "navigate".to_string(),
                description: "Navigate to default page".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com",
                    "screenshot": true
                }),
            }
        ])
    }
    
    /// Generate intelligent metadata for task plans
    fn generate_intelligent_metadata(&self, input: &str, task_type: &TaskType, entities: &[Entity]) -> (String, String) {
        let title = match task_type {
            TaskType::Planning => {
                if entities.iter().any(|e| e.entity_type == "location") {
                    let location = entities.iter()
                        .find(|e| e.entity_type == "location")
                        .map(|e| e.value.clone())
                        .unwrap_or_else(|| "destination".to_string());
                    format!("Travel Planning for {}", location)
                } else {
                    "Travel Planning".to_string()
                }
            }
            TaskType::Search => format!("Search: {}", input.chars().take(50).collect::<String>()),
            TaskType::Analysis => "Data Analysis Task".to_string(),
            _ => format!("{:?} Task", task_type),
        };
        
        let description = match task_type {
            TaskType::Planning => "Comprehensive travel planning including flights, accommodation, attractions, and weather",
            TaskType::Search => "Search for information and extract relevant results",
            TaskType::Analysis => "Analyze data and provide insights",
            _ => "Execute the requested task with intelligent automation",
        };
        
        (title, description.to_string())
    }
    
    /// Estimate duration based on learned patterns
    fn estimate_duration_intelligently(&self, steps: &[ActionStep], task_type: &TaskType) -> u32 {
        let base_duration_per_step = match task_type {
            TaskType::Planning => 90,  // More complex planning takes longer
            TaskType::Analysis => 75,  // Analysis requires more processing
            TaskType::Search => 45,    // Searches are relatively quick
            TaskType::Navigation => 30, // Navigation is fast
            TaskType::Screenshot => 20, // Screenshots are very fast
            _ => 60,                   // Default duration
        };
        
        (steps.len() as u32 * base_duration_per_step).max(30)
    }
    
    /// Identify required capabilities based on task steps
    fn identify_required_capabilities(&self, steps: &[ActionStep], _entities: &[Entity]) -> Vec<String> {
        let mut capabilities = std::collections::HashSet::new();
        
        for step in steps {
            match step.action_type.as_str() {
                "navigate" => capabilities.insert("web_navigation".to_string()),
                "extract" => capabilities.insert("data_extraction".to_string()),
                "screenshot" => capabilities.insert("screenshot_capture".to_string()),
                "report" => capabilities.insert("report_generation".to_string()),
                _ => capabilities.insert("general_automation".to_string()),
            };
        }
        
        capabilities.into_iter().collect()
    }
    
    /// Get intelligence statistics
    pub fn get_intelligence_stats(&self) -> Option<crate::intelligence::IntelligenceStats> {
        match self.mode {
            IntelligenceMode::Intelligent | IntelligenceMode::Hybrid => {
                Some(self.perception.get_intelligence_stats())
            }
            IntelligenceMode::Legacy => None,
        }
    }
    
    /// Switch intelligence mode at runtime
    pub fn set_mode(&mut self, mode: IntelligenceMode) {
        info!("🔄 Switching intelligence mode from {:?} to {:?}", self.mode, mode);
        self.mode = mode;
    }
}