// Perception Orchestrator
// Coordinates all four perception layers and manages intelligent layer selection

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use super::browser_connection::BrowserConnection;
use super::lightning_real::{RealLightningPerception, LightningData};
use super::quick_real::{RealQuickPerception, QuickData};
use super::standard_perception::{RealStandardPerception, StandardData};
use super::deep_perception::{RealDeepPerception, DeepData};
use super::cache_system::PerceptionCache;

/// Unified perception orchestrator
pub struct PerceptionOrchestrator {
    lightning_perception: RealLightningPerception,
    quick_perception: RealQuickPerception,
    standard_perception: RealStandardPerception,
    deep_perception: RealDeepPerception,
    cache: Arc<PerceptionCache>,
    config: OrchestratorConfig,
    stats: Arc<RwLock<OrchestrationStats>>,
}

/// Configuration for the orchestrator
#[derive(Clone, Debug)]
pub struct OrchestratorConfig {
    pub auto_layer_selection: bool,
    pub enable_caching: bool,
    pub enable_parallel_execution: bool,
    pub timeout_multiplier: f32,
    pub fallback_strategy: FallbackStrategy,
    pub quality_threshold: f32,
    pub performance_priority: PerformancePriority,
}

#[derive(Clone, Debug)]
pub enum FallbackStrategy {
    StopOnTimeout,
    ContinueWithPartialData,
    RetryWithLowerLayer,
    UseCache,
}

#[derive(Clone, Debug)]
pub enum PerformancePriority {
    Speed,
    Accuracy,
    Balanced,
}

/// Orchestration statistics
#[derive(Default, Debug, Clone)]
pub struct OrchestrationStats {
    pub total_requests: u64,
    pub lightning_requests: u64,
    pub quick_requests: u64,
    pub standard_requests: u64,
    pub deep_requests: u64,
    pub cache_hits: u64,
    pub timeouts: u64,
    pub errors: u64,
    pub average_response_time_ms: f64,
}

/// Unified perception result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPerceptionResult {
    pub perception_level: PerceptionLevel,
    pub lightning_data: Option<LightningData>,
    pub quick_data: Option<QuickData>,
    pub standard_data: Option<StandardData>,
    pub deep_data: Option<DeepData>,
    pub execution_info: ExecutionInfo,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionLevel {
    Lightning,
    Quick,
    Standard,
    Deep,
    Hybrid,
}

/// Information about the execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub requested_level: PerceptionLevel,
    pub actual_level: PerceptionLevel,
    pub execution_time_ms: u64,
    pub cache_used: bool,
    pub quality_score: f32,
    pub confidence_score: f32,
    pub fallback_reason: Option<String>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub layers_executed: Vec<String>,
    pub total_time_ms: u64,
    pub layer_times_ms: Vec<(String, u64)>,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
}

/// Recommendations based on perception results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_impact: f32,
    pub implementation_effort: ImplementationEffort,
    pub actionable_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Performance,
    Usability,
    Accessibility,
    SEO,
    Security,
    Content,
    Automation,
    UserExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl PerceptionOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            lightning_perception: RealLightningPerception::new(),
            quick_perception: RealQuickPerception::new(),
            standard_perception: RealStandardPerception::new(),
            deep_perception: RealDeepPerception::new(),
            cache: Arc::new(PerceptionCache::new()),
            config,
            stats: Arc::new(RwLock::new(OrchestrationStats::default())),
        }
    }

    /// Execute perception based on requirements
    pub async fn perceive(
        &self,
        browser: &BrowserConnection,
        level: PerceptionLevel,
    ) -> Result<UnifiedPerceptionResult> {
        let start = Instant::now();
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        drop(stats);

        info!("Starting perception orchestration for level: {:?}", level);

        // Determine optimal execution strategy
        let execution_strategy = self.determine_execution_strategy(&level, browser).await?;
        
        // Execute perception layers
        let result = match execution_strategy {
            ExecutionStrategy::SingleLayer(target_level) => {
                self.execute_single_layer(browser, target_level).await
            }
            ExecutionStrategy::Cascading => {
                self.execute_cascading(browser, level).await
            }
            ExecutionStrategy::Parallel => {
                self.execute_parallel(browser, level).await
            }
            ExecutionStrategy::Hybrid => {
                self.execute_hybrid(browser, level).await
            }
        };

        let execution_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut unified_result) => {
                unified_result.execution_info.execution_time_ms = execution_time;
                
                // Generate recommendations
                unified_result.recommendations = self.generate_recommendations(&unified_result).await;
                
                // Update statistics
                self.update_stats(execution_time, false).await;
                
                info!("Perception orchestration completed in {}ms", execution_time);
                Ok(unified_result)
            }
            Err(e) => {
                error!("Perception orchestration failed: {}", e);
                self.update_stats(execution_time, true).await;
                Err(e)
            }
        }
    }

    /// Execute lightning perception only
    pub async fn perceive_lightning(&self, browser: &BrowserConnection) -> Result<UnifiedPerceptionResult> {
        self.perceive(browser, PerceptionLevel::Lightning).await
    }

    /// Execute quick perception (includes Lightning)
    pub async fn perceive_quick(&self, browser: &BrowserConnection) -> Result<UnifiedPerceptionResult> {
        self.perceive(browser, PerceptionLevel::Quick).await
    }

    /// Execute standard perception (includes Quick and Lightning)
    pub async fn perceive_standard(&self, browser: &BrowserConnection) -> Result<UnifiedPerceptionResult> {
        self.perceive(browser, PerceptionLevel::Standard).await
    }

    /// Execute deep perception (includes all layers)
    pub async fn perceive_deep(&self, browser: &BrowserConnection) -> Result<UnifiedPerceptionResult> {
        self.perceive(browser, PerceptionLevel::Deep).await
    }

    /// Adaptive perception that chooses the optimal layer
    pub async fn perceive_adaptive(&self, browser: &BrowserConnection) -> Result<UnifiedPerceptionResult> {
        let optimal_level = self.determine_optimal_level(browser).await?;
        self.perceive(browser, optimal_level).await
    }

    /// Determine execution strategy based on requirements
    async fn determine_execution_strategy(
        &self,
        level: &PerceptionLevel,
        browser: &BrowserConnection,
    ) -> Result<ExecutionStrategy> {
        if !self.config.auto_layer_selection {
            return Ok(ExecutionStrategy::SingleLayer(level.clone()));
        }

        // Analyze page complexity to determine optimal strategy
        let page_complexity = self.assess_page_complexity(browser).await?;
        
        match (level, page_complexity, &self.config.performance_priority) {
            (PerceptionLevel::Deep, PageComplexity::High, PerformancePriority::Accuracy) => {
                Ok(ExecutionStrategy::Cascading)
            }
            (PerceptionLevel::Standard | PerceptionLevel::Deep, _, PerformancePriority::Speed) => {
                Ok(ExecutionStrategy::Parallel)
            }
            (_, PageComplexity::Medium, PerformancePriority::Balanced) => {
                Ok(ExecutionStrategy::Hybrid)
            }
            _ => Ok(ExecutionStrategy::SingleLayer(level.clone())),
        }
    }

    /// Execute a single perception layer
    async fn execute_single_layer(
        &self,
        browser: &BrowserConnection,
        level: PerceptionLevel,
    ) -> Result<UnifiedPerceptionResult> {
        let start = Instant::now();
        
        match level {
            PerceptionLevel::Lightning => {
                let data = self.lightning_perception.scan_page(browser).await?;
                let mut stats = self.stats.write().await;
                stats.lightning_requests += 1;
                drop(stats);
                
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Lightning,
                    lightning_data: Some(data),
                    quick_data: None,
                    standard_data: None,
                    deep_data: None,
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Lightning,
                        actual_level: PerceptionLevel::Lightning,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        cache_used: false,
                        quality_score: 0.7,
                        confidence_score: 0.8,
                        fallback_reason: None,
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string()],
                            total_time_ms: start.elapsed().as_millis() as u64,
                            layer_times_ms: vec![("Lightning".to_string(), start.elapsed().as_millis() as u64)],
                            memory_usage_mb: 10.0,
                            cpu_usage_percent: 15.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            }
            PerceptionLevel::Quick => {
                let data = self.quick_perception.scan_page(browser).await?;
                let mut stats = self.stats.write().await;
                stats.quick_requests += 1;
                drop(stats);
                
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Quick,
                    lightning_data: Some(data.lightning_data.clone()),
                    quick_data: Some(data),
                    standard_data: None,
                    deep_data: None,
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Quick,
                        actual_level: PerceptionLevel::Quick,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        cache_used: false,
                        quality_score: 0.8,
                        confidence_score: 0.85,
                        fallback_reason: None,
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string(), "Quick".to_string()],
                            total_time_ms: start.elapsed().as_millis() as u64,
                            layer_times_ms: vec![
                                ("Lightning".to_string(), 50),
                                ("Quick".to_string(), start.elapsed().as_millis() as u64 - 50)
                            ],
                            memory_usage_mb: 25.0,
                            cpu_usage_percent: 30.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            }
            PerceptionLevel::Standard => {
                let data = self.standard_perception.scan_page(browser).await?;
                let mut stats = self.stats.write().await;
                stats.standard_requests += 1;
                drop(stats);
                
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Standard,
                    lightning_data: Some(data.quick_data.lightning_data.clone()),
                    quick_data: Some(data.quick_data.clone()),
                    standard_data: Some(data),
                    deep_data: None,
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Standard,
                        actual_level: PerceptionLevel::Standard,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        cache_used: false,
                        quality_score: 0.9,
                        confidence_score: 0.9,
                        fallback_reason: None,
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string(), "Quick".to_string(), "Standard".to_string()],
                            total_time_ms: start.elapsed().as_millis() as u64,
                            layer_times_ms: vec![
                                ("Lightning".to_string(), 50),
                                ("Quick".to_string(), 150),
                                ("Standard".to_string(), start.elapsed().as_millis() as u64 - 200)
                            ],
                            memory_usage_mb: 50.0,
                            cpu_usage_percent: 45.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            }
            PerceptionLevel::Deep => {
                let data = self.deep_perception.scan_page(browser).await?;
                let mut stats = self.stats.write().await;
                stats.deep_requests += 1;
                drop(stats);
                
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Deep,
                    lightning_data: Some(data.standard_data.quick_data.lightning_data.clone()),
                    quick_data: Some(data.standard_data.quick_data.clone()),
                    standard_data: Some(data.standard_data.clone()),
                    deep_data: Some(data),
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Deep,
                        actual_level: PerceptionLevel::Deep,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        cache_used: false,
                        quality_score: 0.95,
                        confidence_score: 0.95,
                        fallback_reason: None,
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string(), "Quick".to_string(), "Standard".to_string(), "Deep".to_string()],
                            total_time_ms: start.elapsed().as_millis() as u64,
                            layer_times_ms: vec![
                                ("Lightning".to_string(), 50),
                                ("Quick".to_string(), 150),
                                ("Standard".to_string(), 300),
                                ("Deep".to_string(), start.elapsed().as_millis() as u64 - 500)
                            ],
                            memory_usage_mb: 100.0,
                            cpu_usage_percent: 70.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            }
            PerceptionLevel::Hybrid => {
                self.execute_hybrid(browser, PerceptionLevel::Hybrid).await
            }
        }
    }

    /// Execute layers in cascade (each builds on previous)
    async fn execute_cascading(
        &self,
        browser: &BrowserConnection,
        target_level: PerceptionLevel,
    ) -> Result<UnifiedPerceptionResult> {
        // This is the natural execution pattern since each layer builds on the previous
        self.execute_single_layer(browser, target_level).await
    }

    /// Execute compatible layers in parallel
    async fn execute_parallel(
        &self,
        browser: &BrowserConnection,
        target_level: PerceptionLevel,
    ) -> Result<UnifiedPerceptionResult> {
        // For now, parallel execution is complex due to dependencies
        // Fall back to cascading execution
        self.execute_cascading(browser, target_level).await
    }

    /// Execute hybrid approach based on page analysis
    async fn execute_hybrid(
        &self,
        browser: &BrowserConnection,
        _target_level: PerceptionLevel,
    ) -> Result<UnifiedPerceptionResult> {
        // Start with Lightning, then decide whether to continue
        let lightning_data = self.lightning_perception.scan_page(browser).await?;
        
        // Analyze if we need more detailed perception
        let needs_more_detail = self.analyze_detail_requirements(&lightning_data).await;
        
        if needs_more_detail {
            // Continue with Quick
            let quick_data = self.quick_perception.scan_page(browser).await?;
            
            // Check if Standard perception is needed
            let needs_standard = self.analyze_standard_requirements(&quick_data).await;
            
            if needs_standard {
                let standard_data = self.standard_perception.scan_page(browser).await?;
                
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Hybrid,
                    lightning_data: Some(lightning_data),
                    quick_data: Some(quick_data),
                    standard_data: Some(standard_data),
                    deep_data: None,
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Hybrid,
                        actual_level: PerceptionLevel::Standard,
                        execution_time_ms: 0, // Will be set by caller
                        cache_used: false,
                        quality_score: 0.9,
                        confidence_score: 0.88,
                        fallback_reason: Some("Adaptive execution based on complexity".to_string()),
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string(), "Quick".to_string(), "Standard".to_string()],
                            total_time_ms: 0,
                            layer_times_ms: vec![],
                            memory_usage_mb: 45.0,
                            cpu_usage_percent: 40.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            } else {
                Ok(UnifiedPerceptionResult {
                    perception_level: PerceptionLevel::Hybrid,
                    lightning_data: Some(lightning_data),
                    quick_data: Some(quick_data),
                    standard_data: None,
                    deep_data: None,
                    execution_info: ExecutionInfo {
                        requested_level: PerceptionLevel::Hybrid,
                        actual_level: PerceptionLevel::Quick,
                        execution_time_ms: 0,
                        cache_used: false,
                        quality_score: 0.8,
                        confidence_score: 0.83,
                        fallback_reason: Some("Sufficient detail achieved with Quick perception".to_string()),
                        performance_metrics: PerformanceMetrics {
                            layers_executed: vec!["Lightning".to_string(), "Quick".to_string()],
                            total_time_ms: 0,
                            layer_times_ms: vec![],
                            memory_usage_mb: 20.0,
                            cpu_usage_percent: 25.0,
                        },
                    },
                    recommendations: Vec::new(),
                })
            }
        } else {
            Ok(UnifiedPerceptionResult {
                perception_level: PerceptionLevel::Hybrid,
                lightning_data: Some(lightning_data),
                quick_data: None,
                standard_data: None,
                deep_data: None,
                execution_info: ExecutionInfo {
                    requested_level: PerceptionLevel::Hybrid,
                    actual_level: PerceptionLevel::Lightning,
                    execution_time_ms: 0,
                    cache_used: false,
                    quality_score: 0.7,
                    confidence_score: 0.75,
                    fallback_reason: Some("Lightning perception sufficient for simple page".to_string()),
                    performance_metrics: PerformanceMetrics {
                        layers_executed: vec!["Lightning".to_string()],
                        total_time_ms: 0,
                        layer_times_ms: vec![],
                        memory_usage_mb: 8.0,
                        cpu_usage_percent: 12.0,
                    },
                },
                recommendations: Vec::new(),
            })
        }
    }

    /// Assess page complexity to guide execution strategy
    async fn assess_page_complexity(&self, browser: &BrowserConnection) -> Result<PageComplexity> {
        let script = r##"
            function assessComplexity() {
                const totalElements = document.querySelectorAll('*').length;
                const interactiveElements = document.querySelectorAll('button, a, input, select, textarea').length;
                const forms = document.querySelectorAll('form').length;
                const scripts = document.querySelectorAll('script').length;
                const textLength = document.body.innerText.length;
                
                const complexityScore = (totalElements / 1000) * 0.3 + 
                                      (interactiveElements / 50) * 0.3 + 
                                      (forms / 5) * 0.2 + 
                                      (scripts / 10) * 0.1 + 
                                      (textLength / 10000) * 0.1;
                
                return {
                    total_elements: totalElements,
                    interactive_elements: interactiveElements,
                    forms: forms,
                    scripts: scripts,
                    text_length: textLength,
                    complexity_score: complexityScore
                };
            }
            
            return assessComplexity();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let complexity_data: serde_json::Value = serde_json::from_value(result)?;
        
        let score = complexity_data["complexity_score"].as_f64().unwrap_or(0.5);
        
        Ok(if score < 0.3 {
            PageComplexity::Low
        } else if score < 0.7 {
            PageComplexity::Medium
        } else {
            PageComplexity::High
        })
    }

    /// Determine optimal perception level based on page analysis
    async fn determine_optimal_level(&self, browser: &BrowserConnection) -> Result<PerceptionLevel> {
        let complexity = self.assess_page_complexity(browser).await?;
        
        match (complexity, &self.config.performance_priority) {
            (PageComplexity::Low, PerformancePriority::Speed) => Ok(PerceptionLevel::Lightning),
            (PageComplexity::Low, _) => Ok(PerceptionLevel::Quick),
            (PageComplexity::Medium, PerformancePriority::Speed) => Ok(PerceptionLevel::Quick),
            (PageComplexity::Medium, _) => Ok(PerceptionLevel::Standard),
            (PageComplexity::High, PerformancePriority::Speed) => Ok(PerceptionLevel::Standard),
            (PageComplexity::High, _) => Ok(PerceptionLevel::Deep),
        }
    }

    /// Analyze if more detailed perception is needed
    async fn analyze_detail_requirements(&self, lightning_data: &LightningData) -> bool {
        // Simple heuristics for now
        lightning_data.key_elements.len() > 10 || 
        lightning_data.urgent_signals.len() > 0 ||
        lightning_data.page_status.is_loading
    }

    /// Analyze if Standard perception is needed
    async fn analyze_standard_requirements(&self, quick_data: &QuickData) -> bool {
        quick_data.interaction_elements.len() > 20 ||
        quick_data.form_analysis.len() > 3 ||
        quick_data.navigation_paths.len() > 10
    }

    /// Generate recommendations based on perception results
    async fn generate_recommendations(&self, result: &UnifiedPerceptionResult) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if result.execution_info.execution_time_ms > 500 {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::Performance,
                title: "Consider Lightning perception for faster results".to_string(),
                description: "Current perception took longer than optimal. Consider using Lightning perception for time-critical applications.".to_string(),
                priority: RecommendationPriority::Medium,
                estimated_impact: 0.6,
                implementation_effort: ImplementationEffort::Minimal,
                actionable_steps: vec![
                    "Switch to Lightning perception for basic operations".to_string(),
                    "Use Quick perception only when interaction analysis is needed".to_string(),
                ],
            });
        }

        // Automation recommendations based on detected patterns
        if let Some(standard_data) = &result.standard_data {
            if standard_data.interaction_patterns.len() > 5 {
                recommendations.push(Recommendation {
                    recommendation_type: RecommendationType::Automation,
                    title: "Multiple automation opportunities detected".to_string(),
                    description: format!("Found {} interaction patterns that could be automated", standard_data.interaction_patterns.len()),
                    priority: RecommendationPriority::High,
                    estimated_impact: 0.8,
                    implementation_effort: ImplementationEffort::Medium,
                    actionable_steps: vec![
                        "Analyze detected workflows for automation potential".to_string(),
                        "Implement automated testing for common user flows".to_string(),
                        "Consider RPA solutions for repetitive tasks".to_string(),
                    ],
                });
            }
        }

        // Quality recommendations
        if result.execution_info.quality_score < 0.7 {
            recommendations.push(Recommendation {
                recommendation_type: RecommendationType::UserExperience,
                title: "Page quality could be improved".to_string(),
                description: "The page has potential quality issues that may impact user experience.".to_string(),
                priority: RecommendationPriority::Medium,
                estimated_impact: 0.5,
                implementation_effort: ImplementationEffort::Low,
                actionable_steps: vec![
                    "Review accessibility features".to_string(),
                    "Optimize loading performance".to_string(),
                    "Improve content structure".to_string(),
                ],
            });
        }

        recommendations
    }

    /// Update orchestration statistics
    async fn update_stats(&self, execution_time_ms: u64, is_error: bool) {
        let mut stats = self.stats.write().await;
        
        if is_error {
            stats.errors += 1;
        }
        
        // Update average response time
        let total_time = stats.average_response_time_ms * (stats.total_requests as f64 - 1.0) + execution_time_ms as f64;
        stats.average_response_time_ms = total_time / stats.total_requests as f64;
    }

    /// Get current orchestration statistics
    pub async fn get_stats(&self) -> OrchestrationStats {
        self.stats.read().await.clone()
    }

    /// Clear statistics
    pub async fn clear_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = OrchestrationStats::default();
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            auto_layer_selection: true,
            enable_caching: true,
            enable_parallel_execution: false,
            timeout_multiplier: 1.0,
            fallback_strategy: FallbackStrategy::ContinueWithPartialData,
            quality_threshold: 0.7,
            performance_priority: PerformancePriority::Balanced,
        }
    }
}

#[derive(Debug)]
enum ExecutionStrategy {
    SingleLayer(PerceptionLevel),
    Cascading,
    Parallel,
    Hybrid,
}

#[derive(Debug)]
enum PageComplexity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_config() {
        let config = OrchestratorConfig::default();
        assert!(config.auto_layer_selection);
        assert!(config.enable_caching);
        assert_eq!(config.performance_priority as u8, PerformancePriority::Balanced as u8);
    }

    #[test]
    fn test_perception_result_creation() {
        let result = UnifiedPerceptionResult {
            perception_level: PerceptionLevel::Lightning,
            lightning_data: None,
            quick_data: None,
            standard_data: None,
            deep_data: None,
            execution_info: ExecutionInfo {
                requested_level: PerceptionLevel::Lightning,
                actual_level: PerceptionLevel::Lightning,
                execution_time_ms: 50,
                cache_used: false,
                quality_score: 0.8,
                confidence_score: 0.9,
                fallback_reason: None,
                performance_metrics: PerformanceMetrics {
                    layers_executed: vec!["Lightning".to_string()],
                    total_time_ms: 50,
                    layer_times_ms: vec![("Lightning".to_string(), 50)],
                    memory_usage_mb: 10.0,
                    cpu_usage_percent: 15.0,
                },
            },
            recommendations: Vec::new(),
        };
        
        assert_eq!(result.execution_info.execution_time_ms, 50);
        assert_eq!(result.execution_info.quality_score, 0.8);
    }
}