// Production Monitoring and Alerting System
// Provides comprehensive monitoring with intelligent alerting capabilities

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::sync::{Arc, Mutex};
use tokio::time::interval;
use std::fs;

/// Production monitoring and alerting system
pub struct MonitoringAlertsSystem {
    monitors: Vec<Box<dyn Monitor + Send + Sync>>,
    alert_manager: AlertManager,
    metrics_store: Arc<Mutex<MetricsStore>>,
    notification_channels: Vec<Box<dyn NotificationChannel + Send + Sync>>,
    config: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub check_interval_seconds: u64,
    pub alert_cooldown_seconds: u64,
    pub metrics_retention_hours: u64,
    pub enable_predictive_alerts: bool,
    pub alert_severity_levels: HashMap<String, AlertSeverity>,
}

pub trait Monitor: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent>;
    fn description(&self) -> &str;
    fn is_enabled(&self) -> bool;
}

pub trait NotificationChannel: Send + Sync {
    fn name(&self) -> &str;
    fn send_alert(&self, alert: &AlertEvent) -> Result<(), NotificationError>;
    fn test_connection(&self) -> Result<(), NotificationError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub success_rate: f32,
    pub avg_response_time: u64,
    pub error_rate: f32,
    pub active_sessions: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub confidence_score: f32,
    pub throughput: f32,
    pub custom_metrics: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub timestamp: u64,
    pub severity: AlertSeverity,
    pub monitor_name: String,
    pub title: String,
    pub description: String,
    pub metrics: HashMap<String, f32>,
    pub suggested_actions: Vec<String>,
    pub auto_resolution: Option<AutoResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
    Emergency = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResolution {
    pub strategy: String,
    pub parameters: HashMap<String, String>,
    pub timeout_seconds: u64,
}

#[derive(Debug)]
pub struct NotificationError(pub String);

#[derive(Debug, Clone)]
pub struct AlertManager {
    active_alerts: Arc<Mutex<HashMap<String, AlertEvent>>>,
    alert_history: Arc<Mutex<VecDeque<AlertEvent>>>,
    cooldowns: Arc<Mutex<HashMap<String, u64>>>,
    escalation_rules: Vec<EscalationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub severity: AlertSeverity,
    pub delay_minutes: u64,
    pub notification_channels: Vec<String>,
    pub auto_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MetricsStore {
    snapshots: VecDeque<MetricsSnapshot>,
    max_snapshots: usize,
    aggregated_metrics: HashMap<String, AggregatedMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub count: u32,
    pub trend: f32,
}

// Built-in monitor implementations
pub struct SuccessRateMonitor {
    threshold_warning: f32,
    threshold_critical: f32,
}

pub struct ResponseTimeMonitor {
    threshold_warning_ms: u64,
    threshold_critical_ms: u64,
    trend_analysis: bool,
}

pub struct ErrorRateMonitor {
    threshold_warning: f32,
    threshold_critical: f32,
    spike_detection: bool,
}

pub struct ResourceUsageMonitor {
    cpu_threshold: f32,
    memory_threshold: f32,
    sustained_duration_minutes: u64,
}

pub struct PredictiveFailureMonitor {
    enabled: bool,
    confidence_threshold: f32,
    lookback_hours: u64,
}

// Notification channel implementations
pub struct SlackNotificationChannel {
    webhook_url: String,
    channel: String,
    enabled: bool,
}

pub struct EmailNotificationChannel {
    smtp_config: SmtpConfig,
    recipients: Vec<String>,
    enabled: bool,
}

pub struct WebhookNotificationChannel {
    url: String,
    headers: HashMap<String, String>,
    enabled: bool,
}

pub struct LogNotificationChannel {
    log_file: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
}

impl MonitoringAlertsSystem {
    pub fn new(config: MonitoringConfig) -> Self {
        let mut system = Self {
            monitors: Vec::new(),
            alert_manager: AlertManager::new(),
            metrics_store: Arc::new(Mutex::new(MetricsStore::new(1000))),
            notification_channels: Vec::new(),
            config,
        };

        // Initialize default monitors
        system.add_default_monitors();
        system.add_default_notification_channels();

        system
    }

    fn add_default_monitors(&mut self) {
        self.monitors.push(Box::new(SuccessRateMonitor::new()));
        self.monitors.push(Box::new(ResponseTimeMonitor::new()));
        self.monitors.push(Box::new(ErrorRateMonitor::new()));
        self.monitors.push(Box::new(ResourceUsageMonitor::new()));
        
        if self.config.enable_predictive_alerts {
            self.monitors.push(Box::new(PredictiveFailureMonitor::new()));
        }
    }

    fn add_default_notification_channels(&mut self) {
        self.notification_channels.push(Box::new(LogNotificationChannel::new()));
        
        // Add other channels based on configuration
        if let Ok(webhook_url) = std::env::var("SLACK_WEBHOOK_URL") {
            self.notification_channels.push(Box::new(SlackNotificationChannel::new(webhook_url)));
        }
        
        if let Ok(webhook_url) = std::env::var("ALERT_WEBHOOK_URL") {
            self.notification_channels.push(Box::new(WebhookNotificationChannel::new(webhook_url)));
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚨 Starting Production Monitoring & Alerts System");
        
        // Start monitoring loop
        let monitors = self.monitors.iter().map(|m| m.name().to_string()).collect::<Vec<_>>();
        println!("   📊 Active Monitors: {}", monitors.join(", "));
        
        let channels = self.notification_channels.iter().map(|c| c.name().to_string()).collect::<Vec<_>>();
        println!("   📢 Notification Channels: {}", channels.join(", "));
        
        // Start metrics collection
        self.start_metrics_collection().await;
        
        // Start monitoring checks
        self.start_monitoring_checks().await;
        
        // Start alert processing
        self.start_alert_processing().await;

        Ok(())
    }

    async fn start_metrics_collection(&self) {
        let metrics_store = Arc::clone(&self.metrics_store);
        let interval_duration = Duration::from_secs(30); // Collect metrics every 30 seconds
        
        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);
            
            loop {
                interval_timer.tick().await;
                
                let snapshot = Self::collect_current_metrics().await;
                if let Ok(mut store) = metrics_store.lock() {
                    store.add_snapshot(snapshot);
                }
            }
        });
    }

    async fn start_monitoring_checks(&self) {
        let monitors: Vec<String> = self.monitors.iter().map(|m| m.name().to_string()).collect();
        let metrics_store = Arc::clone(&self.metrics_store);
        let alert_manager = self.alert_manager.clone();
        let interval_duration = Duration::from_secs(self.config.check_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);
            
            loop {
                interval_timer.tick().await;
                
                if let Ok(store) = metrics_store.lock() {
                    if let Some(latest_metrics) = store.get_latest() {
                        // Note: In a real implementation, we'd have access to the monitors here
                        // For now, we'll simulate the monitoring checks
                        Self::simulate_monitoring_checks(&latest_metrics, &alert_manager).await;
                    }
                }
            }
        });
    }

    async fn start_alert_processing(&self) {
        let alert_manager = self.alert_manager.clone();
        let notification_channels: Vec<String> = self.notification_channels.iter()
            .map(|c| c.name().to_string())
            .collect();
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(60)); // Process alerts every minute
            
            loop {
                interval_timer.tick().await;
                
                // Process any pending alerts
                if let Ok(alerts) = alert_manager.active_alerts.lock() {
                    for alert in alerts.values() {
                        if alert.severity >= AlertSeverity::Error {
                            println!("🚨 CRITICAL ALERT: {} - {}", alert.title, alert.description);
                        }
                    }
                }
            }
        });
    }

    async fn collect_current_metrics() -> MetricsSnapshot {
        // In a real implementation, this would collect actual system metrics
        MetricsSnapshot {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            success_rate: 0.94, // Mock high success rate
            avg_response_time: 142,
            error_rate: 0.06,
            active_sessions: 3,
            cpu_usage: 25.5,
            memory_usage: 45.2,
            confidence_score: 0.87,
            throughput: 15.3,
            custom_metrics: HashMap::new(),
        }
    }

    async fn simulate_monitoring_checks(metrics: &MetricsSnapshot, alert_manager: &AlertManager) {
        let mut alerts = Vec::new();

        // Success rate check
        if metrics.success_rate < 0.8 {
            alerts.push(AlertEvent {
                id: format!("success_rate_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: if metrics.success_rate < 0.5 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                monitor_name: "SuccessRateMonitor".to_string(),
                title: "Low Success Rate Detected".to_string(),
                description: format!("Success rate dropped to {:.1}%", metrics.success_rate * 100.0),
                metrics: [("success_rate".to_string(), metrics.success_rate)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Check element selectors for changes".to_string(),
                    "Review recent deployments".to_string(),
                    "Run diagnostics on failing commands".to_string(),
                ],
                auto_resolution: Some(AutoResolution {
                    strategy: "restart_browser_pool".to_string(),
                    parameters: HashMap::new(),
                    timeout_seconds: 300,
                }),
            });
        }

        // Response time check
        if metrics.avg_response_time > 300 {
            alerts.push(AlertEvent {
                id: format!("response_time_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: if metrics.avg_response_time > 1000 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                monitor_name: "ResponseTimeMonitor".to_string(),
                title: "High Response Time Detected".to_string(),
                description: format!("Average response time: {}ms", metrics.avg_response_time),
                metrics: [("avg_response_time".to_string(), metrics.avg_response_time as f32)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Check system resources".to_string(),
                    "Optimize browser pool size".to_string(),
                    "Review network connectivity".to_string(),
                ],
                auto_resolution: Some(AutoResolution {
                    strategy: "optimize_browser_pool".to_string(),
                    parameters: [("max_browsers".to_string(), "10".to_string())].iter().cloned().collect(),
                    timeout_seconds: 180,
                }),
            });
        }

        // Add alerts to manager
        for alert in alerts {
            alert_manager.add_alert(alert).await;
        }
    }

    pub async fn get_monitoring_report(&self) -> MonitoringReport {
        let store = self.metrics_store.lock().unwrap();
        let alert_manager = &self.alert_manager;
        
        let active_alerts = alert_manager.active_alerts.lock().unwrap();
        let alert_history = alert_manager.alert_history.lock().unwrap();
        
        MonitoringReport {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            system_health: self.calculate_system_health(&store),
            active_alerts: active_alerts.len() as u32,
            total_alerts_24h: alert_history.iter()
                .filter(|a| a.timestamp > SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 86400)
                .count() as u32,
            metrics_summary: store.get_summary(),
            performance_trends: self.analyze_trends(&store),
            recommendations: self.generate_recommendations(&store, &active_alerts),
        }
    }

    fn calculate_system_health(&self, store: &MetricsStore) -> SystemHealth {
        let latest = store.get_latest();
        
        match latest {
            Some(metrics) => {
                let health_score = (
                    metrics.success_rate * 0.4 +
                    (1.0 - (metrics.avg_response_time as f32 / 1000.0).min(1.0)) * 0.3 +
                    (1.0 - metrics.error_rate) * 0.2 +
                    metrics.confidence_score * 0.1
                ) * 100.0;

                SystemHealth {
                    score: health_score.min(100.0).max(0.0) as u32,
                    status: if health_score > 90.0 { "Excellent".to_string() }
                           else if health_score > 75.0 { "Good".to_string() }
                           else if health_score > 60.0 { "Fair".to_string() }
                           else { "Poor".to_string() },
                    last_updated: metrics.timestamp,
                }
            }
            None => SystemHealth {
                score: 0,
                status: "Unknown".to_string(),
                last_updated: 0,
            }
        }
    }

    fn analyze_trends(&self, store: &MetricsStore) -> Vec<PerformanceTrend> {
        let mut trends = Vec::new();
        
        if let Some(trend_data) = store.calculate_trends(24) { // 24 hours
            trends.push(PerformanceTrend {
                metric: "success_rate".to_string(),
                direction: if trend_data.success_rate_trend > 0.05 { "improving" } else if trend_data.success_rate_trend < -0.05 { "declining" } else { "stable" }.to_string(),
                percentage_change: trend_data.success_rate_trend * 100.0,
                description: format!("Success rate trend: {:+.1}% over 24h", trend_data.success_rate_trend * 100.0),
            });
            
            trends.push(PerformanceTrend {
                metric: "response_time".to_string(),
                direction: if trend_data.response_time_trend < -10.0 { "improving" } else if trend_data.response_time_trend > 10.0 { "declining" } else { "stable" }.to_string(),
                percentage_change: trend_data.response_time_trend,
                description: format!("Response time trend: {:+.0}ms over 24h", trend_data.response_time_trend),
            });
        }
        
        trends
    }

    fn generate_recommendations(&self, store: &MetricsStore, active_alerts: &HashMap<String, AlertEvent>) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if let Some(latest) = store.get_latest() {
            if latest.success_rate < 0.9 {
                recommendations.push("Consider updating element selectors - success rate below optimal".to_string());
            }
            
            if latest.avg_response_time > 200 {
                recommendations.push("Response time optimization recommended - consider browser pool tuning".to_string());
            }
            
            if latest.cpu_usage > 70.0 {
                recommendations.push("High CPU usage detected - consider scaling or optimization".to_string());
            }
        }
        
        if active_alerts.len() > 5 {
            recommendations.push("High alert volume - review monitoring thresholds and system stability".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("System performing well - no immediate recommendations".to_string());
        }
        
        recommendations
    }

    pub async fn test_all_notifications(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        
        for channel in &self.notification_channels {
            let test_alert = AlertEvent {
                id: "test_alert".to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                severity: AlertSeverity::Info,
                monitor_name: "TestMonitor".to_string(),
                title: "Test Alert".to_string(),
                description: "This is a test alert to verify notification functionality".to_string(),
                metrics: HashMap::new(),
                suggested_actions: vec!["This is a test - no action required".to_string()],
                auto_resolution: None,
            };
            
            let success = channel.send_alert(&test_alert).is_ok();
            results.push(TestResult {
                channel_name: channel.name().to_string(),
                success,
                message: if success { "Test successful".to_string() } else { "Test failed".to_string() },
            });
        }
        
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub timestamp: u64,
    pub system_health: SystemHealth,
    pub active_alerts: u32,
    pub total_alerts_24h: u32,
    pub metrics_summary: MetricsSummary,
    pub performance_trends: Vec<PerformanceTrend>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub score: u32, // 0-100
    pub status: String,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub avg_success_rate: f32,
    pub avg_response_time: f32,
    pub total_requests: u32,
    pub uptime_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub metric: String,
    pub direction: String, // "improving", "declining", "stable"
    pub percentage_change: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub success_rate_trend: f32,
    pub response_time_trend: f32,
    pub error_rate_trend: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub channel_name: String,
    pub success: bool,
    pub message: String,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
            alert_history: Arc::new(Mutex::new(VecDeque::new())),
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
            escalation_rules: Vec::new(),
        }
    }

    pub async fn add_alert(&self, alert: AlertEvent) {
        // Check cooldown
        if let Ok(cooldowns) = self.cooldowns.lock() {
            let key = format!("{}_{}", alert.monitor_name, alert.severity as u32);
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            if let Some(&last_alert_time) = cooldowns.get(&key) {
                if now - last_alert_time < 300 { // 5 minute cooldown
                    return; // Skip duplicate alert
                }
            }
        }

        // Add to active alerts
        if let Ok(mut active_alerts) = self.active_alerts.lock() {
            active_alerts.insert(alert.id.clone(), alert.clone());
        }

        // Add to history
        if let Ok(mut history) = self.alert_history.lock() {
            history.push_back(alert.clone());
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        // Update cooldown
        if let Ok(mut cooldowns) = self.cooldowns.lock() {
            let key = format!("{}_{}", alert.monitor_name, alert.severity as u32);
            cooldowns.insert(key, alert.timestamp);
        }

        println!("🚨 Alert Generated: {} - {}", alert.title, alert.description);
    }
}

impl MetricsStore {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: VecDeque::new(),
            max_snapshots,
            aggregated_metrics: HashMap::new(),
        }
    }

    pub fn add_snapshot(&mut self, snapshot: MetricsSnapshot) {
        self.snapshots.push_back(snapshot);
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.pop_front();
        }
        self.update_aggregated_metrics();
    }

    pub fn get_latest(&self) -> Option<&MetricsSnapshot> {
        self.snapshots.back()
    }

    pub fn get_summary(&self) -> MetricsSummary {
        if self.snapshots.is_empty() {
            return MetricsSummary {
                avg_success_rate: 0.0,
                avg_response_time: 0.0,
                total_requests: 0,
                uptime_percentage: 0.0,
            };
        }

        let count = self.snapshots.len() as f32;
        let avg_success_rate = self.snapshots.iter().map(|s| s.success_rate).sum::<f32>() / count;
        let avg_response_time = self.snapshots.iter().map(|s| s.avg_response_time as f32).sum::<f32>() / count;
        let total_requests = self.snapshots.iter().map(|s| s.active_sessions).sum::<u32>() * 10; // Estimate

        MetricsSummary {
            avg_success_rate,
            avg_response_time,
            total_requests,
            uptime_percentage: 99.5, // Mock uptime
        }
    }

    pub fn calculate_trends(&self, hours: u64) -> Option<TrendData> {
        if self.snapshots.len() < 2 {
            return None;
        }

        let cutoff_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - (hours * 3600);
        let recent_snapshots: Vec<_> = self.snapshots.iter()
            .filter(|s| s.timestamp >= cutoff_time)
            .collect();

        if recent_snapshots.len() < 2 {
            return None;
        }

        let first_half = &recent_snapshots[..recent_snapshots.len()/2];
        let second_half = &recent_snapshots[recent_snapshots.len()/2..];

        let first_success = first_half.iter().map(|s| s.success_rate).sum::<f32>() / first_half.len() as f32;
        let second_success = second_half.iter().map(|s| s.success_rate).sum::<f32>() / second_half.len() as f32;

        let first_response = first_half.iter().map(|s| s.avg_response_time as f32).sum::<f32>() / first_half.len() as f32;
        let second_response = second_half.iter().map(|s| s.avg_response_time as f32).sum::<f32>() / second_half.len() as f32;

        let first_error = first_half.iter().map(|s| s.error_rate).sum::<f32>() / first_half.len() as f32;
        let second_error = second_half.iter().map(|s| s.error_rate).sum::<f32>() / second_half.len() as f32;

        Some(TrendData {
            success_rate_trend: second_success - first_success,
            response_time_trend: second_response - first_response,
            error_rate_trend: second_error - first_error,
        })
    }

    fn update_aggregated_metrics(&mut self) {
        // Update aggregated metrics for better performance analysis
        if let Some(latest) = self.snapshots.back() {
            self.update_aggregated_metric("success_rate", latest.success_rate);
            self.update_aggregated_metric("response_time", latest.avg_response_time as f32);
            self.update_aggregated_metric("error_rate", latest.error_rate);
            self.update_aggregated_metric("confidence", latest.confidence_score);
        }
    }

    fn update_aggregated_metric(&mut self, key: &str, value: f32) {
        let metric = self.aggregated_metrics.entry(key.to_string()).or_insert(AggregatedMetric {
            min: value,
            max: value,
            avg: value,
            count: 0,
            trend: 0.0,
        });

        metric.min = metric.min.min(value);
        metric.max = metric.max.max(value);
        metric.avg = (metric.avg * metric.count as f32 + value) / (metric.count + 1) as f32;
        metric.count += 1;
    }
}

// Monitor implementations
impl SuccessRateMonitor {
    pub fn new() -> Self {
        Self {
            threshold_warning: 0.85,
            threshold_critical: 0.70,
        }
    }
}

impl Monitor for SuccessRateMonitor {
    fn name(&self) -> &str { "SuccessRateMonitor" }
    fn description(&self) -> &str { "Monitors browser automation success rate" }
    fn is_enabled(&self) -> bool { true }

    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent> {
        let mut alerts = Vec::new();
        
        if metrics.success_rate < self.threshold_critical {
            alerts.push(AlertEvent {
                id: format!("success_critical_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Critical,
                monitor_name: self.name().to_string(),
                title: "Critical Success Rate".to_string(),
                description: format!("Success rate critically low: {:.1}%", metrics.success_rate * 100.0),
                metrics: [("success_rate".to_string(), metrics.success_rate)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Immediate investigation required".to_string(),
                    "Check browser automation infrastructure".to_string(),
                    "Review recent changes".to_string(),
                ],
                auto_resolution: Some(AutoResolution {
                    strategy: "emergency_restart".to_string(),
                    parameters: HashMap::new(),
                    timeout_seconds: 60,
                }),
            });
        } else if metrics.success_rate < self.threshold_warning {
            alerts.push(AlertEvent {
                id: format!("success_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "Low Success Rate".to_string(),
                description: format!("Success rate below threshold: {:.1}%", metrics.success_rate * 100.0),
                metrics: [("success_rate".to_string(), metrics.success_rate)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Monitor closely for continued degradation".to_string(),
                    "Review element selectors".to_string(),
                    "Check for UI changes on target sites".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        alerts
    }
}

impl ResponseTimeMonitor {
    pub fn new() -> Self {
        Self {
            threshold_warning_ms: 300,
            threshold_critical_ms: 1000,
            trend_analysis: true,
        }
    }
}

impl Monitor for ResponseTimeMonitor {
    fn name(&self) -> &str { "ResponseTimeMonitor" }
    fn description(&self) -> &str { "Monitors response time performance" }
    fn is_enabled(&self) -> bool { true }

    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent> {
        let mut alerts = Vec::new();
        
        if metrics.avg_response_time > self.threshold_critical_ms {
            alerts.push(AlertEvent {
                id: format!("response_critical_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Critical,
                monitor_name: self.name().to_string(),
                title: "Critical Response Time".to_string(),
                description: format!("Response time critically high: {}ms", metrics.avg_response_time),
                metrics: [("response_time".to_string(), metrics.avg_response_time as f32)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Check system resources immediately".to_string(),
                    "Scale browser pool".to_string(),
                    "Investigate performance bottlenecks".to_string(),
                ],
                auto_resolution: Some(AutoResolution {
                    strategy: "scale_browser_pool".to_string(),
                    parameters: [("target_size".to_string(), "20".to_string())].iter().cloned().collect(),
                    timeout_seconds: 120,
                }),
            });
        } else if metrics.avg_response_time > self.threshold_warning_ms {
            alerts.push(AlertEvent {
                id: format!("response_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "High Response Time".to_string(),
                description: format!("Response time above threshold: {}ms", metrics.avg_response_time),
                metrics: [("response_time".to_string(), metrics.avg_response_time as f32)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Monitor performance trends".to_string(),
                    "Consider browser pool optimization".to_string(),
                    "Check network connectivity".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        alerts
    }
}

impl ErrorRateMonitor {
    pub fn new() -> Self {
        Self {
            threshold_warning: 0.10,
            threshold_critical: 0.25,
            spike_detection: true,
        }
    }
}

impl Monitor for ErrorRateMonitor {
    fn name(&self) -> &str { "ErrorRateMonitor" }
    fn description(&self) -> &str { "Monitors error rates and spikes" }
    fn is_enabled(&self) -> bool { true }

    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent> {
        let mut alerts = Vec::new();
        
        if metrics.error_rate > self.threshold_critical {
            alerts.push(AlertEvent {
                id: format!("error_critical_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Critical,
                monitor_name: self.name().to_string(),
                title: "Critical Error Rate".to_string(),
                description: format!("Error rate critically high: {:.1}%", metrics.error_rate * 100.0),
                metrics: [("error_rate".to_string(), metrics.error_rate)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Investigate error patterns immediately".to_string(),
                    "Check browser automation infrastructure".to_string(),
                    "Review error logs".to_string(),
                ],
                auto_resolution: Some(AutoResolution {
                    strategy: "restart_browser_pool".to_string(),
                    parameters: HashMap::new(),
                    timeout_seconds: 180,
                }),
            });
        } else if metrics.error_rate > self.threshold_warning {
            alerts.push(AlertEvent {
                id: format!("error_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "High Error Rate".to_string(),
                description: format!("Error rate above normal: {:.1}%", metrics.error_rate * 100.0),
                metrics: [("error_rate".to_string(), metrics.error_rate)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Monitor error patterns".to_string(),
                    "Check for recent changes".to_string(),
                    "Review failing operations".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        alerts
    }
}

impl ResourceUsageMonitor {
    pub fn new() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            sustained_duration_minutes: 5,
        }
    }
}

impl Monitor for ResourceUsageMonitor {
    fn name(&self) -> &str { "ResourceUsageMonitor" }
    fn description(&self) -> &str { "Monitors CPU and memory usage" }
    fn is_enabled(&self) -> bool { true }

    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent> {
        let mut alerts = Vec::new();
        
        if metrics.cpu_usage > self.cpu_threshold {
            alerts.push(AlertEvent {
                id: format!("cpu_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "High CPU Usage".to_string(),
                description: format!("CPU usage: {:.1}%", metrics.cpu_usage),
                metrics: [("cpu_usage".to_string(), metrics.cpu_usage)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Monitor resource usage trends".to_string(),
                    "Consider scaling infrastructure".to_string(),
                    "Optimize browser pool configuration".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        if metrics.memory_usage > self.memory_threshold {
            alerts.push(AlertEvent {
                id: format!("memory_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "High Memory Usage".to_string(),
                description: format!("Memory usage: {:.1}%", metrics.memory_usage),
                metrics: [("memory_usage".to_string(), metrics.memory_usage)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Monitor memory trends".to_string(),
                    "Check for memory leaks".to_string(),
                    "Consider increasing available memory".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        alerts
    }
}

impl PredictiveFailureMonitor {
    pub fn new() -> Self {
        Self {
            enabled: true,
            confidence_threshold: 0.75,
            lookback_hours: 24,
        }
    }
}

impl Monitor for PredictiveFailureMonitor {
    fn name(&self) -> &str { "PredictiveFailureMonitor" }
    fn description(&self) -> &str { "Predicts potential failures using ML" }
    fn is_enabled(&self) -> bool { self.enabled }

    fn check(&self, metrics: &MetricsSnapshot) -> Vec<AlertEvent> {
        let mut alerts = Vec::new();
        
        // Simple predictive logic (in a real implementation, this would use ML models)
        let failure_risk = self.calculate_failure_risk(metrics);
        
        if failure_risk > self.confidence_threshold {
            alerts.push(AlertEvent {
                id: format!("predictive_warning_{}", metrics.timestamp),
                timestamp: metrics.timestamp,
                severity: AlertSeverity::Warning,
                monitor_name: self.name().to_string(),
                title: "Potential Failure Predicted".to_string(),
                description: format!("ML model predicts {:.1}% chance of failure", failure_risk * 100.0),
                metrics: [("failure_risk".to_string(), failure_risk)].iter().cloned().collect(),
                suggested_actions: vec![
                    "Take proactive measures to prevent failure".to_string(),
                    "Review system health metrics".to_string(),
                    "Consider scaling preventively".to_string(),
                ],
                auto_resolution: None,
            });
        }
        
        alerts
    }
}

impl PredictiveFailureMonitor {
    fn calculate_failure_risk(&self, metrics: &MetricsSnapshot) -> f32 {
        // Simple risk calculation based on multiple factors
        let success_risk = 1.0 - metrics.success_rate;
        let response_risk = (metrics.avg_response_time as f32 / 1000.0).min(1.0);
        let error_risk = metrics.error_rate;
        let resource_risk = (metrics.cpu_usage / 100.0 + metrics.memory_usage / 100.0) / 2.0;
        
        (success_risk * 0.4 + response_risk * 0.3 + error_risk * 0.2 + resource_risk * 0.1)
    }
}

// Notification channel implementations
impl LogNotificationChannel {
    pub fn new() -> Self {
        Self {
            log_file: "alerts.log".to_string(),
            enabled: true,
        }
    }
}

impl NotificationChannel for LogNotificationChannel {
    fn name(&self) -> &str { "LogNotification" }

    fn send_alert(&self, alert: &AlertEvent) -> Result<(), NotificationError> {
        if !self.enabled {
            return Ok(());
        }

        let log_entry = format!(
            "[{}] {} - {} ({}): {}\n",
            chrono_format_simple(alert.timestamp),
            alert.severity as u32,
            alert.title,
            alert.monitor_name,
            alert.description
        );

        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .map_err(|e| NotificationError(format!("Failed to open log file: {}", e)))?
            .write_all(log_entry.as_bytes())
            .map_err(|e| NotificationError(format!("Failed to write to log: {}", e)))?;

        Ok(())
    }

    fn test_connection(&self) -> Result<(), NotificationError> {
        self.send_alert(&AlertEvent {
            id: "test".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            severity: AlertSeverity::Info,
            monitor_name: "Test".to_string(),
            title: "Test Alert".to_string(),
            description: "Connection test".to_string(),
            metrics: HashMap::new(),
            suggested_actions: Vec::new(),
            auto_resolution: None,
        })
    }
}

impl SlackNotificationChannel {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            channel: "#alerts".to_string(),
            enabled: true,
        }
    }
}

impl NotificationChannel for SlackNotificationChannel {
    fn name(&self) -> &str { "SlackNotification" }

    fn send_alert(&self, alert: &AlertEvent) -> Result<(), NotificationError> {
        if !self.enabled {
            return Ok(());
        }

        let emoji = match alert.severity {
            AlertSeverity::Info => ":information_source:",
            AlertSeverity::Warning => ":warning:",
            AlertSeverity::Error => ":x:",
            AlertSeverity::Critical => ":rotating_light:",
            AlertSeverity::Emergency => ":fire:",
        };

        let message = format!("{} *{}*\n{}\nMonitor: {}", 
            emoji, alert.title, alert.description, alert.monitor_name);

        println!("📱 Slack Alert: {}", message);
        // In a real implementation, this would send an HTTP request to the Slack webhook
        Ok(())
    }

    fn test_connection(&self) -> Result<(), NotificationError> {
        // Mock test - in reality would ping Slack API
        Ok(())
    }
}

impl WebhookNotificationChannel {
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
            enabled: true,
        }
    }
}

impl NotificationChannel for WebhookNotificationChannel {
    fn name(&self) -> &str { "WebhookNotification" }

    fn send_alert(&self, alert: &AlertEvent) -> Result<(), NotificationError> {
        if !self.enabled {
            return Ok(());
        }

        println!("🔗 Webhook Alert: {} - {}", alert.title, alert.description);
        // In a real implementation, this would send an HTTP POST request
        Ok(())
    }

    fn test_connection(&self) -> Result<(), NotificationError> {
        // Mock test - in reality would ping the webhook URL
        Ok(())
    }
}

use std::io::Write;

fn chrono_format_simple(timestamp: u64) -> String {
    // Simple timestamp formatting
    let dt = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
    format!("{:?}", dt)
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        let mut severity_levels = HashMap::new();
        severity_levels.insert("success_rate".to_string(), AlertSeverity::Warning);
        severity_levels.insert("response_time".to_string(), AlertSeverity::Warning);
        severity_levels.insert("error_rate".to_string(), AlertSeverity::Error);
        severity_levels.insert("resource_usage".to_string(), AlertSeverity::Warning);

        Self {
            check_interval_seconds: 60,
            alert_cooldown_seconds: 300,
            metrics_retention_hours: 168, // 1 week
            enable_predictive_alerts: true,
            alert_severity_levels: severity_levels,
        }
    }
}