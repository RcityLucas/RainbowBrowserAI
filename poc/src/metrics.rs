use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

/// Performance metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<Metrics>>,
    start_time: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    // Counters
    pub operations_total: u64,
    pub operations_success: u64,
    pub operations_failed: u64,
    pub errors_total: u64,
    
    // Gauges
    pub active_browsers: usize,
    pub memory_usage_mb: f64,
    pub cache_size: usize,
    
    // Histograms (storing samples for percentile calculation)
    pub operation_durations_ms: Vec<f64>,
    pub llm_response_times_ms: Vec<f64>,
    pub browser_startup_times_ms: Vec<f64>,
    
    // Cost metrics
    pub total_cost: f64,
    pub llm_cost: f64,
    pub browser_cost: f64,
    
    // Workflow metrics
    pub workflows_executed: u64,
    pub workflow_steps_total: u64,
    pub workflow_steps_failed: u64,
    
    // Resource metrics
    pub screenshots_taken: u64,
    pub data_extracted_bytes: u64,
    
    // Timing
    pub uptime_seconds: u64,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            operations_total: 0,
            operations_success: 0,
            operations_failed: 0,
            errors_total: 0,
            active_browsers: 0,
            memory_usage_mb: 0.0,
            cache_size: 0,
            operation_durations_ms: Vec::new(),
            llm_response_times_ms: Vec::new(),
            browser_startup_times_ms: Vec::new(),
            total_cost: 0.0,
            llm_cost: 0.0,
            browser_cost: 0.0,
            workflows_executed: 0,
            workflow_steps_total: 0,
            workflow_steps_failed: 0,
            screenshots_taken: 0,
            data_extracted_bytes: 0,
            uptime_seconds: 0,
        }
    }
}

impl Metrics {
    /// Calculate percentile from a sorted vector
    fn percentile(sorted_values: &[f64], p: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }
        
        let index = ((p / 100.0) * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index.min(sorted_values.len() - 1)]
    }

    /// Get operation duration percentiles
    pub fn operation_duration_percentiles(&self) -> PercentileStats {
        let mut durations = self.operation_durations_ms.clone();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        PercentileStats {
            p50: Self::percentile(&durations, 50.0),
            p75: Self::percentile(&durations, 75.0),
            p90: Self::percentile(&durations, 90.0),
            p95: Self::percentile(&durations, 95.0),
            p99: Self::percentile(&durations, 99.0),
            mean: if durations.is_empty() { 
                0.0 
            } else { 
                durations.iter().sum::<f64>() / durations.len() as f64 
            },
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.operations_total == 0 {
            return 100.0;
        }
        (self.operations_success as f64 / self.operations_total as f64) * 100.0
    }

    /// Get average cost per operation
    pub fn avg_cost_per_operation(&self) -> f64 {
        if self.operations_total == 0 {
            return 0.0;
        }
        self.total_cost / self.operations_total as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileStats {
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub mean: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        info!("Initializing metrics collector");
        Self {
            metrics: Arc::new(RwLock::new(Metrics::default())),
            start_time: Instant::now(),
        }
    }

    /// Record an operation
    pub async fn record_operation(&self, duration: Duration, success: bool, cost: f64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.operations_total += 1;
        if success {
            metrics.operations_success += 1;
        } else {
            metrics.operations_failed += 1;
        }
        
        metrics.operation_durations_ms.push(duration.as_millis() as f64);
        metrics.total_cost += cost;
        
        // Keep only last 1000 samples for memory efficiency
        if metrics.operation_durations_ms.len() > 1000 {
            metrics.operation_durations_ms.remove(0);
        }
        
        debug!("Operation recorded: duration={:?}, success={}, cost={}", duration, success, cost);
    }

    /// Record an LLM operation
    pub async fn record_llm_operation(&self, duration: Duration, cost: f64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.llm_response_times_ms.push(duration.as_millis() as f64);
        metrics.llm_cost += cost;
        
        // Keep only last 1000 samples
        if metrics.llm_response_times_ms.len() > 1000 {
            metrics.llm_response_times_ms.remove(0);
        }
        
        debug!("LLM operation recorded: duration={:?}, cost={}", duration, cost);
    }

    /// Record browser startup time
    pub async fn record_browser_startup(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        
        metrics.browser_startup_times_ms.push(duration.as_millis() as f64);
        
        // Keep only last 100 samples
        if metrics.browser_startup_times_ms.len() > 100 {
            metrics.browser_startup_times_ms.remove(0);
        }
        
        debug!("Browser startup recorded: duration={:?}", duration);
    }

    /// Record a workflow execution
    pub async fn record_workflow(&self, steps_total: u64, steps_failed: u64, cost: f64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.workflows_executed += 1;
        metrics.workflow_steps_total += steps_total;
        metrics.workflow_steps_failed += steps_failed;
        metrics.browser_cost += cost;
        
        debug!("Workflow recorded: steps={}, failed={}, cost={}", steps_total, steps_failed, cost);
    }

    /// Update active browser count
    pub async fn set_active_browsers(&self, count: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.active_browsers = count;
    }

    /// Update cache size
    pub async fn set_cache_size(&self, size: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_size = size;
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self) {
        let mut metrics = self.metrics.write().await;
        
        // Simplified memory calculation - in production, use proper system metrics
        if let Ok(mem_info) = sys_info::mem_info() {
            let used_mb = (mem_info.total - mem_info.free) as f64 / 1024.0;
            metrics.memory_usage_mb = used_mb;
        }
    }

    /// Increment error counter
    pub async fn increment_errors(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.errors_total += 1;
    }

    /// Record a screenshot
    pub async fn record_screenshot(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.screenshots_taken += 1;
    }

    /// Record data extraction
    pub async fn record_data_extraction(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.data_extracted_bytes += bytes;
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> Metrics {
        let mut metrics = self.metrics.write().await;
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        metrics.clone()
    }

    /// Get metrics summary
    pub async fn get_summary(&self) -> MetricsSummary {
        let metrics = self.get_metrics().await;
        
        MetricsSummary {
            uptime_seconds: metrics.uptime_seconds,
            operations_total: metrics.operations_total,
            success_rate: metrics.success_rate(),
            avg_response_time_ms: metrics.operation_duration_percentiles().mean,
            p95_response_time_ms: metrics.operation_duration_percentiles().p95,
            total_cost: metrics.total_cost,
            avg_cost_per_operation: metrics.avg_cost_per_operation(),
            active_browsers: metrics.active_browsers,
            memory_usage_mb: metrics.memory_usage_mb,
            cache_size: metrics.cache_size,
            workflows_executed: metrics.workflows_executed,
            errors_total: metrics.errors_total,
        }
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.get_metrics().await;
        let mut output = String::new();

        // Counters
        output.push_str(&format!("# HELP rainbow_operations_total Total number of operations\n"));
        output.push_str(&format!("# TYPE rainbow_operations_total counter\n"));
        output.push_str(&format!("rainbow_operations_total {}\n", metrics.operations_total));

        output.push_str(&format!("# HELP rainbow_operations_success Successful operations\n"));
        output.push_str(&format!("# TYPE rainbow_operations_success counter\n"));
        output.push_str(&format!("rainbow_operations_success {}\n", metrics.operations_success));

        output.push_str(&format!("# HELP rainbow_errors_total Total errors\n"));
        output.push_str(&format!("# TYPE rainbow_errors_total counter\n"));
        output.push_str(&format!("rainbow_errors_total {}\n", metrics.errors_total));

        // Gauges
        output.push_str(&format!("# HELP rainbow_active_browsers Number of active browsers\n"));
        output.push_str(&format!("# TYPE rainbow_active_browsers gauge\n"));
        output.push_str(&format!("rainbow_active_browsers {}\n", metrics.active_browsers));

        output.push_str(&format!("# HELP rainbow_memory_usage_mb Memory usage in MB\n"));
        output.push_str(&format!("# TYPE rainbow_memory_usage_mb gauge\n"));
        output.push_str(&format!("rainbow_memory_usage_mb {}\n", metrics.memory_usage_mb));

        // Histograms (simplified - showing percentiles)
        let percentiles = metrics.operation_duration_percentiles();
        output.push_str(&format!("# HELP rainbow_operation_duration_ms Operation duration in milliseconds\n"));
        output.push_str(&format!("# TYPE rainbow_operation_duration_ms summary\n"));
        output.push_str(&format!("rainbow_operation_duration_ms{{quantile=\"0.5\"}} {}\n", percentiles.p50));
        output.push_str(&format!("rainbow_operation_duration_ms{{quantile=\"0.9\"}} {}\n", percentiles.p90));
        output.push_str(&format!("rainbow_operation_duration_ms{{quantile=\"0.95\"}} {}\n", percentiles.p95));
        output.push_str(&format!("rainbow_operation_duration_ms{{quantile=\"0.99\"}} {}\n", percentiles.p99));

        // Cost metrics
        output.push_str(&format!("# HELP rainbow_total_cost_dollars Total cost in dollars\n"));
        output.push_str(&format!("# TYPE rainbow_total_cost_dollars counter\n"));
        output.push_str(&format!("rainbow_total_cost_dollars {}\n", metrics.total_cost));

        output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: u64,
    pub operations_total: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub total_cost: f64,
    pub avg_cost_per_operation: f64,
    pub active_browsers: usize,
    pub memory_usage_mb: f64,
    pub cache_size: usize,
    pub workflows_executed: u64,
    pub errors_total: u64,
}

impl MetricsSummary {
    /// Format as a human-readable report
    pub fn format_report(&self) -> String {
        format!(
            r#"
📊 Performance Metrics Report
============================
⏱️  Uptime: {} hours {} minutes
📈 Operations: {} total ({:.1}% success rate)
⚡ Performance: {:.1}ms avg, {:.1}ms p95
💰 Cost: ${:.4} total (${:.6} per operation)
🌐 Browsers: {} active
💾 Memory: {:.1} MB
🗄️  Cache: {} entries
🎭 Workflows: {} executed
❌ Errors: {} total
"#,
            self.uptime_seconds / 3600,
            (self.uptime_seconds % 3600) / 60,
            self.operations_total,
            self.success_rate,
            self.avg_response_time_ms,
            self.p95_response_time_ms,
            self.total_cost,
            self.avg_cost_per_operation,
            self.active_browsers,
            self.memory_usage_mb,
            self.cache_size,
            self.workflows_executed,
            self.errors_total
        )
    }
}

// Global metrics instance
lazy_static::lazy_static! {
    pub static ref METRICS: MetricsCollector = MetricsCollector::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        let collector = MetricsCollector::new();
        
        // Record some operations
        collector.record_operation(Duration::from_millis(100), true, 0.01).await;
        collector.record_operation(Duration::from_millis(200), false, 0.02).await;
        collector.record_operation(Duration::from_millis(150), true, 0.015).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.operations_total, 3);
        assert_eq!(metrics.operations_success, 2);
        assert_eq!(metrics.operations_failed, 1);
        assert_eq!(metrics.total_cost, 0.045);
    }

    #[tokio::test]
    async fn test_percentile_calculation() {
        let metrics = Metrics {
            operation_durations_ms: vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0],
            ..Default::default()
        };
        
        let percentiles = metrics.operation_duration_percentiles();
        assert_eq!(percentiles.p50, 50.0);
        assert_eq!(percentiles.p90, 90.0);
        assert_eq!(percentiles.mean, 55.0);
    }
}