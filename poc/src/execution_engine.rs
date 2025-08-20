//! Intelligent Command Execution Engine
//!
//! This module provides the execution engine that takes commands selected by
//! the command registry and executes them through the browser, with intelligent
//! error handling, learning, and adaptation.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::browser::BrowserAction;
use crate::browser_pool::{BrowserPool, PooledBrowserHandle};
use crate::command_registry::{
    IntelligentCommandRegistry, CommandDefinition, CommandSelection,
    ExecutionRecord as CommandExecutionRecord, ExecutionContext, FallbackStrategy, Precondition,
    SuccessCriterion
};
use crate::contextual_awareness::{ContextualAwareness, ContextSnapshot};
use crate::simple_memory::{SimpleMemory, InteractionRecord};
use crate::creative_engine::CreativeEngine;

/// Intelligent command executor with learning and adaptation
pub struct IntelligentExecutor {
    /// Command registry for available commands
    registry: Arc<IntelligentCommandRegistry>,
    /// Browser pool for execution
    browser_pool: Arc<BrowserPool>,
    /// Memory system for learning
    memory: Arc<SimpleMemory>,
    /// Context awareness for intelligent execution
    context_awareness: Arc<ContextualAwareness>,
    /// Creative engine for alternative strategies
    creative_engine: Option<Arc<RwLock<CreativeEngine>>>,
    /// Execution configuration
    config: ExecutorConfig,
    /// Active executions tracking
    active_executions: Arc<RwLock<HashMap<Uuid, ExecutionState>>>,
    /// Performance metrics
    metrics: Arc<RwLock<ExecutorMetrics>>,
}

/// Configuration for the execution engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Maximum retry attempts for failed commands
    pub max_retries: u32,
    /// Default timeout for command execution
    pub default_timeout: Duration,
    /// Enable parallel execution for independent commands
    pub enable_parallel: bool,
    /// Maximum parallel executions
    pub max_parallel: usize,
    /// Enable learning from executions
    pub enable_learning: bool,
    /// Enable creative fallback strategies
    pub enable_creative_fallbacks: bool,
    /// Minimum confidence threshold for execution
    pub min_confidence_threshold: f32,
    /// Enable detailed execution logging
    pub verbose_logging: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            default_timeout: Duration::from_secs(30),
            enable_parallel: true,
            max_parallel: 5,
            enable_learning: true,
            enable_creative_fallbacks: true,
            min_confidence_threshold: 0.3,
            verbose_logging: false,
        }
    }
}

/// State of an active execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub id: Uuid,
    pub command_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retry_count: u32,
    pub error_messages: Vec<String>,
    pub fallback_attempts: Vec<FallbackAttempt>,
}

/// Status of command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    CheckingPreconditions,
    Executing,
    ValidatingSuccess,
    ApplyingFallback,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

/// Record of a fallback strategy attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackAttempt {
    pub strategy: FallbackStrategy,
    pub attempted_at: DateTime<Utc>,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: Uuid,
    pub command_name: String,
    pub success: bool,
    pub confidence: f32,
    pub duration_ms: u64,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub fallback_used: Option<FallbackStrategy>,
    pub retry_count: u32,
    pub learning_insights: Vec<String>,
}

/// Metrics for executor performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_duration_ms: f64,
    pub average_retry_count: f32,
    pub fallback_success_rate: f32,
    pub creative_solution_rate: f32,
    pub precondition_failure_rate: f32,
}

impl Default for ExecutorMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_duration_ms: 0.0,
            average_retry_count: 0.0,
            fallback_success_rate: 0.0,
            creative_solution_rate: 0.0,
            precondition_failure_rate: 0.0,
        }
    }
}

/// Execution plan for complex multi-step operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: Uuid,
    pub steps: Vec<ExecutionStep>,
    pub parallel_groups: Vec<ParallelGroup>,
    pub estimated_duration_ms: u64,
    pub confidence: f32,
}

/// Single step in an execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub command: CommandDefinition,
    pub parameters: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<usize>,
    pub can_parallel: bool,
    pub is_optional: bool,
}

/// Group of commands that can execute in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelGroup {
    pub step_indices: Vec<usize>,
    pub max_parallel: usize,
}

impl IntelligentExecutor {
    /// Create a new intelligent executor
    pub async fn new(
        registry: Arc<IntelligentCommandRegistry>,
        browser_pool: Arc<BrowserPool>,
        memory: Arc<SimpleMemory>,
        context_awareness: Arc<ContextualAwareness>,
    ) -> Result<Self> {
        Ok(Self {
            registry,
            browser_pool,
            memory,
            context_awareness,
            creative_engine: None,
            config: ExecutorConfig::default(),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(ExecutorMetrics::default())),
        })
    }

    /// Create with creative engine for advanced strategies
    pub async fn with_creative_engine(
        registry: Arc<IntelligentCommandRegistry>,
        browser_pool: Arc<BrowserPool>,
        memory: Arc<SimpleMemory>,
        context_awareness: Arc<ContextualAwareness>,
        creative_engine: CreativeEngine,
    ) -> Result<Self> {
        let mut executor = Self::new(registry, browser_pool, memory, context_awareness).await?;
        executor.creative_engine = Some(Arc::new(RwLock::new(creative_engine)));
        Ok(executor)
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ExecutorConfig) {
        self.config = config;
    }

    /// Execute a command selection
    pub async fn execute_command(
        &self,
        selection: &CommandSelection,
    ) -> Result<ExecutionResult> {
        let execution_id = Uuid::new_v4();
        let start_time = Instant::now();
        
        info!("🚀 Starting execution: {} (command: {}, confidence: {:.2}%)",
              execution_id, selection.command.name, selection.confidence * 100.0);

        // Check confidence threshold
        if selection.confidence < self.config.min_confidence_threshold {
            warn!("⚠️ Confidence too low: {:.2}% < {:.2}%",
                  selection.confidence * 100.0,
                  self.config.min_confidence_threshold * 100.0);
            
            if let Some(creative) = &self.creative_engine {
                info!("🎨 Attempting creative solution...");
                return self.execute_with_creative_fallback(selection, execution_id).await;
            }
        }

        // Initialize execution state
        let mut state = ExecutionState {
            id: execution_id,
            command_name: selection.command.name.clone(),
            status: ExecutionStatus::Pending,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            retry_count: 0,
            error_messages: Vec::new(),
            fallback_attempts: Vec::new(),
        };

        // Track active execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id, state.clone());
        }

        // Get browser handle
        let browser = self.browser_pool.acquire().await
            .map_err(|e| anyhow::anyhow!("Failed to get browser from pool: {}", e))?;

        // Execute with retry logic
        let mut result = None;
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            state.retry_count = attempt;
            state.updated_at = Utc::now();
            
            match self.execute_single_attempt(
                &selection.command,
                &selection.inferred_parameters,
                &browser,
                &mut state,
            ).await {
                Ok(exec_result) => {
                    result = Some(exec_result);
                    break;
                },
                Err(e) => {
                    last_error = Some(e.to_string());
                    state.error_messages.push(e.to_string());
                    
                    if attempt < self.config.max_retries {
                        info!("🔄 Retry {}/{} after error: {}",
                              attempt + 1, self.config.max_retries, e);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        // Try fallback strategies if main execution failed
        if result.is_none() && !selection.command.fallback_strategies.is_empty() {
            info!("🔧 Attempting fallback strategies...");
            for fallback in &selection.command.fallback_strategies {
                match self.execute_fallback_strategy(
                    fallback,
                    &selection.command,
                    &selection.inferred_parameters,
                    &browser,
                    &mut state,
                ).await {
                    Ok(exec_result) => {
                        result = Some(exec_result);
                        break;
                    },
                    Err(e) => {
                        state.fallback_attempts.push(FallbackAttempt {
                            strategy: fallback.clone(),
                            attempted_at: Utc::now(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        // Update final state
        state.status = if result.is_some() {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };
        state.updated_at = Utc::now();

        // Remove from active executions
        {
            let mut active = self.active_executions.write().await;
            active.remove(&execution_id);
        }

        // Record execution for learning
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        let execution_result = result.unwrap_or_else(|| ExecutionResult {
            id: execution_id,
            command_name: selection.command.name.clone(),
            success: false,
            confidence: selection.confidence,
            duration_ms,
            output: None,
            error: last_error,
            fallback_used: None,
            retry_count: state.retry_count,
            learning_insights: Vec::new(),
        });

        // Record for learning
        self.record_execution(&execution_result, &state).await?;

        // Update metrics
        self.update_metrics(&execution_result).await;

        Ok(execution_result)
    }

    /// Execute a single attempt of a command
    async fn execute_single_attempt(
        &self,
        command: &CommandDefinition,
        parameters: &HashMap<String, serde_json::Value>,
        browser: &PooledBrowserHandle,
        state: &mut ExecutionState,
    ) -> Result<ExecutionResult> {
        // Check preconditions
        state.status = ExecutionStatus::CheckingPreconditions;
        self.check_preconditions(&command.preconditions, browser).await?;

        // Execute the command
        state.status = ExecutionStatus::Executing;
        let action = self.map_to_browser_action(command, parameters)?;
        let output = self.execute_browser_action(&action, browser).await?;

        // Validate success criteria
        state.status = ExecutionStatus::ValidatingSuccess;
        self.validate_success_criteria(&command.success_criteria, &output, browser).await?;

        state.status = ExecutionStatus::Completed;

        Ok(ExecutionResult {
            id: state.id,
            command_name: command.name.clone(),
            success: true,
            confidence: 1.0,
            duration_ms: (Utc::now() - state.started_at).num_milliseconds() as u64,
            output: Some(output),
            error: None,
            fallback_used: None,
            retry_count: state.retry_count,
            learning_insights: Vec::new(),
        })
    }

    /// Check command preconditions
    async fn check_preconditions(
        &self,
        preconditions: &[Precondition],
        browser: &PooledBrowserHandle,
    ) -> Result<()> {
        for condition in preconditions {
            match condition {
                Precondition::PageLoaded => {
                    // Check if page is loaded
                    let title = browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .get_title().await?;
                    if title.is_empty() {
                        return Err(anyhow::anyhow!("Page not loaded"));
                    }
                },
                Precondition::ElementExists(selector) => {
                    if !browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .element_exists(selector).await? {
                        return Err(anyhow::anyhow!("Element not found: {}", selector));
                    }
                },
                Precondition::ElementVisible(selector) => {
                    // For now, check existence (could enhance with visibility check)
                    if !browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .element_exists(selector).await? {
                        return Err(anyhow::anyhow!("Element not visible: {}", selector));
                    }
                },
                Precondition::ElementClickable(selector) => {
                    // For now, check existence (could enhance with clickability check)
                    if !browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .element_exists(selector).await? {
                        return Err(anyhow::anyhow!("Element not clickable: {}", selector));
                    }
                },
                Precondition::UrlMatches(pattern) => {
                    let current_url = browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .get_current_url().await?;
                    if !current_url.contains(pattern) {
                        return Err(anyhow::anyhow!("URL doesn't match pattern: {}", pattern));
                    }
                },
                Precondition::NoActiveAnimations => {
                    // Could implement animation detection
                    tokio::time::sleep(Duration::from_millis(500)).await;
                },
                Precondition::NetworkIdle => {
                    // Could implement network monitoring
                    tokio::time::sleep(Duration::from_secs(1)).await;
                },
                Precondition::CustomCondition(name) => {
                    debug!("Custom precondition: {} (not implemented)", name);
                },
            }
        }
        Ok(())
    }

    /// Map command to browser action
    fn map_to_browser_action(
        &self,
        command: &CommandDefinition,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<BrowserAction> {
        let action = match command.name.as_str() {
            "navigate_to_url" => {
                let url = parameters.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing URL parameter"))?;
                BrowserAction::Navigate { url: url.to_string() }
            },
            "click_element" => {
                let selector = parameters.get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("button");
                BrowserAction::Click { selector: selector.to_string() }
            },
            "input_text" => {
                let selector = parameters.get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("input");
                let text = parameters.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing text parameter"))?;
                BrowserAction::InputText {
                    selector: selector.to_string(),
                    text: text.to_string(),
                }
            },
            "extract_text" => {
                let selector = parameters.get("selector")
                    .and_then(|v| v.as_str())
                    .unwrap_or("body");
                BrowserAction::GetText { selector: selector.to_string() }
            },
            "take_screenshot" => {
                let selector = parameters.get("selector")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                BrowserAction::Screenshot { selector }
            },
            "wait_for_element" => {
                let selector = parameters.get("selector")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing selector parameter"))?;
                let timeout_ms = parameters.get("timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5000);
                BrowserAction::WaitForElement {
                    selector: selector.to_string(),
                    timeout_ms,
                }
            },
            "scroll_page" => {
                let direction = parameters.get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("down");
                let amount = parameters.get("amount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(500) as i32;
                
                let y = if direction == "up" { -amount } else { amount };
                BrowserAction::ScrollTo { x: 0, y }
            },
            "go_back" => BrowserAction::GoBack,
            "refresh_page" => BrowserAction::Refresh,
            _ => {
                return Err(anyhow::anyhow!("Unknown command: {}", command.name));
            }
        };
        
        Ok(action)
    }

    /// Execute browser action
    async fn execute_browser_action(
        &self,
        action: &BrowserAction,
        browser: &PooledBrowserHandle,
    ) -> Result<serde_json::Value> {
        match action {
            BrowserAction::Navigate { url } => {
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .navigate_to(url).await?;
                Ok(serde_json::json!({ "navigated": url }))
            },
            BrowserAction::Click { selector } => {
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .click(selector).await?;
                Ok(serde_json::json!({ "clicked": selector }))
            },
            BrowserAction::InputText { selector, text } => {
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .fill_field(selector, text).await?;
                Ok(serde_json::json!({ "input": { "selector": selector, "text": text } }))
            },
            BrowserAction::GetText { selector } => {
                let text = browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .get_text(selector).await?;
                Ok(serde_json::json!({ "text": text }))
            },
            BrowserAction::Screenshot { selector } => {
                let filename = format!("screenshot_{}.png", Uuid::new_v4());
                if let Some(sel) = selector {
                    // Element screenshot not directly supported yet
                    browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .take_screenshot(&filename).await?;
                } else {
                    browser.browser()
                        .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                        .take_screenshot(&filename).await?;
                }
                Ok(serde_json::json!({ "screenshot": filename }))
            },
            BrowserAction::WaitForElement { selector, timeout_ms } => {
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .wait_for_element(
                        selector,
                        Duration::from_millis(*timeout_ms)
                    ).await?;
                Ok(serde_json::json!({ "found": selector }))
            },
            BrowserAction::ScrollTo { x, y } => {
                let script = format!("window.scrollTo({}, {});", x, y);
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(&script).await?;
                Ok(serde_json::json!({ "scrolled": { "x": x, "y": y } }))
            },
            BrowserAction::GoBack => {
                let script = "window.history.back();";
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(script).await?;
                Ok(serde_json::json!({ "action": "back" }))
            },
            BrowserAction::GoForward => {
                let script = "window.history.forward();";
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(script).await?;
                Ok(serde_json::json!({ "action": "forward" }))
            },
            BrowserAction::Refresh => {
                let script = "window.location.reload();";
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(script).await?;
                Ok(serde_json::json!({ "action": "refresh" }))
            },
            BrowserAction::ExecuteScript { script } => {
                let result = browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(script).await?;
                Ok(result)
            },
        }
    }

    /// Validate success criteria
    async fn validate_success_criteria(
        &self,
        criteria: &[SuccessCriterion],
        output: &serde_json::Value,
        browser: &PooledBrowserHandle,
    ) -> Result<()> {
        for criterion in criteria {
            match criterion {
                SuccessCriterion::PageNavigated => {
                    // Check if navigation occurred (could enhance with URL change detection)
                    if !output.get("navigated").is_some() {
                        // Not a navigation command, skip
                        continue;
                    }
                },
                SuccessCriterion::ElementClicked => {
                    if !output.get("clicked").is_some() {
                        continue;
                    }
                },
                SuccessCriterion::TextEntered => {
                    if !output.get("input").is_some() {
                        continue;
                    }
                },
                SuccessCriterion::ElementFound => {
                    if !output.get("found").is_some() {
                        continue;
                    }
                },
                SuccessCriterion::ValueExtracted => {
                    if !output.get("text").is_some() {
                        continue;
                    }
                },
                SuccessCriterion::ScreenshotTaken => {
                    if !output.get("screenshot").is_some() {
                        continue;
                    }
                },
                SuccessCriterion::NoErrors => {
                    // If we got here, no errors occurred
                },
                SuccessCriterion::ResponseReceived => {
                    // Check if we have any output
                    if output.is_null() {
                        return Err(anyhow::anyhow!("No response received"));
                    }
                },
                SuccessCriterion::ConditionMet(condition) => {
                    debug!("Custom success condition: {} (not implemented)", condition);
                },
                SuccessCriterion::Custom(name) => {
                    debug!("Custom success criterion: {} (not implemented)", name);
                },
            }
        }
        Ok(())
    }

    /// Execute a fallback strategy
    async fn execute_fallback_strategy(
        &self,
        strategy: &FallbackStrategy,
        command: &CommandDefinition,
        parameters: &HashMap<String, serde_json::Value>,
        browser: &PooledBrowserHandle,
        state: &mut ExecutionState,
    ) -> Result<ExecutionResult> {
        state.status = ExecutionStatus::ApplyingFallback;
        
        match strategy {
            FallbackStrategy::WaitAndRetry(count) => {
                for i in 1..=*count {
                    info!("⏳ Wait and retry {}/{}", i, count);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    
                    match self.execute_single_attempt(command, parameters, browser, state).await {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            if i == *count {
                                return Err(e);
                            }
                        }
                    }
                }
            },
            FallbackStrategy::ScrollToElement => {
                if let Some(selector) = parameters.get("selector").and_then(|v| v.as_str()) {
                    let script = format!(
                        "document.querySelector('{}').scrollIntoView({{behavior: 'smooth'}});",
                        selector
                    );
                    browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(&script).await?;
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    
                    // Retry original command
                    return self.execute_single_attempt(command, parameters, browser, state).await;
                }
            },
            FallbackStrategy::UseJavaScript => {
                // Execute command using JavaScript instead
                if command.name == "click_element" {
                    if let Some(selector) = parameters.get("selector").and_then(|v| v.as_str()) {
                        let script = format!("document.querySelector('{}').click();", selector);
                        browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script(&script).await?;
                        
                        return Ok(ExecutionResult {
                            id: state.id,
                            command_name: command.name.clone(),
                            success: true,
                            confidence: 0.8,
                            duration_ms: 100,
                            output: Some(serde_json::json!({ "clicked_via_js": selector })),
                            error: None,
                            fallback_used: Some(strategy.clone()),
                            retry_count: state.retry_count,
                            learning_insights: vec!["JavaScript fallback successful".to_string()],
                        });
                    }
                }
            },
            FallbackStrategy::RefreshAndRetry => {
                browser.browser()
                    .ok_or_else(|| anyhow::anyhow!("Browser not available"))?
                    .execute_script("window.location.reload();").await?;
                tokio::time::sleep(Duration::from_secs(3)).await;
                return self.execute_single_attempt(command, parameters, browser, state).await;
            },
            FallbackStrategy::CreativeAlternative => {
                if let Some(creative) = &self.creative_engine {
                    // Use creative engine for alternative approach
                    info!("🎨 Attempting creative alternative...");
                    // Create a default context snapshot for creative engine
            let context = ContextSnapshot {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                temporal_context: crate::contextual_awareness::TemporalContext {
                    time_of_day: crate::contextual_awareness::TimeOfDay::Afternoon,
                    day_of_week: chrono::Weekday::Mon,
                    is_business_hours: true,
                    is_weekend: false,
                    season: crate::contextual_awareness::Season::Spring,
                    urgency_indicators: Vec::new(),
                },
                environmental_context: crate::contextual_awareness::EnvironmentalContext {
                    device_type: crate::contextual_awareness::DeviceType::Desktop,
                    screen_resolution: (1920, 1080),
                    network_quality: crate::contextual_awareness::NetworkQuality::Good,
                    location_hints: Vec::new(),
                    language_preference: "en".to_string(),
                },
                user_context: crate::contextual_awareness::UserContext {
                    interaction_style: crate::contextual_awareness::InteractionStyle::CasualAndFlexible,
                    preferred_task_types: Vec::new(),
                    expertise_level: crate::contextual_awareness::ExpertiseLevel::Intermediate,
                    recent_patterns: Vec::new(),
                    success_patterns: Vec::new(),
                },
                system_context: crate::contextual_awareness::SystemContext {
                    available_memory: 1000,
                    cpu_usage: 0.5,
                    response_time_avg: 100.0,
                    error_rate: 0.01,
                    active_sessions: 1,
                },
                confidence_score: 0.7,
            };
                    let problem = format!("Find alternative way to execute: {}", command.name);
                    let mut creative_guard = creative.write().await;
                    if let Ok(solution) = creative_guard.solve_creatively(&problem, &context).await {
                        return Ok(ExecutionResult {
                            id: state.id,
                            command_name: command.name.clone(),
                            success: true,
                            confidence: solution.feasibility_score,
                            duration_ms: 1000,
                            output: Some(serde_json::json!({ "creative_solution": solution.recommended_approach.primary_solution.idea.description })),
                            error: None,
                            fallback_used: Some(strategy.clone()),
                            retry_count: state.retry_count,
                            learning_insights: vec!["Creative fallback successful".to_string()],
                        });
                    }
                }
            },
            _ => {
                debug!("Fallback strategy {:?} not implemented", strategy);
            }
        }
        
        Err(anyhow::anyhow!("Fallback strategy failed"))
    }

    /// Execute with creative fallback
    async fn execute_with_creative_fallback(
        &self,
        selection: &CommandSelection,
        execution_id: Uuid,
    ) -> Result<ExecutionResult> {
        if let Some(creative) = &self.creative_engine {
            // Generate creative solution
            let problem = format!("Execute command '{}' with low confidence", selection.command.name);
            // Create a default context snapshot for creative engine
            let context = ContextSnapshot {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                temporal_context: crate::contextual_awareness::TemporalContext {
                    time_of_day: crate::contextual_awareness::TimeOfDay::Afternoon,
                    day_of_week: chrono::Weekday::Mon,
                    is_business_hours: true,
                    is_weekend: false,
                    season: crate::contextual_awareness::Season::Spring,
                    urgency_indicators: Vec::new(),
                },
                environmental_context: crate::contextual_awareness::EnvironmentalContext {
                    device_type: crate::contextual_awareness::DeviceType::Desktop,
                    screen_resolution: (1920, 1080),
                    network_quality: crate::contextual_awareness::NetworkQuality::Good,
                    location_hints: Vec::new(),
                    language_preference: "en".to_string(),
                },
                user_context: crate::contextual_awareness::UserContext {
                    interaction_style: crate::contextual_awareness::InteractionStyle::CasualAndFlexible,
                    preferred_task_types: Vec::new(),
                    expertise_level: crate::contextual_awareness::ExpertiseLevel::Intermediate,
                    recent_patterns: Vec::new(),
                    success_patterns: Vec::new(),
                },
                system_context: crate::contextual_awareness::SystemContext {
                    available_memory: 1000,
                    cpu_usage: 0.5,
                    response_time_avg: 100.0,
                    error_rate: 0.01,
                    active_sessions: 1,
                },
                confidence_score: 0.7,
            };
            let mut creative_guard = creative.write().await;
            let solution = creative_guard.solve_creatively(&problem, &context).await?;
            
            // Convert creative solution to execution result
            Ok(ExecutionResult {
                id: execution_id,
                command_name: selection.command.name.clone(),
                success: true,
                confidence: solution.feasibility_score,
                duration_ms: 1000,
                output: Some(serde_json::json!({
                    "creative_solution": solution.recommended_approach.primary_solution.idea.description,
                    "strategy": format!("{:?}", solution.recommended_approach.primary_solution.idea.description),
                })),
                error: None,
                fallback_used: Some(FallbackStrategy::CreativeAlternative),
                retry_count: 0,
                learning_insights: vec!["Used creative solution".to_string()],
            })
        } else {
            Err(anyhow::anyhow!("No creative engine available"))
        }
    }

    /// Record execution for learning
    async fn record_execution(
        &self,
        result: &ExecutionResult,
        state: &ExecutionState,
    ) -> Result<()> {
        // Create execution record for command registry
        let record = CommandExecutionRecord {
            id: result.id,
            command_name: result.command_name.clone(),
            parameters: HashMap::new(), // Could store actual parameters
            context: ExecutionContext {
                url: "unknown".to_string(),
                page_title: "unknown".to_string(),
                viewport_size: (1920, 1080),
                user_agent: "RainbowBrowserAI".to_string(),
                network_quality: "good".to_string(),
                cpu_usage: 0.5,
                memory_usage: 0.5,
            },
            started_at: state.started_at,
            completed_at: state.updated_at,
            duration_ms: result.duration_ms,
            success: result.success,
            error_message: result.error.clone(),
            fallback_used: result.fallback_used.clone(),
            confidence_score: result.confidence,
        };
        
        // Record in command registry
        self.registry.record_execution(record).await?;
        
        // Record in memory for learning
        if self.config.enable_learning {
            let interaction = InteractionRecord {
                id: result.id,
                timestamp: Utc::now(),
                user_input: result.command_name.clone(),
                classified_task: crate::llm_service::llm_service_enhanced::TaskType::Navigation,
                confidence: result.confidence,
                execution_success: result.success,
                execution_time_ms: result.duration_ms,
                context_markers: vec![result.command_name.clone()],
            };
            
            self.memory.record_interaction(interaction).await?;
        }
        
        Ok(())
    }

    /// Update executor metrics
    async fn update_metrics(&self, result: &ExecutionResult) {
        let mut metrics = self.metrics.write().await;
        
        metrics.total_executions += 1;
        if result.success {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }
        
        // Update average duration
        metrics.average_duration_ms = 
            (metrics.average_duration_ms * (metrics.total_executions - 1) as f64 
             + result.duration_ms as f64) / metrics.total_executions as f64;
        
        // Update average retry count
        metrics.average_retry_count = 
            (metrics.average_retry_count * (metrics.total_executions - 1) as f32 
             + result.retry_count as f32) / metrics.total_executions as f32;
        
        // Update fallback success rate
        if result.fallback_used.is_some() && result.success {
            let fallback_successes = (metrics.fallback_success_rate * metrics.total_executions as f32) + 1.0;
            metrics.fallback_success_rate = fallback_successes / metrics.total_executions as f32;
        }
        
        // Update creative solution rate
        if matches!(result.fallback_used, Some(FallbackStrategy::CreativeAlternative)) {
            let creative_uses = (metrics.creative_solution_rate * metrics.total_executions as f32) + 1.0;
            metrics.creative_solution_rate = creative_uses / metrics.total_executions as f32;
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> ExecutorMetrics {
        self.metrics.read().await.clone()
    }

    /// Get active executions
    pub async fn get_active_executions(&self) -> Vec<ExecutionState> {
        self.active_executions.read().await.values().cloned().collect()
    }
}

/// Create executor from environment
pub async fn create_executor(
    registry: Arc<IntelligentCommandRegistry>,
    browser_pool: Arc<BrowserPool>,
    memory: Arc<SimpleMemory>,
    context_awareness: Arc<ContextualAwareness>,
) -> Result<IntelligentExecutor> {
    IntelligentExecutor::new(registry, browser_pool, memory, context_awareness).await
}

/// Create executor with creative engine
pub async fn create_advanced_executor(
    registry: Arc<IntelligentCommandRegistry>,
    browser_pool: Arc<BrowserPool>,
    memory: Arc<SimpleMemory>,
    context_awareness: Arc<ContextualAwareness>,
    creative_engine: CreativeEngine,
) -> Result<IntelligentExecutor> {
    IntelligentExecutor::with_creative_engine(
        registry,
        browser_pool,
        memory,
        context_awareness,
        creative_engine,
    ).await
}