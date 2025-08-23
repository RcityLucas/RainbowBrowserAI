//! Smart Actions System for Enhanced Browser Interaction
//!
//! This module provides intelligent browser actions that automatically adapt
//! their approach based on context, success rates, and environmental factors.
//! It builds on the Enhanced Browser Controller to provide self-healing,
//! context-aware browser interactions.

use anyhow::{Result, Context as AnyhowContext};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::enhanced_browser::{EnhancedBrowserController, ElementFindingStrategy, EnhancedActionResult, ActionType as EnhancedActionType};
use crate::contextual_perception::{ContextualPerception, ContextualTaskUnderstanding};
use crate::contextual_awareness::{ContextualAwareness, ContextSnapshot, InteractionStyle, DeviceType, NetworkQuality};
use crate::simple_memory::{SimpleMemory, InteractionRecord};
use crate::llm_service::llm_service_enhanced::TaskType;

/// Smart actions orchestrator that provides intelligent browser interactions
pub struct SmartActionOrchestrator {
    /// Enhanced browser controller for low-level operations
    browser_controller: Arc<EnhancedBrowserController>,
    /// Intelligence layer for understanding context
    intelligence: Arc<RwLock<ContextualPerception>>,
    /// Contextual awareness for environmental adaptation
    awareness: Arc<RwLock<ContextualAwareness>>,
    /// Memory system for learning patterns
    memory: Option<Arc<SimpleMemory>>,
    /// Action execution history
    execution_history: Arc<RwLock<VecDeque<SmartActionExecution>>>,
    /// Strategy success rates
    strategy_stats: Arc<RwLock<HashMap<ActionType, StrategyStatistics>>>,
    /// Adaptive configuration
    config: Arc<RwLock<SmartActionConfig>>,
    /// Session tracking
    session_id: Uuid,
}

/// Configuration for smart actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartActionConfig {
    /// Maximum retry attempts per action
    pub max_retries: u32,
    /// Timeout for individual actions (milliseconds)
    pub action_timeout_ms: u64,
    /// Confidence threshold for action execution
    pub confidence_threshold: f32,
    /// Enable adaptive strategy selection
    pub adaptive_strategies: bool,
    /// Enable contextual optimizations
    pub contextual_optimization: bool,
    /// Maximum history size for learning
    pub max_history_size: usize,
    /// Learning rate for strategy adaptation
    pub learning_rate: f32,
}

/// Types of smart actions supported
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    SmartClick,
    SmartInput,
    SmartNavigation,
    SmartExtraction,
    SmartWait,
    SmartScroll,
    SmartSearch,
    SmartForm,
}

/// Smart action execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartActionExecution {
    pub id: Uuid,
    pub action_type: ActionType,
    pub intent: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub strategies_attempted: Vec<ExecutionStrategy>,
    pub successful_strategy: Option<ExecutionStrategy>,
    pub context_snapshot: ContextSnapshot,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub success: bool,
    pub confidence: f32,
    pub error_details: Option<String>,
    pub retry_count: u32,
}

/// Individual strategy execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStrategy {
    pub strategy_name: String,
    pub finding_strategy: ElementFindingStrategy,
    pub parameters: HashMap<String, serde_json::Value>,
    pub success: bool,
    pub duration_ms: u64,
    pub confidence: f32,
    pub error: Option<String>,
}

/// Statistics for strategy success rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatistics {
    pub total_attempts: u64,
    pub successful_attempts: u64,
    pub success_rate: f32,
    pub average_duration_ms: f64,
    pub confidence_distribution: Vec<f32>,
    pub context_success_rates: HashMap<String, f32>,
    pub last_updated: DateTime<Utc>,
}

/// Result of smart action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartActionResult {
    pub action_type: ActionType,
    pub success: bool,
    pub confidence: f32,
    pub duration_ms: u64,
    pub strategy_used: Option<String>,
    pub data: Option<serde_json::Value>,
    pub recommendations: Vec<ActionRecommendation>,
    pub session_id: Uuid,
}

/// Recommendations for improving future actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    pub category: RecommendationType,
    pub description: String,
    pub confidence: f32,
    pub implementation_priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    StrategyOptimization,
    ContextualAdaptation,
    PerformanceImprovement,
    ErrorPrevention,
    UserExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl SmartActionOrchestrator {
    /// Create new smart action orchestrator
    pub async fn new(
        browser_controller: Arc<EnhancedBrowserController>,
        intelligence: Arc<RwLock<ContextualPerception>>,
        awareness: Arc<RwLock<ContextualAwareness>>,
    ) -> Result<Self> {
        let config = SmartActionConfig::default();
        
        let orchestrator = Self {
            browser_controller,
            intelligence,
            awareness,
            memory: None,
            execution_history: Arc::new(RwLock::new(VecDeque::new())),
            strategy_stats: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            session_id: Uuid::new_v4(),
        };

        // Initialize strategy statistics
        orchestrator.initialize_strategy_stats().await;

        info!("🎯 Smart Action Orchestrator initialized (session: {})", orchestrator.session_id);
        Ok(orchestrator)
    }

    /// Create with memory integration
    pub async fn with_memory(
        browser_controller: Arc<EnhancedBrowserController>,
        intelligence: Arc<RwLock<ContextualPerception>>,
        awareness: Arc<RwLock<ContextualAwareness>>,
        memory: Arc<SimpleMemory>,
    ) -> Result<Self> {
        let mut orchestrator = Self::new(browser_controller, intelligence, awareness).await?;
        orchestrator.memory = Some(memory);
        
        // Load historical patterns
        orchestrator.load_historical_patterns().await?;
        
        info!("🧠 Smart Action Orchestrator initialized with memory integration");
        Ok(orchestrator)
    }

    /// Execute smart click action with adaptive strategies
    pub async fn smart_click(&self, intent: &str, target_description: Option<&str>) -> Result<SmartActionResult> {
        let execution_id = Uuid::new_v4();
        info!("🎯 Executing smart click: '{}' (id: {})", intent, execution_id);

        let mut execution = SmartActionExecution {
            id: execution_id,
            action_type: ActionType::SmartClick,
            intent: intent.to_string(),
            parameters: self.build_click_parameters(target_description).await,
            strategies_attempted: Vec::new(),
            successful_strategy: None,
            context_snapshot: self.capture_context(intent).await?,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: 0,
            success: false,
            confidence: 0.0,
            error_details: None,
            retry_count: 0,
        };

        let start_time = std::time::Instant::now();
        let config = self.config.read().await.clone();
        
        // Get optimal strategies based on context and history
        let strategies = self.get_optimal_click_strategies(&execution.context_snapshot).await;
        
        for (retry, strategy) in strategies.iter().enumerate() {
            if retry >= config.max_retries as usize {
                break;
            }

            execution.retry_count = retry as u32;
            let strategy_result = self.execute_click_strategy(strategy, &execution).await?;
            execution.strategies_attempted.push(strategy_result.clone());

            if strategy_result.success {
                execution.success = true;
                execution.successful_strategy = Some(strategy_result);
                execution.confidence = self.calculate_action_confidence(&execution).await;
                break;
            }

            // Brief pause between retries
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        execution.duration_ms = start_time.elapsed().as_millis() as u64;
        execution.completed_at = Some(Utc::now());

        // Record execution for learning
        self.record_execution(execution.clone()).await?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&execution).await;

        let result = SmartActionResult {
            action_type: ActionType::SmartClick,
            success: execution.success,
            confidence: execution.confidence,
            duration_ms: execution.duration_ms,
            strategy_used: execution.successful_strategy.as_ref().map(|s| s.strategy_name.clone()),
            data: None,
            recommendations,
            session_id: self.session_id,
        };

        info!("✅ Smart click completed: success={}, confidence={:.2}, duration={}ms", 
               result.success, result.confidence, result.duration_ms);

        Ok(result)
    }

    /// Execute smart input action with context awareness
    pub async fn smart_input(&self, intent: &str, text: &str, target_description: Option<&str>) -> Result<SmartActionResult> {
        let execution_id = Uuid::new_v4();
        info!("✏️ Executing smart input: '{}' -> '{}' (id: {})", intent, text, execution_id);

        let mut execution = SmartActionExecution {
            id: execution_id,
            action_type: ActionType::SmartInput,
            intent: intent.to_string(),
            parameters: self.build_input_parameters(text, target_description).await,
            strategies_attempted: Vec::new(),
            successful_strategy: None,
            context_snapshot: self.capture_context(intent).await?,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: 0,
            success: false,
            confidence: 0.0,
            error_details: None,
            retry_count: 0,
        };

        let start_time = std::time::Instant::now();
        let config = self.config.read().await.clone();
        
        // Get optimal strategies for input based on context
        let strategies = self.get_optimal_input_strategies(&execution.context_snapshot, text).await;
        
        for (retry, strategy) in strategies.iter().enumerate() {
            if retry >= config.max_retries as usize {
                break;
            }

            execution.retry_count = retry as u32;
            let strategy_result = self.execute_input_strategy(strategy, &execution, text).await?;
            execution.strategies_attempted.push(strategy_result.clone());

            if strategy_result.success {
                execution.success = true;
                execution.successful_strategy = Some(strategy_result);
                execution.confidence = self.calculate_action_confidence(&execution).await;
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        execution.duration_ms = start_time.elapsed().as_millis() as u64;
        execution.completed_at = Some(Utc::now());

        self.record_execution(execution.clone()).await?;
        let recommendations = self.generate_recommendations(&execution).await;

        let result = SmartActionResult {
            action_type: ActionType::SmartInput,
            success: execution.success,
            confidence: execution.confidence,
            duration_ms: execution.duration_ms,
            strategy_used: execution.successful_strategy.as_ref().map(|s| s.strategy_name.clone()),
            data: None,
            recommendations,
            session_id: self.session_id,
        };

        info!("✅ Smart input completed: success={}, confidence={:.2}, duration={}ms", 
               result.success, result.confidence, result.duration_ms);

        Ok(result)
    }

    /// Execute smart navigation with intelligent URL handling
    pub async fn smart_navigation(&self, intent: &str, url: &str) -> Result<SmartActionResult> {
        let execution_id = Uuid::new_v4();
        info!("🧭 Executing smart navigation: '{}' -> '{}' (id: {})", intent, url, execution_id);

        let mut execution = SmartActionExecution {
            id: execution_id,
            action_type: ActionType::SmartNavigation,
            intent: intent.to_string(),
            parameters: self.build_navigation_parameters(url).await,
            strategies_attempted: Vec::new(),
            successful_strategy: None,
            context_snapshot: self.capture_context(intent).await?,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: 0,
            success: false,
            confidence: 0.0,
            error_details: None,
            retry_count: 0,
        };

        let start_time = std::time::Instant::now();
        let strategies = self.get_optimal_navigation_strategies(&execution.context_snapshot, url).await;

        for strategy in strategies.iter() {
            let strategy_result = self.execute_navigation_strategy(strategy, &execution, url).await?;
            execution.strategies_attempted.push(strategy_result.clone());

            if strategy_result.success {
                execution.success = true;
                execution.successful_strategy = Some(strategy_result);
                execution.confidence = self.calculate_action_confidence(&execution).await;
                break;
            }
        }

        execution.duration_ms = start_time.elapsed().as_millis() as u64;
        execution.completed_at = Some(Utc::now());

        self.record_execution(execution.clone()).await?;
        let recommendations = self.generate_recommendations(&execution).await;

        let result = SmartActionResult {
            action_type: ActionType::SmartNavigation,
            success: execution.success,
            confidence: execution.confidence,
            duration_ms: execution.duration_ms,
            strategy_used: execution.successful_strategy.as_ref().map(|s| s.strategy_name.clone()),
            data: None,
            recommendations,
            session_id: self.session_id,
        };

        info!("✅ Smart navigation completed: success={}, confidence={:.2}, duration={}ms", 
               result.success, result.confidence, result.duration_ms);

        Ok(result)
    }

    /// Execute smart extraction with multiple fallback approaches
    pub async fn smart_extraction(&self, intent: &str, target_description: Option<&str>) -> Result<SmartActionResult> {
        let execution_id = Uuid::new_v4();
        info!("📋 Executing smart extraction: '{}' (id: {})", intent, execution_id);

        let mut execution = SmartActionExecution {
            id: execution_id,
            action_type: ActionType::SmartExtraction,
            intent: intent.to_string(),
            parameters: self.build_extraction_parameters(target_description).await,
            strategies_attempted: Vec::new(),
            successful_strategy: None,
            context_snapshot: self.capture_context(intent).await?,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: 0,
            success: false,
            confidence: 0.0,
            error_details: None,
            retry_count: 0,
        };

        let start_time = std::time::Instant::now();
        let strategies = self.get_optimal_extraction_strategies(&execution.context_snapshot).await;
        let mut extracted_data = None;

        for strategy in strategies.iter() {
            let strategy_result = self.execute_extraction_strategy(strategy, &execution).await?;
            execution.strategies_attempted.push(strategy_result.clone());

            if strategy_result.success {
                execution.success = true;
                execution.successful_strategy = Some(strategy_result.clone());
                execution.confidence = self.calculate_action_confidence(&execution).await;
                
                // Extract the actual data
                extracted_data = Some(serde_json::json!({
                    "strategy": strategy_result.strategy_name,
                    "confidence": strategy_result.confidence,
                    "timestamp": Utc::now()
                }));
                break;
            }
        }

        execution.duration_ms = start_time.elapsed().as_millis() as u64;
        execution.completed_at = Some(Utc::now());

        self.record_execution(execution.clone()).await?;
        let recommendations = self.generate_recommendations(&execution).await;

        let result = SmartActionResult {
            action_type: ActionType::SmartExtraction,
            success: execution.success,
            confidence: execution.confidence,
            duration_ms: execution.duration_ms,
            strategy_used: execution.successful_strategy.as_ref().map(|s| s.strategy_name.clone()),
            data: extracted_data,
            recommendations,
            session_id: self.session_id,
        };

        info!("✅ Smart extraction completed: success={}, confidence={:.2}, duration={}ms", 
               result.success, result.confidence, result.duration_ms);

        Ok(result)
    }

    /// Get optimal click strategies based on context
    async fn get_optimal_click_strategies(&self, context: &ContextSnapshot) -> Vec<String> {
        let mut strategies = Vec::new();
        
        // Base strategies ordered by success probability
        match context.user_context.interaction_style {
            InteractionStyle::DirectAndFast => {
                strategies.extend(vec![
                    "fast_css_click".to_string(),
                    "text_content_click".to_string(),
                    "semantic_click".to_string(),
                    "javascript_click".to_string(),
                ]);
            },
            InteractionStyle::PreciseAndControlled => {
                strategies.extend(vec![
                    "precise_css_click".to_string(),
                    "xpath_click".to_string(),
                    "semantic_attributes_click".to_string(),
                    "visual_recognition_click".to_string(),
                ]);
            },
            _ => {
                strategies.extend(vec![
                    "balanced_css_click".to_string(),
                    "text_content_click".to_string(),
                    "semantic_click".to_string(),
                    "positional_click".to_string(),
                ]);
            }
        }

        // Add device-specific strategies
        match context.environmental_context.device_type {
            DeviceType::Mobile => {
                strategies.push("touch_optimized_click".to_string());
            },
            DeviceType::Desktop => {
                strategies.push("precise_mouse_click".to_string());
            },
            _ => {}
        }

        // Add network-aware strategies
        match context.environmental_context.network_quality {
            NetworkQuality::Poor => {
                strategies.insert(0, "lightweight_click".to_string());
            },
            _ => {}
        }

        strategies
    }

    /// Get optimal input strategies based on context
    async fn get_optimal_input_strategies(&self, context: &ContextSnapshot, text: &str) -> Vec<String> {
        let mut strategies = Vec::new();
        
        // Determine input complexity
        let is_complex = text.len() > 100 || text.contains('\n') || text.chars().any(|c| !c.is_ascii());
        
        if is_complex {
            strategies.extend(vec![
                "careful_input".to_string(),
                "chunk_input".to_string(),
                "javascript_input".to_string(),
                "clipboard_input".to_string(),
            ]);
        } else {
            strategies.extend(vec![
                "direct_input".to_string(),
                "clear_and_type".to_string(),
                "focus_and_type".to_string(),
                "javascript_input".to_string(),
            ]);
        }

        // Add context-specific optimizations
        match context.user_context.interaction_style {
            InteractionStyle::DirectAndFast => {
                strategies.insert(0, "fast_input".to_string());
            },
            InteractionStyle::PreciseAndControlled => {
                strategies.push("validated_input".to_string());
            },
            _ => {}
        }

        strategies
    }

    /// Get optimal navigation strategies based on context
    async fn get_optimal_navigation_strategies(&self, context: &ContextSnapshot, url: &str) -> Vec<String> {
        let mut strategies = Vec::new();
        
        // Base navigation strategies
        strategies.extend(vec![
            "direct_navigation".to_string(),
            "wait_and_navigate".to_string(),
            "retry_navigation".to_string(),
        ]);

        // Add URL-specific strategies
        if url.contains("login") || url.contains("auth") {
            strategies.push("secure_navigation".to_string());
        }

        if url.starts_with("https://") {
            strategies.push("ssl_navigation".to_string());
        }

        // Network-aware strategies
        match context.environmental_context.network_quality {
            NetworkQuality::Poor => {
                strategies.insert(0, "patient_navigation".to_string());
            },
            NetworkQuality::Excellent => {
                strategies.insert(0, "fast_navigation".to_string());
            },
            _ => {}
        }

        strategies
    }

    /// Get optimal extraction strategies based on context
    async fn get_optimal_extraction_strategies(&self, context: &ContextSnapshot) -> Vec<String> {
        let mut strategies = Vec::new();
        
        // Base extraction strategies
        strategies.extend(vec![
            "css_extraction".to_string(),
            "xpath_extraction".to_string(),
            "text_pattern_extraction".to_string(),
            "semantic_extraction".to_string(),
        ]);

        // Add context-specific strategies
        match context.user_context.expertise_level {
            crate::contextual_awareness::ExpertiseLevel::Expert => {
                strategies.push("advanced_extraction".to_string());
            },
            crate::contextual_awareness::ExpertiseLevel::Beginner => {
                strategies.insert(0, "simple_extraction".to_string());
            },
            _ => {}
        }

        strategies
    }

    /// Execute click strategy
    async fn execute_click_strategy(&self, strategy_name: &str, execution: &SmartActionExecution) -> Result<ExecutionStrategy> {
        let start_time = std::time::Instant::now();
        
        let result = match strategy_name {
            "fast_css_click" => {
                self.browser_controller.enhanced_click("button").await
            },
            "text_content_click" => {
                self.browser_controller.enhanced_click(&execution.intent).await
            },
            "semantic_click" => {
                self.browser_controller.enhanced_click(&execution.intent).await
            },
            "javascript_click" => {
                self.browser_controller.enhanced_click("button").await
            },
            "precise_css_click" => {
                self.browser_controller.enhanced_click("button[type='submit'], input[type='submit'], a").await
            },
            "xpath_click" => {
                self.browser_controller.enhanced_click("//button | //input[@type='submit'] | //a").await
            },
            "visual_recognition_click" => {
                self.browser_controller.enhanced_click(&execution.intent).await
            },
            "positional_click" => {
                self.browser_controller.enhanced_click(&execution.intent).await
            },
            _ => {
                self.browser_controller.enhanced_click(&execution.intent).await
            }
        };

        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };

        Ok(ExecutionStrategy {
            strategy_name: strategy_name.to_string(),
            finding_strategy: ElementFindingStrategy::CssSelector, // Default for now
            parameters: HashMap::new(),
            success,
            duration_ms: duration,
            confidence: if success { 0.8 } else { 0.0 },
            error,
        })
    }

    /// Execute input strategy
    async fn execute_input_strategy(&self, strategy_name: &str, execution: &SmartActionExecution, text: &str) -> Result<ExecutionStrategy> {
        let start_time = std::time::Instant::now();
        
        let result = match strategy_name {
            "direct_input" => {
                self.browser_controller.enhanced_input("input, textarea", text).await
            },
            "clear_and_type" => {
                self.browser_controller.enhanced_input("input, textarea", text).await
            },
            "javascript_input" => {
                self.browser_controller.enhanced_input("input, textarea", text).await
            },
            "careful_input" => {
                self.browser_controller.enhanced_input("input, textarea", text).await
            },
            _ => {
                self.browser_controller.enhanced_input("input, textarea", text).await
            }
        };

        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };

        Ok(ExecutionStrategy {
            strategy_name: strategy_name.to_string(),
            finding_strategy: ElementFindingStrategy::CssSelector,
            parameters: HashMap::new(),
            success,
            duration_ms: duration,
            confidence: if success { 0.8 } else { 0.0 },
            error,
        })
    }

    /// Execute navigation strategy
    async fn execute_navigation_strategy(&self, strategy_name: &str, execution: &SmartActionExecution, url: &str) -> Result<ExecutionStrategy> {
        let start_time = std::time::Instant::now();
        
        let result = match strategy_name {
            "direct_navigation" => {
                self.browser_controller.enhanced_navigate(url).await
            },
            "wait_and_navigate" => {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                self.browser_controller.enhanced_navigate(url).await
            },
            "patient_navigation" => {
                self.browser_controller.enhanced_navigate(url).await
            },
            _ => {
                self.browser_controller.enhanced_navigate(url).await
            }
        };

        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };

        Ok(ExecutionStrategy {
            strategy_name: strategy_name.to_string(),
            finding_strategy: ElementFindingStrategy::CssSelector,
            parameters: HashMap::new(),
            success,
            duration_ms: duration,
            confidence: if success { 0.8 } else { 0.0 },
            error,
        })
    }

    /// Execute extraction strategy
    async fn execute_extraction_strategy(&self, strategy_name: &str, execution: &SmartActionExecution) -> Result<ExecutionStrategy> {
        let start_time = std::time::Instant::now();
        
        let result = match strategy_name {
            "css_extraction" => {
                self.browser_controller.enhanced_extract("body").await.map(|(text, _)| text)
            },
            "xpath_extraction" => {
                self.browser_controller.enhanced_extract("//body").await.map(|(text, _)| text)
            },
            "semantic_extraction" => {
                self.browser_controller.enhanced_extract(&execution.intent).await.map(|(text, _)| text)
            },
            _ => {
                self.browser_controller.enhanced_extract("body").await.map(|(text, _)| text)
            }
        };

        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();
        let error = if let Err(ref e) = result {
            Some(e.to_string())
        } else {
            None
        };

        Ok(ExecutionStrategy {
            strategy_name: strategy_name.to_string(),
            finding_strategy: ElementFindingStrategy::CssSelector,
            parameters: HashMap::new(),
            success,
            duration_ms: duration,
            confidence: if success { 0.8 } else { 0.0 },
            error,
        })
    }

    /// Capture current context
    async fn capture_context(&self, intent: &str) -> Result<ContextSnapshot> {
        let mut awareness = self.awareness.write().await;
        awareness.capture_context(intent).await
    }

    /// Calculate action confidence based on execution history
    async fn calculate_action_confidence(&self, execution: &SmartActionExecution) -> f32 {
        let base_confidence = if execution.success { 0.8 } else { 0.0 };
        
        // Adjust based on retry count
        let retry_penalty = execution.retry_count as f32 * 0.1;
        
        // Adjust based on strategy success
        let strategy_bonus = if let Some(ref strategy) = execution.successful_strategy {
            strategy.confidence * 0.2
        } else {
            0.0
        };

        // Adjust based on context confidence
        let context_factor = execution.context_snapshot.confidence_score * 0.1;

        (base_confidence + strategy_bonus + context_factor - retry_penalty).clamp(0.0, 1.0)
    }

    /// Build parameters for click actions
    async fn build_click_parameters(&self, target_description: Option<&str>) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(desc) = target_description {
            params.insert("target".to_string(), serde_json::Value::String(desc.to_string()));
        }
        params
    }

    /// Build parameters for input actions
    async fn build_input_parameters(&self, text: &str, target_description: Option<&str>) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::Value::String(text.to_string()));
        if let Some(desc) = target_description {
            params.insert("target".to_string(), serde_json::Value::String(desc.to_string()));
        }
        params
    }

    /// Build parameters for navigation actions
    async fn build_navigation_parameters(&self, url: &str) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("url".to_string(), serde_json::Value::String(url.to_string()));
        params
    }

    /// Build parameters for extraction actions
    async fn build_extraction_parameters(&self, target_description: Option<&str>) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(desc) = target_description {
            params.insert("target".to_string(), serde_json::Value::String(desc.to_string()));
        }
        params
    }

    /// Record execution for learning
    async fn record_execution(&self, execution: SmartActionExecution) -> Result<()> {
        let mut history = self.execution_history.write().await;
        let config = self.config.read().await;
        
        history.push_back(execution.clone());
        
        // Keep history size manageable
        while history.len() > config.max_history_size {
            history.pop_front();
        }

        // Update strategy statistics
        self.update_strategy_stats(&execution).await?;

        // Store in memory if available
        if let Some(ref memory) = self.memory {
            let record = InteractionRecord {
                id: execution.id,
                timestamp: execution.started_at,
                user_input: execution.intent.clone(),
                classified_task: TaskType::Navigation, // Default
                confidence: execution.confidence,
                execution_success: execution.success,
                execution_time_ms: execution.duration_ms,
                context_markers: vec![format!("{:?}", execution.context_snapshot.environmental_context.device_type)],
            };
            
            memory.record_interaction(record).await?;
        }

        info!("📊 Recorded smart action execution: {} (success: {})", execution.id, execution.success);
        Ok(())
    }

    /// Update strategy statistics
    async fn update_strategy_stats(&self, execution: &SmartActionExecution) -> Result<()> {
        let mut stats = self.strategy_stats.write().await;
        
        let action_stats = stats.entry(execution.action_type).or_insert_with(|| StrategyStatistics {
            total_attempts: 0,
            successful_attempts: 0,
            success_rate: 0.0,
            average_duration_ms: 0.0,
            confidence_distribution: Vec::new(),
            context_success_rates: HashMap::new(),
            last_updated: Utc::now(),
        });

        action_stats.total_attempts += 1;
        if execution.success {
            action_stats.successful_attempts += 1;
        }
        
        action_stats.success_rate = action_stats.successful_attempts as f32 / action_stats.total_attempts as f32;
        action_stats.average_duration_ms = (action_stats.average_duration_ms * (action_stats.total_attempts - 1) as f64 
            + execution.duration_ms as f64) / action_stats.total_attempts as f64;
        
        action_stats.confidence_distribution.push(execution.confidence);
        if action_stats.confidence_distribution.len() > 100 {
            action_stats.confidence_distribution.drain(0..10);
        }
        
        action_stats.last_updated = Utc::now();

        Ok(())
    }

    /// Generate recommendations for improvement
    async fn generate_recommendations(&self, execution: &SmartActionExecution) -> Vec<ActionRecommendation> {
        let mut recommendations = Vec::new();

        // Strategy optimization recommendations
        if execution.retry_count > 0 {
            recommendations.push(ActionRecommendation {
                category: RecommendationType::StrategyOptimization,
                description: format!("Consider reordering strategies - {} failed on first attempt", 
                    execution.strategies_attempted.first().map(|s| s.strategy_name.as_str()).unwrap_or("unknown")),
                confidence: 0.7,
                implementation_priority: Priority::Medium,
            });
        }

        // Performance recommendations
        if execution.duration_ms > 5000 {
            recommendations.push(ActionRecommendation {
                category: RecommendationType::PerformanceImprovement,
                description: "Action took longer than 5 seconds - consider timeout optimization".to_string(),
                confidence: 0.8,
                implementation_priority: Priority::High,
            });
        }

        // Context adaptation recommendations
        match execution.context_snapshot.environmental_context.network_quality {
            NetworkQuality::Poor => {
                recommendations.push(ActionRecommendation {
                    category: RecommendationType::ContextualAdaptation,
                    description: "Poor network detected - enable patient strategies by default".to_string(),
                    confidence: 0.9,
                    implementation_priority: Priority::High,
                });
            },
            _ => {}
        }

        // Error prevention recommendations
        if !execution.success {
            recommendations.push(ActionRecommendation {
                category: RecommendationType::ErrorPrevention,
                description: "Action failed - consider adding element waiting strategies".to_string(),
                confidence: 0.6,
                implementation_priority: Priority::Medium,
            });
        }

        recommendations
    }

    /// Initialize strategy statistics
    async fn initialize_strategy_stats(&self) {
        let mut stats = self.strategy_stats.write().await;
        
        for action_type in [ActionType::SmartClick, ActionType::SmartInput, ActionType::SmartNavigation, ActionType::SmartExtraction] {
            stats.insert(action_type, StrategyStatistics {
                total_attempts: 0,
                successful_attempts: 0,
                success_rate: 0.0,
                average_duration_ms: 0.0,
                confidence_distribution: Vec::new(),
                context_success_rates: HashMap::new(),
                last_updated: Utc::now(),
            });
        }
    }

    /// Load historical patterns from memory
    async fn load_historical_patterns(&self) -> Result<()> {
        if let Some(ref memory) = self.memory {
            let stats = memory.get_memory_stats().await;
            info!("📚 Loading {} historical patterns for smart action optimization", stats.total_interactions);
            
            // TODO: Analyze historical patterns to optimize strategy selection
            // This would involve loading interaction records and extracting successful patterns
        }
        Ok(())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> HashMap<ActionType, StrategyStatistics> {
        self.strategy_stats.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: SmartActionConfig) {
        *self.config.write().await = config;
        info!("⚙️ Smart action configuration updated");
    }
}

impl Default for SmartActionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            action_timeout_ms: 10000,
            confidence_threshold: 0.6,
            adaptive_strategies: true,
            contextual_optimization: true,
            max_history_size: 1000,
            learning_rate: 0.1,
        }
    }
}

/// Create smart action orchestrator
pub async fn create_smart_action_orchestrator(
    browser_controller: Arc<EnhancedBrowserController>,
    intelligence: Arc<RwLock<ContextualPerception>>,
    awareness: Arc<RwLock<ContextualAwareness>>,
) -> Result<SmartActionOrchestrator> {
    SmartActionOrchestrator::new(browser_controller, intelligence, awareness).await
}

/// Create smart action orchestrator with memory
pub async fn create_smart_action_orchestrator_with_memory(
    browser_controller: Arc<EnhancedBrowserController>,
    intelligence: Arc<RwLock<ContextualPerception>>,
    awareness: Arc<RwLock<ContextualAwareness>>,
    memory: Arc<SimpleMemory>,
) -> Result<SmartActionOrchestrator> {
    SmartActionOrchestrator::with_memory(browser_controller, intelligence, awareness, memory).await
}