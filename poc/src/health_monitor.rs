//! Health Monitoring & Diagnostics System
//!
//! This module provides comprehensive health monitoring, system diagnostics,
//! performance tracking, and operational insights for the RainbowBrowserAI system.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::llm_integration::LLMMetrics;
use crate::contextual_awareness::ContextSnapshot;

/// Health monitoring system for production operations
pub struct HealthMonitor {
    /// Unique monitoring session identifier
    session_id: Uuid,
    /// System health metrics
    health_metrics: Arc<RwLock<SystemHealthMetrics>>,
    /// Performance metrics
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Health check registry
    health_checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
    /// Diagnostic data collection
    diagnostics: Arc<RwLock<DiagnosticData>>,
    /// Monitoring configuration
    config: Arc<RwLock<HealthMonitorConfig>>,
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<HealthAlert>>>,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Enable continuous health monitoring
    pub enable_monitoring: bool,
    /// Health check interval (seconds)
    pub health_check_interval_seconds: u64,
    /// Performance metrics collection interval (seconds)
    pub metrics_collection_interval_seconds: u64,
    /// Maximum alert history size
    pub max_alert_history: usize,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Enable detailed diagnostics
    pub enable_detailed_diagnostics: bool,
    /// Diagnostic data retention period (hours)
    pub diagnostic_retention_hours: u64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            health_check_interval_seconds: 30,
            metrics_collection_interval_seconds: 60,
            max_alert_history: 1000,
            alert_thresholds: AlertThresholds::default(),
            enable_detailed_diagnostics: true,
            diagnostic_retention_hours: 24,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_usage_threshold: f64,
    /// Memory usage threshold (percentage)
    pub memory_usage_threshold: f64,
    /// Error rate threshold (errors per minute)
    pub error_rate_threshold: f64,
    /// Response time threshold (milliseconds)
    pub response_time_threshold: u64,
    /// Success rate threshold (percentage)
    pub success_rate_threshold: f64,
    /// Browser automation failure threshold
    pub browser_failure_threshold: u32,
    /// LLM service failure threshold
    pub llm_failure_threshold: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_threshold: 80.0,
            memory_usage_threshold: 85.0,
            error_rate_threshold: 10.0,
            response_time_threshold: 5000,
            success_rate_threshold: 90.0,
            browser_failure_threshold: 3,
            llm_failure_threshold: 5,
        }
    }
}

/// System health metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Individual component health
    pub component_health: HashMap<String, ComponentHealth>,
    /// System resource usage
    pub resource_usage: ResourceUsage,
    /// Service availability metrics
    pub service_availability: HashMap<String, ServiceAvailability>,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operating normally
    Healthy,
    /// Minor issues detected, system functional
    Warning,
    /// Significant issues, degraded performance
    Degraded,
    /// Critical issues, system may be unstable
    Critical,
    /// System is down or unreachable
    Down,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Current health status
    pub status: HealthStatus,
    /// Health check timestamp
    pub last_check: DateTime<Utc>,
    /// Detailed status message
    pub message: String,
    /// Response time for health check (ms)
    pub response_time_ms: u64,
    /// Success/failure counts
    pub success_count: u64,
    pub failure_count: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System resource usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Available memory (MB)
    pub available_memory_mb: u64,
    /// Total memory (MB)
    pub total_memory_mb: u64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Available disk space (MB)
    pub available_disk_mb: u64,
    /// Network I/O metrics
    pub network_io: NetworkIO,
    /// Process count
    pub active_processes: u32,
}

/// Network I/O metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkIO {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Active connections
    pub active_connections: u32,
}

/// Service availability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAvailability {
    /// Service name
    pub service_name: String,
    /// Current availability status
    pub is_available: bool,
    /// Uptime percentage (last 24h)
    pub uptime_percentage: f64,
    /// Total uptime (seconds)
    pub total_uptime_seconds: u64,
    /// Total downtime (seconds)
    pub total_downtime_seconds: u64,
    /// Last availability check
    pub last_check: DateTime<Utc>,
    /// Response times (last 10 checks)
    pub recent_response_times: Vec<u64>,
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Request processing metrics
    pub request_metrics: RequestMetrics,
    /// Browser automation metrics
    pub browser_metrics: BrowserMetrics,
    /// LLM service metrics
    pub llm_metrics: Option<LLMMetrics>,
    /// Cache performance metrics
    pub cache_metrics: CacheMetrics,
    /// Error tracking metrics
    pub error_metrics: ErrorMetrics,
    /// Throughput metrics
    pub throughput_metrics: ThroughputMetrics,
}

/// Request processing metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time (ms)
    pub average_response_time_ms: f64,
    /// 95th percentile response time (ms)
    pub p95_response_time_ms: u64,
    /// 99th percentile response time (ms)
    pub p99_response_time_ms: u64,
    /// Requests per second
    pub requests_per_second: f64,
}

/// Browser automation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserMetrics {
    /// Active browser sessions
    pub active_sessions: u32,
    /// Total browser actions
    pub total_actions: u64,
    /// Successful browser actions
    pub successful_actions: u64,
    /// Failed browser actions
    pub failed_actions: u64,
    /// Average action execution time (ms)
    pub average_action_time_ms: f64,
    /// Browser pool utilization
    pub pool_utilization_percent: f64,
    /// Screenshot count
    pub screenshots_taken: u64,
    /// Page navigation count
    pub pages_navigated: u64,
}

/// Cache performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Hit ratio percentage
    pub hit_ratio_percent: f64,
    /// Cache size (entries)
    pub cache_size: u64,
    /// Cache memory usage (MB)
    pub cache_memory_mb: u64,
    /// Average cache lookup time (ms)
    pub average_lookup_time_ms: f64,
}

/// Error tracking metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors
    pub total_errors: u64,
    /// Errors by category
    pub errors_by_category: HashMap<String, u64>,
    /// Error rate (errors per minute)
    pub error_rate_per_minute: f64,
    /// Recent errors (last 100)
    pub recent_errors: Vec<ErrorSummary>,
    /// Top error patterns
    pub top_error_patterns: Vec<ErrorPattern>,
}

/// Error summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    /// Error category
    pub category: String,
    /// Error message (truncated)
    pub message: String,
    /// Error severity
    pub severity: String,
}

/// Error pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern signature
    pub pattern: String,
    /// Occurrence count
    pub count: u32,
    /// First seen
    pub first_seen: DateTime<Utc>,
    /// Last seen
    pub last_seen: DateTime<Utc>,
}

/// Throughput metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Tasks completed per hour
    pub tasks_per_hour: f64,
    /// Pages processed per hour
    pub pages_per_hour: f64,
    /// Screenshots per hour
    pub screenshots_per_hour: f64,
    /// Data extracted per hour (MB)
    pub data_extracted_mb_per_hour: f64,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert ID
    pub alert_id: Uuid,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Alert level
    pub level: AlertLevel,
    /// Component that triggered the alert
    pub component: String,
    /// Alert message
    pub message: String,
    /// Alert details
    pub details: HashMap<String, serde_json::Value>,
    /// Whether alert is resolved
    pub resolved: bool,
    /// Resolution timestamp
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

/// Diagnostic data collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticData {
    /// System information
    pub system_info: SystemInfo,
    /// Configuration snapshot
    pub configuration_snapshot: HashMap<String, serde_json::Value>,
    /// Recent performance samples
    pub performance_samples: VecDeque<PerformanceSample>,
    /// Component diagnostics
    pub component_diagnostics: HashMap<String, ComponentDiagnostic>,
    /// Environment information
    pub environment_info: EnvironmentInfo,
}

/// System information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Architecture
    pub architecture: String,
    /// Hostname
    pub hostname: String,
    /// Process ID
    pub process_id: u32,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// Uptime (seconds)
    pub uptime_seconds: u64,
}

/// Performance sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    /// Sample timestamp
    pub timestamp: DateTime<Utc>,
    /// CPU usage at sample time
    pub cpu_usage: f64,
    /// Memory usage at sample time
    pub memory_usage: f64,
    /// Active requests at sample time
    pub active_requests: u32,
    /// Response time at sample time
    pub response_time_ms: u64,
}

/// Component diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDiagnostic {
    /// Component name
    pub component_name: String,
    /// Component version
    pub version: String,
    /// Configuration
    pub configuration: HashMap<String, serde_json::Value>,
    /// Status information
    pub status_info: HashMap<String, serde_json::Value>,
    /// Performance stats
    pub performance_stats: HashMap<String, f64>,
    /// Recent operations
    pub recent_operations: Vec<String>,
}

/// Environment information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Environment variables (filtered)
    pub environment_variables: HashMap<String, String>,
    /// Runtime information
    pub runtime_info: HashMap<String, String>,
    /// Dependency versions
    pub dependency_versions: HashMap<String, String>,
}

/// Health check trait
pub trait HealthCheck {
    /// Perform health check
    fn check(&self) -> Result<ComponentHealth>;
    
    /// Get check name
    fn name(&self) -> &str;
    
    /// Get check description
    fn description(&self) -> &str;
}

/// Basic health check implementation
pub struct BasicHealthCheck {
    name: String,
    description: String,
    check_fn: Box<dyn Fn() -> Result<bool> + Send + Sync>,
}

impl BasicHealthCheck {
    pub fn new<F>(name: String, description: String, check_fn: F) -> Self
    where
        F: Fn() -> Result<bool> + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            check_fn: Box::new(check_fn),
        }
    }
}

impl HealthCheck for BasicHealthCheck {
    fn check(&self) -> Result<ComponentHealth> {
        let start_time = std::time::Instant::now();
        let result = (self.check_fn)();
        let response_time = start_time.elapsed().as_millis() as u64;

        let (status, message) = match result {
            Ok(true) => (HealthStatus::Healthy, "Health check passed".to_string()),
            Ok(false) => (HealthStatus::Warning, "Health check returned false".to_string()),
            Err(e) => (HealthStatus::Critical, format!("Health check failed: {}", e)),
        };

        Ok(ComponentHealth {
            name: self.name.clone(),
            status,
            last_check: Utc::now(),
            message,
            response_time_ms: response_time,
            success_count: if matches!(status, HealthStatus::Healthy) { 1 } else { 0 },
            failure_count: if matches!(status, HealthStatus::Healthy) { 0 } else { 1 },
            metadata: HashMap::new(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl HealthMonitor {
    /// Create new health monitor
    pub async fn new(config: HealthMonitorConfig) -> Result<Self> {
        let monitor = Self {
            session_id: Uuid::new_v4(),
            health_metrics: Arc::new(RwLock::new(SystemHealthMetrics::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            diagnostics: Arc::new(RwLock::new(DiagnosticData::default())),
            config: Arc::new(RwLock::new(config)),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
        };

        // Initialize system diagnostics
        monitor.initialize_system_diagnostics().await?;

        // Register default health checks
        monitor.register_default_health_checks().await?;

        info!("🏥 Health Monitor initialized (session: {})", monitor.session_id);
        Ok(monitor)
    }

    /// Start continuous monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.enable_monitoring {
            warn!("Health monitoring is disabled");
            return Ok(());
        }

        info!("🔍 Starting continuous health monitoring");

        // Start health check loop
        let health_check_interval = config.health_check_interval_seconds;
        let health_metrics = self.health_metrics.clone();
        let health_checks = self.health_checks.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(health_check_interval));
            loop {
                interval.tick().await;
                if let Err(e) = Self::run_health_checks(health_metrics.clone(), health_checks.clone()).await {
                    warn!("Health check error: {}", e);
                }
            }
        });

        // Start metrics collection loop
        let metrics_interval = config.metrics_collection_interval_seconds;
        let performance_metrics = self.performance_metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(metrics_interval));
            loop {
                interval.tick().await;
                if let Err(e) = Self::collect_performance_metrics(performance_metrics.clone()).await {
                    warn!("Metrics collection error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Run all registered health checks
    async fn run_health_checks(
        health_metrics: Arc<RwLock<SystemHealthMetrics>>,
        health_checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>
    ) -> Result<()> {
        let checks = health_checks.read().await;
        let mut component_healths = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for (name, check) in checks.iter() {
            match check.check() {
                Ok(health) => {
                    // Update overall status based on component status
                    match health.status {
                        HealthStatus::Critical | HealthStatus::Down => overall_status = HealthStatus::Critical,
                        HealthStatus::Degraded if overall_status == HealthStatus::Healthy => overall_status = HealthStatus::Degraded,
                        HealthStatus::Warning if matches!(overall_status, HealthStatus::Healthy) => overall_status = HealthStatus::Warning,
                        _ => {}
                    }
                    component_healths.insert(name.clone(), health);
                },
                Err(e) => {
                    warn!("Health check failed for {}: {}", name, e);
                    overall_status = HealthStatus::Critical;
                    component_healths.insert(name.clone(), ComponentHealth {
                        name: name.clone(),
                        status: HealthStatus::Critical,
                        last_check: Utc::now(),
                        message: format!("Health check error: {}", e),
                        response_time_ms: 0,
                        success_count: 0,
                        failure_count: 1,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Update health metrics
        {
            let mut metrics = health_metrics.write().await;
            metrics.overall_status = overall_status;
            metrics.component_health = component_healths;
            metrics.last_updated = Some(Utc::now());
        }

        debug!("Health checks completed: overall status = {:?}", overall_status);
        Ok(())
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(performance_metrics: Arc<RwLock<PerformanceMetrics>>) -> Result<()> {
        // Simulate metrics collection (in production, this would gather real metrics)
        let mut metrics = performance_metrics.write().await;
        
        // Update request metrics
        metrics.request_metrics.total_requests += 1;
        metrics.request_metrics.successful_requests += 1;
        metrics.request_metrics.average_response_time_ms = 150.0;
        metrics.request_metrics.requests_per_second = 2.5;

        // Update browser metrics
        metrics.browser_metrics.active_sessions = 2;
        metrics.browser_metrics.total_actions += 5;
        metrics.browser_metrics.successful_actions += 4;
        metrics.browser_metrics.pool_utilization_percent = 40.0;

        // Update cache metrics
        metrics.cache_metrics.cache_hits += 3;
        metrics.cache_metrics.cache_misses += 1;
        metrics.cache_metrics.hit_ratio_percent = 75.0;

        // Update throughput metrics
        metrics.throughput_metrics.tasks_per_hour = 120.0;
        metrics.throughput_metrics.pages_per_hour = 480.0;

        debug!("Performance metrics collected");
        Ok(())
    }

    /// Register a health check
    pub async fn register_health_check<T>(&self, health_check: T) -> Result<()>
    where
        T: HealthCheck + Send + Sync + 'static,
    {
        let name = health_check.name().to_string();
        let mut checks = self.health_checks.write().await;
        checks.insert(name.clone(), Box::new(health_check));
        info!("Registered health check: {}", name);
        Ok(())
    }

    /// Register default health checks
    async fn register_default_health_checks(&self) -> Result<()> {
        // System health check
        let system_check = BasicHealthCheck::new(
            "system".to_string(),
            "Basic system health check".to_string(),
            || Ok(true), // Always healthy in mock mode
        );
        self.register_health_check(system_check).await?;

        // Browser pool health check
        let browser_check = BasicHealthCheck::new(
            "browser_pool".to_string(),
            "Browser pool availability check".to_string(),
            || Ok(true), // Always healthy in mock mode
        );
        self.register_health_check(browser_check).await?;

        // LLM service health check
        let llm_check = BasicHealthCheck::new(
            "llm_service".to_string(),
            "LLM service connectivity check".to_string(),
            || Ok(true), // Always healthy in mock mode
        );
        self.register_health_check(llm_check).await?;

        info!("Registered {} default health checks", 3);
        Ok(())
    }

    /// Initialize system diagnostics
    async fn initialize_system_diagnostics(&self) -> Result<()> {
        let mut diagnostics = self.diagnostics.write().await;
        
        // Initialize system info
        diagnostics.system_info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            os_version: "Unknown".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            hostname: "localhost".to_string(),
            process_id: std::process::id(),
            start_time: Utc::now(),
            uptime_seconds: 0,
        };

        // Initialize environment info
        diagnostics.environment_info.runtime_info.insert(
            "rust_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string()
        );

        info!("System diagnostics initialized");
        Ok(())
    }

    /// Generate health report
    pub async fn generate_health_report(&self) -> Result<HealthReport> {
        let health_metrics = self.health_metrics.read().await;
        let performance_metrics = self.performance_metrics.read().await;
        let diagnostics = self.diagnostics.read().await;
        let recent_alerts = self.get_recent_alerts(10).await;

        let report = HealthReport {
            report_id: Uuid::new_v4(),
            generated_at: Utc::now(),
            overall_status: health_metrics.overall_status,
            component_count: health_metrics.component_health.len() as u32,
            healthy_components: health_metrics.component_health.values()
                .filter(|c| c.status == HealthStatus::Healthy).count() as u32,
            warning_components: health_metrics.component_health.values()
                .filter(|c| c.status == HealthStatus::Warning).count() as u32,
            critical_components: health_metrics.component_health.values()
                .filter(|c| matches!(c.status, HealthStatus::Critical | HealthStatus::Down)).count() as u32,
            performance_summary: PerformanceSummary {
                avg_response_time_ms: performance_metrics.request_metrics.average_response_time_ms,
                success_rate_percent: if performance_metrics.request_metrics.total_requests > 0 {
                    (performance_metrics.request_metrics.successful_requests as f64 / 
                     performance_metrics.request_metrics.total_requests as f64) * 100.0
                } else { 100.0 },
                throughput_per_hour: performance_metrics.throughput_metrics.tasks_per_hour,
                cache_hit_rate_percent: performance_metrics.cache_metrics.hit_ratio_percent,
            },
            system_resources: health_metrics.resource_usage.clone(),
            recent_alerts,
            uptime_seconds: diagnostics.system_info.uptime_seconds,
            recommendations: self.generate_recommendations(&health_metrics, &performance_metrics).await,
        };

        Ok(report)
    }

    /// Generate system recommendations
    async fn generate_recommendations(&self, health_metrics: &SystemHealthMetrics, performance_metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check resource usage
        if health_metrics.resource_usage.memory_usage_percent > 80.0 {
            recommendations.push("Consider increasing available memory or optimizing memory usage".to_string());
        }

        if health_metrics.resource_usage.cpu_usage_percent > 75.0 {
            recommendations.push("High CPU usage detected - consider scaling or optimization".to_string());
        }

        // Check performance metrics
        if performance_metrics.request_metrics.average_response_time_ms > 1000.0 {
            recommendations.push("Response times are elevated - investigate performance bottlenecks".to_string());
        }

        if performance_metrics.cache_metrics.hit_ratio_percent < 50.0 {
            recommendations.push("Low cache hit ratio - review caching strategy".to_string());
        }

        // Check component health
        let unhealthy_count = health_metrics.component_health.values()
            .filter(|c| !matches!(c.status, HealthStatus::Healthy)).count();
        
        if unhealthy_count > 0 {
            recommendations.push(format!("Address {} unhealthy components", unhealthy_count));
        }

        if recommendations.is_empty() {
            recommendations.push("System is operating within normal parameters".to_string());
        }

        recommendations
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<HealthAlert> {
        let alerts = self.alert_history.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Get system health metrics
    pub async fn get_health_metrics(&self) -> SystemHealthMetrics {
        self.health_metrics.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }
}

/// Health report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Report ID
    pub report_id: Uuid,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Overall system status
    pub overall_status: HealthStatus,
    /// Total component count
    pub component_count: u32,
    /// Healthy component count
    pub healthy_components: u32,
    /// Warning component count
    pub warning_components: u32,
    /// Critical component count
    pub critical_components: u32,
    /// Performance summary
    pub performance_summary: PerformanceSummary,
    /// System resource usage
    pub system_resources: ResourceUsage,
    /// Recent alerts
    pub recent_alerts: Vec<HealthAlert>,
    /// System uptime (seconds)
    pub uptime_seconds: u64,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Success rate percentage
    pub success_rate_percent: f64,
    /// Throughput per hour
    pub throughput_per_hour: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate_percent: f64,
}

/// Create health monitor with default configuration
pub async fn create_health_monitor() -> Result<HealthMonitor> {
    let config = HealthMonitorConfig::default();
    HealthMonitor::new(config).await
}

/// Create health monitor with custom configuration
pub async fn create_custom_health_monitor(config: HealthMonitorConfig) -> Result<HealthMonitor> {
    HealthMonitor::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = create_health_monitor().await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_registration() {
        let monitor = create_health_monitor().await.unwrap();
        
        let test_check = BasicHealthCheck::new(
            "test".to_string(),
            "Test health check".to_string(),
            || Ok(true),
        );
        
        let result = monitor.register_health_check(test_check).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_report_generation() {
        let monitor = create_health_monitor().await.unwrap();
        let report = monitor.generate_health_report().await;
        assert!(report.is_ok());
    }
}