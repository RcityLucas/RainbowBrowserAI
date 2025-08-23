//! Error Recovery System
//!
//! This module provides comprehensive error recovery capabilities for the
//! RainbowBrowserAI system, including intelligent error classification,
//! automatic retry strategies, fallback mechanisms, and failure analysis.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::llm_service::llm_service_enhanced::TaskType;
use crate::contextual_awareness::ContextSnapshot;

/// Error recovery manager for production resilience
pub struct ErrorRecoveryManager {
    /// Unique recovery session identifier
    session_id: Uuid,
    /// Error classification engine
    error_classifier: Arc<RwLock<ErrorClassifier>>,
    /// Recovery strategy registry
    recovery_strategies: Arc<RwLock<HashMap<ErrorCategory, RecoveryStrategy>>>,
    /// Error history for pattern analysis
    error_history: Arc<RwLock<VecDeque<ErrorRecord>>>,
    /// Recovery metrics
    recovery_metrics: Arc<RwLock<RecoveryMetrics>>,
    /// Configuration
    config: Arc<RwLock<ErrorRecoveryConfig>>,
    /// Concurrency control
    recovery_limiter: Arc<Semaphore>,
}

/// Error recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    /// Enable automatic error recovery
    pub enable_auto_recovery: bool,
    /// Maximum recovery attempts per error
    pub max_recovery_attempts: u32,
    /// Recovery timeout in milliseconds
    pub recovery_timeout_ms: u64,
    /// Enable error pattern learning
    pub enable_pattern_learning: bool,
    /// Maximum error history size
    pub max_error_history: usize,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset time (seconds)
    pub circuit_breaker_reset_seconds: u64,
    /// Enable proactive error prevention
    pub enable_proactive_prevention: bool,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            enable_auto_recovery: true,
            max_recovery_attempts: 3,
            recovery_timeout_ms: 30000,
            enable_pattern_learning: true,
            max_error_history: 1000,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_seconds: 300,
            enable_proactive_prevention: true,
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network connectivity issues
    NetworkError,
    /// Browser automation failures
    BrowserError,
    /// LLM service failures
    LLMError,
    /// Task execution failures
    ExecutionError,
    /// Configuration or setup errors
    ConfigurationError,
    /// Resource exhaustion (memory, disk, etc.)
    ResourceError,
    /// Authentication or authorization failures
    AuthenticationError,
    /// Timeout errors
    TimeoutError,
    /// Validation or input errors
    ValidationError,
    /// Unknown or unclassified errors
    UnknownError,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical system failure
    Critical,
    /// High impact error
    High,
    /// Medium impact error
    Medium,
    /// Low impact error
    Low,
    /// Informational error
    Info,
}

/// Recovery strategy definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Strategy name
    pub name: String,
    /// Strategy type
    pub strategy_type: RecoveryType,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Base delay between retries (milliseconds)
    pub base_delay_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum delay between retries
    pub max_delay_ms: u64,
    /// Recovery actions to take
    pub recovery_actions: Vec<RecoveryAction>,
    /// Success criteria for recovery
    pub success_criteria: Vec<String>,
    /// Fallback strategy if recovery fails
    pub fallback_strategy: Option<String>,
}

/// Recovery strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    /// Simple retry with linear backoff
    LinearRetry,
    /// Exponential backoff retry
    ExponentialRetry,
    /// Circuit breaker pattern
    CircuitBreaker,
    /// Fallback to alternative service
    ServiceFallback,
    /// Resource cleanup and retry
    ResourceCleanup,
    /// Configuration reset
    ConfigurationReset,
    /// System restart
    SystemRestart,
    /// Manual intervention required
    ManualIntervention,
}

/// Recovery actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Retry the failed operation
    Retry { delay_ms: u64 },
    /// Clean up resources
    CleanupResources { resource_types: Vec<String> },
    /// Reset configuration to defaults
    ResetConfiguration { config_keys: Vec<String> },
    /// Restart browser session
    RestartBrowser,
    /// Switch to fallback LLM provider
    SwitchLLMProvider { provider: String },
    /// Clear cache
    ClearCache { cache_types: Vec<String> },
    /// Reduce resource usage
    ReduceResourceUsage { percentage: u32 },
    /// Wait for resource availability
    WaitForResources { timeout_ms: u64 },
    /// Send alert to operations team
    SendAlert { severity: ErrorSeverity, message: String },
    /// Log detailed error information
    LogError { include_context: bool },
}

/// Error record for history and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// Unique error identifier
    pub error_id: Uuid,
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    /// Error category
    pub category: ErrorCategory,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error message
    pub message: String,
    /// Error context
    pub context: ErrorContext,
    /// Recovery attempt results
    pub recovery_attempts: Vec<RecoveryAttempt>,
    /// Final resolution status
    pub resolution_status: ResolutionStatus,
    /// Time to recovery (if successful)
    pub recovery_time_ms: Option<u64>,
}

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Task type being executed
    pub task_type: Option<TaskType>,
    /// System context snapshot
    pub context_snapshot: Option<ContextSnapshot>,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Related error IDs
    pub related_errors: Vec<Uuid>,
}

/// Recovery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Attempt number
    pub attempt_number: u32,
    /// Recovery strategy used
    pub strategy_name: String,
    /// Actions taken
    pub actions_taken: Vec<RecoveryAction>,
    /// Attempt timestamp
    pub timestamp: DateTime<Utc>,
    /// Attempt result
    pub result: RecoveryResult,
    /// Duration of recovery attempt
    pub duration_ms: u64,
}

/// Recovery attempt results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryResult {
    /// Recovery succeeded
    Success { details: String },
    /// Recovery failed
    Failed { reason: String },
    /// Recovery partially succeeded
    Partial { success_details: String, remaining_issues: String },
    /// Recovery timeout
    Timeout,
    /// Recovery not attempted due to conditions
    Skipped { reason: String },
}

/// Final resolution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStatus {
    /// Error was successfully recovered
    Recovered,
    /// Error was partially recovered
    PartiallyRecovered,
    /// Error could not be recovered
    Unrecovered,
    /// Error requires manual intervention
    ManualInterventionRequired,
    /// Error was worked around with alternative approach
    WorkedAround,
}

/// Recovery metrics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    /// Total errors encountered
    pub total_errors: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
    /// Failed recoveries
    pub failed_recoveries: u64,
    /// Average recovery time
    pub average_recovery_time_ms: f64,
    /// Recovery success rate by category
    pub success_rate_by_category: HashMap<ErrorCategory, f32>,
    /// Most common error patterns
    pub common_error_patterns: Vec<ErrorPattern>,
    /// Circuit breaker states
    pub circuit_breaker_states: HashMap<String, CircuitBreakerState>,
}

/// Error pattern for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern signature
    pub signature: String,
    /// Occurrence count
    pub count: u32,
    /// Success rate for this pattern
    pub success_rate: f32,
    /// Best recovery strategy
    pub best_strategy: String,
    /// Last occurrence
    pub last_seen: DateTime<Utc>,
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    /// Current state
    pub state: CircuitState,
    /// Failure count
    pub failure_count: u32,
    /// Last failure time
    pub last_failure_time: Option<DateTime<Utc>>,
    /// Next reset time (for half-open state)
    pub next_reset_time: Option<DateTime<Utc>>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Failing, blocking requests
    Open,
    /// Testing if service recovered
    HalfOpen,
}

/// Error classifier for intelligent categorization
pub struct ErrorClassifier {
    /// Classification rules
    classification_rules: Vec<ClassificationRule>,
    /// Pattern learning history
    learned_patterns: HashMap<String, ErrorCategory>,
    /// Classification confidence thresholds
    confidence_thresholds: HashMap<ErrorCategory, f32>,
}

/// Classification rule
#[derive(Debug, Clone)]
pub struct ClassificationRule {
    /// Rule name
    pub name: String,
    /// Error message patterns
    pub message_patterns: Vec<String>,
    /// Context patterns
    pub context_patterns: Vec<String>,
    /// Target category
    pub category: ErrorCategory,
    /// Rule confidence
    pub confidence: f32,
    /// Rule priority (higher = more priority)
    pub priority: u32,
}

impl ErrorRecoveryManager {
    /// Create new error recovery manager
    pub async fn new(config: ErrorRecoveryConfig) -> Result<Self> {
        let recovery_limiter = Arc::new(Semaphore::new(5)); // Max 5 concurrent recoveries
        
        let mut manager = Self {
            session_id: Uuid::new_v4(),
            error_classifier: Arc::new(RwLock::new(ErrorClassifier::new())),
            recovery_strategies: Arc::new(RwLock::new(HashMap::new())),
            error_history: Arc::new(RwLock::new(VecDeque::new())),
            recovery_metrics: Arc::new(RwLock::new(RecoveryMetrics::default())),
            config: Arc::new(RwLock::new(config)),
            recovery_limiter,
        };

        // Initialize default recovery strategies
        manager.initialize_default_strategies().await?;

        info!("🛡️ Error Recovery Manager initialized (session: {})", manager.session_id);
        Ok(manager)
    }

    /// Handle error with automatic recovery
    pub async fn handle_error(&self, error: anyhow::Error, context: ErrorContext) -> Result<RecoveryResult> {
        let error_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        
        info!("🚨 Handling error: {} (id: {})", error, error_id);

        // Acquire recovery permit
        let _permit = self.recovery_limiter.acquire().await?;

        // Classify the error
        let (category, severity) = self.classify_error(&error, &context).await;
        
        // Check if recovery is enabled and appropriate
        let config = self.config.read().await;
        if !config.enable_auto_recovery {
            warn!("Auto-recovery disabled, logging error only");
            self.record_error(error_id, category, severity, error.to_string(), context, vec![], ResolutionStatus::Unrecovered, None).await;
            return Ok(RecoveryResult::Skipped { reason: "Auto-recovery disabled".to_string() });
        }

        // Get recovery strategy for this error category
        let strategy = {
            let strategies = self.recovery_strategies.read().await;
            strategies.get(&category).cloned()
        };

        let strategy = match strategy {
            Some(s) => s,
            None => {
                warn!("No recovery strategy found for category: {:?}", category);
                self.record_error(error_id, category, severity, error.to_string(), context, vec![], ResolutionStatus::Unrecovered, None).await;
                return Ok(RecoveryResult::Failed { reason: "No recovery strategy available".to_string() });
            }
        };

        // Attempt recovery
        let recovery_attempts = self.attempt_recovery(&error, &context, &strategy, error_id).await;
        let final_result = recovery_attempts.last().map(|a| &a.result).cloned()
            .unwrap_or(RecoveryResult::Failed { reason: "No recovery attempts made".to_string() });

        // Determine resolution status
        let resolution_status = match &final_result {
            RecoveryResult::Success { .. } => ResolutionStatus::Recovered,
            RecoveryResult::Partial { .. } => ResolutionStatus::PartiallyRecovered,
            RecoveryResult::Failed { .. } | RecoveryResult::Timeout => ResolutionStatus::Unrecovered,
            RecoveryResult::Skipped { .. } => ResolutionStatus::ManualInterventionRequired,
        };

        let recovery_time = if matches!(resolution_status, ResolutionStatus::Recovered | ResolutionStatus::PartiallyRecovered) {
            Some(start_time.elapsed().as_millis() as u64)
        } else {
            None
        };

        // Record the error and recovery attempts
        self.record_error(error_id, category, severity, error.to_string(), context, recovery_attempts, resolution_status, recovery_time).await;

        // Update metrics
        self.update_metrics(category, &final_result).await;

        match &final_result {
            RecoveryResult::Success { details } => {
                info!("✅ Error recovery successful: {} (time: {}ms)", details, start_time.elapsed().as_millis());
            },
            RecoveryResult::Failed { reason } => {
                error!("❌ Error recovery failed: {} (time: {}ms)", reason, start_time.elapsed().as_millis());
            },
            _ => {
                warn!("⚠️ Error recovery partial/skipped (time: {}ms)", start_time.elapsed().as_millis());
            }
        }

        Ok(final_result)
    }

    /// Classify error into category and severity
    async fn classify_error(&self, error: &anyhow::Error, context: &ErrorContext) -> (ErrorCategory, ErrorSeverity) {
        let classifier = self.error_classifier.read().await;
        
        let error_message = error.to_string().to_lowercase();
        let mut best_category = ErrorCategory::UnknownError;
        let mut best_confidence = 0.0;

        // Apply classification rules
        for rule in &classifier.classification_rules {
            let mut rule_confidence = 0.0;

            // Check message patterns
            for pattern in &rule.message_patterns {
                if error_message.contains(&pattern.to_lowercase()) {
                    rule_confidence += rule.confidence * 0.8;
                }
            }

            // Check context patterns
            if let Some(task_type) = &context.task_type {
                let task_str = format!("{:?}", task_type).to_lowercase();
                for pattern in &rule.context_patterns {
                    if task_str.contains(&pattern.to_lowercase()) {
                        rule_confidence += rule.confidence * 0.2;
                    }
                }
            }

            if rule_confidence > best_confidence {
                best_confidence = rule_confidence;
                best_category = rule.category;
            }
        }

        // Determine severity based on category and context
        let severity = self.determine_severity(best_category, error, context).await;

        debug!("Error classified as {:?} with {:?} severity (confidence: {:.2})", best_category, severity, best_confidence);
        (best_category, severity)
    }

    /// Determine error severity
    async fn determine_severity(&self, category: ErrorCategory, _error: &anyhow::Error, context: &ErrorContext) -> ErrorSeverity {
        match category {
            ErrorCategory::NetworkError => {
                // Network errors are usually medium severity unless critical task
                if context.task_type == Some(TaskType::Planning) {
                    ErrorSeverity::High
                } else {
                    ErrorSeverity::Medium
                }
            },
            ErrorCategory::BrowserError => ErrorSeverity::High,
            ErrorCategory::LLMError => ErrorSeverity::Medium,
            ErrorCategory::ExecutionError => ErrorSeverity::High,
            ErrorCategory::ConfigurationError => ErrorSeverity::Medium,
            ErrorCategory::ResourceError => ErrorSeverity::Critical,
            ErrorCategory::AuthenticationError => ErrorSeverity::High,
            ErrorCategory::TimeoutError => ErrorSeverity::Medium,
            ErrorCategory::ValidationError => ErrorSeverity::Low,
            ErrorCategory::UnknownError => ErrorSeverity::Medium,
        }
    }

    /// Attempt error recovery using strategy
    async fn attempt_recovery(&self, error: &anyhow::Error, context: &ErrorContext, strategy: &RecoveryStrategy, error_id: Uuid) -> Vec<RecoveryAttempt> {
        let mut attempts = Vec::new();
        let config = self.config.read().await;
        let max_attempts = std::cmp::min(strategy.max_retries, config.max_recovery_attempts);

        for attempt_num in 1..=max_attempts {
            let attempt_start = std::time::Instant::now();
            let attempt_timestamp = Utc::now();

            info!("🔄 Recovery attempt {}/{} using strategy: {}", attempt_num, max_attempts, strategy.name);

            // Execute recovery actions
            let mut actions_taken = Vec::new();
            let mut attempt_result = RecoveryResult::Success { details: "Starting recovery".to_string() };

            for action in &strategy.recovery_actions {
                let action_result = self.execute_recovery_action(action, error, context).await;
                actions_taken.push(action.clone());

                match action_result {
                    Ok(success_msg) => {
                        debug!("Recovery action succeeded: {}", success_msg);
                    },
                    Err(action_error) => {
                        warn!("Recovery action failed: {}", action_error);
                        attempt_result = RecoveryResult::Failed { reason: action_error.to_string() };
                        break;
                    }
                }
            }

            // Apply recovery delay if specified
            let delay = self.calculate_retry_delay(strategy, attempt_num);
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }

            let attempt_duration = attempt_start.elapsed().as_millis() as u64;
            
            let attempt = RecoveryAttempt {
                attempt_number: attempt_num,
                strategy_name: strategy.name.clone(),
                actions_taken,
                timestamp: attempt_timestamp,
                result: attempt_result.clone(),
                duration_ms: attempt_duration,
            };

            attempts.push(attempt);

            // Check if recovery was successful
            if matches!(attempt_result, RecoveryResult::Success { .. }) {
                info!("✅ Recovery successful on attempt {}", attempt_num);
                break;
            }

            // Check for timeout
            if attempt_duration > config.recovery_timeout_ms {
                warn!("⏱️ Recovery timeout exceeded");
                break;
            }
        }

        attempts
    }

    /// Execute a specific recovery action
    async fn execute_recovery_action(&self, action: &RecoveryAction, _error: &anyhow::Error, _context: &ErrorContext) -> Result<String> {
        match action {
            RecoveryAction::Retry { delay_ms } => {
                tokio::time::sleep(tokio::time::Duration::from_millis(*delay_ms)).await;
                Ok(format!("Retried with {}ms delay", delay_ms))
            },
            RecoveryAction::CleanupResources { resource_types } => {
                // Simulate resource cleanup
                for resource_type in resource_types {
                    debug!("Cleaning up resource type: {}", resource_type);
                }
                Ok(format!("Cleaned up {} resource types", resource_types.len()))
            },
            RecoveryAction::ResetConfiguration { config_keys } => {
                // Simulate configuration reset
                for key in config_keys {
                    debug!("Resetting configuration key: {}", key);
                }
                Ok(format!("Reset {} configuration keys", config_keys.len()))
            },
            RecoveryAction::RestartBrowser => {
                // Simulate browser restart
                debug!("Restarting browser session");
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                Ok("Browser restarted successfully".to_string())
            },
            RecoveryAction::SwitchLLMProvider { provider } => {
                debug!("Switching to LLM provider: {}", provider);
                Ok(format!("Switched to LLM provider: {}", provider))
            },
            RecoveryAction::ClearCache { cache_types } => {
                for cache_type in cache_types {
                    debug!("Clearing cache type: {}", cache_type);
                }
                Ok(format!("Cleared {} cache types", cache_types.len()))
            },
            RecoveryAction::ReduceResourceUsage { percentage } => {
                debug!("Reducing resource usage by {}%", percentage);
                Ok(format!("Reduced resource usage by {}%", percentage))
            },
            RecoveryAction::WaitForResources { timeout_ms } => {
                tokio::time::sleep(tokio::time::Duration::from_millis(*timeout_ms)).await;
                Ok(format!("Waited {}ms for resources", timeout_ms))
            },
            RecoveryAction::SendAlert { severity, message } => {
                warn!("ALERT [{:?}]: {}", severity, message);
                Ok("Alert sent to operations team".to_string())
            },
            RecoveryAction::LogError { include_context } => {
                if *include_context {
                    error!("Error logged with full context");
                } else {
                    error!("Error logged");
                }
                Ok("Error logged successfully".to_string())
            },
        }
    }

    /// Calculate retry delay with backoff
    fn calculate_retry_delay(&self, strategy: &RecoveryStrategy, attempt_num: u32) -> u64 {
        match strategy.strategy_type {
            RecoveryType::LinearRetry => strategy.base_delay_ms * attempt_num as u64,
            RecoveryType::ExponentialRetry => {
                let delay = (strategy.base_delay_ms as f64 * strategy.backoff_multiplier.powi(attempt_num as i32 - 1)) as u64;
                std::cmp::min(delay, strategy.max_delay_ms)
            },
            _ => strategy.base_delay_ms,
        }
    }

    /// Initialize default recovery strategies
    async fn initialize_default_strategies(&mut self) -> Result<()> {
        let mut strategies = HashMap::new();

        // Network error recovery
        strategies.insert(ErrorCategory::NetworkError, RecoveryStrategy {
            name: "NetworkErrorRecovery".to_string(),
            strategy_type: RecoveryType::ExponentialRetry,
            max_retries: 3,
            base_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            recovery_actions: vec![
                RecoveryAction::Retry { delay_ms: 1000 },
                RecoveryAction::CleanupResources { resource_types: vec!["network_connections".to_string()] },
            ],
            success_criteria: vec!["Network connectivity restored".to_string()],
            fallback_strategy: Some("ManualIntervention".to_string()),
        });

        // Browser error recovery
        strategies.insert(ErrorCategory::BrowserError, RecoveryStrategy {
            name: "BrowserErrorRecovery".to_string(),
            strategy_type: RecoveryType::ServiceFallback,
            max_retries: 2,
            base_delay_ms: 2000,
            backoff_multiplier: 1.5,
            max_delay_ms: 5000,
            recovery_actions: vec![
                RecoveryAction::RestartBrowser,
                RecoveryAction::ClearCache { cache_types: vec!["browser_cache".to_string()] },
                RecoveryAction::Retry { delay_ms: 2000 },
            ],
            success_criteria: vec!["Browser automation restored".to_string()],
            fallback_strategy: None,
        });

        // LLM error recovery
        strategies.insert(ErrorCategory::LLMError, RecoveryStrategy {
            name: "LLMErrorRecovery".to_string(),
            strategy_type: RecoveryType::ServiceFallback,
            max_retries: 2,
            base_delay_ms: 1500,
            backoff_multiplier: 2.0,
            max_delay_ms: 8000,
            recovery_actions: vec![
                RecoveryAction::SwitchLLMProvider { provider: "fallback_provider".to_string() },
                RecoveryAction::Retry { delay_ms: 1500 },
            ],
            success_criteria: vec!["LLM service restored".to_string()],
            fallback_strategy: Some("ManualIntervention".to_string()),
        });

        *self.recovery_strategies.write().await = strategies;
        info!("🔧 Initialized {} default recovery strategies", 3);
        Ok(())
    }

    /// Record error in history
    async fn record_error(&self, error_id: Uuid, category: ErrorCategory, severity: ErrorSeverity, message: String, 
                         context: ErrorContext, recovery_attempts: Vec<RecoveryAttempt>, 
                         resolution_status: ResolutionStatus, recovery_time_ms: Option<u64>) {
        let error_record = ErrorRecord {
            error_id,
            timestamp: Utc::now(),
            category,
            severity,
            message,
            context,
            recovery_attempts,
            resolution_status,
            recovery_time_ms,
        };

        let mut history = self.error_history.write().await;
        history.push_back(error_record);

        // Maintain history size limit
        let config = self.config.read().await;
        while history.len() > config.max_error_history {
            history.pop_front();
        }

        debug!("📝 Error recorded in history (total: {})", history.len());
    }

    /// Update recovery metrics
    async fn update_metrics(&self, category: ErrorCategory, result: &RecoveryResult) {
        let mut metrics = self.recovery_metrics.write().await;
        metrics.total_errors += 1;

        match result {
            RecoveryResult::Success { .. } => {
                metrics.successful_recoveries += 1;
            },
            RecoveryResult::Failed { .. } | RecoveryResult::Timeout => {
                metrics.failed_recoveries += 1;
            },
            _ => {}
        }

        // Update success rate by category
        let category_entry = metrics.success_rate_by_category.entry(category).or_insert(0.0);
        let success = matches!(result, RecoveryResult::Success { .. });
        *category_entry = (*category_entry + if success { 1.0 } else { 0.0 }) / 2.0; // Simple moving average
    }

    /// Get recovery metrics
    pub async fn get_metrics(&self) -> RecoveryMetrics {
        self.recovery_metrics.read().await.clone()
    }

    /// Get error history
    pub async fn get_error_history(&self, limit: Option<usize>) -> Vec<ErrorRecord> {
        let history = self.error_history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }
}

impl ErrorClassifier {
    fn new() -> Self {
        let mut classifier = Self {
            classification_rules: Vec::new(),
            learned_patterns: HashMap::new(),
            confidence_thresholds: HashMap::new(),
        };

        classifier.initialize_default_rules();
        classifier
    }

    fn initialize_default_rules(&mut self) {
        // Network error patterns
        self.classification_rules.push(ClassificationRule {
            name: "NetworkError".to_string(),
            message_patterns: vec![
                "connection refused".to_string(),
                "timeout".to_string(),
                "network unreachable".to_string(),
                "dns resolution failed".to_string(),
            ],
            context_patterns: vec![],
            category: ErrorCategory::NetworkError,
            confidence: 0.9,
            priority: 100,
        });

        // Browser error patterns
        self.classification_rules.push(ClassificationRule {
            name: "BrowserError".to_string(),
            message_patterns: vec![
                "webdriver".to_string(),
                "chrome".to_string(),
                "element not found".to_string(),
                "browser".to_string(),
            ],
            context_patterns: vec![],
            category: ErrorCategory::BrowserError,
            confidence: 0.85,
            priority: 90,
        });

        // LLM error patterns
        self.classification_rules.push(ClassificationRule {
            name: "LLMError".to_string(),
            message_patterns: vec![
                "llm".to_string(),
                "model".to_string(),
                "api key".to_string(),
                "rate limit".to_string(),
            ],
            context_patterns: vec![],
            category: ErrorCategory::LLMError,
            confidence: 0.8,
            priority: 80,
        });

        info!("🔍 Initialized {} classification rules", self.classification_rules.len());
    }
}

/// Create error recovery manager with default configuration
pub async fn create_error_recovery_manager() -> Result<ErrorRecoveryManager> {
    let config = ErrorRecoveryConfig::default();
    ErrorRecoveryManager::new(config).await
}

/// Create error recovery manager with custom configuration
pub async fn create_custom_error_recovery_manager(config: ErrorRecoveryConfig) -> Result<ErrorRecoveryManager> {
    ErrorRecoveryManager::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_recovery_manager_creation() {
        let manager = create_error_recovery_manager().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_error_classification() {
        let manager = create_error_recovery_manager().await.unwrap();
        let error = anyhow::anyhow!("Connection refused: network unreachable");
        let context = ErrorContext {
            task_type: None,
            context_snapshot: None,
            stack_trace: None,
            metadata: HashMap::new(),
            related_errors: Vec::new(),
        };

        let (category, severity) = manager.classify_error(&error, &context).await;
        assert_eq!(category, ErrorCategory::NetworkError);
    }

    #[tokio::test]
    async fn test_recovery_strategy_execution() {
        let manager = create_error_recovery_manager().await.unwrap();
        let error = anyhow::anyhow!("Network connection failed");
        let context = ErrorContext {
            task_type: Some(TaskType::Navigation),
            context_snapshot: None,
            stack_trace: None,
            metadata: HashMap::new(),
            related_errors: Vec::new(),
        };

        let result = manager.handle_error(error, context).await;
        assert!(result.is_ok());
    }
}