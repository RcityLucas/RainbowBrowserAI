// Performance Monitor Tool Implementation
// Week 12 - Phase 3 Advanced Automation Tools
// Comprehensive performance metrics and monitoring

use crate::tools::{Tool, ToolError};
use crate::browser::SimpleBrowser;
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{By, WebDriver, WebElement};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use async_trait::async_trait;
use tokio::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Performance monitor for comprehensive metrics collection
pub struct PerformanceMonitor {
    driver: Arc<WebDriver>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Capture navigation timing metrics
    async fn capture_navigation_timing(&self) -> Result<NavigationTiming> {
        let script = r#"
            const timing = window.performance.timing;
            const navigation = window.performance.navigation;
            return {
                navigationStart: timing.navigationStart,
                unloadEventStart: timing.unloadEventStart,
                unloadEventEnd: timing.unloadEventEnd,
                redirectStart: timing.redirectStart,
                redirectEnd: timing.redirectEnd,
                fetchStart: timing.fetchStart,
                domainLookupStart: timing.domainLookupStart,
                domainLookupEnd: timing.domainLookupEnd,
                connectStart: timing.connectStart,
                connectEnd: timing.connectEnd,
                secureConnectionStart: timing.secureConnectionStart,
                requestStart: timing.requestStart,
                responseStart: timing.responseStart,
                responseEnd: timing.responseEnd,
                domLoading: timing.domLoading,
                domInteractive: timing.domInteractive,
                domContentLoadedEventStart: timing.domContentLoadedEventStart,
                domContentLoadedEventEnd: timing.domContentLoadedEventEnd,
                domComplete: timing.domComplete,
                loadEventStart: timing.loadEventStart,
                loadEventEnd: timing.loadEventEnd,
                redirectCount: navigation.redirectCount
            };
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        let timing_data: NavigationTimingData = serde_json::from_value(result.json().clone())?;
        
        Ok(NavigationTiming::from_raw(timing_data))
    }
    
    /// Capture resource timing metrics
    async fn capture_resource_timing(&self) -> Result<Vec<ResourceTiming>> {
        let script = r#"
            return window.performance.getEntriesByType('resource').map(entry => ({
                name: entry.name,
                entryType: entry.entryType,
                startTime: entry.startTime,
                duration: entry.duration,
                initiatorType: entry.initiatorType,
                nextHopProtocol: entry.nextHopProtocol,
                workerStart: entry.workerStart,
                redirectStart: entry.redirectStart,
                redirectEnd: entry.redirectEnd,
                fetchStart: entry.fetchStart,
                domainLookupStart: entry.domainLookupStart,
                domainLookupEnd: entry.domainLookupEnd,
                connectStart: entry.connectStart,
                connectEnd: entry.connectEnd,
                secureConnectionStart: entry.secureConnectionStart,
                requestStart: entry.requestStart,
                responseStart: entry.responseStart,
                responseEnd: entry.responseEnd,
                transferSize: entry.transferSize,
                encodedBodySize: entry.encodedBodySize,
                decodedBodySize: entry.decodedBodySize,
                serverTiming: entry.serverTiming
            }));
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        let resources: Vec<ResourceTimingData> = serde_json::from_value(result.json().clone())?;
        
        Ok(resources.into_iter().map(ResourceTiming::from_raw).collect())
    }
    
    /// Capture paint timing metrics
    async fn capture_paint_timing(&self) -> Result<PaintTiming> {
        let script = r#"
            const paintEntries = window.performance.getEntriesByType('paint');
            const result = {};
            paintEntries.forEach(entry => {
                result[entry.name] = entry.startTime;
            });
            return result;
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        let paint_data: HashMap<String, f64> = serde_json::from_value(result.json().clone())?;
        
        Ok(PaintTiming {
            first_paint: paint_data.get("first-paint").copied(),
            first_contentful_paint: paint_data.get("first-contentful-paint").copied(),
        })
    }
    
    /// Capture layout shift metrics
    async fn capture_layout_shift(&self) -> Result<LayoutShiftMetrics> {
        let script = r#"
            const observer = new PerformanceObserver((list) => {});
            const entries = window.performance.getEntriesByType('layout-shift');
            let totalShift = 0;
            let maxShift = 0;
            entries.forEach(entry => {
                if (!entry.hadRecentInput) {
                    totalShift += entry.value;
                    maxShift = Math.max(maxShift, entry.value);
                }
            });
            return {
                cumulativeLayoutShift: totalShift,
                maxLayoutShift: maxShift,
                shiftCount: entries.length
            };
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        let shift_data: LayoutShiftData = serde_json::from_value(result.json().clone())?;
        
        Ok(LayoutShiftMetrics {
            cumulative_layout_shift: shift_data.cumulative_layout_shift,
            max_layout_shift: shift_data.max_layout_shift,
            shift_count: shift_data.shift_count,
        })
    }
    
    /// Capture largest contentful paint
    async fn capture_lcp(&self) -> Result<f64> {
        let script = r#"
            return new Promise((resolve) => {
                const observer = new PerformanceObserver((list) => {
                    const entries = list.getEntries();
                    const lastEntry = entries[entries.length - 1];
                    resolve(lastEntry.renderTime || lastEntry.loadTime);
                });
                observer.observe({ entryTypes: ['largest-contentful-paint'] });
                
                // Fallback for immediate measurement
                setTimeout(() => {
                    const entries = window.performance.getEntriesByType('largest-contentful-paint');
                    if (entries.length > 0) {
                        const lastEntry = entries[entries.length - 1];
                        resolve(lastEntry.renderTime || lastEntry.loadTime);
                    } else {
                        resolve(0);
                    }
                }, 100);
            });
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        Ok(result.convert::<f64>().unwrap_or(0.0))
    }
    
    /// Capture first input delay
    async fn capture_fid(&self) -> Result<Option<f64>> {
        let script = r#"
            return new Promise((resolve) => {
                const observer = new PerformanceObserver((list) => {
                    const entries = list.getEntries();
                    if (entries.length > 0) {
                        resolve(entries[0].processingStart - entries[0].startTime);
                    }
                });
                observer.observe({ entryTypes: ['first-input'] });
                
                // Fallback timeout
                setTimeout(() => resolve(null), 5000);
            });
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        Ok(result.convert::<f64>().ok())
    }
    
    /// Capture memory usage metrics
    async fn capture_memory_usage(&self) -> Result<MemoryMetrics> {
        let script = r#"
            if (window.performance.memory) {
                return {
                    usedJSHeapSize: window.performance.memory.usedJSHeapSize,
                    totalJSHeapSize: window.performance.memory.totalJSHeapSize,
                    jsHeapSizeLimit: window.performance.memory.jsHeapSizeLimit
                };
            }
            return null;
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        
        if result.json().is_null() {
            return Ok(MemoryMetrics::default());
        }
        
        let memory_data: MemoryData = serde_json::from_value(result.json().clone())?;
        
        Ok(MemoryMetrics {
            used_js_heap_size: memory_data.used_js_heap_size,
            total_js_heap_size: memory_data.total_js_heap_size,
            js_heap_size_limit: memory_data.js_heap_size_limit,
        })
    }
    
    /// Capture network metrics
    async fn capture_network_metrics(&self) -> Result<NetworkMetrics> {
        let script = r#"
            const connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection;
            if (connection) {
                return {
                    effectiveType: connection.effectiveType,
                    downlink: connection.downlink,
                    rtt: connection.rtt,
                    saveData: connection.saveData
                };
            }
            return null;
        "#;
        
        let result = self.driver.execute(script, vec![]).await?;
        
        if result.json().is_null() {
            return Ok(NetworkMetrics::default());
        }
        
        let network_data: NetworkData = serde_json::from_value(result.json().clone())?;
        
        Ok(NetworkMetrics {
            effective_type: network_data.effective_type,
            downlink: network_data.downlink,
            rtt: network_data.rtt,
            save_data: network_data.save_data,
        })
    }
    
    /// Analyze Core Web Vitals
    fn analyze_core_web_vitals(&self, lcp: f64, fid: Option<f64>, cls: f64) -> CoreWebVitals {
        CoreWebVitals {
            largest_contentful_paint: lcp,
            first_input_delay: fid,
            cumulative_layout_shift: cls,
            lcp_rating: Self::rate_lcp(lcp),
            fid_rating: fid.map(Self::rate_fid).unwrap_or(PerformanceRating::NotAvailable),
            cls_rating: Self::rate_cls(cls),
        }
    }
    
    /// Rate LCP performance
    fn rate_lcp(lcp: f64) -> PerformanceRating {
        if lcp <= 2500.0 {
            PerformanceRating::Good
        } else if lcp <= 4000.0 {
            PerformanceRating::NeedsImprovement
        } else {
            PerformanceRating::Poor
        }
    }
    
    /// Rate FID performance
    fn rate_fid(fid: f64) -> PerformanceRating {
        if fid <= 100.0 {
            PerformanceRating::Good
        } else if fid <= 300.0 {
            PerformanceRating::NeedsImprovement
        } else {
            PerformanceRating::Poor
        }
    }
    
    /// Rate CLS performance
    fn rate_cls(cls: f64) -> PerformanceRating {
        if cls <= 0.1 {
            PerformanceRating::Good
        } else if cls <= 0.25 {
            PerformanceRating::NeedsImprovement
        } else {
            PerformanceRating::Poor
        }
    }
    
    /// Generate performance recommendations
    fn generate_recommendations(&self, metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // LCP recommendations
        if metrics.core_web_vitals.lcp_rating != PerformanceRating::Good {
            recommendations.push("Optimize Largest Contentful Paint: Consider lazy loading, optimizing images, and reducing server response time".to_string());
        }
        
        // FID recommendations
        if metrics.core_web_vitals.fid_rating != PerformanceRating::Good && 
           metrics.core_web_vitals.fid_rating != PerformanceRating::NotAvailable {
            recommendations.push("Improve First Input Delay: Reduce JavaScript execution time and break up long tasks".to_string());
        }
        
        // CLS recommendations
        if metrics.core_web_vitals.cls_rating != PerformanceRating::Good {
            recommendations.push("Reduce Cumulative Layout Shift: Add size attributes to images and avoid inserting content above existing content".to_string());
        }
        
        // Resource optimization
        if metrics.resource_metrics.total_resources > 100 {
            recommendations.push("High resource count detected: Consider bundling resources and implementing lazy loading".to_string());
        }
        
        // Memory recommendations
        if metrics.memory_metrics.used_js_heap_size > 50_000_000 {
            recommendations.push("High memory usage detected: Review memory leaks and optimize JavaScript memory consumption".to_string());
        }
        
        recommendations
    }
}

#[async_trait]
impl Tool for PerformanceMonitor {
    type Input = PerformanceMonitorInput;
    type Output = PerformanceMonitorOutput;
    
    fn name(&self) -> &str {
        "performance_monitor"
    }
    
    fn description(&self) -> &str {
        "Monitor and analyze comprehensive performance metrics including Core Web Vitals"
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start_time = Instant::now();
        let mut metrics_history = Vec::new();
        let mut aggregated_metrics = AggregatedMetrics::default();
        
        // Navigate to URL if provided
        if let Some(url) = &input.url {
            self.driver.goto(url).await?;
            
            // Wait for page load
            tokio::time::sleep(Duration::from_millis(input.config.wait_after_load_ms)).await;
        }
        
        // Collect metrics for specified duration or iterations
        let iterations = input.iterations.unwrap_or(1);
        
        for i in 0..iterations {
            tracing::info!("Collecting performance metrics - iteration {}/{}", i + 1, iterations);
            
            // Capture all metrics
            let navigation_timing = self.capture_navigation_timing().await?;
            let resource_timing = self.capture_resource_timing().await?;
            let paint_timing = self.capture_paint_timing().await?;
            let layout_shift = self.capture_layout_shift().await?;
            let lcp = self.capture_lcp().await?;
            let fid = self.capture_fid().await?;
            let memory = self.capture_memory_usage().await?;
            let network = self.capture_network_metrics().await?;
            
            // Calculate derived metrics
            let page_load_time = navigation_timing.load_event_end - navigation_timing.navigation_start;
            let dom_content_loaded = navigation_timing.dom_content_loaded_event_end - navigation_timing.navigation_start;
            let time_to_first_byte = navigation_timing.response_start - navigation_timing.request_start;
            
            // Analyze Core Web Vitals
            let core_web_vitals = self.analyze_core_web_vitals(lcp, fid, layout_shift.cumulative_layout_shift);
            
            // Resource analysis
            let total_resources = resource_timing.len();
            let total_transfer_size: u64 = resource_timing.iter().map(|r| r.transfer_size).sum();
            let slowest_resource = resource_timing.iter()
                .max_by(|a, b| a.duration.partial_cmp(&b.duration).unwrap())
                .map(|r| r.name.clone());
            
            let resource_metrics = ResourceMetrics {
                total_resources,
                total_transfer_size,
                average_duration: if !resource_timing.is_empty() {
                    resource_timing.iter().map(|r| r.duration).sum::<f64>() / total_resources as f64
                } else {
                    0.0
                },
                slowest_resource,
                resource_breakdown: Self::analyze_resource_breakdown(&resource_timing),
            };
            
            // Create performance snapshot
            let current_url = self.driver.current_url().await.ok().map(|u| u.to_string()).unwrap_or_else(|| "unknown".to_string());
            let metrics = PerformanceMetrics {
                timestamp: Utc::now(),
                url: input.url.clone().unwrap_or(current_url),
                page_load_time,
                dom_content_loaded,
                time_to_first_byte,
                first_paint: paint_timing.first_paint,
                first_contentful_paint: paint_timing.first_contentful_paint,
                core_web_vitals: core_web_vitals.clone(),
                navigation_metrics: navigation_timing.clone(),
                resource_metrics: resource_metrics.clone(),
                memory_metrics: memory.clone(),
                network_metrics: network.clone(),
                custom_metrics: HashMap::new(),
            };
            
            // Update aggregated metrics
            aggregated_metrics.update(&metrics);
            
            metrics_history.push(metrics);
            
            // Wait between iterations if specified
            if i < iterations - 1 && input.config.interval_ms > 0 {
                tokio::time::sleep(Duration::from_millis(input.config.interval_ms)).await;
            }
        }
        
        // Generate performance score
        let performance_score = self.calculate_performance_score(&metrics_history.last().unwrap());
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&metrics_history.last().unwrap());
        
        // Generate bottlenecks analysis
        let bottlenecks = self.analyze_bottlenecks(&metrics_history);
        
        // Create detailed report if requested
        let report = if input.config.generate_report {
            Some(self.generate_detailed_report(&metrics_history, &aggregated_metrics))
        } else {
            None
        };
        
        Ok(PerformanceMonitorOutput {
            metrics_history,
            aggregated_metrics,
            performance_score,
            bottlenecks,
            recommendations,
            report,
            monitoring_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

impl PerformanceMonitor {
    /// Analyze resource breakdown by type
    fn analyze_resource_breakdown(resources: &[ResourceTiming]) -> HashMap<String, ResourceTypeMetrics> {
        let mut breakdown = HashMap::new();
        
        for resource in resources {
            let entry = breakdown.entry(resource.initiator_type.clone()).or_insert(ResourceTypeMetrics {
                count: 0,
                total_size: 0,
                total_duration: 0.0,
            });
            
            entry.count += 1;
            entry.total_size += resource.transfer_size;
            entry.total_duration += resource.duration;
        }
        
        breakdown
    }
    
    /// Calculate overall performance score
    fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> PerformanceScore {
        let mut score = 100.0;
        let mut breakdown = HashMap::new();
        
        // Core Web Vitals impact (40% of score)
        let cwv_score = match metrics.core_web_vitals.lcp_rating {
            PerformanceRating::Good => 100.0,
            PerformanceRating::NeedsImprovement => 70.0,
            PerformanceRating::Poor => 40.0,
            PerformanceRating::NotAvailable => 100.0,
        } * 0.2 +
        match metrics.core_web_vitals.fid_rating {
            PerformanceRating::Good => 100.0,
            PerformanceRating::NeedsImprovement => 70.0,
            PerformanceRating::Poor => 40.0,
            PerformanceRating::NotAvailable => 100.0,
        } * 0.1 +
        match metrics.core_web_vitals.cls_rating {
            PerformanceRating::Good => 100.0,
            PerformanceRating::NeedsImprovement => 70.0,
            PerformanceRating::Poor => 40.0,
            PerformanceRating::NotAvailable => 100.0,
        } * 0.1;
        
        breakdown.insert("core_web_vitals".to_string(), cwv_score);
        
        // Page load time impact (30% of score)
        let load_score = if metrics.page_load_time < 2000.0 {
            100.0
        } else if metrics.page_load_time < 4000.0 {
            80.0
        } else if metrics.page_load_time < 6000.0 {
            60.0
        } else {
            40.0
        } * 0.3;
        
        breakdown.insert("page_load".to_string(), load_score);
        
        // Resource efficiency (20% of score)
        let resource_score = if metrics.resource_metrics.total_resources < 50 {
            100.0
        } else if metrics.resource_metrics.total_resources < 100 {
            80.0
        } else {
            60.0
        } * 0.2;
        
        breakdown.insert("resource_efficiency".to_string(), resource_score);
        
        // Memory usage (10% of score)
        let memory_score = if metrics.memory_metrics.used_js_heap_size < 20_000_000 {
            100.0
        } else if metrics.memory_metrics.used_js_heap_size < 50_000_000 {
            80.0
        } else {
            60.0
        } * 0.1;
        
        breakdown.insert("memory_usage".to_string(), memory_score);
        
        score = cwv_score + load_score + resource_score + memory_score;
        
        PerformanceScore {
            overall_score: score,
            rating: if score >= 90.0 {
                PerformanceRating::Good
            } else if score >= 70.0 {
                PerformanceRating::NeedsImprovement
            } else {
                PerformanceRating::Poor
            },
            breakdown,
        }
    }
    
    /// Analyze performance bottlenecks
    fn analyze_bottlenecks(&self, metrics_history: &[PerformanceMetrics]) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        if let Some(latest) = metrics_history.last() {
            // Slow page load
            if latest.page_load_time > 4000.0 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::SlowPageLoad,
                    severity: if latest.page_load_time > 6000.0 { Severity::High } else { Severity::Medium },
                    impact: format!("Page load time is {:.1}s", latest.page_load_time / 1000.0),
                    recommendation: "Optimize critical rendering path and reduce resource blocking".to_string(),
                });
            }
            
            // High resource count
            if latest.resource_metrics.total_resources > 100 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::ExcessiveResources,
                    severity: if latest.resource_metrics.total_resources > 150 { Severity::High } else { Severity::Medium },
                    impact: format!("{} resources loaded", latest.resource_metrics.total_resources),
                    recommendation: "Bundle resources and implement lazy loading".to_string(),
                });
            }
            
            // Large transfer size
            if latest.resource_metrics.total_transfer_size > 3_000_000 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::LargePayload,
                    severity: if latest.resource_metrics.total_transfer_size > 5_000_000 { Severity::High } else { Severity::Medium },
                    impact: format!("{:.1}MB total transfer size", latest.resource_metrics.total_transfer_size as f64 / 1_000_000.0),
                    recommendation: "Compress resources and optimize images".to_string(),
                });
            }
            
            // Memory issues
            if latest.memory_metrics.used_js_heap_size > 50_000_000 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::HighMemoryUsage,
                    severity: if latest.memory_metrics.used_js_heap_size > 100_000_000 { Severity::High } else { Severity::Medium },
                    impact: format!("{:.1}MB JavaScript heap used", latest.memory_metrics.used_js_heap_size as f64 / 1_000_000.0),
                    recommendation: "Review memory leaks and optimize JavaScript".to_string(),
                });
            }
            
            // Layout shift issues
            if latest.core_web_vitals.cumulative_layout_shift > 0.1 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::LayoutInstability,
                    severity: if latest.core_web_vitals.cumulative_layout_shift > 0.25 { Severity::High } else { Severity::Medium },
                    impact: format!("CLS score: {:.3}", latest.core_web_vitals.cumulative_layout_shift),
                    recommendation: "Add dimensions to media and avoid dynamic content injection".to_string(),
                });
            }
        }
        
        bottlenecks
    }
    
    /// Generate detailed performance report
    fn generate_detailed_report(&self, metrics_history: &[PerformanceMetrics], aggregated: &AggregatedMetrics) -> PerformanceReport {
        PerformanceReport {
            summary: ReportSummary {
                total_measurements: metrics_history.len(),
                average_page_load: aggregated.average_page_load,
                average_lcp: aggregated.average_lcp,
                average_fid: aggregated.average_fid,
                average_cls: aggregated.average_cls,
                p95_page_load: aggregated.p95_page_load,
                p95_lcp: aggregated.p95_lcp,
            },
            trend_analysis: self.analyze_trends(metrics_history),
            resource_analysis: self.analyze_resources_detailed(metrics_history),
            recommendations: self.generate_detailed_recommendations(metrics_history),
        }
    }
    
    /// Analyze performance trends
    fn analyze_trends(&self, metrics_history: &[PerformanceMetrics]) -> TrendAnalysis {
        if metrics_history.len() < 2 {
            return TrendAnalysis::default();
        }
        
        let first = metrics_history.first().unwrap();
        let last = metrics_history.last().unwrap();
        
        TrendAnalysis {
            page_load_trend: (last.page_load_time - first.page_load_time) / first.page_load_time * 100.0,
            lcp_trend: (last.core_web_vitals.largest_contentful_paint - first.core_web_vitals.largest_contentful_paint) 
                / first.core_web_vitals.largest_contentful_paint * 100.0,
            cls_trend: (last.core_web_vitals.cumulative_layout_shift - first.core_web_vitals.cumulative_layout_shift)
                / first.core_web_vitals.cumulative_layout_shift.max(0.001) * 100.0,
            memory_trend: (last.memory_metrics.used_js_heap_size as f64 - first.memory_metrics.used_js_heap_size as f64)
                / first.memory_metrics.used_js_heap_size.max(1) as f64 * 100.0,
        }
    }
    
    /// Analyze resources in detail
    fn analyze_resources_detailed(&self, metrics_history: &[PerformanceMetrics]) -> ResourceAnalysis {
        let mut all_resources = HashMap::new();
        
        for metrics in metrics_history {
            for (resource_type, type_metrics) in &metrics.resource_metrics.resource_breakdown {
                let entry = all_resources.entry(resource_type.clone()).or_insert(ResourceTypeMetrics {
                    count: 0,
                    total_size: 0,
                    total_duration: 0.0,
                });
                
                entry.count += type_metrics.count;
                entry.total_size += type_metrics.total_size;
                entry.total_duration += type_metrics.total_duration;
            }
        }
        
        ResourceAnalysis {
            by_type: all_resources,
            critical_resources: Vec::new(), // Would need more detailed analysis
            optimization_opportunities: Vec::new(),
        }
    }
    
    /// Generate detailed recommendations
    fn generate_detailed_recommendations(&self, metrics_history: &[PerformanceMetrics]) -> Vec<DetailedRecommendation> {
        let mut recommendations = Vec::new();
        
        if let Some(latest) = metrics_history.last() {
            // Add detailed recommendations based on metrics
            if latest.core_web_vitals.largest_contentful_paint > 2500.0 {
                recommendations.push(DetailedRecommendation {
                    category: "Core Web Vitals".to_string(),
                    issue: "Slow Largest Contentful Paint".to_string(),
                    impact: "High".to_string(),
                    solution: "Optimize server response times, use CDN, preload critical resources".to_string(),
                    estimated_improvement: "30-50% reduction in LCP".to_string(),
                });
            }
        }
        
        recommendations
    }
}

// Input/Output structures

/// Input for performance monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitorInput {
    /// URL to monitor (optional if already on page)
    pub url: Option<String>,
    
    /// Number of measurement iterations
    pub iterations: Option<usize>,
    
    /// Monitoring configuration
    pub config: PerformanceConfig,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Wait time after page load (ms)
    pub wait_after_load_ms: u64,
    
    /// Interval between measurements (ms)
    pub interval_ms: u64,
    
    /// Capture Core Web Vitals
    pub capture_core_web_vitals: bool,
    
    /// Capture resource timing
    pub capture_resource_timing: bool,
    
    /// Capture memory metrics
    pub capture_memory_metrics: bool,
    
    /// Capture network metrics
    pub capture_network_metrics: bool,
    
    /// Generate detailed report
    pub generate_report: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            wait_after_load_ms: 3000,
            interval_ms: 1000,
            capture_core_web_vitals: true,
            capture_resource_timing: true,
            capture_memory_metrics: true,
            capture_network_metrics: true,
            generate_report: true,
        }
    }
}

/// Output from performance monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitorOutput {
    /// History of performance metrics
    pub metrics_history: Vec<PerformanceMetrics>,
    
    /// Aggregated metrics across all measurements
    pub aggregated_metrics: AggregatedMetrics,
    
    /// Overall performance score
    pub performance_score: PerformanceScore,
    
    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
    
    /// Performance recommendations
    pub recommendations: Vec<String>,
    
    /// Detailed report (if generated)
    pub report: Option<PerformanceReport>,
    
    /// Total monitoring duration
    pub monitoring_duration_ms: u64,
}

// Performance metrics structures

/// Complete performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub page_load_time: f64,
    pub dom_content_loaded: f64,
    pub time_to_first_byte: f64,
    pub first_paint: Option<f64>,
    pub first_contentful_paint: Option<f64>,
    pub core_web_vitals: CoreWebVitals,
    pub navigation_metrics: NavigationTiming,
    pub resource_metrics: ResourceMetrics,
    pub memory_metrics: MemoryMetrics,
    pub network_metrics: NetworkMetrics,
    pub custom_metrics: HashMap<String, f64>,
}

/// Core Web Vitals metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreWebVitals {
    pub largest_contentful_paint: f64,
    pub first_input_delay: Option<f64>,
    pub cumulative_layout_shift: f64,
    pub lcp_rating: PerformanceRating,
    pub fid_rating: PerformanceRating,
    pub cls_rating: PerformanceRating,
}

/// Performance rating
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceRating {
    Good,
    NeedsImprovement,
    Poor,
    NotAvailable,
}

/// Navigation timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTiming {
    pub navigation_start: f64,
    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub connect_end: f64,
    pub secure_connection_start: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub dom_loading: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub redirect_count: u32,
}

impl NavigationTiming {
    fn from_raw(data: NavigationTimingData) -> Self {
        Self {
            navigation_start: data.navigation_start,
            unload_event_start: data.unload_event_start,
            unload_event_end: data.unload_event_end,
            redirect_start: data.redirect_start,
            redirect_end: data.redirect_end,
            fetch_start: data.fetch_start,
            domain_lookup_start: data.domain_lookup_start,
            domain_lookup_end: data.domain_lookup_end,
            connect_start: data.connect_start,
            connect_end: data.connect_end,
            secure_connection_start: data.secure_connection_start,
            request_start: data.request_start,
            response_start: data.response_start,
            response_end: data.response_end,
            dom_loading: data.dom_loading,
            dom_interactive: data.dom_interactive,
            dom_content_loaded_event_start: data.dom_content_loaded_event_start,
            dom_content_loaded_event_end: data.dom_content_loaded_event_end,
            dom_complete: data.dom_complete,
            load_event_start: data.load_event_start,
            load_event_end: data.load_event_end,
            redirect_count: data.redirect_count,
        }
    }
}

/// Resource timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    pub name: String,
    pub initiator_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
}

impl ResourceTiming {
    fn from_raw(data: ResourceTimingData) -> Self {
        Self {
            name: data.name,
            initiator_type: data.initiator_type,
            start_time: data.start_time,
            duration: data.duration,
            transfer_size: data.transfer_size.unwrap_or(0),
            encoded_body_size: data.encoded_body_size.unwrap_or(0),
            decoded_body_size: data.decoded_body_size.unwrap_or(0),
        }
    }
}

/// Paint timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintTiming {
    pub first_paint: Option<f64>,
    pub first_contentful_paint: Option<f64>,
}

/// Layout shift metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutShiftMetrics {
    pub cumulative_layout_shift: f64,
    pub max_layout_shift: f64,
    pub shift_count: u32,
}

/// Memory metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_js_heap_size: u64,
    pub total_js_heap_size: u64,
    pub js_heap_size_limit: u64,
}

/// Network metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub effective_type: Option<String>,
    pub downlink: Option<f64>,
    pub rtt: Option<f64>,
    pub save_data: Option<bool>,
}

/// Resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub total_resources: usize,
    pub total_transfer_size: u64,
    pub average_duration: f64,
    pub slowest_resource: Option<String>,
    pub resource_breakdown: HashMap<String, ResourceTypeMetrics>,
}

/// Resource type metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTypeMetrics {
    pub count: usize,
    pub total_size: u64,
    pub total_duration: f64,
}

/// Aggregated metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub average_page_load: f64,
    pub average_lcp: f64,
    pub average_fid: Option<f64>,
    pub average_cls: f64,
    pub p95_page_load: f64,
    pub p95_lcp: f64,
    pub min_page_load: f64,
    pub max_page_load: f64,
}

impl AggregatedMetrics {
    fn update(&mut self, metrics: &PerformanceMetrics) {
        // Simple averaging for now - would implement proper aggregation
        self.average_page_load = (self.average_page_load + metrics.page_load_time) / 2.0;
        self.average_lcp = (self.average_lcp + metrics.core_web_vitals.largest_contentful_paint) / 2.0;
        self.average_cls = (self.average_cls + metrics.core_web_vitals.cumulative_layout_shift) / 2.0;
        
        if let Some(fid) = metrics.core_web_vitals.first_input_delay {
            self.average_fid = Some(self.average_fid.unwrap_or(0.0) + fid / 2.0);
        }
    }
}

/// Performance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    pub overall_score: f64,
    pub rating: PerformanceRating,
    pub breakdown: HashMap<String, f64>,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub bottleneck_type: BottleneckType,
    pub severity: Severity,
    pub impact: String,
    pub recommendation: String,
}

/// Bottleneck types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BottleneckType {
    SlowPageLoad,
    HighLatency,
    ExcessiveResources,
    LargePayload,
    RenderBlocking,
    HighMemoryUsage,
    LayoutInstability,
    SlowInteractivity,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub summary: ReportSummary,
    pub trend_analysis: TrendAnalysis,
    pub resource_analysis: ResourceAnalysis,
    pub recommendations: Vec<DetailedRecommendation>,
}

/// Report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_measurements: usize,
    pub average_page_load: f64,
    pub average_lcp: f64,
    pub average_fid: Option<f64>,
    pub average_cls: f64,
    pub p95_page_load: f64,
    pub p95_lcp: f64,
}

/// Trend analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub page_load_trend: f64,
    pub lcp_trend: f64,
    pub cls_trend: f64,
    pub memory_trend: f64,
}

/// Resource analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    pub by_type: HashMap<String, ResourceTypeMetrics>,
    pub critical_resources: Vec<String>,
    pub optimization_opportunities: Vec<String>,
}

/// Detailed recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedRecommendation {
    pub category: String,
    pub issue: String,
    pub impact: String,
    pub solution: String,
    pub estimated_improvement: String,
}

// Raw data structures for JavaScript interop

#[derive(Debug, Deserialize)]
struct NavigationTimingData {
    #[serde(rename = "navigationStart")]
    navigation_start: f64,
    #[serde(rename = "unloadEventStart")]
    unload_event_start: f64,
    #[serde(rename = "unloadEventEnd")]
    unload_event_end: f64,
    #[serde(rename = "redirectStart")]
    redirect_start: f64,
    #[serde(rename = "redirectEnd")]
    redirect_end: f64,
    #[serde(rename = "fetchStart")]
    fetch_start: f64,
    #[serde(rename = "domainLookupStart")]
    domain_lookup_start: f64,
    #[serde(rename = "domainLookupEnd")]
    domain_lookup_end: f64,
    #[serde(rename = "connectStart")]
    connect_start: f64,
    #[serde(rename = "connectEnd")]
    connect_end: f64,
    #[serde(rename = "secureConnectionStart")]
    secure_connection_start: f64,
    #[serde(rename = "requestStart")]
    request_start: f64,
    #[serde(rename = "responseStart")]
    response_start: f64,
    #[serde(rename = "responseEnd")]
    response_end: f64,
    #[serde(rename = "domLoading")]
    dom_loading: f64,
    #[serde(rename = "domInteractive")]
    dom_interactive: f64,
    #[serde(rename = "domContentLoadedEventStart")]
    dom_content_loaded_event_start: f64,
    #[serde(rename = "domContentLoadedEventEnd")]
    dom_content_loaded_event_end: f64,
    #[serde(rename = "domComplete")]
    dom_complete: f64,
    #[serde(rename = "loadEventStart")]
    load_event_start: f64,
    #[serde(rename = "loadEventEnd")]
    load_event_end: f64,
    #[serde(rename = "redirectCount")]
    redirect_count: u32,
}

#[derive(Debug, Deserialize)]
struct ResourceTimingData {
    name: String,
    #[serde(rename = "initiatorType")]
    initiator_type: String,
    #[serde(rename = "startTime")]
    start_time: f64,
    duration: f64,
    #[serde(rename = "transferSize")]
    transfer_size: Option<u64>,
    #[serde(rename = "encodedBodySize")]
    encoded_body_size: Option<u64>,
    #[serde(rename = "decodedBodySize")]
    decoded_body_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct LayoutShiftData {
    #[serde(rename = "cumulativeLayoutShift")]
    cumulative_layout_shift: f64,
    #[serde(rename = "maxLayoutShift")]
    max_layout_shift: f64,
    #[serde(rename = "shiftCount")]
    shift_count: u32,
}

#[derive(Debug, Deserialize)]
struct MemoryData {
    #[serde(rename = "usedJSHeapSize")]
    used_js_heap_size: u64,
    #[serde(rename = "totalJSHeapSize")]
    total_js_heap_size: u64,
    #[serde(rename = "jsHeapSizeLimit")]
    js_heap_size_limit: u64,
}

#[derive(Debug, Deserialize)]
struct NetworkData {
    #[serde(rename = "effectiveType")]
    effective_type: Option<String>,
    downlink: Option<f64>,
    rtt: Option<f64>,
    #[serde(rename = "saveData")]
    save_data: Option<bool>,
}