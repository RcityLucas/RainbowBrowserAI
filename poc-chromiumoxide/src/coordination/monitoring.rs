// Unified Monitoring System for Cross-Module Health and Performance
// Provides health checks, metrics collection, and alerting

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::events::{Event, EventBus};
use super::session::SessionBundle;

/// Unified monitoring system
pub struct UnifiedMonitoring {
    _event_bus: Arc<EventBus>,
    metrics_collector: Arc<MetricsCollector>,
    _health_monitor: Arc<HealthMonitor>,
    alerting: Arc<AlertingSystem>,
}

impl UnifiedMonitoring {
    pub async fn new(event_bus: Arc<EventBus>) -> Result<Self> {
        Ok(Self {
            _event_bus: event_bus.clone(),
            metrics_collector: Arc::new(MetricsCollector::new()),
            _health_monitor: Arc::new(HealthMonitor::new()),
            alerting: Arc::new(AlertingSystem::new(event_bus)),
        })
    }

    /// Monitor a session bundle
    pub async fn monitor_session_bundle(&self, bundle: &SessionBundle) {
        // Collect performance metrics
        self.metrics_collector
            .track_session_performance(&bundle.session_id)
            .await;

        // Check health
        let health = self.collect_module_health(bundle).await;
        if health.overall_score < 0.8 {
            self.alerting
                .send_health_alert(&bundle.session_id, health.clone())
                .await;
        }

        // Check resource usage
        self.monitor_resource_usage(bundle).await;
    }

    async fn collect_module_health(&self, bundle: &SessionBundle) -> OverallHealth {
        let perception_health = bundle.perception.health_check();
        let tools_health = bundle.tools.health_check();
        let intelligence_health = bundle.intelligence.health_check();

        OverallHealth::calculate(vec![perception_health, tools_health, intelligence_health])
    }

    async fn monitor_resource_usage(&self, bundle: &SessionBundle) {
        let usage = bundle.context.get_resource_usage().await;

        // Check thresholds
        if usage.memory_bytes > 500_000_000 {
            // 500MB
            warn!(
                "Session {} memory usage high: {} bytes",
                bundle.session_id, usage.memory_bytes
            );
        }

        if usage.cpu_percent > 80.0 {
            warn!(
                "Session {} CPU usage high: {}%",
                bundle.session_id, usage.cpu_percent
            );
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    session_metrics: Arc<RwLock<HashMap<String, SessionMetrics>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            session_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn track_session_performance(&self, session_id: &str) {
        let mut metrics = self.session_metrics.write().await;
        let entry = metrics
            .entry(session_id.to_string())
            .or_insert_with(|| SessionMetrics {
                session_id: session_id.to_string(),
                started_at: Instant::now(),
                operation_count: 0,
                total_duration_ms: 0,
                error_count: 0,
            });

        entry.operation_count += 1;
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Health monitor
pub struct HealthMonitor {
    health_checks: Arc<RwLock<Vec<HealthCheck>>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_health_check(&self, check: HealthCheck) {
        self.health_checks.write().await.push(check);
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Alerting system
pub struct AlertingSystem {
    event_bus: Arc<EventBus>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
}

impl AlertingSystem {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn send_health_alert(&self, session_id: &str, health: OverallHealth) {
        let alert = Alert {
            timestamp: Instant::now(),
            severity: if health.overall_score < 0.5 {
                AlertSeverity::Critical
            } else if health.overall_score < 0.8 {
                AlertSeverity::Warning
            } else {
                AlertSeverity::Info
            },
            message: format!(
                "Session {} health degraded: score {:.2}",
                session_id, health.overall_score
            ),
            context: HashMap::new(),
        };

        // Log alert
        match alert.severity {
            AlertSeverity::Critical => error!("{}", alert.message),
            AlertSeverity::Warning => warn!("{}", alert.message),
            AlertSeverity::Info => info!("{}", alert.message),
        }

        // Store alert
        self.alert_history.write().await.push(alert.clone());

        // Emit event
        self.event_bus
            .emit(Event::ResourceWarning {
                resource_type: "health".to_string(),
                usage_percent: (1.0 - health.overall_score) * 100.0,
                threshold: 80.0,
                timestamp: Instant::now(),
            })
            .await
            .ok();
    }
}

// Module health types

#[derive(Debug, Clone)]
pub struct ModuleHealth {
    pub module_name: String,
    pub status: HealthStatus,
    pub score: f64, // 0.0 to 1.0
    pub checks: Vec<HealthCheckResult>,
    pub last_check: Instant,
}

impl ModuleHealth {
    pub fn healthy() -> Self {
        Self {
            module_name: String::new(),
            status: HealthStatus::Healthy,
            score: 1.0,
            checks: Vec::new(),
            last_check: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct OverallHealth {
    pub overall_score: f64,
    pub module_scores: HashMap<String, f64>,
    pub status: HealthStatus,
}

impl OverallHealth {
    pub fn calculate(modules: Vec<ModuleHealth>) -> Self {
        let mut total_score = 0.0;
        let mut module_scores = HashMap::new();

        for module in &modules {
            total_score += module.score;
            module_scores.insert(module.module_name.clone(), module.score);
        }

        let overall_score = if modules.is_empty() {
            1.0
        } else {
            total_score / modules.len() as f64
        };

        let status = if overall_score >= 0.9 {
            HealthStatus::Healthy
        } else if overall_score >= 0.7 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Critical
        };

        Self {
            overall_score,
            module_scores,
            status,
        }
    }
}

// Metrics types

#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub session_id: String,
    pub started_at: Instant,
    pub operation_count: u64,
    pub total_duration_ms: u64,
    pub error_count: u64,
}

pub struct HealthCheck {
    pub name: String,
    pub check_fn: Arc<dyn Fn() -> bool + Send + Sync>,
    pub interval: Duration,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub timestamp: Instant,
    pub severity: AlertSeverity,
    pub message: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}
