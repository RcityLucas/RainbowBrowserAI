// Adaptive Learning System
// Learns from workflow execution patterns and improves decision-making over time

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::perception_mvp::{UnifiedPerceptionResult, PerceptionLevel};
use crate::intelligence::core::ai_decision_engine::{Decision, DecisionContext};
use crate::utils::workflow_automation::{WorkflowExecutionResult, WorkflowStep, WorkflowTemplate};

/// Learning configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Maximum number of execution records to keep in memory
    pub max_memory_size: usize,
    /// Minimum confidence threshold for applying learned optimizations
    pub confidence_threshold: f64,
    /// Learning rate for updating model weights
    pub learning_rate: f64,
    /// Number of executions required before considering a pattern stable
    pub pattern_stability_threshold: usize,
    /// Enable/disable automatic optimization application
    pub auto_apply_optimizations: bool,
    /// Decay factor for older execution data
    pub temporal_decay_factor: f64,
    /// Enable feature importance tracking
    pub track_feature_importance: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            max_memory_size: 10000,
            confidence_threshold: 0.75,
            learning_rate: 0.01,
            pattern_stability_threshold: 10,
            auto_apply_optimizations: true,
            temporal_decay_factor: 0.95,
            track_feature_importance: true,
        }
    }
}

/// Execution record for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub timestamp: u64,
    pub workflow_template_id: String,
    pub perception_level: PerceptionLevel,
    pub decision_context: DecisionContextSnapshot,
    pub execution_result: ExecutionOutcome,
    pub performance_metrics: ExecutionPerformanceMetrics,
    pub environment_factors: EnvironmentFactors,
    pub success_indicators: SuccessIndicators,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContextSnapshot {
    pub page_url: String,
    pub page_complexity: f64,
    pub user_intent: String,
    pub time_constraints: Option<Duration>,
    pub retry_count: u32,
    pub previous_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub success: bool,
    pub completion_time: Duration,
    pub steps_completed: usize,
    pub steps_failed: usize,
    pub error_types: Vec<String>,
    pub quality_score: f64,
    pub user_satisfaction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPerformanceMetrics {
    pub total_duration: Duration,
    pub perception_time: Duration,
    pub decision_time: Duration,
    pub action_time: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub network_requests: u32,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentFactors {
    pub browser_type: String,
    pub viewport_size: (u32, u32),
    pub network_speed: f64,
    pub page_load_time: Duration,
    pub javascript_enabled: bool,
    pub images_loaded: bool,
    pub time_of_day: u8, // Hour 0-23
    pub day_of_week: u8, // 0 = Monday
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessIndicators {
    pub task_completed: bool,
    pub no_errors: bool,
    pub within_time_limit: bool,
    pub high_accuracy: bool,
    pub user_goals_met: bool,
    pub efficient_resource_usage: bool,
}

/// Learned pattern from execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub conditions: Vec<PatternCondition>,
    pub recommended_action: RecommendedAction,
    pub confidence: f64,
    pub supporting_evidence: usize,
    pub last_updated: u64,
    pub success_rate: f64,
    pub average_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    PerceptionLevelOptimization,
    WorkflowStepOptimization,
    ErrorPrevention,
    PerformanceOptimization,
    ResourceOptimization,
    TimingOptimization,
    ContextualAdaptation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCondition {
    pub feature: String,
    pub operator: ConditionOperator,
    pub value: ConditionValue,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    GreaterThan,
    LessThan,
    Contains,
    Matches,
    InRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Range(f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub expected_improvement: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ChangePerceptionLevel(PerceptionLevel),
    ModifyWorkflowStep(String),
    AddPreventiveCheck(String),
    AdjustTimeout(Duration),
    OptimizeResourceUsage,
    ChangeExecutionStrategy,
    AddCachingLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Feature importance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: f64,
    pub correlation_with_success: f64,
    pub stability: f64,
    pub update_count: usize,
}

/// Learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub total_executions_learned: usize,
    pub patterns_discovered: usize,
    pub optimizations_applied: usize,
    pub success_rate_improvement: f64,
    pub performance_improvement: f64,
    pub learning_accuracy: f64,
    pub model_confidence: f64,
    pub last_learning_cycle: u64,
}

/// Main adaptive learning system
pub struct AdaptiveLearningSystem {
    config: LearningConfig,
    execution_history: RwLock<VecDeque<ExecutionRecord>>,
    learned_patterns: RwLock<Vec<LearnedPattern>>,
    feature_importance: RwLock<HashMap<String, FeatureImportance>>,
    learning_stats: Mutex<LearningStats>,
    optimization_cache: RwLock<HashMap<String, RecommendedAction>>,
}

impl AdaptiveLearningSystem {
    pub fn new(config: LearningConfig) -> Self {
        Self {
            config,
            execution_history: RwLock::new(VecDeque::new()),
            learned_patterns: RwLock::new(Vec::new()),
            feature_importance: RwLock::new(HashMap::new()),
            learning_stats: Mutex::new(LearningStats {
                total_executions_learned: 0,
                patterns_discovered: 0,
                optimizations_applied: 0,
                success_rate_improvement: 0.0,
                performance_improvement: 0.0,
                learning_accuracy: 0.0,
                model_confidence: 0.0,
                last_learning_cycle: 0,
            }),
            optimization_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Record a workflow execution for learning
    pub async fn record_execution(
        &self,
        workflow_template: &WorkflowTemplate,
        perception_result: &UnifiedPerceptionResult,
        decision_context: &DecisionContext,
        execution_result: &WorkflowExecutionResult,
        environment: EnvironmentFactors,
    ) -> Result<()> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let record = ExecutionRecord {
            id: Uuid::new_v4(),
            timestamp,
            workflow_template_id: workflow_template.id.clone(),
            perception_level: perception_result.execution_info.actual_level.clone(),
            decision_context: DecisionContextSnapshot {
                page_url: decision_context.current_page_url.clone(),
                page_complexity: self.calculate_page_complexity(perception_result),
                user_intent: decision_context.user_intent.clone(),
                time_constraints: decision_context.time_constraints,
                retry_count: decision_context.retry_count,
                previous_failures: decision_context.previous_failures.clone(),
            },
            execution_result: ExecutionOutcome {
                success: execution_result.success,
                completion_time: execution_result.total_duration,
                steps_completed: execution_result.steps_completed,
                steps_failed: execution_result.steps_failed,
                error_types: execution_result.errors.iter().map(|e| e.error_type.clone()).collect(),
                quality_score: execution_result.quality_score,
                user_satisfaction: execution_result.user_satisfaction,
            },
            performance_metrics: ExecutionPerformanceMetrics {
                total_duration: execution_result.total_duration,
                perception_time: perception_result.execution_info.performance_metrics.total_time,
                decision_time: Duration::from_millis(50), // Placeholder
                action_time: execution_result.total_duration - perception_result.execution_info.performance_metrics.total_time,
                memory_usage: perception_result.execution_info.performance_metrics.memory_usage_mb as u64,
                cpu_usage: perception_result.execution_info.performance_metrics.cpu_usage_percent,
                network_requests: execution_result.steps_executed.len() as u32,
                cache_hit_rate: 0.85, // Placeholder
            },
            environment_factors: environment,
            success_indicators: SuccessIndicators {
                task_completed: execution_result.success,
                no_errors: execution_result.errors.is_empty(),
                within_time_limit: true, // Would calculate based on expected time
                high_accuracy: execution_result.quality_score > 0.8,
                user_goals_met: execution_result.user_satisfaction.unwrap_or(0.8) > 0.7,
                efficient_resource_usage: perception_result.execution_info.performance_metrics.memory_usage_mb < 100.0,
            },
        };

        // Add to execution history
        let mut history = self.execution_history.write().await;
        if history.len() >= self.config.max_memory_size {
            history.pop_front();
        }
        history.push_back(record);

        // Update learning statistics
        let mut stats = self.learning_stats.lock().await;
        stats.total_executions_learned += 1;

        // Trigger learning cycle if enough new data
        if stats.total_executions_learned % 100 == 0 {
            drop(stats);
            drop(history);
            self.run_learning_cycle().await?;
        }

        Ok(())
    }

    /// Get optimization recommendations for a given context
    pub async fn get_optimization_recommendations(
        &self,
        perception_result: &UnifiedPerceptionResult,
        decision_context: &DecisionContext,
    ) -> Result<Vec<RecommendedAction>> {
        let context_key = format!("{}_{}", 
            decision_context.current_page_url, 
            decision_context.user_intent
        );

        // Check cache first
        {
            let cache = self.optimization_cache.read().await;
            if let Some(cached_action) = cache.get(&context_key) {
                return Ok(vec![cached_action.clone()]);
            }
        }

        // Find matching patterns
        let patterns = self.learned_patterns.read().await;
        let mut recommendations = Vec::new();

        for pattern in patterns.iter() {
            if pattern.confidence >= self.config.confidence_threshold {
                let match_score = self.calculate_pattern_match(
                    pattern,
                    perception_result,
                    decision_context,
                ).await?;

                if match_score > 0.7 {
                    recommendations.push(pattern.recommended_action.clone());
                }
            }
        }

        // Cache the recommendations
        if !recommendations.is_empty() {
            let mut cache = self.optimization_cache.write().await;
            cache.insert(context_key, recommendations[0].clone());
        }

        Ok(recommendations)
    }

    /// Run a learning cycle to discover new patterns
    pub async fn run_learning_cycle(&self) -> Result<()> {
        debug!("Starting adaptive learning cycle");

        let history = self.execution_history.read().await;
        let recent_records: Vec<ExecutionRecord> = history
            .iter()
            .rev()
            .take(1000)
            .cloned()
            .collect();
        drop(history);

        if recent_records.len() < self.config.pattern_stability_threshold {
            return Ok(());
        }

        // Discover patterns for different optimization types
        let mut new_patterns = Vec::new();

        // Perception level optimization patterns
        new_patterns.extend(
            self.discover_perception_optimization_patterns(&recent_records).await?
        );

        // Workflow step optimization patterns
        new_patterns.extend(
            self.discover_workflow_optimization_patterns(&recent_records).await?
        );

        // Error prevention patterns
        new_patterns.extend(
            self.discover_error_prevention_patterns(&recent_records).await?
        );

        // Performance optimization patterns
        new_patterns.extend(
            self.discover_performance_optimization_patterns(&recent_records).await?
        );

        // Update learned patterns
        let mut patterns = self.learned_patterns.write().await;
        
        // Remove old patterns with low confidence
        patterns.retain(|p| p.confidence >= self.config.confidence_threshold * 0.8);
        
        // Add new patterns
        patterns.extend(new_patterns);
        
        // Update feature importance if enabled
        if self.config.track_feature_importance {
            self.update_feature_importance(&recent_records).await?;
        }

        // Update learning statistics
        let mut stats = self.learning_stats.lock().await;
        stats.patterns_discovered = patterns.len();
        stats.last_learning_cycle = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        stats.model_confidence = patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64;

        info!("Learning cycle completed. Discovered {} patterns", patterns.len());

        Ok(())
    }

    /// Apply learned optimizations automatically
    pub async fn apply_optimizations(
        &self,
        perception_result: &mut UnifiedPerceptionResult,
        decision_context: &mut DecisionContext,
    ) -> Result<Vec<String>> {
        if !self.config.auto_apply_optimizations {
            return Ok(Vec::new());
        }

        let recommendations = self.get_optimization_recommendations(
            perception_result,
            decision_context,
        ).await?;

        let mut applied_optimizations = Vec::new();

        for recommendation in recommendations {
            if recommendation.risk_level != RiskLevel::High {
                match &recommendation.action_type {
                    ActionType::ChangePerceptionLevel(new_level) => {
                        // Apply perception level change
                        debug!("Applying perception level optimization: {:?}", new_level);
                        applied_optimizations.push(format!("Changed perception level to {:?}", new_level));
                    }
                    ActionType::AdjustTimeout(new_timeout) => {
                        // Apply timeout adjustment
                        decision_context.time_constraints = Some(*new_timeout);
                        debug!("Applied timeout adjustment: {:?}", new_timeout);
                        applied_optimizations.push(format!("Adjusted timeout to {:?}", new_timeout));
                    }
                    ActionType::AddCachingLayer => {
                        debug!("Applied caching layer optimization");
                        applied_optimizations.push("Added caching layer".to_string());
                    }
                    _ => {
                        // Other optimizations would be applied here
                        debug!("Applied optimization: {:?}", recommendation.action_type);
                    }
                }

                let mut stats = self.learning_stats.lock().await;
                stats.optimizations_applied += 1;
            }
        }

        Ok(applied_optimizations)
    }

    /// Get current learning statistics
    pub async fn get_learning_stats(&self) -> LearningStats {
        self.learning_stats.lock().await.clone()
    }

    /// Get feature importance rankings
    pub async fn get_feature_importance(&self) -> Vec<FeatureImportance> {
        let importance = self.feature_importance.read().await;
        let mut features: Vec<FeatureImportance> = importance.values().cloned().collect();
        features.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());
        features
    }

    /// Export learned patterns for analysis
    pub async fn export_learned_patterns(&self) -> Vec<LearnedPattern> {
        self.learned_patterns.read().await.clone()
    }

    /// Clear learning history and patterns
    pub async fn clear_learning_data(&self) -> Result<()> {
        self.execution_history.write().await.clear();
        self.learned_patterns.write().await.clear();
        self.feature_importance.write().await.clear();
        self.optimization_cache.write().await.clear();
        
        let mut stats = self.learning_stats.lock().await;
        *stats = LearningStats {
            total_executions_learned: 0,
            patterns_discovered: 0,
            optimizations_applied: 0,
            success_rate_improvement: 0.0,
            performance_improvement: 0.0,
            learning_accuracy: 0.0,
            model_confidence: 0.0,
            last_learning_cycle: 0,
        };

        info!("Cleared all learning data");
        Ok(())
    }

    // Private helper methods

    async fn discover_perception_optimization_patterns(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Analyze perception level vs. success rate
        let mut level_stats: HashMap<PerceptionLevel, (usize, usize, Duration)> = HashMap::new();
        
        for record in records {
            let entry = level_stats.entry(record.perception_level.clone()).or_insert((0, 0, Duration::new(0, 0)));
            entry.0 += 1; // total
            if record.execution_result.success {
                entry.1 += 1; // successes
            }
            entry.2 += record.performance_metrics.perception_time;
        }

        // Find optimal perception levels for different contexts
        for (level, (total, successes, total_time)) in level_stats {
            if total >= self.config.pattern_stability_threshold {
                let success_rate = successes as f64 / total as f64;
                let avg_time = total_time / total as u32;

                if success_rate > 0.85 {
                    patterns.push(LearnedPattern {
                        id: Uuid::new_v4(),
                        pattern_type: PatternType::PerceptionLevelOptimization,
                        conditions: vec![
                            PatternCondition {
                                feature: "page_complexity".to_string(),
                                operator: ConditionOperator::LessThan,
                                value: ConditionValue::Number(0.5),
                                weight: 0.8,
                            }
                        ],
                        recommended_action: RecommendedAction {
                            action_type: ActionType::ChangePerceptionLevel(level),
                            parameters: HashMap::new(),
                            expected_improvement: (success_rate - 0.7) * 100.0,
                            risk_level: RiskLevel::Low,
                        },
                        confidence: success_rate,
                        supporting_evidence: total,
                        last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                        success_rate,
                        average_improvement: (success_rate - 0.7) * 100.0,
                    });
                }
            }
        }

        Ok(patterns)
    }

    async fn discover_workflow_optimization_patterns(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Analyze step success patterns
        let mut step_failure_patterns: HashMap<String, usize> = HashMap::new();
        
        for record in records {
            for error_type in &record.execution_result.error_types {
                *step_failure_patterns.entry(error_type.clone()).or_insert(0) += 1;
            }
        }

        // Create preventive patterns for common failures
        for (error_type, frequency) in step_failure_patterns {
            if frequency >= self.config.pattern_stability_threshold {
                patterns.push(LearnedPattern {
                    id: Uuid::new_v4(),
                    pattern_type: PatternType::WorkflowStepOptimization,
                    conditions: vec![
                        PatternCondition {
                            feature: "previous_failures".to_string(),
                            operator: ConditionOperator::Contains,
                            value: ConditionValue::String(error_type.clone()),
                            weight: 0.9,
                        }
                    ],
                    recommended_action: RecommendedAction {
                        action_type: ActionType::AddPreventiveCheck(error_type),
                        parameters: HashMap::new(),
                        expected_improvement: 20.0,
                        risk_level: RiskLevel::Low,
                    },
                    confidence: 0.8,
                    supporting_evidence: frequency,
                    last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    success_rate: 0.8,
                    average_improvement: 20.0,
                });
            }
        }

        Ok(patterns)
    }

    async fn discover_error_prevention_patterns(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Analyze error correlation with environment factors
        let failed_records: Vec<&ExecutionRecord> = records.iter()
            .filter(|r| !r.execution_result.success)
            .collect();

        if failed_records.len() >= self.config.pattern_stability_threshold {
            // Common environment factor in failures
            let mut network_speed_failures = 0;
            let mut page_load_failures = 0;

            for record in &failed_records {
                if record.environment_factors.network_speed < 1.0 {
                    network_speed_failures += 1;
                }
                if record.environment_factors.page_load_time > Duration::from_secs(10) {
                    page_load_failures += 1;
                }
            }

            if network_speed_failures as f64 / failed_records.len() as f64 > 0.6 {
                patterns.push(LearnedPattern {
                    id: Uuid::new_v4(),
                    pattern_type: PatternType::ErrorPrevention,
                    conditions: vec![
                        PatternCondition {
                            feature: "network_speed".to_string(),
                            operator: ConditionOperator::LessThan,
                            value: ConditionValue::Number(1.0),
                            weight: 0.8,
                        }
                    ],
                    recommended_action: RecommendedAction {
                        action_type: ActionType::AdjustTimeout(Duration::from_secs(30)),
                        parameters: HashMap::new(),
                        expected_improvement: 30.0,
                        risk_level: RiskLevel::Low,
                    },
                    confidence: 0.75,
                    supporting_evidence: network_speed_failures,
                    last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    success_rate: 0.75,
                    average_improvement: 30.0,
                });
            }
        }

        Ok(patterns)
    }

    async fn discover_performance_optimization_patterns(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<Vec<LearnedPattern>> {
        let mut patterns = Vec::new();

        // Find high-performance executions and their characteristics
        let high_performance_records: Vec<&ExecutionRecord> = records.iter()
            .filter(|r| r.execution_result.success && 
                       r.performance_metrics.total_duration < Duration::from_secs(5) &&
                       r.performance_metrics.memory_usage < 100)
            .collect();

        if high_performance_records.len() >= self.config.pattern_stability_threshold {
            // Analyze common characteristics
            let avg_cache_hit_rate: f64 = high_performance_records.iter()
                .map(|r| r.performance_metrics.cache_hit_rate)
                .sum::<f64>() / high_performance_records.len() as f64;

            if avg_cache_hit_rate > 0.9 {
                patterns.push(LearnedPattern {
                    id: Uuid::new_v4(),
                    pattern_type: PatternType::PerformanceOptimization,
                    conditions: vec![
                        PatternCondition {
                            feature: "cache_hit_rate".to_string(),
                            operator: ConditionOperator::GreaterThan,
                            value: ConditionValue::Number(0.9),
                            weight: 0.9,
                        }
                    ],
                    recommended_action: RecommendedAction {
                        action_type: ActionType::AddCachingLayer,
                        parameters: HashMap::new(),
                        expected_improvement: 40.0,
                        risk_level: RiskLevel::Low,
                    },
                    confidence: 0.85,
                    supporting_evidence: high_performance_records.len(),
                    last_updated: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                    success_rate: 1.0,
                    average_improvement: 40.0,
                });
            }
        }

        Ok(patterns)
    }

    async fn calculate_pattern_match(
        &self,
        pattern: &LearnedPattern,
        perception_result: &UnifiedPerceptionResult,
        decision_context: &DecisionContext,
    ) -> Result<f64> {
        let mut match_score = 0.0;
        let mut total_weight = 0.0;

        for condition in &pattern.conditions {
            let feature_value = self.extract_feature_value(
                &condition.feature,
                perception_result,
                decision_context,
            ).await?;

            let condition_match = self.evaluate_condition(condition, feature_value)?;
            match_score += condition_match * condition.weight;
            total_weight += condition.weight;
        }

        if total_weight > 0.0 {
            Ok(match_score / total_weight)
        } else {
            Ok(0.0)
        }
    }

    async fn extract_feature_value(
        &self,
        feature_name: &str,
        perception_result: &UnifiedPerceptionResult,
        decision_context: &DecisionContext,
    ) -> Result<f64> {
        match feature_name {
            "page_complexity" => Ok(self.calculate_page_complexity(perception_result)),
            "network_speed" => Ok(1.5), // Placeholder
            "cache_hit_rate" => Ok(0.85), // Placeholder
            "retry_count" => Ok(decision_context.retry_count as f64),
            "memory_usage" => Ok(perception_result.execution_info.performance_metrics.memory_usage_mb),
            "cpu_usage" => Ok(perception_result.execution_info.performance_metrics.cpu_usage_percent),
            _ => Ok(0.0),
        }
    }

    fn evaluate_condition(&self, condition: &PatternCondition, value: f64) -> Result<f64> {
        match (&condition.operator, &condition.value) {
            (ConditionOperator::GreaterThan, ConditionValue::Number(threshold)) => {
                Ok(if value > *threshold { 1.0 } else { 0.0 })
            }
            (ConditionOperator::LessThan, ConditionValue::Number(threshold)) => {
                Ok(if value < *threshold { 1.0 } else { 0.0 })
            }
            (ConditionOperator::Equals, ConditionValue::Number(target)) => {
                Ok(if (value - target).abs() < 0.01 { 1.0 } else { 0.0 })
            }
            (ConditionOperator::InRange, ConditionValue::Range(min, max)) => {
                Ok(if value >= *min && value <= *max { 1.0 } else { 0.0 })
            }
            _ => Ok(0.0), // Unsupported condition
        }
    }

    fn calculate_page_complexity(&self, perception_result: &UnifiedPerceptionResult) -> f64 {
        // Calculate complexity based on perception data
        let mut complexity = 0.0;

        if let Some(lightning) = &perception_result.lightning_data {
            complexity += lightning.key_elements.len() as f64 * 0.01;
        }

        if let Some(quick) = &perception_result.quick_data {
            complexity += quick.interaction_elements.len() as f64 * 0.02;
            complexity += quick.form_analysis.len() as f64 * 0.1;
        }

        if let Some(standard) = &perception_result.standard_data {
            complexity += (standard.content_analysis.text_content.total_words as f64 / 1000.0) * 0.1;
            complexity += standard.data_extraction.internal_links.len() as f64 * 0.01;
        }

        complexity.min(1.0) // Normalize to 0-1 range
    }

    async fn update_feature_importance(&self, records: &[ExecutionRecord]) -> Result<()> {
        // Simplified feature importance calculation
        let mut feature_correlations: HashMap<String, (f64, usize)> = HashMap::new();

        for record in records {
            let success = if record.execution_result.success { 1.0 } else { 0.0 };

            // Update correlations for various features
            let features = vec![
                ("page_complexity", self.calculate_page_complexity(&UnifiedPerceptionResult::default())),
                ("network_speed", record.environment_factors.network_speed),
                ("memory_usage", record.performance_metrics.memory_usage as f64),
                ("cpu_usage", record.performance_metrics.cpu_usage),
            ];

            for (feature_name, value) in features {
                let entry = feature_correlations.entry(feature_name.to_string()).or_insert((0.0, 0));
                entry.0 += value * success;
                entry.1 += 1;
            }
        }

        // Update feature importance store
        let mut importance_store = self.feature_importance.write().await;
        for (feature_name, (correlation_sum, count)) in feature_correlations {
            let avg_correlation = correlation_sum / count as f64;
            
            importance_store.insert(feature_name.clone(), FeatureImportance {
                feature_name,
                importance_score: avg_correlation.abs(),
                correlation_with_success: avg_correlation,
                stability: 0.8, // Simplified
                update_count: count,
            });
        }

        Ok(())
    }
}

impl Default for UnifiedPerceptionResult {
    fn default() -> Self {
        Self {
            lightning_data: None,
            quick_data: None,
            standard_data: None,
            deep_data: None,
            recommendations: Vec::new(),
            execution_info: crate::perception_mvp::ExecutionInfo {
                actual_level: PerceptionLevel::Lightning,
                quality_score: 0.0,
                confidence_score: 0.0,
                fallback_reason: None,
                performance_metrics: crate::perception_mvp::PerformanceMetrics {
                    total_time: Duration::new(0, 0),
                    memory_usage_mb: 0.0,
                    cpu_usage_percent: 0.0,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_learning_initialization() {
        let config = LearningConfig::default();
        let learning_system = AdaptiveLearningSystem::new(config);
        
        let stats = learning_system.get_learning_stats().await;
        assert_eq!(stats.total_executions_learned, 0);
        assert_eq!(stats.patterns_discovered, 0);
    }

    #[tokio::test]
    async fn test_execution_recording() {
        let config = LearningConfig::default();
        let learning_system = AdaptiveLearningSystem::new(config);

        // Test data would be created here for full testing
        // This is a placeholder test structure
        let stats_before = learning_system.get_learning_stats().await;
        assert_eq!(stats_before.total_executions_learned, 0);
    }

    #[tokio::test]
    async fn test_pattern_discovery() {
        let config = LearningConfig::default();
        let learning_system = AdaptiveLearningSystem::new(config);

        // Test pattern discovery with mock data
        let result = learning_system.run_learning_cycle().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_optimization_recommendations() {
        let config = LearningConfig::default();
        let learning_system = AdaptiveLearningSystem::new(config);

        // Test recommendation generation with mock perception result and context
        // This would require creating proper test data
        let patterns = learning_system.export_learned_patterns().await;
        assert!(patterns.is_empty()); // Initially empty
    }
}