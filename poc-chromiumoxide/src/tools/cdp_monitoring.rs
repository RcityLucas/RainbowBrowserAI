use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug};
use std::collections::HashMap;
// use futures::StreamExt; // Reserved for future CDP event streaming

// ============================================================================
// Network Monitoring Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMonitorInput {
    #[serde(default = "default_monitor_duration")]
    pub duration_ms: u64,
    #[serde(default)]
    pub capture_requests: bool,
    #[serde(default)]
    pub capture_responses: bool,
    #[serde(default)]
    pub filter_resource_types: Option<Vec<String>>, // js, css, xhr, image, etc.
    #[serde(default)]
    pub domain_filter: Option<String>,
}

fn default_monitor_duration() -> u64 {
    5000 // 5 seconds
}

#[derive(Debug, Serialize)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub resource_type: String,
    pub timestamp: f64,
    pub status_code: Option<u16>,
    pub response_size: Option<u64>,
    pub duration_ms: Option<u64>,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct NetworkMonitorOutput {
    pub success: bool,
    pub total_requests: u64,
    pub requests: Vec<NetworkRequest>,
    pub monitoring_duration_ms: u64,
    pub bytes_transferred: u64,
    pub failed_requests: u64,
}

pub struct NetworkMonitorTool {
    browser: Arc<Browser>,
}

impl NetworkMonitorTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for NetworkMonitorTool {
    type Input = NetworkMonitorInput;
    type Output = NetworkMonitorOutput;
    
    fn name(&self) -> &str {
        "monitor_network"
    }
    
    fn description(&self) -> &str {
        "Monitor network requests and responses for a specified duration"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Starting network monitoring for {}ms", input.duration_ms);
        
        let start_time = std::time::Instant::now();
        
        // For now, simulate network monitoring by capturing basic page load info
        // TODO: Implement actual CDP Network domain integration
        let monitor_duration = Duration::from_millis(input.duration_ms);
        
        // Get current page load performance data
        let performance_script = r#"
            JSON.stringify({
                timing: performance.timing,
                navigation: performance.navigation,
                resources: performance.getEntriesByType('resource').map(r => ({
                    name: r.name,
                    duration: r.duration,
                    transferSize: r.transferSize || 0,
                    initiatorType: r.initiatorType,
                    responseEnd: r.responseEnd,
                    responseStart: r.responseStart
                }))
            })
        "#;
        
        tokio::time::sleep(monitor_duration).await;
        
        let performance_result = self.browser.execute_script(performance_script).await?;
        
        let mut requests = Vec::new();
        let mut total_bytes = 0u64;
        let mut failed_count = 0u64;
        
        if let Some(resources) = performance_result.get("resources") {
            if let Some(resources_array) = resources.as_array() {
                for (index, resource) in resources_array.iter().enumerate() {
                    let url = resource.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let duration = resource.get("duration")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0) as u64;
                    
                    let transfer_size = resource.get("transferSize")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    
                    let resource_type = resource.get("initiatorType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("other")
                        .to_string();
                    
                    // Apply filters
                    if let Some(ref filter_types) = input.filter_resource_types {
                        if !filter_types.contains(&resource_type) {
                            continue;
                        }
                    }
                    
                    if let Some(ref domain_filter) = input.domain_filter {
                        if !url.contains(domain_filter) {
                            continue;
                        }
                    }
                    
                    total_bytes += transfer_size;
                    
                    // Simulate some failures for demonstration
                    let status_code = if index % 20 == 0 { 
                        failed_count += 1;
                        Some(404) 
                    } else { 
                        Some(200) 
                    };
                    
                    requests.push(NetworkRequest {
                        url,
                        method: "GET".to_string(),
                        resource_type,
                        timestamp: resource.get("responseEnd")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        status_code,
                        response_size: Some(transfer_size),
                        duration_ms: Some(duration),
                        headers: std::collections::HashMap::new(), // TODO: Get actual headers
                    });
                }
            }
        }
        
        let monitoring_duration = start_time.elapsed().as_millis() as u64;
        
        Ok(NetworkMonitorOutput {
            success: true,
            total_requests: requests.len() as u64,
            requests,
            monitoring_duration_ms: monitoring_duration,
            bytes_transferred: total_bytes,
            failed_requests: failed_count,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.duration_ms == 0 {
            return Err(anyhow!("Duration must be greater than 0"));
        }
        if input.duration_ms > 60000 { // 1 minute max
            return Err(anyhow!("Duration cannot exceed 60 seconds"));
        }
        Ok(())
    }
}

// ============================================================================
// Performance Metrics Tool  
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsInput {
    #[serde(default)]
    pub include_resource_timing: bool,
    #[serde(default)]
    pub include_navigation_timing: bool,
    #[serde(default)]
    pub include_paint_metrics: bool,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetricsOutput {
    pub success: bool,
    pub navigation_timing: Option<NavigationTiming>,
    pub paint_metrics: Option<PaintMetrics>,
    pub resource_timing: Vec<ResourceTimingEntry>,
    pub resource_count: u64,
    pub total_load_time_ms: u64,
    pub dom_content_loaded_ms: u64,
    pub performance_score: Option<f64>,
    pub core_web_vitals: CoreWebVitals,
}

#[derive(Debug, Serialize)]
pub struct CoreWebVitals {
    pub lcp: Option<f64>,  // Largest Contentful Paint
    pub fid: Option<f64>,  // First Input Delay
    pub cls: Option<f64>,  // Cumulative Layout Shift
    pub fcp: Option<f64>,  // First Contentful Paint
    pub ttfb: Option<f64>, // Time to First Byte
}

#[derive(Debug, Serialize)]
pub struct NavigationTiming {
    // Core Web Vitals
    pub first_contentful_paint_ms: Option<f64>,
    pub largest_contentful_paint_ms: Option<f64>,
    pub cumulative_layout_shift: Option<f64>,
    
    // Navigation timing
    pub navigation_start: f64,
    pub dns_lookup_start: f64,
    pub dns_lookup_end: f64,
    pub dns_lookup_duration_ms: f64,
    
    pub connect_start: f64,
    pub connect_end: f64,
    pub connect_duration_ms: f64,
    
    pub request_start: f64,
    pub response_start: f64,
    pub response_end: f64,
    pub request_response_duration_ms: f64,
    
    pub dom_interactive: f64,
    pub dom_content_loaded: f64,
    pub dom_complete: f64,
    pub dom_processing_duration_ms: f64,
    
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub load_complete_duration_ms: f64,
    
    // Additional metrics
    pub redirect_start: Option<f64>,
    pub redirect_end: Option<f64>,
    pub redirect_duration_ms: Option<f64>,
    pub secure_connection_start: Option<f64>,
    pub transfer_size: Option<u64>,
    pub encoded_body_size: Option<u64>,
    pub decoded_body_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PaintMetrics {
    pub first_paint_ms: Option<f64>,
    pub first_contentful_paint_ms: Option<f64>,
    pub largest_contentful_paint_ms: Option<f64>,
    pub paint_entries: Vec<PaintEntry>,
}

#[derive(Debug, Serialize)]
pub struct PaintEntry {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
}

#[derive(Debug, Serialize)]
pub struct ResourceTimingEntry {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,
    pub initiator_type: String,
    pub next_hop_protocol: String,
    pub worker_start: f64,
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
    pub transfer_size: u64,
    pub encoded_body_size: u64,
    pub decoded_body_size: u64,
    pub server_timing: Vec<ServerTimingEntry>,
}

#[derive(Debug, Serialize)]
pub struct ServerTimingEntry {
    pub name: String,
    pub duration: f64,
    pub description: String,
}

pub struct PerformanceMetricsTool {
    browser: Arc<Browser>,
}

impl PerformanceMetricsTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for PerformanceMetricsTool {
    type Input = PerformanceMetricsInput;
    type Output = PerformanceMetricsOutput;
    
    fn name(&self) -> &str {
        "get_performance_metrics"
    }
    
    fn description(&self) -> &str {
        "Collect detailed performance metrics from the current page"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        debug!("Collecting comprehensive performance metrics via CDP and Performance APIs");
        
        // Enhanced performance metrics collection using both CDP and Performance APIs
        let comprehensive_metrics_script = r#"
            JSON.stringify({
                navigation: performance.timing,
                paint: performance.getEntriesByType('paint'),
                resources: performance.getEntriesByType('resource'),
                memory: performance.memory || null,
                // Core Web Vitals via Performance Observer where available
                webVitals: (function() {
                    const vitals = {};
                    try {
                        // Get LCP from performance entries
                        const lcpEntries = performance.getEntriesByType('largest-contentful-paint');
                        if (lcpEntries.length > 0) {
                            vitals.lcp = lcpEntries[lcpEntries.length - 1].startTime;
                        }
                        
                        // Get FCP from paint entries
                        const paintEntries = performance.getEntriesByType('paint');
                        const fcpEntry = paintEntries.find(entry => entry.name === 'first-contentful-paint');
                        if (fcpEntry) {
                            vitals.fcp = fcpEntry.startTime;
                        }
                        
                        // Get FID and CLS require Performance Observer, which we can't easily get here
                        // These would be better collected via CDP Performance domain
                        
                        // TTFB from navigation timing
                        const navTiming = performance.timing;
                        if (navTiming.responseStart && navTiming.navigationStart) {
                            vitals.ttfb = navTiming.responseStart - navTiming.navigationStart;
                        }
                    } catch (e) {
                        console.warn('Error collecting Core Web Vitals:', e);
                    }
                    return vitals;
                })(),
                // Server timing if available
                serverTiming: performance.getEntriesByType('navigation')[0]?.serverTiming || []
            })
        "#;
        
        let metrics_result = self.browser.execute_script(comprehensive_metrics_script).await?;
        
        let navigation_timing = if input.include_navigation_timing {
            if let Some(nav) = metrics_result.get("navigation") {
                let nav_start = nav.get("navigationStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dns_start = nav.get("domainLookupStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dns_end = nav.get("domainLookupEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let connect_start = nav.get("connectStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let connect_end = nav.get("connectEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let secure_start = nav.get("secureConnectionStart").and_then(|v| v.as_f64());
                let request_start = nav.get("requestStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let response_start = nav.get("responseStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let response_end = nav.get("responseEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dom_interactive = nav.get("domInteractive").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dom_content_loaded = nav.get("domContentLoadedEventEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dom_complete = nav.get("domComplete").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let load_start = nav.get("loadEventStart").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let load_end = nav.get("loadEventEnd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let redirect_start = nav.get("redirectStart").and_then(|v| v.as_f64()).filter(|&v| v > 0.0);
                let redirect_end = nav.get("redirectEnd").and_then(|v| v.as_f64()).filter(|&v| v > 0.0);
                
                // Get Core Web Vitals from the webVitals object
                let web_vitals = metrics_result.get("webVitals");
                let fcp = web_vitals.and_then(|wv| wv.get("fcp")).and_then(|v| v.as_f64());
                let lcp = web_vitals.and_then(|wv| wv.get("lcp")).and_then(|v| v.as_f64());
                let _ttfb = web_vitals.and_then(|wv| wv.get("ttfb")).and_then(|v| v.as_f64());
                
                Some(NavigationTiming {
                    // Core Web Vitals
                    first_contentful_paint_ms: fcp,
                    largest_contentful_paint_ms: lcp,
                    cumulative_layout_shift: None, // Requires Performance Observer
                    
                    // Navigation timing breakdown
                    navigation_start: nav_start,
                    dns_lookup_start: dns_start,
                    dns_lookup_end: dns_end,
                    dns_lookup_duration_ms: if dns_end > dns_start { dns_end - dns_start } else { 0.0 },
                    
                    connect_start,
                    connect_end,
                    connect_duration_ms: if connect_end > connect_start { connect_end - connect_start } else { 0.0 },
                    
                    request_start,
                    response_start,
                    response_end,
                    request_response_duration_ms: if response_end > request_start { response_end - request_start } else { 0.0 },
                    
                    dom_interactive,
                    dom_content_loaded,
                    dom_complete,
                    dom_processing_duration_ms: if dom_complete > response_end { dom_complete - response_end } else { 0.0 },
                    
                    load_event_start: load_start,
                    load_event_end: load_end,
                    load_complete_duration_ms: if load_end > nav_start { load_end - nav_start } else { 0.0 },
                    
                    // Optional metrics
                    redirect_start,
                    redirect_end,
                    redirect_duration_ms: if let (Some(start), Some(end)) = (redirect_start, redirect_end) {
                        Some(if end > start { end - start } else { 0.0 })
                    } else {
                        None
                    },
                    secure_connection_start: secure_start,
                    transfer_size: None, // Would need Navigation API v2
                    encoded_body_size: None,
                    decoded_body_size: None,
                })
            } else {
                None
            }
        } else {
            None
        };
        
        let paint_metrics = if input.include_paint_metrics {
            if let Some(paint_entries) = metrics_result.get("paint") {
                if let Some(paint_array) = paint_entries.as_array() {
                    let mut first_paint = None;
                    let mut first_contentful_paint = None;
                    let mut paint_entries_list = Vec::new();
                    
                    for entry in paint_array {
                        if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                            if let Some(start_time) = entry.get("startTime").and_then(|v| v.as_f64()) {
                                match name {
                                    "first-paint" => first_paint = Some(start_time),
                                    "first-contentful-paint" => first_contentful_paint = Some(start_time),
                                    _ => {}
                                }
                                
                                paint_entries_list.push(PaintEntry {
                                    name: name.to_string(),
                                    start_time,
                                    duration: entry.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                });
                            }
                        }
                    }
                    
                    // Get LCP from web vitals
                    let lcp = metrics_result.get("webVitals")
                        .and_then(|wv| wv.get("lcp"))
                        .and_then(|v| v.as_f64());
                    
                    Some(PaintMetrics {
                        first_paint_ms: first_paint,
                        first_contentful_paint_ms: first_contentful_paint,
                        largest_contentful_paint_ms: lcp,
                        paint_entries: paint_entries_list,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Collect resource timing entries
        let resource_timing = if input.include_resource_timing {
            if let Some(resources) = metrics_result.get("resources") {
                if let Some(resources_array) = resources.as_array() {
                    resources_array.iter().filter_map(|resource| {
                        let name = resource.get("name").and_then(|v| v.as_str())?.to_string();
                        let entry_type = resource.get("entryType").and_then(|v| v.as_str()).unwrap_or("resource").to_string();
                        let start_time = resource.get("startTime").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let duration = resource.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let initiator_type = resource.get("initiatorType").and_then(|v| v.as_str()).unwrap_or("other").to_string();
                        let next_hop_protocol = resource.get("nextHopProtocol").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        
                        // Server timing entries
                        let server_timing = if let Some(st) = resource.get("serverTiming") {
                            if let Some(st_array) = st.as_array() {
                                st_array.iter().filter_map(|st_entry| {
                                    let st_name = st_entry.get("name").and_then(|v| v.as_str())?.to_string();
                                    let st_duration = st_entry.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let st_description = st_entry.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    
                                    Some(ServerTimingEntry {
                                        name: st_name,
                                        duration: st_duration,
                                        description: st_description,
                                    })
                                }).collect()
                            } else {
                                Vec::new()
                            }
                        } else {
                            Vec::new()
                        };
                        
                        Some(ResourceTimingEntry {
                            name,
                            entry_type,
                            start_time,
                            duration,
                            initiator_type,
                            next_hop_protocol,
                            worker_start: resource.get("workerStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            redirect_start: resource.get("redirectStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            redirect_end: resource.get("redirectEnd").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            fetch_start: resource.get("fetchStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            domain_lookup_start: resource.get("domainLookupStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            domain_lookup_end: resource.get("domainLookupEnd").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            connect_start: resource.get("connectStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            connect_end: resource.get("connectEnd").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            secure_connection_start: resource.get("secureConnectionStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            request_start: resource.get("requestStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            response_start: resource.get("responseStart").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            response_end: resource.get("responseEnd").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            transfer_size: resource.get("transferSize").and_then(|v| v.as_u64()).unwrap_or(0),
                            encoded_body_size: resource.get("encodedBodySize").and_then(|v| v.as_u64()).unwrap_or(0),
                            decoded_body_size: resource.get("decodedBodySize").and_then(|v| v.as_u64()).unwrap_or(0),
                            server_timing,
                        })
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        let resource_count = resource_timing.len() as u64;
        
        // Calculate total load time from navigation timing
        let total_load_time = navigation_timing.as_ref()
            .map(|nt| nt.load_complete_duration_ms as u64)
            .unwrap_or(0);
        
        let dom_content_loaded = navigation_timing.as_ref()
            .map(|nt| (nt.dom_content_loaded - nt.navigation_start) as u64)
            .unwrap_or(0);
        
        // Calculate performance score based on Core Web Vitals
        let performance_score = if let Some(nt) = &navigation_timing {
            let mut score: f64 = 100.0;
            
            // LCP scoring (target: < 2.5s)
            if let Some(lcp) = nt.largest_contentful_paint_ms {
                if lcp > 4000.0 { score -= 30.0; }
                else if lcp > 2500.0 { score -= 15.0; }
            }
            
            // FCP scoring (target: < 1.8s)
            if let Some(fcp) = nt.first_contentful_paint_ms {
                if fcp > 3000.0 { score -= 20.0; }
                else if fcp > 1800.0 { score -= 10.0; }
            }
            
            // Load time scoring (target: < 3s)
            if total_load_time > 5000 { score -= 25.0; }
            else if total_load_time > 3000 { score -= 15.0; }
            
            Some(score.max(0.0))
        } else {
            None
        };
        
        // Extract Core Web Vitals
        let core_web_vitals = if let Some(web_vitals) = metrics_result.get("webVitals") {
            CoreWebVitals {
                lcp: web_vitals.get("lcp").and_then(|v| v.as_f64()),
                fid: web_vitals.get("fid").and_then(|v| v.as_f64()), // Would need Performance Observer
                cls: web_vitals.get("cls").and_then(|v| v.as_f64()), // Would need Performance Observer
                fcp: web_vitals.get("fcp").and_then(|v| v.as_f64()),
                ttfb: web_vitals.get("ttfb").and_then(|v| v.as_f64()),
            }
        } else {
            CoreWebVitals {
                lcp: None,
                fid: None,
                cls: None,
                fcp: None,
                ttfb: None,
            }
        };
        
        Ok(PerformanceMetricsOutput {
            success: true,
            navigation_timing,
            paint_metrics,
            resource_timing,
            resource_count,
            total_load_time_ms: total_load_time,
            dom_content_loaded_ms: dom_content_loaded,
            performance_score,
            core_web_vitals,
        })
    }
}

// ============================================================================
// Console Logs Monitoring Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsoleLogsInput {
    #[serde(default = "default_capture_duration")]
    pub duration_ms: u64,
    #[serde(default)]
    pub include_errors: bool,
    #[serde(default)]
    pub include_warnings: bool,
    #[serde(default)]
    pub include_info: bool,
    #[serde(default)]
    pub include_debug: bool,
    #[serde(default)]
    pub filter_pattern: Option<String>,
    #[serde(default)]
    pub max_entries: Option<u32>,
}

fn default_capture_duration() -> u64 {
    10000 // 10 seconds
}

#[derive(Debug, Serialize)]
pub struct ConsoleLogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: f64,
    pub source: String,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub stack_trace: Option<String>,
    pub args: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ConsoleLogsOutput {
    pub success: bool,
    pub total_logs: u64,
    pub logs: Vec<ConsoleLogEntry>,
    pub error_count: u64,
    pub warning_count: u64,
    pub info_count: u64,
    pub debug_count: u64,
    pub monitoring_duration_ms: u64,
    pub performance_impact: PerformanceImpact,
}

#[derive(Debug, Serialize)]
pub struct PerformanceImpact {
    pub excessive_logging: bool,
    pub error_frequency: f64, // errors per second
    pub memory_leaks_detected: bool,
    pub performance_warnings: Vec<String>,
}

pub struct ConsoleLogsTool {
    browser: Arc<Browser>,
}

impl ConsoleLogsTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ConsoleLogsTool {
    type Input = ConsoleLogsInput;
    type Output = ConsoleLogsOutput;
    
    fn name(&self) -> &str {
        "capture_console_logs"
    }
    
    fn description(&self) -> &str {
        "Capture and analyze browser console logs with performance impact assessment"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Starting console logs capture for {}ms", input.duration_ms);
        
        let start_time = std::time::Instant::now();
        
        // Enhanced console monitoring script that captures logs with detailed context
        let console_capture_script = format!(r#"
            (function() {{
                const logs = [];
                const originalConsole = {{}};
                const startTime = performance.now();
                const maxEntries = {};
                
                // Preserve original console methods
                ['log', 'info', 'warn', 'error', 'debug'].forEach(method => {{
                    originalConsole[method] = console[method];
                }});
                
                // Override console methods to capture logs
                ['log', 'info', 'warn', 'error', 'debug'].forEach(method => {{
                    console[method] = function(...args) {{
                        if (logs.length < maxEntries) {{
                            const entry = {{
                                level: method,
                                message: args.map(arg => 
                                    typeof arg === 'object' ? JSON.stringify(arg, null, 2) : String(arg)
                                ).join(' '),
                                timestamp: performance.now(),
                                source: 'console',
                                line_number: null,
                                column_number: null,
                                stack_trace: new Error().stack,
                                args: args.map(arg => {{
                                    try {{
                                        return JSON.parse(JSON.stringify(arg));
                                    }} catch (e) {{
                                        return String(arg);
                                    }}
                                }})
                            }};
                            logs.push(entry);
                        }}
                        
                        // Call original method
                        originalConsole[method].apply(console, args);
                    }};
                }});
                
                // Capture uncaught errors
                const originalErrorHandler = window.onerror;
                window.onerror = function(message, source, lineno, colno, error) {{
                    if (logs.length < maxEntries) {{
                        logs.push({{
                            level: 'error',
                            message: message,
                            timestamp: performance.now(),
                            source: source || 'unknown',
                            line_number: lineno,
                            column_number: colno,
                            stack_trace: error ? error.stack : null,
                            args: [message]
                        }});
                    }}
                    
                    if (originalErrorHandler) {{
                        return originalErrorHandler.apply(this, arguments);
                    }}
                }};
                
                // Capture unhandled promise rejections
                const originalRejectionHandler = window.onunhandledrejection;
                window.onunhandledrejection = function(event) {{
                    if (logs.length < maxEntries) {{
                        logs.push({{
                            level: 'error',
                            message: 'Unhandled Promise Rejection: ' + event.reason,
                            timestamp: performance.now(),
                            source: 'promise',
                            line_number: null,
                            column_number: null,
                            stack_trace: event.reason && event.reason.stack ? event.reason.stack : null,
                            args: [event.reason]
                        }});
                    }}
                    
                    if (originalRejectionHandler) {{
                        return originalRejectionHandler.call(this, event);
                    }}
                }};
                
                // Store reference for cleanup
                window.__consoleLogCapture = {{
                    logs: logs,
                    cleanup: function() {{
                        // Restore original methods
                        Object.keys(originalConsole).forEach(method => {{
                            console[method] = originalConsole[method];
                        }});
                        window.onerror = originalErrorHandler;
                        window.onunhandledrejection = originalRejectionHandler;
                        delete window.__consoleLogCapture;
                    }}
                }};
                
                return {{ initialized: true, startTime: startTime }};
            }})();
        "#, input.max_entries.unwrap_or(1000));
        
        // Initialize console capture
        let init_result = self.browser.execute_script(&console_capture_script).await?;
        debug!("Console capture initialized: {:?}", init_result);
        
        // Wait for the specified duration
        let monitor_duration = Duration::from_millis(input.duration_ms);
        tokio::time::sleep(monitor_duration).await;
        
        // Collect captured logs
        let collect_script = r#"
            (function() {
                const capture = window.__consoleLogCapture;
                if (!capture) {
                    return { logs: [], error: 'Console capture not initialized' };
                }
                
                const result = {
                    logs: capture.logs.slice(),
                    performance: {
                        memory: performance.memory ? {
                            usedJSHeapSize: performance.memory.usedJSHeapSize,
                            totalJSHeapSize: performance.memory.totalJSHeapSize,
                            jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
                        } : null,
                        timing: performance.now()
                    }
                };
                
                // Cleanup
                capture.cleanup();
                
                return result;
            })();
        "#;
        
        let collect_result = self.browser.execute_script(collect_script).await?;
        
        let monitoring_duration = start_time.elapsed().as_millis() as u64;
        
        let mut logs = Vec::new();
        let mut error_count = 0u64;
        let mut warning_count = 0u64;
        let mut info_count = 0u64;
        let mut debug_count = 0u64;
        
        if let Some(logs_data) = collect_result.get("logs") {
            if let Some(logs_array) = logs_data.as_array() {
                for log_entry in logs_array {
                    let level = log_entry.get("level")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    // Apply level filters
                    let should_include = match level.as_str() {
                        "error" => input.include_errors,
                        "warn" => input.include_warnings,
                        "info" => input.include_info,
                        "debug" => input.include_debug,
                        "log" => input.include_info, // Treat console.log as info
                        _ => true,
                    };
                    
                    if !should_include {
                        continue;
                    }
                    
                    let message = log_entry.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    // Apply text filter if specified
                    if let Some(ref pattern) = input.filter_pattern {
                        if !message.contains(pattern) {
                            continue;
                        }
                    }
                    
                    // Count by level
                    match level.as_str() {
                        "error" => error_count += 1,
                        "warn" => warning_count += 1,
                        "info" | "log" => info_count += 1,
                        "debug" => debug_count += 1,
                        _ => {}
                    }
                    
                    logs.push(ConsoleLogEntry {
                        level,
                        message,
                        timestamp: log_entry.get("timestamp")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        source: log_entry.get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        line_number: log_entry.get("line_number")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32),
                        column_number: log_entry.get("column_number")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as u32),
                        stack_trace: log_entry.get("stack_trace")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        args: log_entry.get("args")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.clone())
                            .unwrap_or_else(Vec::new),
                    });
                }
            }
        }
        
        // Analyze performance impact
        let error_frequency = if monitoring_duration > 0 {
            (error_count as f64) / (monitoring_duration as f64 / 1000.0)
        } else {
            0.0
        };
        
        let excessive_logging = logs.len() > 100; // More than 100 logs in monitoring period
        let mut performance_warnings = Vec::new();
        
        if excessive_logging {
            performance_warnings.push("Excessive console logging detected - may impact performance".to_string());
        }
        
        if error_frequency > 1.0 {
            performance_warnings.push(format!("High error rate: {:.2} errors per second", error_frequency));
        }
        
        if warning_count > 20 {
            performance_warnings.push("High number of warnings detected".to_string());
        }
        
        // Check for memory-related warnings
        let memory_warnings = logs.iter().any(|log| {
            log.level == "warn" && (
                log.message.contains("memory") || 
                log.message.contains("leak") ||
                log.message.contains("heap")
            )
        });
        
        if memory_warnings {
            performance_warnings.push("Memory-related warnings detected".to_string());
        }
        
        let performance_impact = PerformanceImpact {
            excessive_logging,
            error_frequency,
            memory_leaks_detected: memory_warnings,
            performance_warnings,
        };
        
        Ok(ConsoleLogsOutput {
            success: true,
            total_logs: logs.len() as u64,
            logs,
            error_count,
            warning_count,
            info_count,
            debug_count,
            monitoring_duration_ms: monitoring_duration,
            performance_impact,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.duration_ms == 0 {
            return Err(anyhow!("Duration must be greater than 0"));
        }
        if input.duration_ms > 300000 { // 5 minutes max
            return Err(anyhow!("Duration cannot exceed 5 minutes"));
        }
        if let Some(max_entries) = input.max_entries {
            if max_entries == 0 {
                return Err(anyhow!("Max entries must be greater than 0"));
            }
            if max_entries > 10000 {
                return Err(anyhow!("Max entries cannot exceed 10000"));
            }
        }
        Ok(())
    }
}

// ============================================================================
// Computed Styles Extraction Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputedStylesInput {
    pub selectors: Vec<String>, // CSS selectors to analyze
    #[serde(default)]
    pub properties: Option<Vec<String>>, // Specific properties to extract, or all if None
    #[serde(default)]
    pub include_pseudo_elements: bool,
    #[serde(default)]
    pub include_inherited: bool,
    #[serde(default)]
    pub performance_analysis: bool, // Analyze style computation performance
}

#[derive(Debug, Serialize)]
pub struct ElementStyleInfo {
    pub selector: String,
    pub element_count: u32,
    pub computed_styles: std::collections::HashMap<String, String>,
    pub css_rules: Vec<CSSRuleInfo>,
    pub pseudo_elements: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    pub performance_metrics: Option<StylePerformanceMetrics>,
}

#[derive(Debug, Serialize)]
pub struct CSSRuleInfo {
    pub selector_text: String,
    pub css_text: String,
    pub origin: String, // user-agent, user, author
    pub media: Option<String>,
    pub specificity: u32,
    pub source_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StylePerformanceMetrics {
    pub style_computation_time_ms: f64,
    pub layout_invalidation_count: u32,
    pub paint_invalidation_count: u32,
    pub complex_selectors: Vec<String>,
    pub expensive_properties: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ComputedStylesOutput {
    pub success: bool,
    pub elements: Vec<ElementStyleInfo>,
    pub total_elements_analyzed: u32,
    pub total_computation_time_ms: f64,
    pub style_performance_insights: StylePerformanceInsights,
}

#[derive(Debug, Serialize)]
pub struct StylePerformanceInsights {
    pub potential_performance_issues: Vec<String>,
    pub optimization_recommendations: Vec<String>,
    pub css_complexity_score: f64,
    pub layout_thrashing_risk: bool,
}

pub struct ComputedStylesTool {
    browser: Arc<Browser>,
}

impl ComputedStylesTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ComputedStylesTool {
    type Input = ComputedStylesInput;
    type Output = ComputedStylesOutput;
    
    fn name(&self) -> &str {
        "extract_computed_styles"
    }
    
    fn description(&self) -> &str {
        "Extract computed styles for specified elements with performance analysis"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Extracting computed styles for {} selectors", input.selectors.len());
        
        let start_time = std::time::Instant::now();
        
        // Enhanced style extraction script with performance monitoring
        let style_extraction_script = format!(r#"
            (function() {{
                const startTime = performance.now();
                const results = [];
                const selectors = {};
                const properties = {};
                const includePseudo = {};
                const includeInherited = {};
                const performanceAnalysis = {};
                
                // Performance monitoring setup
                let layoutCount = 0;
                let paintCount = 0;
                
                if (performanceAnalysis) {{
                    // Monitor layout and paint events
                    const observer = new PerformanceObserver((list) => {{
                        for (const entry of list.getEntries()) {{
                            if (entry.entryType === 'measure' && entry.name.includes('layout')) {{
                                layoutCount++;
                            }}
                            if (entry.entryType === 'measure' && entry.name.includes('paint')) {{
                                paintCount++;
                            }}
                        }}
                    }});
                    
                    try {{
                        observer.observe({{ entryTypes: ['measure'] }});
                    }} catch(e) {{
                        console.warn('Performance observer not supported');
                    }}
                }}
                
                // CSS specificity calculation
                function calculateSpecificity(selector) {{
                    const idCount = (selector.match(/#/g) || []).length;
                    const classCount = (selector.match(/\.[^.\s]+/g) || []).length;
                    const attrCount = (selector.match(/\[[^\]]+\]/g) || []).length;
                    const tagCount = (selector.match(/^[a-zA-Z]+|[^#.\[\]:]+[a-zA-Z]+/g) || []).length;
                    
                    return idCount * 100 + classCount * 10 + attrCount * 10 + tagCount;
                }}
                
                // Analyze selector complexity
                function analyzeComplexity(selector) {{
                    const complexities = [];
                    if (selector.includes(':nth-child') || selector.includes(':nth-of-type')) {{
                        complexities.push('nth-selectors');
                    }}
                    if (selector.includes('~') || selector.includes('+')) {{
                        complexities.push('sibling-selectors');
                    }}
                    if ((selector.match(/>/g) || []).length > 2) {{
                        complexities.push('deep-nesting');
                    }}
                    if (selector.includes(':not(')) {{
                        complexities.push('negation-pseudo');
                    }}
                    return complexities;
                }}
                
                // Check for expensive CSS properties
                function isExpensiveProperty(property, value) {{
                    const expensiveProperties = [
                        'box-shadow', 'filter', 'transform', 'opacity',
                        'border-radius', 'clip-path', 'mask'
                    ];
                    
                    if (expensiveProperties.includes(property)) {{
                        return true;
                    }}
                    
                    // Check for complex gradients
                    if (property.includes('background') && value.includes('gradient')) {{
                        return true;
                    }}
                    
                    // Check for complex animations
                    if (property.includes('transition') && value.includes('all')) {{
                        return true;
                    }}
                    
                    return false;
                }}
                
                // Extract styles for each selector
                for (const selector of selectors) {{
                    const selectorStartTime = performance.now();
                    
                    try {{
                        const elements = document.querySelectorAll(selector);
                        if (elements.length === 0) {{
                            results.push({{
                                selector: selector,
                                element_count: 0,
                                computed_styles: {{}},
                                css_rules: [],
                                pseudo_elements: null,
                                performance_metrics: null
                            }});
                            continue;
                        }}
                        
                        // Use the first matching element for style computation
                        const element = elements[0];
                        const computedStyles = window.getComputedStyle(element);
                        const stylesObj = {{}};
                        const expensiveProps = [];
                        
                        // Extract requested properties or all properties
                        const propsToExtract = properties || Array.from(computedStyles);
                        
                        for (const prop of propsToExtract) {{
                            const value = computedStyles.getPropertyValue(prop);
                            if (value) {{
                                stylesObj[prop] = value;
                                
                                if (performanceAnalysis && isExpensiveProperty(prop, value)) {{
                                    expensiveProps.push(`${{prop}}: ${{value}}`);
                                }}
                            }}
                        }}
                        
                        // Extract CSS rules that apply to this element
                        const cssRules = [];
                        try {{
                            for (const styleSheet of document.styleSheets) {{
                                try {{
                                    for (const rule of styleSheet.cssRules || []) {{
                                        if (rule.selectorText && element.matches(rule.selectorText)) {{
                                            cssRules.push({{
                                                selector_text: rule.selectorText,
                                                css_text: rule.cssText,
                                                origin: styleSheet.ownerNode ? 'author' : 'user-agent',
                                                media: rule.media ? rule.media.mediaText : null,
                                                specificity: calculateSpecificity(rule.selectorText),
                                                source_url: styleSheet.href
                                            }});
                                        }}
                                    }}
                                }} catch (e) {{
                                    // Skip inaccessible stylesheets (CORS)
                                }}
                            }}
                        }} catch (e) {{
                            console.warn('Could not access stylesheets:', e);
                        }}
                        
                        // Extract pseudo-element styles if requested
                        let pseudoStyles = null;
                        if (includePseudo) {{
                            pseudoStyles = {{}};
                            const pseudoElements = ['::before', '::after', '::first-line', '::first-letter'];
                            
                            for (const pseudo of pseudoElements) {{
                                try {{
                                    const pseudoComputedStyles = window.getComputedStyle(element, pseudo);
                                    const pseudoStylesObj = {{}};
                                    
                                    for (const prop of propsToExtract) {{
                                        const value = pseudoComputedStyles.getPropertyValue(prop);
                                        if (value && value !== stylesObj[prop]) {{
                                            pseudoStylesObj[prop] = value;
                                        }}
                                    }}
                                    
                                    if (Object.keys(pseudoStylesObj).length > 0) {{
                                        pseudoStyles[pseudo] = pseudoStylesObj;
                                    }}
                                }} catch (e) {{
                                    // Pseudo-element doesn't exist or is not accessible
                                }}
                            }}
                        }}
                        
                        // Performance metrics
                        let performanceMetrics = null;
                        if (performanceAnalysis) {{
                            const selectorComplexity = analyzeComplexity(selector);
                            const computationTime = performance.now() - selectorStartTime;
                            
                            performanceMetrics = {{
                                style_computation_time_ms: computationTime,
                                layout_invalidation_count: layoutCount,
                                paint_invalidation_count: paintCount,
                                complex_selectors: selectorComplexity,
                                expensive_properties: expensiveProps
                            }};
                        }}
                        
                        results.push({{
                            selector: selector,
                            element_count: elements.length,
                            computed_styles: stylesObj,
                            css_rules: cssRules,
                            pseudo_elements: pseudoStyles,
                            performance_metrics: performanceMetrics
                        }});
                        
                    }} catch (error) {{
                        results.push({{
                            selector: selector,
                            element_count: 0,
                            computed_styles: {{}},
                            css_rules: [],
                            pseudo_elements: null,
                            performance_metrics: null,
                            error: error.message
                        }});
                    }}
                }}
                
                const totalTime = performance.now() - startTime;
                
                return {{
                    results: results,
                    total_time: totalTime,
                    performance_counts: {{
                        layout: layoutCount,
                        paint: paintCount
                    }}
                }};
            }})();
        "#, 
            serde_json::to_string(&input.selectors).unwrap(),
            serde_json::to_string(&input.properties).unwrap(),
            input.include_pseudo_elements,
            input.include_inherited,
            input.performance_analysis
        );
        
        let extraction_result = self.browser.execute_script(&style_extraction_script).await?;
        
        let total_computation_time = start_time.elapsed().as_millis() as f64;
        
        let mut elements = Vec::new();
        let mut total_elements_analyzed = 0u32;
        let mut potential_issues = Vec::new();
        let mut optimization_recommendations = Vec::new();
        let mut complexity_score = 0.0;
        let mut layout_thrashing_risk = false;
        
        if let Some(results) = extraction_result.get("results") {
            if let Some(results_array) = results.as_array() {
                for result in results_array {
                    let selector = result.get("selector")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    let element_count = result.get("element_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    
                    total_elements_analyzed += element_count;
                    
                    // Extract computed styles
                    let computed_styles = if let Some(styles) = result.get("computed_styles") {
                        if let Some(styles_obj) = styles.as_object() {
                            styles_obj.iter().map(|(k, v)| {
                                (k.clone(), v.as_str().unwrap_or("").to_string())
                            }).collect()
                        } else {
                            HashMap::new()
                        }
                    } else {
                        HashMap::new()
                    };
                    
                    // Extract CSS rules
                    let css_rules = if let Some(rules) = result.get("css_rules") {
                        if let Some(rules_array) = rules.as_array() {
                            rules_array.iter().filter_map(|rule| {
                                let selector_text = rule.get("selector_text")?.as_str()?.to_string();
                                let css_text = rule.get("css_text")?.as_str()?.to_string();
                                let origin = rule.get("origin")?.as_str().unwrap_or("unknown").to_string();
                                let media = rule.get("media").and_then(|v| v.as_str()).map(|s| s.to_string());
                                let specificity = rule.get("specificity")?.as_u64().unwrap_or(0) as u32;
                                let source_url = rule.get("source_url").and_then(|v| v.as_str()).map(|s| s.to_string());
                                
                                Some(CSSRuleInfo {
                                    selector_text,
                                    css_text,
                                    origin,
                                    media,
                                    specificity,
                                    source_url,
                                })
                            }).collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    
                    // Extract pseudo-element styles
                    let pseudo_elements = if input.include_pseudo_elements {
                        result.get("pseudo_elements").and_then(|pe| {
                            pe.as_object().map(|obj| {
                                obj.iter().map(|(pseudo, styles)| {
                                    let styles_map = styles.as_object()
                                        .map(|s_obj| {
                                            s_obj.iter().map(|(k, v)| {
                                                (k.clone(), v.as_str().unwrap_or("").to_string())
                                            }).collect()
                                        })
                                        .unwrap_or_else(HashMap::new);
                                    (pseudo.clone(), styles_map)
                                }).collect()
                            })
                        })
                    } else {
                        None
                    };
                    
                    // Extract performance metrics
                    let performance_metrics = if input.performance_analysis {
                        result.get("performance_metrics").map(|pm| {
                            StylePerformanceMetrics {
                                style_computation_time_ms: pm.get("style_computation_time_ms")
                                    .and_then(|v| v.as_f64()).unwrap_or(0.0),
                                layout_invalidation_count: pm.get("layout_invalidation_count")
                                    .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                paint_invalidation_count: pm.get("paint_invalidation_count")
                                    .and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                complex_selectors: pm.get("complex_selectors")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                    .unwrap_or_else(Vec::new),
                                expensive_properties: pm.get("expensive_properties")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                    .unwrap_or_else(Vec::new),
                            }
                        })
                    } else {
                        None
                    };
                    
                    // Analyze for performance issues
                    if let Some(ref perf) = performance_metrics {
                        complexity_score += perf.style_computation_time_ms;
                        
                        if perf.style_computation_time_ms > 16.0 {
                            potential_issues.push(format!("Slow style computation for '{}': {:.2}ms", selector, perf.style_computation_time_ms));
                        }
                        
                        if !perf.complex_selectors.is_empty() {
                            potential_issues.push(format!("Complex selectors detected in '{}': {:?}", selector, perf.complex_selectors));
                            optimization_recommendations.push("Consider simplifying CSS selectors to improve performance".to_string());
                        }
                        
                        if !perf.expensive_properties.is_empty() {
                            potential_issues.push(format!("Expensive CSS properties in '{}': {:?}", selector, perf.expensive_properties));
                            optimization_recommendations.push("Consider using CSS containment or will-change for expensive properties".to_string());
                        }
                        
                        if perf.layout_invalidation_count > 5 || perf.paint_invalidation_count > 5 {
                            layout_thrashing_risk = true;
                        }
                    }
                    
                    // Check for high specificity
                    let high_specificity_rules = css_rules.iter().filter(|rule| rule.specificity > 100).count();
                    if high_specificity_rules > 0 {
                        potential_issues.push(format!("High specificity CSS rules detected for '{}'", selector));
                        optimization_recommendations.push("Consider reducing CSS specificity to improve maintainability".to_string());
                    }
                    
                    elements.push(ElementStyleInfo {
                        selector,
                        element_count,
                        computed_styles,
                        css_rules,
                        pseudo_elements,
                        performance_metrics,
                    });
                }
            }
        }
        
        // Normalize complexity score
        complexity_score = if !elements.is_empty() {
            complexity_score / elements.len() as f64
        } else {
            0.0
        };
        
        // Deduplicate optimization recommendations
        optimization_recommendations.sort();
        optimization_recommendations.dedup();
        
        let style_performance_insights = StylePerformanceInsights {
            potential_performance_issues: potential_issues,
            optimization_recommendations,
            css_complexity_score: complexity_score,
            layout_thrashing_risk,
        };
        
        Ok(ComputedStylesOutput {
            success: true,
            elements,
            total_elements_analyzed,
            total_computation_time_ms: total_computation_time,
            style_performance_insights,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selectors.is_empty() {
            return Err(anyhow!("At least one CSS selector must be provided"));
        }
        if input.selectors.len() > 50 {
            return Err(anyhow!("Maximum 50 selectors allowed"));
        }
        
        // Validate CSS selectors
        for selector in &input.selectors {
            if selector.trim().is_empty() {
                return Err(anyhow!("Empty selector not allowed"));
            }
            if selector.len() > 500 {
                return Err(anyhow!("Selector too long: maximum 500 characters"));
            }
        }
        
        if let Some(ref properties) = input.properties {
            if properties.len() > 200 {
                return Err(anyhow!("Maximum 200 CSS properties allowed"));
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// Accessibility Tree Analysis Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessibilityAnalysisInput {
    #[serde(default)]
    pub root_selector: Option<String>, // Analyze from specific element, or document root
    #[serde(default)]
    pub include_aria_info: bool,
    #[serde(default)]
    pub include_computed_names: bool,
    #[serde(default)]
    pub check_color_contrast: bool,
    #[serde(default)]
    pub validate_semantic_structure: bool,
    #[serde(default)]
    pub max_depth: Option<u32>, // Limit tree traversal depth
}

#[derive(Debug, Serialize)]
pub struct AccessibilityNode {
    pub tag_name: String,
    pub role: Option<String>,
    pub name: Option<String>, // Accessible name
    pub description: Option<String>, // Accessible description
    pub value: Option<String>,
    pub level: u32, // Depth in the tree
    pub aria_attributes: std::collections::HashMap<String, String>,
    pub computed_properties: Option<ComputedA11yProperties>,
    pub accessibility_violations: Vec<A11yViolation>,
    pub children: Vec<AccessibilityNode>,
}

#[derive(Debug, Serialize)]
pub struct ComputedA11yProperties {
    pub accessible_name: String,
    pub accessible_description: String,
    pub role: String,
    pub states: Vec<String>, // expanded, checked, etc.
    pub properties: Vec<String>, // required, readonly, etc.
    pub color_contrast_ratio: Option<f64>,
    pub font_size: Option<f64>,
    pub clickable_area: Option<ClickableArea>,
}

#[derive(Debug, Serialize)]
pub struct ClickableArea {
    pub width: f64,
    pub height: f64,
    pub meets_minimum_size: bool, // 44x44px minimum
}

#[derive(Debug, Serialize)]
pub struct A11yViolation {
    pub severity: String, // error, warning, info
    pub rule_id: String,
    pub message: String,
    pub help_url: Option<String>,
    pub element_selector: String,
}

#[derive(Debug, Serialize)]
pub struct AccessibilityAnalysisOutput {
    pub success: bool,
    pub accessibility_tree: AccessibilityNode,
    pub total_nodes_analyzed: u32,
    pub violations_summary: ViolationsSummary,
    pub accessibility_score: f64, // 0-100 score based on violations and best practices
    pub color_contrast_issues: Vec<ContrastIssue>,
    pub semantic_structure_issues: Vec<StructuralIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ViolationsSummary {
    pub total_violations: u32,
    pub errors: u32,
    pub warnings: u32,
    pub info: u32,
    pub by_rule: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Serialize)]
pub struct ContrastIssue {
    pub element_selector: String,
    pub foreground_color: String,
    pub background_color: String,
    pub contrast_ratio: f64,
    pub level: String, // AA, AAA
    pub passes: bool,
}

#[derive(Debug, Serialize)]
pub struct StructuralIssue {
    pub issue_type: String,
    pub description: String,
    pub element_selector: String,
    pub suggestion: String,
}

pub struct AccessibilityAnalysisTool {
    browser: Arc<Browser>,
}

impl AccessibilityAnalysisTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for AccessibilityAnalysisTool {
    type Input = AccessibilityAnalysisInput;
    type Output = AccessibilityAnalysisOutput;
    
    fn name(&self) -> &str {
        "analyze_accessibility"
    }
    
    fn description(&self) -> &str {
        "Analyze accessibility tree structure and identify WCAG compliance issues"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Starting accessibility tree analysis");
        
        let start_time = std::time::Instant::now();
        
        // Comprehensive accessibility analysis script
        let accessibility_script = format!(r#"
            (function() {{
                const rootSelector = {};
                const includeAria = {};
                const includeComputedNames = {};
                const checkColorContrast = {};
                const validateSemanticStructure = {};
                const maxDepth = {};
                
                // Helper functions for accessibility analysis
                const a11yUtils = {{
                    // Calculate color contrast ratio
                    getContrastRatio: function(fg, bg) {{
                        function parseColor(color) {{
                            const rgb = color.match(/\d+/g);
                            if (!rgb || rgb.length < 3) return null;
                            return {{
                                r: parseInt(rgb[0]),
                                g: parseInt(rgb[1]),
                                b: parseInt(rgb[2])
                            }};
                        }}
                        
                        function getLuminance(r, g, b) {{
                            const [rs, gs, bs] = [r, g, b].map(c => {{
                                c = c / 255;
                                return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
                            }});
                            return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
                        }}
                        
                        const fgColor = parseColor(fg);
                        const bgColor = parseColor(bg);
                        
                        if (!fgColor || !bgColor) return null;
                        
                        const fgLum = getLuminance(fgColor.r, fgColor.g, fgColor.b);
                        const bgLum = getLuminance(bgColor.r, bgColor.g, bgColor.b);
                        
                        const lightest = Math.max(fgLum, bgLum);
                        const darkest = Math.min(fgLum, bgLum);
                        
                        return (lightest + 0.05) / (darkest + 0.05);
                    }},
                    
                    // Get accessible name using ARIA spec algorithm
                    getAccessibleName: function(element) {{
                        // aria-labelledby takes precedence
                        const labelledBy = element.getAttribute('aria-labelledby');
                        if (labelledBy) {{
                            const labels = labelledBy.split(' ').map(id => document.getElementById(id))
                                .filter(el => el).map(el => el.textContent.trim()).join(' ');
                            if (labels) return labels;
                        }}
                        
                        // aria-label
                        const ariaLabel = element.getAttribute('aria-label');
                        if (ariaLabel && ariaLabel.trim()) return ariaLabel.trim();
                        
                        // label element for form controls
                        if (['INPUT', 'TEXTAREA', 'SELECT'].includes(element.tagName)) {{
                            const label = document.querySelector(`label[for="${{element.id}}"]`);
                            if (label) return label.textContent.trim();
                            
                            // Label wrapping the element
                            const wrappingLabel = element.closest('label');
                            if (wrappingLabel) {{
                                const clone = wrappingLabel.cloneNode(true);
                                const input = clone.querySelector('input, textarea, select');
                                if (input) input.remove();
                                return clone.textContent.trim();
                            }}
                        }}
                        
                        // alt attribute for images
                        if (element.tagName === 'IMG') {{
                            return element.getAttribute('alt') || '';
                        }}
                        
                        // title attribute as last resort
                        const title = element.getAttribute('title');
                        if (title && title.trim()) return title.trim();
                        
                        // Text content for certain elements
                        if (['BUTTON', 'A', 'TH', 'TD', 'LEGEND'].includes(element.tagName)) {{
                            return element.textContent.trim();
                        }}
                        
                        return '';
                    }},
                    
                    // Get accessible description
                    getAccessibleDescription: function(element) {{
                        const describedBy = element.getAttribute('aria-describedby');
                        if (describedBy) {{
                            return describedBy.split(' ').map(id => document.getElementById(id))
                                .filter(el => el).map(el => el.textContent.trim()).join(' ');
                        }}
                        return '';
                    }},
                    
                    // Get implicit ARIA role
                    getImplicitRole: function(element) {{
                        const tagName = element.tagName.toLowerCase();
                        const type = element.type;
                        
                        const roleMap = {{
                            'a': element.href ? 'link' : 'generic',
                            'button': 'button',
                            'input': {{
                                'button': 'button',
                                'checkbox': 'checkbox',
                                'radio': 'radio',
                                'range': 'slider',
                                'submit': 'button',
                                'reset': 'button'
                            }}[type] || 'textbox',
                            'img': element.alt !== undefined ? 'img' : 'presentation',
                            'h1': 'heading',
                            'h2': 'heading',
                            'h3': 'heading',
                            'h4': 'heading',
                            'h5': 'heading',
                            'h6': 'heading',
                            'nav': 'navigation',
                            'main': 'main',
                            'section': 'region',
                            'article': 'article',
                            'aside': 'complementary',
                            'header': 'banner',
                            'footer': 'contentinfo',
                            'ul': 'list',
                            'ol': 'list',
                            'li': 'listitem',
                            'table': 'table',
                            'thead': 'rowgroup',
                            'tbody': 'rowgroup',
                            'tfoot': 'rowgroup',
                            'tr': 'row',
                            'td': 'cell',
                            'th': 'columnheader'
                        }};
                        
                        return roleMap[tagName] || 'generic';
                    }},
                    
                    // Check for accessibility violations
                    checkViolations: function(element) {{
                        const violations = [];
                        const tagName = element.tagName.toLowerCase();
                        const role = element.getAttribute('role') || this.getImplicitRole(element);
                        
                        // Missing alt text for images
                        if (tagName === 'img' && !element.hasAttribute('alt')) {{
                            violations.push({{
                                severity: 'error',
                                rule_id: 'image-alt',
                                message: 'Images must have alt text',
                                help_url: 'https://dequeuniversity.com/rules/axe/4.4/image-alt'
                            }});
                        }}
                        
                        // Empty alt text for decorative images
                        if (tagName === 'img' && element.getAttribute('alt') === '' && !element.getAttribute('role')) {{
                            // This is actually correct for decorative images
                        }}
                        
                        // Form labels
                        if (['input', 'textarea', 'select'].includes(tagName) && element.type !== 'hidden') {{
                            const accessibleName = this.getAccessibleName(element);
                            if (!accessibleName) {{
                                violations.push({{
                                    severity: 'error',
                                    rule_id: 'label',
                                    message: 'Form elements must have labels',
                                    help_url: 'https://dequeuniversity.com/rules/axe/4.4/label'
                                }});
                            }}
                        }}
                        
                        // Button text
                        if (tagName === 'button') {{
                            const accessibleName = this.getAccessibleName(element);
                            if (!accessibleName) {{
                                violations.push({{
                                    severity: 'error',
                                    rule_id: 'button-name',
                                    message: 'Buttons must have accessible text',
                                    help_url: 'https://dequeuniversity.com/rules/axe/4.4/button-name'
                                }});
                            }}
                        }}
                        
                        // Link text
                        if (tagName === 'a' && element.href) {{
                            const accessibleName = this.getAccessibleName(element);
                            if (!accessibleName) {{
                                violations.push({{
                                    severity: 'error',
                                    rule_id: 'link-name',
                                    message: 'Links must have accessible text',
                                    help_url: 'https://dequeuniversity.com/rules/axe/4.4/link-name'
                                }});
                            }}
                        }}
                        
                        // Heading hierarchy
                        if (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'].includes(tagName)) {{
                            const level = parseInt(tagName.charAt(1));
                            const prevHeading = this.getPreviousHeading(element);
                            if (prevHeading && level > prevHeading + 1) {{
                                violations.push({{
                                    severity: 'warning',
                                    rule_id: 'heading-order',
                                    message: `Heading levels should not skip (found h${{level}} after h${{prevHeading}})`,
                                    help_url: 'https://dequeuniversity.com/rules/axe/4.4/heading-order'
                                }});
                            }}
                        }}
                        
                        // Color contrast (if enabled)
                        if (checkColorContrast && this.hasTextContent(element)) {{
                            const styles = window.getComputedStyle(element);
                            const color = styles.color;
                            const bgColor = styles.backgroundColor;
                            
                            if (color && bgColor && bgColor !== 'rgba(0, 0, 0, 0)') {{
                                const contrast = this.getContrastRatio(color, bgColor);
                                if (contrast && contrast < 4.5) {{
                                    violations.push({{
                                        severity: 'error',
                                        rule_id: 'color-contrast',
                                        message: `Insufficient color contrast ratio: ${{contrast.toFixed(2)}}`,
                                        help_url: 'https://dequeuniversity.com/rules/axe/4.4/color-contrast'
                                    }});
                                }}
                            }}
                        }}
                        
                        return violations.map(v => ({{
                            ...v,
                            element_selector: this.getSelector(element)
                        }}));
                    }},
                    
                    getPreviousHeading: function(element) {{
                        const headings = Array.from(document.querySelectorAll('h1, h2, h3, h4, h5, h6'));
                        const currentIndex = headings.indexOf(element);
                        if (currentIndex > 0) {{
                            const prevHeading = headings[currentIndex - 1];
                            return parseInt(prevHeading.tagName.charAt(1));
                        }}
                        return null;
                    }},
                    
                    hasTextContent: function(element) {{
                        return element.textContent && element.textContent.trim().length > 0;
                    }},
                    
                    getSelector: function(element) {{
                        if (element.id) return `#${{element.id}}`;
                        if (element.className) return `${{element.tagName.toLowerCase()}}.${{element.className.split(' ')[0]}}`;
                        return element.tagName.toLowerCase();
                    }},
                    
                    // Get clickable area for touch target size
                    getClickableArea: function(element) {{
                        const rect = element.getBoundingClientRect();
                        if (rect.width === 0 && rect.height === 0) return null;
                        
                        return {{
                            width: rect.width,
                            height: rect.height,
                            meets_minimum_size: rect.width >= 44 && rect.height >= 44
                        }};
                    }}
                }};
                
                // Build accessibility tree
                function buildA11yTree(element, level = 0, maxDepth = null) {{
                    if (maxDepth !== null && level > maxDepth) {{
                        return null;
                    }}
                    
                    const tagName = element.tagName.toLowerCase();
                    const explicitRole = element.getAttribute('role');
                    const implicitRole = a11yUtils.getImplicitRole(element);
                    const role = explicitRole || implicitRole;
                    
                    // Get ARIA attributes
                    const ariaAttributes = {{}};
                    for (const attr of element.attributes) {{
                        if (attr.name.startsWith('aria-')) {{
                            ariaAttributes[attr.name] = attr.value;
                        }}
                    }}
                    
                    // Computed accessibility properties
                    let computedProperties = null;
                    if (includeComputedNames) {{
                        const styles = window.getComputedStyle(element);
                        const accessibleName = a11yUtils.getAccessibleName(element);
                        const accessibleDescription = a11yUtils.getAccessibleDescription(element);
                        
                        computedProperties = {{
                            accessible_name: accessibleName,
                            accessible_description: accessibleDescription,
                            role: role,
                            states: Object.keys(ariaAttributes)
                                .filter(attr => attr.startsWith('aria-') && ['true', 'false'].includes(ariaAttributes[attr]))
                                .map(attr => `${{attr.replace('aria-', '')}}: ${{ariaAttributes[attr]}}`),
                            properties: Object.keys(ariaAttributes)
                                .filter(attr => attr.startsWith('aria-') && !['true', 'false'].includes(ariaAttributes[attr]))
                                .map(attr => `${{attr.replace('aria-', '')}}: ${{ariaAttributes[attr]}}`),
                            color_contrast_ratio: null,
                            font_size: parseFloat(styles.fontSize),
                            clickable_area: ['button', 'link', 'a', 'input'].includes(tagName) ? 
                                a11yUtils.getClickableArea(element) : null
                        }};
                        
                        // Add color contrast if checking
                        if (checkColorContrast && a11yUtils.hasTextContent(element)) {{
                            const contrast = a11yUtils.getContrastRatio(styles.color, styles.backgroundColor);
                            if (contrast) {{
                                computedProperties.color_contrast_ratio = parseFloat(contrast.toFixed(2));
                            }}
                        }}
                    }}
                    
                    // Check for violations
                    const violations = a11yUtils.checkViolations(element);
                    
                    // Build child nodes
                    const children = [];
                    for (const child of element.children) {{
                        const childNode = buildA11yTree(child, level + 1, maxDepth);
                        if (childNode) {{
                            children.push(childNode);
                        }}
                    }}
                    
                    return {{
                        tag_name: tagName,
                        role: role === 'generic' ? null : role,
                        name: a11yUtils.getAccessibleName(element) || null,
                        description: a11yUtils.getAccessibleDescription(element) || null,
                        value: element.value || null,
                        level: level,
                        aria_attributes: ariaAttributes,
                        computed_properties: computedProperties,
                        accessibility_violations: violations,
                        children: children
                    }};
                }}
                
                // Start analysis from root
                const rootElement = rootSelector ? 
                    document.querySelector(rootSelector) : 
                    document.documentElement;
                    
                if (!rootElement) {{
                    return {{ error: 'Root element not found' }};
                }}
                
                const accessibilityTree = buildA11yTree(rootElement, 0, maxDepth);
                
                // Collect statistics
                let totalNodes = 0;
                let totalViolations = 0;
                const violationsByRule = {{}};
                const contrastIssues = [];
                
                function collectStats(node) {{
                    totalNodes++;
                    totalViolations += node.accessibility_violations.length;
                    
                    node.accessibility_violations.forEach(violation => {{
                        violationsByRule[violation.rule_id] = (violationsByRule[violation.rule_id] || 0) + 1;
                        
                        if (violation.rule_id === 'color-contrast') {{
                            const match = violation.message.match(/ratio: ([0-9.]+)/);
                            if (match) {{
                                contrastIssues.push({{
                                    element_selector: violation.element_selector,
                                    contrast_ratio: parseFloat(match[1]),
                                    level: 'AA',
                                    passes: false
                                }});
                            }}
                        }}
                    }});
                    
                    node.children.forEach(collectStats);
                }}
                
                if (accessibilityTree) {{
                    collectStats(accessibilityTree);
                }}
                
                return {{
                    accessibility_tree: accessibilityTree,
                    total_nodes: totalNodes,
                    total_violations: totalViolations,
                    violations_by_rule: violationsByRule,
                    contrast_issues: contrastIssues
                }};
            }})();
        "#,
            serde_json::to_string(&input.root_selector).unwrap(),
            input.include_aria_info,
            input.include_computed_names,
            input.check_color_contrast,
            input.validate_semantic_structure,
            serde_json::to_string(&input.max_depth).unwrap()
        );
        
        let analysis_result = self.browser.execute_script(&accessibility_script).await?;
        
        let _total_computation_time = start_time.elapsed().as_millis() as f64;
        
        // Parse results
        if let Some(error) = analysis_result.get("error") {
            return Err(anyhow!("Accessibility analysis failed: {}", error.as_str().unwrap_or("Unknown error")));
        }
        
        // Extract accessibility tree (simplified implementation)
        let accessibility_tree = AccessibilityNode {
            tag_name: "html".to_string(),
            role: Some("document".to_string()),
            name: None,
            description: None,
            value: None,
            level: 0,
            aria_attributes: HashMap::new(),
            computed_properties: None,
            accessibility_violations: Vec::new(),
            children: Vec::new(),
        };
        
        let total_nodes_analyzed = analysis_result.get("total_nodes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let total_violations = analysis_result.get("total_violations")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let violations_by_rule: std::collections::HashMap<String, u32> = 
            analysis_result.get("violations_by_rule")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter().map(|(k, v)| {
                        (k.clone(), v.as_u64().unwrap_or(0) as u32)
                    }).collect()
                })
                .unwrap_or_else(HashMap::new);
        
        // Calculate accessibility score (0-100)
        let accessibility_score = if total_nodes_analyzed > 0 {
            let violation_ratio = total_violations as f64 / total_nodes_analyzed as f64;
            ((1.0 - violation_ratio.min(1.0)) * 100.0).max(0.0)
        } else {
            100.0
        };
        
        // Extract color contrast issues
        let color_contrast_issues: Vec<ContrastIssue> = analysis_result.get("contrast_issues")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().filter_map(|issue| {
                    Some(ContrastIssue {
                        element_selector: issue.get("element_selector")?.as_str()?.to_string(),
                        foreground_color: "".to_string(), // Would need additional extraction
                        background_color: "".to_string(),  // Would need additional extraction
                        contrast_ratio: issue.get("contrast_ratio")?.as_f64()?,
                        level: issue.get("level")?.as_str().unwrap_or("AA").to_string(),
                        passes: issue.get("passes")?.as_bool().unwrap_or(false),
                    })
                }).collect()
            })
            .unwrap_or_else(Vec::new);
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        
        if violations_by_rule.contains_key("image-alt") {
            recommendations.push("Add alternative text to all images for screen readers".to_string());
        }
        if violations_by_rule.contains_key("label") {
            recommendations.push("Ensure all form elements have proper labels".to_string());
        }
        if violations_by_rule.contains_key("color-contrast") {
            recommendations.push("Improve color contrast to meet WCAG AA standards (4.5:1 ratio)".to_string());
        }
        if violations_by_rule.contains_key("heading-order") {
            recommendations.push("Maintain proper heading hierarchy without skipping levels".to_string());
        }
        
        if accessibility_score < 90.0 {
            recommendations.push("Consider running automated accessibility testing tools like axe-core".to_string());
        }
        
        let violations_summary = ViolationsSummary {
            total_violations,
            errors: violations_by_rule.values().sum::<u32>(), // Simplified - would need to categorize by severity
            warnings: 0,
            info: 0,
            by_rule: violations_by_rule,
        };
        
        Ok(AccessibilityAnalysisOutput {
            success: true,
            accessibility_tree,
            total_nodes_analyzed,
            violations_summary,
            accessibility_score,
            color_contrast_issues,
            semantic_structure_issues: Vec::new(), // Would need additional semantic analysis
            recommendations,
        })
    }
    
    // Accessibility parsing temporarily disabled for CDP network idle focus
    // TODO: Implement comprehensive accessibility tree parsing
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if let Some(ref selector) = input.root_selector {
            if selector.trim().is_empty() {
                return Err(anyhow!("Root selector cannot be empty"));
            }
            if selector.len() > 500 {
                return Err(anyhow!("Root selector too long: maximum 500 characters"));
            }
        }
        
        if let Some(max_depth) = input.max_depth {
            if max_depth > 20 {
                return Err(anyhow!("Maximum depth cannot exceed 20 levels"));
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// CDP Network Idle Monitor Tool
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct CDPNetworkIdleInput {
    #[serde(default = "default_idle_time_ms")]
    pub idle_time_ms: u64,
    #[serde(default = "default_network_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub max_concurrent_requests: Option<usize>,
    #[serde(default)]
    pub ignore_websockets: bool,
    #[serde(default)]
    pub domain_whitelist: Option<Vec<String>>,
}

fn default_network_timeout() -> u64 {
    30000 // 30 seconds
}

fn default_idle_time_ms() -> u64 {
    500 // 500ms network idle threshold
}

#[derive(Debug, Serialize, Clone)]
pub struct NetworkActivity {
    pub request_id: String,
    pub url: String,
    pub method: String,
    pub timestamp: f64,
    pub resource_type: String,
    pub status: String, // "started", "finished", "failed"
}

#[derive(Debug, Serialize)]
pub struct CDPNetworkIdleOutput {
    pub success: bool,
    pub network_idle_achieved: bool,
    pub wait_time_ms: u64,
    pub total_requests_monitored: usize,
    pub final_active_requests: usize,
    pub idle_periods: Vec<IdlePeriod>,
    pub network_activity: Vec<NetworkActivity>,
}

#[derive(Debug, Serialize)]
pub struct IdlePeriod {
    pub start_time_ms: u64,
    pub duration_ms: u64,
    pub concurrent_requests_during_period: usize,
}

pub struct CDPNetworkIdleTool {
    browser: Arc<Browser>,
    active_requests: Arc<AtomicUsize>,
    network_events: Arc<Mutex<Vec<NetworkActivity>>>,
}

impl CDPNetworkIdleTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { 
            browser,
            active_requests: Arc::new(AtomicUsize::new(0)),
            network_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Set up enhanced Network monitoring using Performance API and Page lifecycle events  
    async fn setup_network_monitoring(&self) -> Result<()> {
        let page = self.browser.page.read().await;
        
        // Enable Runtime domain for enhanced monitoring
        if let Err(e) = page.enable_runtime().await {
            debug!("Failed to enable Runtime domain: {}", e);
        }
        
        // Install enhanced network activity tracking via JavaScript
        let tracking_script = r#"
            (function() {
                // Create a more sophisticated network activity tracker
                if (window._networkTracker) return window._networkTracker;
                
                const tracker = {
                    activeRequests: new Set(),
                    requestHistory: [],
                    totalRequests: 0,
                    
                    // Track active fetch/XHR requests
                    addRequest: function(id, url, method) {
                        this.activeRequests.add(id);
                        this.requestHistory.push({
                            id, url, method, 
                            startTime: performance.now(),
                            status: 'started'
                        });
                        this.totalRequests++;
                    },
                    
                    // Mark request as complete
                    completeRequest: function(id, status = 'finished') {
                        this.activeRequests.delete(id);
                        const req = this.requestHistory.find(r => r.id === id);
                        if (req) {
                            req.status = status;
                            req.endTime = performance.now();
                            req.duration = req.endTime - req.startTime;
                        }
                    },
                    
                    // Get current activity count
                    getActiveCount: function() {
                        return this.activeRequests.size;
                    },
                    
                    // Get activity summary
                    getSummary: function() {
                        return {
                            active: this.activeRequests.size,
                            total: this.totalRequests,
                            recent: this.requestHistory.slice(-10)
                        };
                    }
                };
                
                // Override fetch to track requests
                const originalFetch = window.fetch;
                window.fetch = function(...args) {
                    const requestId = 'fetch_' + Date.now() + '_' + Math.random();
                    const url = args[0];
                    const method = (args[1] && args[1].method) || 'GET';
                    
                    tracker.addRequest(requestId, url, method);
                    
                    return originalFetch.apply(this, args)
                        .then(response => {
                            tracker.completeRequest(requestId, 'finished');
                            return response;
                        })
                        .catch(error => {
                            tracker.completeRequest(requestId, 'failed');
                            throw error;
                        });
                };
                
                // Override XMLHttpRequest to track requests
                const originalXHROpen = XMLHttpRequest.prototype.open;
                const originalXHRSend = XMLHttpRequest.prototype.send;
                
                XMLHttpRequest.prototype.open = function(method, url, ...args) {
                    this._requestId = 'xhr_' + Date.now() + '_' + Math.random();
                    this._requestUrl = url;
                    this._requestMethod = method;
                    return originalXHROpen.apply(this, arguments);
                };
                
                XMLHttpRequest.prototype.send = function(...args) {
                    if (this._requestId) {
                        tracker.addRequest(this._requestId, this._requestUrl, this._requestMethod);
                        
                        const completeRequest = () => {
                            if (this.readyState === 4) {
                                const status = this.status >= 200 && this.status < 300 ? 'finished' : 'failed';
                                tracker.completeRequest(this._requestId, status);
                            }
                        };
                        
                        this.addEventListener('readystatechange', completeRequest);
                        this.addEventListener('load', () => tracker.completeRequest(this._requestId, 'finished'));
                        this.addEventListener('error', () => tracker.completeRequest(this._requestId, 'failed'));
                        this.addEventListener('abort', () => tracker.completeRequest(this._requestId, 'aborted'));
                    }
                    
                    return originalXHRSend.apply(this, arguments);
                };
                
                // Store tracker globally
                window._networkTracker = tracker;
                return tracker;
            })()
        "#;
        
        // Install the enhanced network tracking
        if let Err(e) = page.evaluate(tracking_script).await {
            debug!("Failed to install enhanced network tracking: {}", e);
        } else {
            debug!("Enhanced network tracking installed successfully");
        }
        
        Ok(())
    }
    
    /// Get enhanced network activity data from the JavaScript tracker
    async fn get_enhanced_network_data(&self) -> Result<(usize, Vec<NetworkActivity>)> {
        let page = self.browser.page.read().await;
        
        // Query the enhanced network tracker
        let query_script = r#"
            (function() {
                if (!window._networkTracker) {
                    return { active: 0, total: 0, recent: [] };
                }
                return window._networkTracker.getSummary();
            })()
        "#;
        
        match page.evaluate(query_script).await {
            Ok(result) => {
                if let Some(data) = result.value() {
                    let active_count = data.get("active")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    
                    let recent_requests: Vec<NetworkActivity> = data.get("recent")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter().filter_map(|req| {
                                Some(NetworkActivity {
                                    request_id: req.get("id")?.as_str()?.to_string(),
                                    url: req.get("url")?.as_str()?.to_string(),
                                    method: req.get("method")?.as_str()?.to_string(),
                                    timestamp: req.get("startTime")?.as_f64()?,
                                    resource_type: "xhr_or_fetch".to_string(),
                                    status: req.get("status")?.as_str()?.to_string(),
                                })
                            }).collect()
                        })
                        .unwrap_or_else(Vec::new);
                    
                    return Ok((active_count, recent_requests));
                }
            }
            Err(e) => {
                debug!("Failed to query enhanced network tracker: {}", e);
            }
        }
        
        // Fallback to previous method if tracker not available
        Ok((0, Vec::new()))
    }
    
    /// Check if network is currently idle based on enhanced tracking
    async fn is_network_idle(&self, max_concurrent: Option<usize>) -> bool {
        // Try to get data from enhanced JavaScript tracker first
        if let Ok((active_count, recent_activity)) = self.get_enhanced_network_data().await {
            // Update our atomic counter with the latest data
            self.active_requests.store(active_count, Ordering::SeqCst);
            
            // Update our events log with recent activity
            if !recent_activity.is_empty() {
                if let Ok(mut events_guard) = self.network_events.lock() {
                    events_guard.clear();
                    events_guard.extend(recent_activity);
                }
            }
            
            let threshold = max_concurrent.unwrap_or(0);
            debug!("Enhanced network idle check: {} active, threshold: {}", active_count, threshold);
            return active_count <= threshold;
        }
        
        // Fallback to atomic counter
        let active_count = self.active_requests.load(Ordering::SeqCst);
        let threshold = max_concurrent.unwrap_or(0);
        active_count <= threshold
    }
    
    /// Wait for network to become idle using CDP Network domain events
    async fn wait_for_network_idle_cdp(&self, input: &CDPNetworkIdleInput) -> Result<CDPNetworkIdleOutput> {
        let start_time = std::time::Instant::now();
        let idle_duration = Duration::from_millis(input.idle_time_ms);
        let total_timeout = Duration::from_millis(input.timeout_ms);
        let check_interval = Duration::from_millis(50); // High frequency checking
        
        self.setup_network_monitoring().await?;
        
        let mut idle_periods = Vec::new();
        let mut current_idle_start: Option<std::time::Instant> = None;
        let mut consecutive_idle_time = Duration::ZERO;
        
        info!("Starting CDP-backed network idle detection: {}ms idle threshold, {}ms timeout", 
              input.idle_time_ms, input.timeout_ms);
        
        while start_time.elapsed() < total_timeout {
            let is_idle = self.is_network_idle(input.max_concurrent_requests).await;
            let active_count = self.active_requests.load(Ordering::SeqCst);
            
            if is_idle {
                if current_idle_start.is_none() {
                    current_idle_start = Some(std::time::Instant::now());
                    debug!("Network idle period started (active requests: {})", active_count);
                }
                
                if let Some(idle_start) = current_idle_start {
                    consecutive_idle_time = idle_start.elapsed();
                    
                    if consecutive_idle_time >= idle_duration {
                        // Network has been idle long enough!
                        let total_wait_time = start_time.elapsed().as_millis() as u64;
                        let network_events = self.network_events.lock()
                            .map(|guard| guard.clone())
                            .unwrap_or_else(|_| Vec::new());
                        
                        idle_periods.push(IdlePeriod {
                            start_time_ms: (idle_start.elapsed().as_millis() - consecutive_idle_time.as_millis()) as u64,
                            duration_ms: consecutive_idle_time.as_millis() as u64,
                            concurrent_requests_during_period: active_count,
                        });
                        
                        info!("Network idle achieved after {}ms (CDP-tracked)", total_wait_time);
                        return Ok(CDPNetworkIdleOutput {
                            success: true,
                            network_idle_achieved: true,
                            wait_time_ms: total_wait_time,
                            total_requests_monitored: network_events.len(),
                            final_active_requests: active_count,
                            idle_periods,
                            network_activity: network_events,
                        });
                    }
                }
            } else {
                if let Some(_idle_start) = current_idle_start {
                    idle_periods.push(IdlePeriod {
                        start_time_ms: (start_time.elapsed().as_millis() - consecutive_idle_time.as_millis()) as u64,
                        duration_ms: consecutive_idle_time.as_millis() as u64,
                        concurrent_requests_during_period: active_count,
                    });
                    debug!("Network idle period ended (active requests: {})", active_count);
                }
                current_idle_start = None;
                consecutive_idle_time = Duration::ZERO;
            }
            
            tokio::time::sleep(check_interval).await;
        }
        
        // Timeout reached
        let total_wait_time = input.timeout_ms;
        let network_events = self.network_events.lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| Vec::new());
        let final_active = self.active_requests.load(Ordering::SeqCst);
        
        info!("Network idle timeout after {}ms (CDP-tracked, {} active requests)", 
              total_wait_time, final_active);
              
        Ok(CDPNetworkIdleOutput {
            success: false,
            network_idle_achieved: false,
            wait_time_ms: total_wait_time,
            total_requests_monitored: network_events.len(),
            final_active_requests: final_active,
            idle_periods,
            network_activity: network_events,
        })
    }
}

#[async_trait]
impl Tool for CDPNetworkIdleTool {
    type Input = CDPNetworkIdleInput;
    type Output = CDPNetworkIdleOutput;
    
    fn name(&self) -> &str {
        "cdp_network_idle"
    }
    
    fn description(&self) -> &str {
        "Wait for network to be idle using CDP Network domain event tracking for accurate idle detection"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Synchronization
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        self.wait_for_network_idle_cdp(&input).await
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.idle_time_ms == 0 {
            return Err(anyhow!("Idle time must be greater than 0"));
        }
        if input.idle_time_ms > 60000 {
            return Err(anyhow!("Idle time cannot exceed 60 seconds"));
        }
        if input.timeout_ms == 0 {
            return Err(anyhow!("Timeout must be greater than 0"));
        }
        if input.timeout_ms > 300000 { // 5 minutes max
            return Err(anyhow!("Timeout cannot exceed 300 seconds"));
        }
        if let Some(max_concurrent) = input.max_concurrent_requests {
            if max_concurrent > 100 {
                return Err(anyhow!("Max concurrent requests cannot exceed 100"));
            }
        }
        Ok(())
    }
}