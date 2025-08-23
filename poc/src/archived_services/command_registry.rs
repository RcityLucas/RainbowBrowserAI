//! Intelligent Command Registry System
//!
//! This module provides a command registration and management system that
//! integrates with our 5-layer intelligence architecture to enable intelligent
//! command selection, parameter inference, and execution optimization.

use anyhow::{Result, Context as AnyhowContext};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::contextual_perception::{ContextualPerception, ContextualTaskUnderstanding};
use crate::creative_engine::CreativeEngine;
use crate::simple_memory::SimpleMemory;
use crate::browser::{SimpleBrowser, BrowserAction};

/// Intelligent command registry that combines command definitions with AI understanding
pub struct IntelligentCommandRegistry {
    /// Registered commands indexed by name
    commands: Arc<RwLock<HashMap<String, CommandDefinition>>>,
    /// Command categories for better organization
    categories: Arc<RwLock<HashMap<CommandCategory, Vec<String>>>>,
    /// Intelligence layer for understanding
    intelligence: Arc<RwLock<ContextualPerception>>,
    /// Creative engine for alternative strategies
    creative_engine: Option<Arc<CreativeEngine>>,
    /// Execution history for learning
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
    /// Command success statistics
    command_stats: Arc<RwLock<HashMap<String, CommandStatistics>>>,
    /// Session ID for tracking
    session_id: Uuid,
}

/// Definition of a browser command with intelligent features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Unique command name
    pub name: String,
    /// Human-readable description for LLM understanding
    pub description: String,
    /// Command category
    pub category: CommandCategory,
    /// Parameters the command accepts
    pub parameters: Vec<ParameterDefinition>,
    /// Conditions that must be met before execution
    pub preconditions: Vec<Precondition>,
    /// Criteria to determine if execution was successful
    pub success_criteria: Vec<SuccessCriterion>,
    /// Alternative strategies if primary execution fails
    pub fallback_strategies: Vec<FallbackStrategy>,
    /// Semantic tags for better AI understanding
    pub semantic_tags: Vec<String>,
    /// Complexity score (0.0 - 1.0)
    pub complexity: f32,
    /// Typical execution time in milliseconds
    pub typical_duration_ms: u64,
    /// Whether this command modifies page state
    pub modifies_state: bool,
    /// Whether this command requires user interaction
    pub requires_interaction: bool,
}

/// Command categories for organization and selection
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandCategory {
    Navigation,      // URL navigation, back, forward, refresh
    Interaction,     // Click, input, select, drag
    Extraction,      // Get text, attributes, screenshots
    Validation,      // Check elements, wait for conditions
    PageManagement,  // Tabs, windows, frames
    Advanced,        // Complex multi-step operations
    Custom,          // User-defined commands
}

/// Parameter definition for commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
    pub can_infer_from_context: bool,
}

/// Types of parameters commands can accept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Url,
    Selector,      // CSS selector or XPath
    Coordinate,    // (x, y) position
    Duration,      // Time duration
    KeySequence,   // Keyboard keys
    Json,          // Arbitrary JSON data
}

/// Validation rules for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),  // Regex pattern
    Range(f64, f64),
    OneOf(Vec<String>),
    Custom(String),   // Custom validation function name
}

/// Preconditions that must be met before command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Precondition {
    PageLoaded,
    ElementExists(String),      // Selector
    ElementVisible(String),     // Selector
    ElementClickable(String),   // Selector
    UrlMatches(String),         // URL pattern
    CustomCondition(String),    // Custom condition name
    NoActiveAnimations,
    NetworkIdle,
}

/// Success criteria for command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuccessCriterion {
    PageNavigated,
    ElementClicked,
    TextEntered,
    ElementFound,
    ValueExtracted,
    ScreenshotTaken,
    ConditionMet(String),
    NoErrors,
    ResponseReceived,
    Custom(String),
}

/// Fallback strategies when primary execution fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FallbackStrategy {
    WaitAndRetry(u32),           // Retry count
    ScrollToElement,
    UseAlternativeSelector(String),
    ClickParentElement,
    UseJavaScript,
    VisualElementDetection,
    ForceClick,
    ClearAndType,
    RefreshAndRetry,
    CreativeAlternative,          // Use creative engine
}

/// Record of command execution for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub command_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: ExecutionContext,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub fallback_used: Option<FallbackStrategy>,
    pub confidence_score: f32,
}

/// Context information during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub url: String,
    pub page_title: String,
    pub viewport_size: (u32, u32),
    pub user_agent: String,
    pub network_quality: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

/// Statistics for command performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatistics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_duration_ms: f64,
    pub success_rate: f32,
    pub fallback_usage_rate: f32,
    pub last_execution: Option<DateTime<Utc>>,
    pub common_errors: Vec<(String, u32)>,
}

/// Result of command selection process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSelection {
    pub command: CommandDefinition,
    pub confidence: f32,
    pub inferred_parameters: HashMap<String, serde_json::Value>,
    pub alternative_commands: Vec<(CommandDefinition, f32)>,
    pub reasoning: String,
}

impl IntelligentCommandRegistry {
    /// Create a new intelligent command registry
    pub async fn new(intelligence: Arc<RwLock<ContextualPerception>>) -> Result<Self> {
        let registry = Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            intelligence,
            creative_engine: None,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            command_stats: Arc::new(RwLock::new(HashMap::new())),
            session_id: Uuid::new_v4(),
        };

        // Register built-in commands
        registry.register_builtin_commands().await?;

        info!("📝 Intelligent Command Registry initialized with {} built-in commands", 
              registry.commands.read().await.len());

        Ok(registry)
    }

    /// Create with creative engine for advanced strategies
    pub async fn with_creative_engine(
        intelligence: Arc<RwLock<ContextualPerception>>,
        creative_engine: Arc<CreativeEngine>,
    ) -> Result<Self> {
        let mut registry = Self::new(intelligence).await?;
        registry.creative_engine = Some(creative_engine);
        Ok(registry)
    }

    /// Register a new command
    pub async fn register_command(&self, command: CommandDefinition) -> Result<()> {
        let name = command.name.clone();
        let category = command.category;

        // Add to commands
        {
            let mut commands = self.commands.write().await;
            commands.insert(name.clone(), command);
        }

        // Add to category index
        {
            let mut categories = self.categories.write().await;
            categories.entry(category)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        // Initialize statistics
        {
            let mut stats = self.command_stats.write().await;
            stats.entry(name.clone()).or_insert_with(|| CommandStatistics {
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_duration_ms: 0.0,
                success_rate: 0.0,
                fallback_usage_rate: 0.0,
                last_execution: None,
                common_errors: Vec::new(),
            });
        }

        info!("✅ Registered command: {} (category: {:?})", name, category);
        Ok(())
    }

    /// Select the best command for a given user intent
    pub async fn select_command(&self, user_intent: &str) -> Result<CommandSelection> {
        info!("🔍 Selecting command for intent: {}", user_intent);

        // Use intelligence layer to understand intent
        let mut intelligence = self.intelligence.write().await;
        let understanding = intelligence.understand_intent(user_intent).await?;

        // Get all commands
        let commands = self.commands.read().await;
        
        // Score each command based on understanding
        let mut scored_commands = Vec::new();
        
        for (name, command) in commands.iter() {
            let score = self.score_command_match(command, &understanding, user_intent).await?;
            if score > 0.0 {
                scored_commands.push((command.clone(), score));
            }
        }

        // Sort by score
        scored_commands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if scored_commands.is_empty() {
            return Err(anyhow::anyhow!("No suitable command found for intent: {}", user_intent));
        }

        // Select best command
        let (best_command, confidence) = scored_commands[0].clone();

        // Infer parameters
        let inferred_parameters = self.infer_parameters(&best_command, &understanding, user_intent).await?;

        // Get alternatives
        let alternative_commands = scored_commands.iter()
            .skip(1)
            .take(3)
            .cloned()
            .collect();

        let reasoning = format!(
            "Selected '{}' with {:.2}% confidence based on intent understanding. \
             Task type: {:?}, Context confidence: {:.2}",
            best_command.name,
            confidence * 100.0,
            understanding.task_type,
            understanding.context_snapshot.confidence_score
        );

        Ok(CommandSelection {
            command: best_command,
            confidence,
            inferred_parameters,
            alternative_commands,
            reasoning,
        })
    }

    /// Score how well a command matches the user intent
    async fn score_command_match(
        &self,
        command: &CommandDefinition,
        understanding: &ContextualTaskUnderstanding,
        user_intent: &str,
    ) -> Result<f32> {
        let mut score = 0.0;
        let intent_lower = user_intent.to_lowercase();

        // Category matching
        score += match (understanding.task_type, command.category) {
            (crate::llm_service::llm_service_enhanced::TaskType::Navigation, CommandCategory::Navigation) => 0.3,
            (crate::llm_service::llm_service_enhanced::TaskType::Search, CommandCategory::Interaction) => 0.25,
            (crate::llm_service::llm_service_enhanced::TaskType::Extraction, CommandCategory::Extraction) => 0.3,
            _ => 0.0,
        };

        // Semantic tag matching
        for tag in &command.semantic_tags {
            if intent_lower.contains(tag) {
                score += 0.15;
            }
        }

        // Description relevance
        let desc_lower = command.description.to_lowercase();
        let desc_words: Vec<&str> = desc_lower.split_whitespace().collect();
        let intent_words: Vec<&str> = intent_lower.split_whitespace().collect();
        let matching_words = desc_words.iter()
            .filter(|w| intent_words.contains(w))
            .count();
        score += (matching_words as f32 / desc_words.len().max(1) as f32) * 0.2;

        // Command name relevance
        if intent_lower.contains(&command.name.replace('_', " ")) {
            score += 0.25;
        }

        // Complexity alignment
        let intent_complexity = (intent_words.len() as f32 / 10.0).min(1.0);
        let complexity_diff = (command.complexity - intent_complexity).abs();
        score += (1.0 - complexity_diff) * 0.1;

        // Context-based adjustment
        score *= understanding.confidence;

        // Historical success rate bonus
        if let Some(stats) = self.command_stats.read().await.get(&command.name) {
            if stats.total_executions > 5 {
                score += stats.success_rate * 0.1;
            }
        }

        Ok(score.min(1.0))
    }

    /// Infer parameters for a command based on context
    async fn infer_parameters(
        &self,
        command: &CommandDefinition,
        understanding: &ContextualTaskUnderstanding,
        user_intent: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut parameters = HashMap::new();

        for param_def in &command.parameters {
            if param_def.can_infer_from_context {
                // Try to infer from entities
                for entity in &understanding.entities {
                    if Self::entity_matches_parameter(&entity.entity_type, &param_def.param_type) {
                        parameters.insert(
                            param_def.name.clone(),
                            serde_json::Value::String(entity.value.clone()),
                        );
                        break;
                    }
                }
            }

            // Use default if not inferred and not required
            if !parameters.contains_key(&param_def.name) {
                if let Some(default) = &param_def.default_value {
                    parameters.insert(param_def.name.clone(), default.clone());
                } else if param_def.required {
                    // Try to extract from intent string
                    if let Some(value) = self.extract_parameter_from_intent(user_intent, &param_def).await {
                        parameters.insert(param_def.name.clone(), value);
                    }
                }
            }
        }

        Ok(parameters)
    }

    /// Check if an entity type matches a parameter type
    fn entity_matches_parameter(entity_type: &str, param_type: &ParameterType) -> bool {
        match (entity_type, param_type) {
            ("url", ParameterType::Url) => true,
            ("text", ParameterType::String) => true,
            ("number", ParameterType::Integer) | ("number", ParameterType::Float) => true,
            ("location", ParameterType::String) => true,
            ("time", ParameterType::Duration) => true,
            _ => false,
        }
    }

    /// Extract parameter value from intent string
    async fn extract_parameter_from_intent(
        &self,
        intent: &str,
        param_def: &ParameterDefinition,
    ) -> Option<serde_json::Value> {
        // Simple extraction based on parameter type
        match param_def.param_type {
            ParameterType::Url => {
                // Look for URL patterns
                if let Some(url_match) = intent.split_whitespace()
                    .find(|word| word.starts_with("http://") || word.starts_with("https://")) {
                    return Some(serde_json::Value::String(url_match.to_string()));
                }
            },
            ParameterType::String => {
                // Extract quoted strings or specific patterns
                if intent.contains('"') {
                    if let Some(start) = intent.find('"') {
                        if let Some(end) = intent[start + 1..].find('"') {
                            let extracted = &intent[start + 1..start + 1 + end];
                            return Some(serde_json::Value::String(extracted.to_string()));
                        }
                    }
                }
            },
            _ => {},
        }

        None
    }

    /// Record command execution for learning
    pub async fn record_execution(&self, record: ExecutionRecord) -> Result<()> {
        let command_name = record.command_name.clone();
        let success = record.success;
        let duration = record.duration_ms;

        // Update history
        {
            let mut history = self.execution_history.write().await;
            history.push(record.clone());
            
            // Keep only last 1000 records
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }

        // Update statistics
        {
            let mut stats = self.command_stats.write().await;
            if let Some(stat) = stats.get_mut(&command_name) {
                stat.total_executions += 1;
                if success {
                    stat.successful_executions += 1;
                } else {
                    stat.failed_executions += 1;
                    if let Some(error) = &record.error_message {
                        // Track common errors
                        if let Some(error_count) = stat.common_errors.iter_mut()
                            .find(|(e, _)| e == error) {
                            error_count.1 += 1;
                        } else {
                            stat.common_errors.push((error.clone(), 1));
                        }
                    }
                }
                
                // Update success rate
                stat.success_rate = stat.successful_executions as f32 / stat.total_executions as f32;
                
                // Update average duration
                stat.average_duration_ms = 
                    (stat.average_duration_ms * (stat.total_executions - 1) as f64 + duration as f64) 
                    / stat.total_executions as f64;
                
                // Update fallback usage
                if record.fallback_used.is_some() {
                    let fallback_count = stat.total_executions - stat.successful_executions;
                    stat.fallback_usage_rate = fallback_count as f32 / stat.total_executions as f32;
                }
                
                stat.last_execution = Some(record.completed_at);
            }
        }

        info!("📊 Recorded execution: {} (success: {}, duration: {}ms)", 
              command_name, success, duration);

        Ok(())
    }

    /// Get command statistics
    pub async fn get_command_stats(&self, command_name: &str) -> Option<CommandStatistics> {
        self.command_stats.read().await.get(command_name).cloned()
    }

    /// Get all registered commands
    pub async fn get_all_commands(&self) -> Vec<CommandDefinition> {
        self.commands.read().await.values().cloned().collect()
    }

    /// Get commands by category
    pub async fn get_commands_by_category(&self, category: CommandCategory) -> Vec<CommandDefinition> {
        let categories = self.categories.read().await;
        let commands = self.commands.read().await;
        
        if let Some(command_names) = categories.get(&category) {
            command_names.iter()
                .filter_map(|name| commands.get(name).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Register built-in browser commands
    async fn register_builtin_commands(&self) -> Result<()> {
        // Navigation: Go to URL
        self.register_command(CommandDefinition {
            name: "navigate_to_url".to_string(),
            description: "Navigate to a specific URL in the browser".to_string(),
            category: CommandCategory::Navigation,
            parameters: vec![
                ParameterDefinition {
                    name: "url".to_string(),
                    param_type: ParameterType::Url,
                    required: true,
                    default_value: None,
                    description: "The URL to navigate to".to_string(),
                    validation_rules: vec![ValidationRule::Pattern(r"^https?://".to_string())],
                    can_infer_from_context: true,
                },
            ],
            preconditions: vec![],
            success_criteria: vec![SuccessCriterion::PageNavigated],
            fallback_strategies: vec![
                FallbackStrategy::WaitAndRetry(3),
                FallbackStrategy::RefreshAndRetry,
            ],
            semantic_tags: vec!["go".to_string(), "visit".to_string(), "open".to_string(), "browse".to_string()],
            complexity: 0.2,
            typical_duration_ms: 2000,
            modifies_state: true,
            requires_interaction: false,
        }).await?;

        // Interaction: Click element
        self.register_command(CommandDefinition {
            name: "click_element".to_string(),
            description: "Click on an element in the page".to_string(),
            category: CommandCategory::Interaction,
            parameters: vec![
                ParameterDefinition {
                    name: "selector".to_string(),
                    param_type: ParameterType::Selector,
                    required: false,
                    default_value: None,
                    description: "CSS selector or element description".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
                ParameterDefinition {
                    name: "text".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default_value: None,
                    description: "Text content of the element to click".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
            ],
            preconditions: vec![
                Precondition::PageLoaded,
            ],
            success_criteria: vec![SuccessCriterion::ElementClicked],
            fallback_strategies: vec![
                FallbackStrategy::ScrollToElement,
                FallbackStrategy::WaitAndRetry(3),
                FallbackStrategy::UseJavaScript,
                FallbackStrategy::VisualElementDetection,
                FallbackStrategy::ClickParentElement,
            ],
            semantic_tags: vec!["click".to_string(), "press".to_string(), "tap".to_string(), "select".to_string()],
            complexity: 0.3,
            typical_duration_ms: 500,
            modifies_state: true,
            requires_interaction: true,
        }).await?;

        // Interaction: Input text
        self.register_command(CommandDefinition {
            name: "input_text".to_string(),
            description: "Type text into an input field".to_string(),
            category: CommandCategory::Interaction,
            parameters: vec![
                ParameterDefinition {
                    name: "selector".to_string(),
                    param_type: ParameterType::Selector,
                    required: false,
                    default_value: None,
                    description: "Input field selector".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
                ParameterDefinition {
                    name: "text".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default_value: None,
                    description: "Text to input".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
            ],
            preconditions: vec![
                Precondition::PageLoaded,
            ],
            success_criteria: vec![SuccessCriterion::TextEntered],
            fallback_strategies: vec![
                FallbackStrategy::ClearAndType,
                FallbackStrategy::UseJavaScript,
                FallbackStrategy::WaitAndRetry(2),
            ],
            semantic_tags: vec!["type".to_string(), "enter".to_string(), "input".to_string(), "fill".to_string(), "write".to_string()],
            complexity: 0.25,
            typical_duration_ms: 1000,
            modifies_state: true,
            requires_interaction: true,
        }).await?;

        // Extraction: Get text
        self.register_command(CommandDefinition {
            name: "extract_text".to_string(),
            description: "Extract text content from the page or element".to_string(),
            category: CommandCategory::Extraction,
            parameters: vec![
                ParameterDefinition {
                    name: "selector".to_string(),
                    param_type: ParameterType::Selector,
                    required: false,
                    default_value: Some(serde_json::Value::String("body".to_string())),
                    description: "Element to extract text from".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
            ],
            preconditions: vec![Precondition::PageLoaded],
            success_criteria: vec![SuccessCriterion::ValueExtracted],
            fallback_strategies: vec![
                FallbackStrategy::WaitAndRetry(2),
                FallbackStrategy::UseJavaScript,
            ],
            semantic_tags: vec!["get".to_string(), "extract".to_string(), "read".to_string(), "copy".to_string(), "fetch".to_string()],
            complexity: 0.1,
            typical_duration_ms: 100,
            modifies_state: false,
            requires_interaction: false,
        }).await?;

        // Navigation: Go back
        self.register_command(CommandDefinition {
            name: "go_back".to_string(),
            description: "Navigate back to the previous page".to_string(),
            category: CommandCategory::Navigation,
            parameters: vec![],
            preconditions: vec![],
            success_criteria: vec![SuccessCriterion::PageNavigated],
            fallback_strategies: vec![],
            semantic_tags: vec!["back".to_string(), "previous".to_string(), "return".to_string()],
            complexity: 0.1,
            typical_duration_ms: 1000,
            modifies_state: true,
            requires_interaction: false,
        }).await?;

        // Navigation: Refresh
        self.register_command(CommandDefinition {
            name: "refresh_page".to_string(),
            description: "Refresh the current page".to_string(),
            category: CommandCategory::Navigation,
            parameters: vec![],
            preconditions: vec![],
            success_criteria: vec![SuccessCriterion::PageNavigated],
            fallback_strategies: vec![],
            semantic_tags: vec!["refresh".to_string(), "reload".to_string(), "update".to_string()],
            complexity: 0.1,
            typical_duration_ms: 2000,
            modifies_state: false,
            requires_interaction: false,
        }).await?;

        // Extraction: Take screenshot
        self.register_command(CommandDefinition {
            name: "take_screenshot".to_string(),
            description: "Capture a screenshot of the page or element".to_string(),
            category: CommandCategory::Extraction,
            parameters: vec![
                ParameterDefinition {
                    name: "selector".to_string(),
                    param_type: ParameterType::Selector,
                    required: false,
                    default_value: None,
                    description: "Element to screenshot (full page if not specified)".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: false,
                },
                ParameterDefinition {
                    name: "filename".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default_value: None,
                    description: "Filename to save screenshot".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: false,
                },
            ],
            preconditions: vec![Precondition::PageLoaded],
            success_criteria: vec![SuccessCriterion::ScreenshotTaken],
            fallback_strategies: vec![
                FallbackStrategy::WaitAndRetry(2),
            ],
            semantic_tags: vec!["screenshot".to_string(), "capture".to_string(), "snap".to_string(), "photo".to_string()],
            complexity: 0.15,
            typical_duration_ms: 500,
            modifies_state: false,
            requires_interaction: false,
        }).await?;

        // Validation: Wait for element
        self.register_command(CommandDefinition {
            name: "wait_for_element".to_string(),
            description: "Wait for an element to appear on the page".to_string(),
            category: CommandCategory::Validation,
            parameters: vec![
                ParameterDefinition {
                    name: "selector".to_string(),
                    param_type: ParameterType::Selector,
                    required: true,
                    default_value: None,
                    description: "Element selector to wait for".to_string(),
                    validation_rules: vec![],
                    can_infer_from_context: true,
                },
                ParameterDefinition {
                    name: "timeout".to_string(),
                    param_type: ParameterType::Duration,
                    required: false,
                    default_value: Some(serde_json::Value::Number(serde_json::Number::from(5000))),
                    description: "Maximum wait time in milliseconds".to_string(),
                    validation_rules: vec![ValidationRule::Range(0.0, 30000.0)],
                    can_infer_from_context: false,
                },
            ],
            preconditions: vec![],
            success_criteria: vec![SuccessCriterion::ElementFound],
            fallback_strategies: vec![
                FallbackStrategy::RefreshAndRetry,
            ],
            semantic_tags: vec!["wait".to_string(), "appear".to_string(), "load".to_string(), "show".to_string()],
            complexity: 0.2,
            typical_duration_ms: 2000,
            modifies_state: false,
            requires_interaction: false,
        }).await?;

        // Interaction: Scroll
        self.register_command(CommandDefinition {
            name: "scroll_page".to_string(),
            description: "Scroll the page up or down".to_string(),
            category: CommandCategory::Interaction,
            parameters: vec![
                ParameterDefinition {
                    name: "direction".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default_value: Some(serde_json::Value::String("down".to_string())),
                    description: "Scroll direction (up/down)".to_string(),
                    validation_rules: vec![ValidationRule::OneOf(vec!["up".to_string(), "down".to_string()])],
                    can_infer_from_context: true,
                },
                ParameterDefinition {
                    name: "amount".to_string(),
                    param_type: ParameterType::Integer,
                    required: false,
                    default_value: Some(serde_json::Value::Number(serde_json::Number::from(500))),
                    description: "Pixels to scroll".to_string(),
                    validation_rules: vec![ValidationRule::Range(0.0, 10000.0)],
                    can_infer_from_context: false,
                },
            ],
            preconditions: vec![Precondition::PageLoaded],
            success_criteria: vec![SuccessCriterion::NoErrors],
            fallback_strategies: vec![],
            semantic_tags: vec!["scroll".to_string(), "move".to_string(), "slide".to_string()],
            complexity: 0.1,
            typical_duration_ms: 200,
            modifies_state: false,
            requires_interaction: false,
        }).await?;

        info!("✅ Registered {} built-in commands", 9);
        Ok(())
    }
}

/// Command builder for easier command creation
pub struct CommandBuilder {
    command: CommandDefinition,
}

impl CommandBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            command: CommandDefinition {
                name: name.into(),
                description: String::new(),
                category: CommandCategory::Custom,
                parameters: Vec::new(),
                preconditions: Vec::new(),
                success_criteria: Vec::new(),
                fallback_strategies: Vec::new(),
                semantic_tags: Vec::new(),
                complexity: 0.5,
                typical_duration_ms: 1000,
                modifies_state: false,
                requires_interaction: false,
            },
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.command.description = desc.into();
        self
    }

    pub fn category(mut self, category: CommandCategory) -> Self {
        self.command.category = category;
        self
    }

    pub fn parameter(mut self, param: ParameterDefinition) -> Self {
        self.command.parameters.push(param);
        self
    }

    pub fn precondition(mut self, condition: Precondition) -> Self {
        self.command.preconditions.push(condition);
        self
    }

    pub fn success_criterion(mut self, criterion: SuccessCriterion) -> Self {
        self.command.success_criteria.push(criterion);
        self
    }

    pub fn fallback(mut self, strategy: FallbackStrategy) -> Self {
        self.command.fallback_strategies.push(strategy);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.command.semantic_tags.push(tag.into());
        self
    }

    pub fn complexity(mut self, complexity: f32) -> Self {
        self.command.complexity = complexity;
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.command.typical_duration_ms = ms;
        self
    }

    pub fn modifies_state(mut self, modifies: bool) -> Self {
        self.command.modifies_state = modifies;
        self
    }

    pub fn requires_interaction(mut self, requires: bool) -> Self {
        self.command.requires_interaction = requires;
        self
    }

    pub fn build(self) -> CommandDefinition {
        self.command
    }
}

/// Create command registry from environment
pub async fn create_command_registry(intelligence: Arc<RwLock<ContextualPerception>>) -> Result<IntelligentCommandRegistry> {
    IntelligentCommandRegistry::new(intelligence).await
}

/// Create command registry with creative engine
pub async fn create_advanced_registry(
    intelligence: Arc<RwLock<ContextualPerception>>,
    creative_engine: Arc<CreativeEngine>,
) -> Result<IntelligentCommandRegistry> {
    IntelligentCommandRegistry::with_creative_engine(intelligence, creative_engine).await
}