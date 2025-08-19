// Enhanced LLM Service with Intelligent Task Understanding
// This module adds task classification and decomposition capabilities

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Task types that the system can understand and execute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Navigation,      // Navigate to URL
    Screenshot,      // Take screenshot
    Search,         // Search for information
    Planning,       // Create plans (travel, shopping, etc.)
    Analysis,       // Analyze content
    Execution,      // Execute multi-step tasks
    Extraction,     // Extract data from pages
    Monitoring,     // Monitor websites
    Testing,        // Test multiple sites
    Reporting,      // Generate reports
    Unknown,        // Unknown task type
}

/// Represents a single action step in a complex task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub step_number: u32,
    pub description: String,
    pub action_type: String,
    pub parameters: serde_json::Value,
    pub depends_on: Option<u32>,
    pub optional: bool,
}

/// Enhanced parsed command with task understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentCommand {
    pub task_type: TaskType,
    pub intent: String,
    pub entities: Vec<Entity>,
    pub steps: Vec<ActionStep>,
    pub context: serde_json::Value,
    pub confidence: f32,
    pub original_command: String,
}

/// Entity extracted from the command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
}

/// Task decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub title: String,
    pub description: String,
    pub steps: Vec<ActionStep>,
    pub estimated_duration: u32, // in seconds
    pub required_capabilities: Vec<String>,
}

/// Enhanced LLM Service trait for task understanding
pub trait TaskUnderstanding {
    /// Classify the intent of a user command
    fn classify_intent(&self, input: &str) -> Result<TaskType>;
    
    /// Extract entities from the command
    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>>;
    
    /// Decompose a complex task into actionable steps
    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>>;
    
    /// Create a complete task plan
    fn create_task_plan(&self, input: &str) -> Result<TaskPlan>;
}

/// Implementation of task understanding for mock mode
pub struct MockTaskUnderstanding;

impl TaskUnderstanding for MockTaskUnderstanding {
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        let input_lower = input.to_lowercase();
        
        // Enhanced intent classification
        if input_lower.contains("plan") || input_lower.contains("itinerary") || 
           input_lower.contains("trip") || input_lower.contains("vacation") {
            return Ok(TaskType::Planning);
        }
        
        if input_lower.contains("search") || input_lower.contains("find") || 
           input_lower.contains("look for") {
            return Ok(TaskType::Search);
        }
        
        if input_lower.contains("analyze") || input_lower.contains("review") || 
           input_lower.contains("evaluate") {
            return Ok(TaskType::Analysis);
        }
        
        if input_lower.contains("extract") || input_lower.contains("scrape") || 
           input_lower.contains("collect") {
            return Ok(TaskType::Extraction);
        }
        
        if input_lower.contains("monitor") || input_lower.contains("watch") || 
           input_lower.contains("track") {
            return Ok(TaskType::Monitoring);
        }
        
        if input_lower.contains("test") && (input_lower.contains("sites") || 
           input_lower.contains("websites")) {
            return Ok(TaskType::Testing);
        }
        
        if input_lower.contains("report") || input_lower.contains("summary") || 
           input_lower.contains("statistics") {
            return Ok(TaskType::Reporting);
        }
        
        if input_lower.contains("navigate") || input_lower.contains("go to") || 
           input_lower.contains("open") || input_lower.contains("visit") {
            return Ok(TaskType::Navigation);
        }
        
        if input_lower.contains("screenshot") || input_lower.contains("capture") || 
           input_lower.contains("snap") {
            return Ok(TaskType::Screenshot);
        }
        
        Ok(TaskType::Unknown)
    }
    
    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        let input_lower = input.to_lowercase();
        
        // Extract locations (for travel planning)
        let locations = ["paris", "tokyo", "new york", "london", "rome", "barcelona", 
                        "berlin", "amsterdam", "prague", "vienna"];
        for location in &locations {
            if input_lower.contains(location) {
                entities.push(Entity {
                    entity_type: "location".to_string(),
                    value: location.to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        // Extract dates (simplified)
        if input_lower.contains("tomorrow") {
            entities.push(Entity {
                entity_type: "date".to_string(),
                value: "tomorrow".to_string(),
                confidence: 0.95,
            });
        }
        
        if input_lower.contains("next week") {
            entities.push(Entity {
                entity_type: "date".to_string(),
                value: "next_week".to_string(),
                confidence: 0.95,
            });
        }
        
        // Extract websites
        let websites = ["google", "github", "stackoverflow", "reddit", "youtube"];
        for website in &websites {
            if input_lower.contains(website) {
                entities.push(Entity {
                    entity_type: "website".to_string(),
                    value: format!("{}.com", website),
                    confidence: 0.95,
                });
            }
        }
        
        // Extract actions
        let actions = ["flights", "hotels", "restaurants", "attractions", "weather"];
        for action in &actions {
            if input_lower.contains(action) {
                entities.push(Entity {
                    entity_type: "search_category".to_string(),
                    value: action.to_string(),
                    confidence: 0.85,
                });
            }
        }
        
        Ok(entities)
    }
    
    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        match task_type {
            TaskType::Planning => self.create_planning_steps(input),
            TaskType::Search => self.create_search_steps(input),
            TaskType::Analysis => self.create_analysis_steps(input),
            TaskType::Testing => self.create_testing_steps(input),
            _ => self.create_default_steps(input),
        }
    }
    
    fn create_task_plan(&self, input: &str) -> Result<TaskPlan> {
        let task_type = self.classify_intent(input)?;
        let steps = self.decompose_task(input, task_type.clone())?;
        let entities = self.extract_entities(input)?;
        
        let title = match task_type {
            TaskType::Planning => "Travel Planning Task".to_string(),
            TaskType::Search => "Information Search Task".to_string(),
            TaskType::Analysis => "Content Analysis Task".to_string(),
            _ => "Custom Task".to_string(),
        };
        
        let description = format!(
            "Executing {} task with {} steps and {} entities identified",
            match task_type {
                TaskType::Planning => "planning",
                TaskType::Search => "search",
                TaskType::Analysis => "analysis",
                _ => "custom",
            },
            steps.len(),
            entities.len()
        );
        
        let estimated_duration = 60 * steps.len() as u32; // 60 seconds per step
        
        Ok(TaskPlan {
            title,
            description,
            steps,
            estimated_duration,
            required_capabilities: vec![
                "browser_automation".to_string(),
                "content_extraction".to_string(),
                "llm_processing".to_string(),
            ],
        })
    }
}

impl MockTaskUnderstanding {
    fn create_planning_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        let input_lower = input.to_lowercase();
        
        // Check if it's a travel plan
        if input_lower.contains("travel") || input_lower.contains("trip") {
            steps.push(ActionStep {
                step_number: 1,
                description: "Search for destination information".to_string(),
                action_type: "navigate".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/search?q=travel+guide",
                    "screenshot": false
                }),
                depends_on: None,
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 2,
                description: "Search for flights".to_string(),
                action_type: "navigate".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.google.com/flights",
                    "screenshot": true
                }),
                depends_on: Some(1),
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 3,
                description: "Search for hotels".to_string(),
                action_type: "navigate".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.booking.com",
                    "screenshot": true
                }),
                depends_on: Some(2),
                optional: false,
            });
            
            steps.push(ActionStep {
                step_number: 4,
                description: "Research local attractions".to_string(),
                action_type: "navigate".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.tripadvisor.com",
                    "screenshot": true
                }),
                depends_on: Some(3),
                optional: true,
            });
            
            steps.push(ActionStep {
                step_number: 5,
                description: "Check weather forecast".to_string(),
                action_type: "navigate".to_string(),
                parameters: serde_json::json!({
                    "url": "https://www.weather.com",
                    "screenshot": false
                }),
                depends_on: None,
                optional: true,
            });
            
            steps.push(ActionStep {
                step_number: 6,
                description: "Compile travel plan summary".to_string(),
                action_type: "report".to_string(),
                parameters: serde_json::json!({
                    "format": "markdown",
                    "include_screenshots": true
                }),
                depends_on: Some(4),
                optional: false,
            });
        }
        
        Ok(steps)
    }
    
    fn create_search_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        
        steps.push(ActionStep {
            step_number: 1,
            description: "Perform web search".to_string(),
            action_type: "navigate".to_string(),
            parameters: serde_json::json!({
                "url": "https://www.google.com",
                "screenshot": false
            }),
            depends_on: None,
            optional: false,
        });
        
        steps.push(ActionStep {
            step_number: 2,
            description: "Extract search results".to_string(),
            action_type: "extract".to_string(),
            parameters: serde_json::json!({
                "selector": ".g",
                "limit": 10
            }),
            depends_on: Some(1),
            optional: false,
        });
        
        Ok(steps)
    }
    
    fn create_analysis_steps(&self, _input: &str) -> Result<Vec<ActionStep>> {
        let mut steps = Vec::new();
        
        steps.push(ActionStep {
            step_number: 1,
            description: "Navigate to target page".to_string(),
            action_type: "navigate".to_string(),
            parameters: serde_json::json!({
                "screenshot": true
            }),
            depends_on: None,
            optional: false,
        });
        
        steps.push(ActionStep {
            step_number: 2,
            description: "Extract page content".to_string(),
            action_type: "extract".to_string(),
            parameters: serde_json::json!({
                "full_page": true
            }),
            depends_on: Some(1),
            optional: false,
        });
        
        steps.push(ActionStep {
            step_number: 3,
            description: "Analyze content".to_string(),
            action_type: "analyze".to_string(),
            parameters: serde_json::json!({
                "metrics": ["readability", "sentiment", "keywords"]
            }),
            depends_on: Some(2),
            optional: false,
        });
        
        Ok(steps)
    }
    
    fn create_testing_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        let entities = self.extract_entities(input)?;
        let mut steps = Vec::new();
        let mut step_num = 1;
        
        for entity in entities {
            if entity.entity_type == "website" {
                steps.push(ActionStep {
                    step_number: step_num,
                    description: format!("Test {}", entity.value),
                    action_type: "navigate".to_string(),
                    parameters: serde_json::json!({
                        "url": format!("https://{}", entity.value),
                        "screenshot": true,
                        "measure_performance": true
                    }),
                    depends_on: if step_num > 1 { Some(step_num - 1) } else { None },
                    optional: false,
                });
                step_num += 1;
            }
        }
        
        Ok(steps)
    }
    
    fn create_default_steps(&self, input: &str) -> Result<Vec<ActionStep>> {
        Ok(vec![
            ActionStep {
                step_number: 1,
                description: "Execute default action".to_string(),
                action_type: "unknown".to_string(),
                parameters: serde_json::json!({
                    "input": input
                }),
                depends_on: None,
                optional: false,
            }
        ])
    }
}

/// Enhanced parse result that includes task understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedParseResult {
    pub success: bool,
    pub action: String,
    pub confidence: f32,
    pub task_type: TaskType,
    pub task_plan: Option<TaskPlan>,
    pub result: serde_json::Value,
    pub explanation: String,
}

/// Convert enhanced understanding to standard ParsedCommand format
pub fn enhance_parsed_command(
    original_command: super::ParsedCommand,
    input: &str,
) -> Result<EnhancedParseResult> {
    let task_understanding = MockTaskUnderstanding;
    
    let task_type = task_understanding.classify_intent(input)?;
    let task_plan = if task_type != TaskType::Unknown {
        Some(task_understanding.create_task_plan(input)?)
    } else {
        None
    };
    
    let explanation = match &task_plan {
        Some(plan) => format!(
            "I'll help you with {}. This involves {} steps: {}",
            plan.title,
            plan.steps.len(),
            plan.steps.iter()
                .map(|s| s.description.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => format!(
            "Mock mode: Parsed '{}' as {} action (confidence: {:.0}%)",
            input,
            original_command.action,
            original_command.confidence * 100.0
        ),
    };
    
    Ok(EnhancedParseResult {
        success: task_type != TaskType::Unknown || original_command.action != "unknown",
        action: if task_type != TaskType::Unknown {
            format!("{:?}", task_type).to_lowercase()
        } else {
            original_command.action.clone()
        },
        confidence: if task_type != TaskType::Unknown { 0.85 } else { original_command.confidence },
        task_type: task_type.clone(),
        task_plan,
        result: serde_json::json!({
            "original_command": original_command,
            "enhanced": task_type != TaskType::Unknown,
        }),
        explanation,
    })
}