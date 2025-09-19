// Unified Error Handling and Recovery Strategy
// Addresses inconsistent error handling across modules

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use super::{Event, EventBus};

/// Unified error types for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationError {
    /// Browser-related errors
    BrowserError { message: String, recoverable: bool },

    /// Perception analysis errors
    PerceptionError {
        message: String,
        fallback_available: bool,
    },

    /// Tool execution errors
    ToolError {
        tool_name: String,
        message: String,
        can_retry: bool,
    },

    /// Resource exhaustion errors
    ResourceError {
        resource_type: String,
        message: String,
    },

    /// Session-related errors
    SessionError { session_id: String, message: String },

    /// Timeout errors
    TimeoutError { operation: String, timeout_ms: u64 },

    /// Coordination errors
    CoordinationError {
        message: String,
        affected_modules: Vec<String>,
    },
}

impl std::error::Error for CoordinationError {}

impl std::fmt::Display for CoordinationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinationError::BrowserError { message, .. } => {
                write!(f, "Browser error: {}", message)
            }
            CoordinationError::PerceptionError { message, .. } => {
                write!(f, "Perception error: {}", message)
            }
            CoordinationError::ToolError {
                tool_name, message, ..
            } => {
                write!(f, "Tool {} error: {}", tool_name, message)
            }
            CoordinationError::ResourceError {
                resource_type,
                message,
            } => {
                write!(f, "Resource {} error: {}", resource_type, message)
            }
            CoordinationError::SessionError {
                session_id,
                message,
            } => {
                write!(f, "Session {} error: {}", session_id, message)
            }
            CoordinationError::TimeoutError {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "Operation {} timed out after {}ms",
                    operation, timeout_ms
                )
            }
            CoordinationError::CoordinationError { message, .. } => {
                write!(f, "Coordination error: {}", message)
            }
        }
    }
}

/// Unified error handler with consistent recovery strategies
pub struct UnifiedErrorHandler {
    event_bus: Arc<EventBus>,
    retry_config: RetryConfig,
    fallback_strategies: FallbackStrategies,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackStrategies {
    pub enable_legacy_fallback: bool,
    pub enable_graceful_degradation: bool,
    pub enable_circuit_breaker: bool,
}

impl Default for FallbackStrategies {
    fn default() -> Self {
        Self {
            enable_legacy_fallback: true,
            enable_graceful_degradation: true,
            enable_circuit_breaker: true,
        }
    }
}

impl UnifiedErrorHandler {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            retry_config: RetryConfig::default(),
            fallback_strategies: FallbackStrategies::default(),
        }
    }

    /// Handle perception errors with consistent fallback
    pub async fn handle_perception_error<T>(
        &self,
        error: anyhow::Error,
        fallback: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        error!("Perception error occurred: {}", error);

        // Emit error event
        self.event_bus
            .emit(Event::ModuleError {
                module_type: "perception".to_string(),
                error: error.to_string(),
                timestamp: Instant::now(),
            })
            .await?;

        // Try fallback if enabled
        if self.fallback_strategies.enable_legacy_fallback {
            info!("Attempting legacy fallback for perception error");
            match fallback() {
                Ok(result) => {
                    warn!("Successfully used legacy fallback for perception");
                    return Ok(result);
                }
                Err(fallback_err) => {
                    error!("Legacy fallback also failed: {}", fallback_err);
                }
            }
        }

        Err(error).context("Perception operation failed with no successful fallback")
    }

    /// Handle tool errors with retry logic
    pub async fn handle_tool_error<T>(
        &self,
        tool_name: &str,
        error: anyhow::Error,
        retry_fn: impl FnMut() -> Result<T>,
    ) -> Result<T> {
        error!("Tool {} error occurred: {}", tool_name, error);

        // Check if error is retryable
        let can_retry =
            !error.to_string().contains("not found") && !error.to_string().contains("invalid");

        if can_retry {
            return self
                .retry_with_backoff(retry_fn, &format!("tool_{}", tool_name))
                .await;
        }

        // Emit error event
        self.event_bus
            .emit(Event::ModuleError {
                module_type: "tools".to_string(),
                error: format!("{}: {}", tool_name, error),
                timestamp: Instant::now(),
            })
            .await?;

        Err(error).context(format!("Tool {} execution failed", tool_name))
    }

    /// Retry with exponential backoff
    pub async fn retry_with_backoff<T>(
        &self,
        mut operation: impl FnMut() -> Result<T>,
        operation_name: &str,
    ) -> Result<T> {
        let mut attempt = 0;
        let mut delay = Duration::from_millis(self.retry_config.initial_delay_ms);

        loop {
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        info!(
                            "Operation {} succeeded after {} retries",
                            operation_name, attempt
                        );
                    }
                    return Ok(result);
                }
                Err(error) if attempt < self.retry_config.max_retries => {
                    warn!(
                        "Operation {} failed (attempt {}/{}): {}",
                        operation_name,
                        attempt + 1,
                        self.retry_config.max_retries,
                        error
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.retry_config.exponential_base) as u64,
                    )
                    .min(Duration::from_millis(self.retry_config.max_delay_ms));

                    attempt += 1;
                }
                Err(error) => {
                    error!(
                        "Operation {} failed after {} retries: {}",
                        operation_name, self.retry_config.max_retries, error
                    );
                    return Err(error).context(format!(
                        "Operation {} failed after {} retries",
                        operation_name, self.retry_config.max_retries
                    ));
                }
            }
        }
    }

    /// Handle resource errors with graceful degradation
    pub async fn handle_resource_error<T>(
        &self,
        resource_type: &str,
        error: anyhow::Error,
        degraded_fn: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        error!("Resource {} error: {}", resource_type, error);

        if self.fallback_strategies.enable_graceful_degradation {
            info!(
                "Attempting graceful degradation for resource {}",
                resource_type
            );
            match degraded_fn() {
                Ok(result) => {
                    warn!("Operating in degraded mode for resource {}", resource_type);
                    return Ok(result);
                }
                Err(degraded_err) => {
                    error!("Graceful degradation failed: {}", degraded_err);
                }
            }
        }

        Err(CoordinationError::ResourceError {
            resource_type: resource_type.to_string(),
            message: error.to_string(),
        }
        .into())
    }

    /// Unified error recovery strategy
    pub async fn recover_from_error(
        &self,
        error: &CoordinationError,
        session_id: Option<&str>,
    ) -> Result<RecoveryAction> {
        match error {
            CoordinationError::BrowserError { recoverable, .. } if *recoverable => {
                Ok(RecoveryAction::RetryWithNewBrowser)
            }
            CoordinationError::PerceptionError {
                fallback_available, ..
            } if *fallback_available => Ok(RecoveryAction::UseFallback),
            CoordinationError::ToolError { can_retry, .. } if *can_retry => {
                Ok(RecoveryAction::RetryOperation)
            }
            CoordinationError::ResourceError { .. } => Ok(RecoveryAction::GracefulDegradation),
            CoordinationError::TimeoutError { .. } => Ok(RecoveryAction::ExtendTimeout),
            CoordinationError::SessionError { .. } => {
                if let Some(sid) = session_id {
                    Ok(RecoveryAction::RecreateSession(sid.to_string()))
                } else {
                    Ok(RecoveryAction::Abort)
                }
            }
            _ => Ok(RecoveryAction::Abort),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    RetryOperation,
    RetryWithNewBrowser,
    UseFallback,
    GracefulDegradation,
    ExtendTimeout,
    RecreateSession(String),
    Abort,
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    failure_count: Arc<tokio::sync::RwLock<u32>>,
    last_failure: Arc<tokio::sync::RwLock<Option<Instant>>>,
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    threshold: u32,
    timeout: Duration,
}

#[derive(Debug, Clone, Copy)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: Arc::new(tokio::sync::RwLock::new(0)),
            last_failure: Arc::new(tokio::sync::RwLock::new(None)),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            threshold,
            timeout,
        }
    }

    pub async fn call<T>(&self, operation: impl FnOnce() -> Result<T>) -> Result<T> {
        // Check circuit state
        let state = *self.state.read().await;

        if let CircuitState::Open = state {
            // Check if timeout has passed
            if let Some(last) = *self.last_failure.read().await {
                if last.elapsed() > self.timeout {
                    // Try half-open
                    *self.state.write().await = CircuitState::HalfOpen;
                } else {
                    return Err(anyhow::anyhow!("Circuit breaker is open"));
                }
            } else {
                return Err(anyhow::anyhow!("Circuit breaker is open"));
            }
        }

        // Execute operation
        match operation() {
            Ok(result) => {
                // Reset on success
                if matches!(state, CircuitState::HalfOpen) {
                    *self.state.write().await = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                }
                Ok(result)
            }
            Err(error) => {
                // Increment failure count
                let mut count = self.failure_count.write().await;
                *count += 1;
                *self.last_failure.write().await = Some(Instant::now());

                // Check if threshold exceeded
                if *count >= self.threshold {
                    *self.state.write().await = CircuitState::Open;
                    warn!("Circuit breaker opened after {} failures", count);
                }

                Err(error)
            }
        }
    }

    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failure_count.write().await = 0;
        *self.last_failure.write().await = None;
    }
}

// Extension trait for Result type to add coordination error handling
pub trait CoordinationResultExt<T> {
    fn or_fallback<F: FnOnce() -> Result<T>>(self, fallback: F) -> Result<T>;
    fn or_degraded<F: FnOnce() -> Result<T>>(self, degraded: F) -> Result<T>;
}

impl<T> CoordinationResultExt<T> for Result<T> {
    fn or_fallback<F: FnOnce() -> Result<T>>(self, fallback: F) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                warn!("Primary operation failed, using fallback: {}", error);
                fallback()
            }
        }
    }

    fn or_degraded<F: FnOnce() -> Result<T>>(self, degraded: F) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                warn!("Normal operation failed, using degraded mode: {}", error);
                degraded()
            }
        }
    }
}
