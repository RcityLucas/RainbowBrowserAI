// Intelligent Retry Mechanism
// Part of the Intelligent Action Engine

use crate::error::{Result, RainbowError};
use serde::{Serialize, Deserialize};
use std::future::Future;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Intelligent retry mechanism with adaptive strategies
#[derive(Debug)]
pub struct RetryMechanism {
    strategies: Vec<Box<dyn RetryStrategy + Send + Sync>>,
    failure_analysis: Arc<RwLock<FailureAnalyzer>>,
}

impl RetryMechanism {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(ExponentialBackoffStrategy),
                Box::new(LinearBackoffStrategy),
                Box::new(AdaptiveDelayStrategy),
                Box::new(CircuitBreakerStrategy),
            ],
            failure_analysis: Arc::new(RwLock::new(FailureAnalyzer::new())),
        }
    }

    /// Execute a function with intelligent retry logic
    pub async fn execute_with_retry<F, T, E>(
        &self,
        mut operation: impl FnMut() -> F,
    ) -> Result<T>
    where
        F: Future<Output = std::result::Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let mut attempt = 0;
        let mut last_error: Option<E> = None;
        let mut retry_context = RetryContext::new();

        loop {
            attempt += 1;
            retry_context.attempt_number = attempt;
            retry_context.elapsed_time = start_time.elapsed();

            // Execute the operation
            match operation().await {
                Ok(result) => {
                    // Success - update failure analyzer
                    self.failure_analysis.write().await
                        .record_success(&retry_context);
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);
                    
                    // Analyze the failure
                    let failure_type = self.analyze_failure(&last_error.as_ref().unwrap());
                    retry_context.failure_types.push(failure_type.clone());
                    
                    // Update failure analyzer
                    self.failure_analysis.write().await
                        .record_failure(&retry_context, &failure_type);

                    // Check if we should continue retrying
                    let retry_decision = self.should_retry(&retry_context, &failure_type).await;
                    
                    if !retry_decision.should_retry {
                        return Err(RainbowError::MaxRetriesExceeded {
                            attempts: attempt,
                            last_error: format!("{}", last_error.unwrap()),
                            reason: retry_decision.reason,
                        });
                    }

                    // Apply retry strategy
                    if let Some(delay) = retry_decision.delay {
                        tokio::time::sleep(delay).await;
                    }

                    // Apply any recovery actions
                    if let Some(recovery_action) = retry_decision.recovery_action {
                        self.execute_recovery_action(recovery_action).await?;
                    }
                }
            }
        }
    }

    async fn should_retry(
        &self,
        context: &RetryContext,
        failure_type: &FailureType,
    ) -> RetryDecision {
        // Basic limits
        if context.attempt_number >= 5 {
            return RetryDecision {
                should_retry: false,
                delay: None,
                recovery_action: None,
                reason: "Maximum attempts reached".to_string(),
            };
        }

        if context.elapsed_time > Duration::from_secs(30) {
            return RetryDecision {
                should_retry: false,
                delay: None,
                recovery_action: None,
                reason: "Maximum retry time exceeded".to_string(),
            };
        }

        // Failure-specific retry decisions
        match failure_type {
            FailureType::ElementNotFound => RetryDecision {
                should_retry: true,
                delay: Some(Duration::from_millis(500 * context.attempt_number as u64)),
                recovery_action: Some(RecoveryAction::WaitForPageStability),
                reason: "Element might load with delay".to_string(),
            },
            FailureType::ElementNotInteractable => RetryDecision {
                should_retry: true,
                delay: Some(Duration::from_millis(200 * context.attempt_number as u64)),
                recovery_action: Some(RecoveryAction::ScrollToElement),
                reason: "Element might become interactable".to_string(),
            },
            FailureType::NetworkTimeout => RetryDecision {
                should_retry: true,
                delay: Some(Duration::from_secs(2_u64.pow(context.attempt_number.min(4)))),
                recovery_action: None,
                reason: "Network issues might be temporary".to_string(),
            },
            FailureType::JavaScriptError => RetryDecision {
                should_retry: context.attempt_number <= 2,
                delay: Some(Duration::from_millis(100)),
                recovery_action: Some(RecoveryAction::RefreshPageContext),
                reason: "JavaScript state might recover".to_string(),
            },
            FailureType::PageNotLoaded => RetryDecision {
                should_retry: true,
                delay: Some(Duration::from_secs(1)),
                recovery_action: Some(RecoveryAction::WaitForPageLoad),
                reason: "Page might still be loading".to_string(),
            },
            FailureType::Unknown => RetryDecision {
                should_retry: context.attempt_number <= 2,
                delay: Some(Duration::from_millis(500)),
                recovery_action: None,
                reason: "Unknown error might be transient".to_string(),
            },
        }
    }

    fn analyze_failure<E: std::error::Error>(&self, error: &E) -> FailureType {
        let error_msg = error.to_string().to_lowercase();

        if error_msg.contains("element not found") || error_msg.contains("no such element") {
            FailureType::ElementNotFound
        } else if error_msg.contains("not interactable") || error_msg.contains("not clickable") {
            FailureType::ElementNotInteractable
        } else if error_msg.contains("timeout") || error_msg.contains("connection") {
            FailureType::NetworkTimeout
        } else if error_msg.contains("javascript") || error_msg.contains("script") {
            FailureType::JavaScriptError
        } else if error_msg.contains("page") && error_msg.contains("load") {
            FailureType::PageNotLoaded
        } else {
            FailureType::Unknown
        }
    }

    async fn execute_recovery_action(&self, action: RecoveryAction) -> Result<()> {
        match action {
            RecoveryAction::WaitForPageStability => {
                // Wait for DOM to stabilize
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            RecoveryAction::ScrollToElement => {
                // In a real implementation, this would scroll to the element
                // For now, just a brief wait
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(())
            }
            RecoveryAction::RefreshPageContext => {
                // Refresh JavaScript context or wait for scripts to load
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            }
            RecoveryAction::WaitForPageLoad => {
                // Wait for page to complete loading
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            }
        }
    }

    /// Get retry statistics for analysis
    pub async fn get_statistics(&self) -> RetryStatistics {
        self.failure_analysis.read().await.get_statistics()
    }

    /// Reset statistics (useful for testing or new sessions)
    pub async fn reset_statistics(&self) {
        *self.failure_analysis.write().await = FailureAnalyzer::new();
    }
}

impl Default for RetryMechanism {
    fn default() -> Self {
        Self::new()
    }
}

/// Context information for retry decisions
#[derive(Debug, Clone)]
struct RetryContext {
    attempt_number: u32,
    elapsed_time: Duration,
    failure_types: Vec<FailureType>,
}

impl RetryContext {
    fn new() -> Self {
        Self {
            attempt_number: 0,
            elapsed_time: Duration::default(),
            failure_types: Vec::new(),
        }
    }
}

/// Types of failures that can be analyzed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureType {
    ElementNotFound,
    ElementNotInteractable,
    NetworkTimeout,
    JavaScriptError,
    PageNotLoaded,
    Unknown,
}

/// Decision about whether to retry an operation
#[derive(Debug, Clone)]
struct RetryDecision {
    should_retry: bool,
    delay: Option<Duration>,
    recovery_action: Option<RecoveryAction>,
    reason: String,
}

/// Recovery actions that can be taken before retry
#[derive(Debug, Clone)]
enum RecoveryAction {
    WaitForPageStability,
    ScrollToElement,
    RefreshPageContext,
    WaitForPageLoad,
}

/// Failure analyzer that tracks patterns and adapts retry strategies
#[derive(Debug)]
struct FailureAnalyzer {
    failure_counts: HashMap<FailureType, u32>,
    success_after_retry_counts: HashMap<FailureType, u32>,
    total_operations: u32,
    total_successes: u32,
    average_attempts_to_success: f64,
}

impl FailureAnalyzer {
    fn new() -> Self {
        Self {
            failure_counts: HashMap::new(),
            success_after_retry_counts: HashMap::new(),
            total_operations: 0,
            total_successes: 0,
            average_attempts_to_success: 1.0,
        }
    }

    fn record_failure(&mut self, context: &RetryContext, failure_type: &FailureType) {
        *self.failure_counts.entry(failure_type.clone()).or_insert(0) += 1;
    }

    fn record_success(&mut self, context: &RetryContext) {
        self.total_operations += 1;
        self.total_successes += 1;

        // Update average attempts to success
        let alpha = 0.1; // Smoothing factor for moving average
        self.average_attempts_to_success = alpha * context.attempt_number as f64 
            + (1.0 - alpha) * self.average_attempts_to_success;

        // Record which failure types were overcome
        for failure_type in &context.failure_types {
            *self.success_after_retry_counts.entry(failure_type.clone()).or_insert(0) += 1;
        }
    }

    fn get_statistics(&self) -> RetryStatistics {
        let success_rate = if self.total_operations > 0 {
            self.total_successes as f64 / self.total_operations as f64
        } else {
            0.0
        };

        RetryStatistics {
            total_operations: self.total_operations,
            success_rate,
            average_attempts_to_success: self.average_attempts_to_success,
            failure_breakdown: self.failure_counts.clone(),
            recovery_rates: self.calculate_recovery_rates(),
        }
    }

    fn calculate_recovery_rates(&self) -> HashMap<FailureType, f64> {
        let mut recovery_rates = HashMap::new();

        for (failure_type, &total_failures) in &self.failure_counts {
            let recoveries = self.success_after_retry_counts
                .get(failure_type)
                .unwrap_or(&0);

            let recovery_rate = if total_failures > 0 {
                *recoveries as f64 / total_failures as f64
            } else {
                0.0
            };

            recovery_rates.insert(failure_type.clone(), recovery_rate);
        }

        recovery_rates
    }
}

/// Statistics about retry performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStatistics {
    pub total_operations: u32,
    pub success_rate: f64,
    pub average_attempts_to_success: f64,
    pub failure_breakdown: HashMap<FailureType, u32>,
    pub recovery_rates: HashMap<FailureType, f64>,
}

/// Trait for different retry strategies
trait RetryStrategy: std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn calculate_delay(&self, attempt: u32, failure_type: &FailureType) -> Duration;
    fn should_abort(&self, context: &RetryContext, failure_type: &FailureType) -> bool;
}

/// Exponential backoff retry strategy
#[derive(Debug)]
struct ExponentialBackoffStrategy;

impl RetryStrategy for ExponentialBackoffStrategy {
    fn name(&self) -> &'static str {
        "ExponentialBackoff"
    }

    fn calculate_delay(&self, attempt: u32, _failure_type: &FailureType) -> Duration {
        let base_delay_ms = 100;
        let max_delay_ms = 5000;
        let delay_ms = (base_delay_ms * 2_u64.pow(attempt.min(6))).min(max_delay_ms);
        Duration::from_millis(delay_ms)
    }

    fn should_abort(&self, context: &RetryContext, _failure_type: &FailureType) -> bool {
        context.attempt_number >= 5 || context.elapsed_time > Duration::from_secs(30)
    }
}

/// Linear backoff retry strategy
#[derive(Debug)]
struct LinearBackoffStrategy;

impl RetryStrategy for LinearBackoffStrategy {
    fn name(&self) -> &'static str {
        "LinearBackoff"
    }

    fn calculate_delay(&self, attempt: u32, _failure_type: &FailureType) -> Duration {
        let base_delay_ms = 200;
        Duration::from_millis(base_delay_ms * attempt as u64)
    }

    fn should_abort(&self, context: &RetryContext, _failure_type: &FailureType) -> bool {
        context.attempt_number >= 4
    }
}

/// Adaptive delay strategy based on failure type
#[derive(Debug)]
struct AdaptiveDelayStrategy;

impl RetryStrategy for AdaptiveDelayStrategy {
    fn name(&self) -> &'static str {
        "AdaptiveDelay"
    }

    fn calculate_delay(&self, attempt: u32, failure_type: &FailureType) -> Duration {
        match failure_type {
            FailureType::ElementNotFound => Duration::from_millis(500 * attempt as u64),
            FailureType::ElementNotInteractable => Duration::from_millis(200 * attempt as u64),
            FailureType::NetworkTimeout => Duration::from_secs(2_u64.pow(attempt.min(3))),
            FailureType::JavaScriptError => Duration::from_millis(100),
            FailureType::PageNotLoaded => Duration::from_secs(1),
            FailureType::Unknown => Duration::from_millis(300 * attempt as u64),
        }
    }

    fn should_abort(&self, context: &RetryContext, failure_type: &FailureType) -> bool {
        match failure_type {
            FailureType::JavaScriptError => context.attempt_number >= 2,
            FailureType::NetworkTimeout => context.attempt_number >= 3,
            _ => context.attempt_number >= 4,
        }
    }
}

/// Circuit breaker strategy to prevent cascading failures
#[derive(Debug)]
struct CircuitBreakerStrategy {
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl Default for CircuitBreakerStrategy {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
        }
    }
}

impl RetryStrategy for CircuitBreakerStrategy {
    fn name(&self) -> &'static str {
        "CircuitBreaker"
    }

    fn calculate_delay(&self, _attempt: u32, _failure_type: &FailureType) -> Duration {
        self.recovery_timeout
    }

    fn should_abort(&self, context: &RetryContext, _failure_type: &FailureType) -> bool {
        // If we've seen too many of the same failure type recently, abort
        let same_failure_count = context.failure_types
            .iter()
            .filter(|&f| f == context.failure_types.last().unwrap())
            .count() as u32;

        same_failure_count >= self.failure_threshold
    }
}

/// Builder for configuring retry behavior
#[derive(Debug)]
pub struct RetryConfigBuilder {
    max_attempts: u32,
    max_duration: Duration,
    base_delay: Duration,
    max_delay: Duration,
    backoff_factor: f64,
}

impl Default for RetryConfigBuilder {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            max_duration: Duration::from_secs(30),
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        }
    }
}

impl RetryConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    pub fn max_duration(mut self, duration: Duration) -> Self {
        self.max_duration = duration;
        self
    }

    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor;
        self
    }

    pub fn build(self) -> RetryConfig {
        RetryConfig {
            max_attempts: self.max_attempts,
            max_duration: self.max_duration,
            base_delay: self.base_delay,
            max_delay: self.max_delay,
            backoff_factor: self.backoff_factor,
        }
    }
}

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub max_duration: Duration,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_mechanism_creation() {
        let mechanism = RetryMechanism::new();
        assert_eq!(mechanism.strategies.len(), 4);
    }

    #[test]
    fn test_failure_type_analysis() {
        let mechanism = RetryMechanism::new();
        
        // Test different error messages
        let errors = [
            ("Element not found", FailureType::ElementNotFound),
            ("Connection timeout", FailureType::NetworkTimeout),
            ("JavaScript error occurred", FailureType::JavaScriptError),
            ("Element not interactable", FailureType::ElementNotInteractable),
            ("Page failed to load", FailureType::PageNotLoaded),
            ("Something weird happened", FailureType::Unknown),
        ];

        for (error_msg, expected_type) in errors {
            let error = RainbowError::ExecutionError(error_msg.to_string());
            let failure_type = mechanism.analyze_failure(&error);
            assert_eq!(failure_type, expected_type);
        }
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfigBuilder::new()
            .max_attempts(5)
            .max_duration(Duration::from_secs(60))
            .base_delay(Duration::from_millis(200))
            .backoff_factor(1.5)
            .build();

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.max_duration, Duration::from_secs(60));
        assert_eq!(config.base_delay, Duration::from_millis(200));
        assert_eq!(config.backoff_factor, 1.5);
    }

    #[tokio::test]
    async fn test_retry_mechanism_success() {
        let mechanism = RetryMechanism::new();
        let mut call_count = 0;

        let result = mechanism.execute_with_retry(|| {
            call_count += 1;
            async move {
                if call_count >= 3 {
                    Ok("Success!")
                } else {
                    Err(RainbowError::ElementNotFound("Not found".to_string()))
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(call_count, 3);
    }

    #[tokio::test]
    async fn test_failure_analyzer() {
        let mut analyzer = FailureAnalyzer::new();
        
        let context = RetryContext {
            attempt_number: 2,
            elapsed_time: Duration::from_millis(500),
            failure_types: vec![FailureType::ElementNotFound],
        };

        analyzer.record_failure(&context, &FailureType::ElementNotFound);
        analyzer.record_success(&context);

        let stats = analyzer.get_statistics();
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.success_rate, 1.0);
        assert!(stats.recovery_rates.contains_key(&FailureType::ElementNotFound));
    }
}